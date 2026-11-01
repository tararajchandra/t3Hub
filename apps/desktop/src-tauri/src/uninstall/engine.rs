use std::path::PathBuf;
use std::sync::Arc;
use std::process::Command;
use chrono::Utc;
use log::{info, warn, error};

use crate::database::DatabaseManager;
use crate::installer::adapters::MsiInstaller;
use crate::installer::detection::AppDetector;
use crate::manifest::parser::parse_manifest_from_str;
use super::UninstallResult;

pub struct UninstallEngine {
    db: Arc<DatabaseManager>,
    apps_dir: PathBuf,
}

impl UninstallEngine {
    pub fn new(db: Arc<DatabaseManager>, apps_dir: PathBuf) -> Self {
        Self { db, apps_dir }
    }

    pub fn default_engine(db: Arc<DatabaseManager>) -> Self {
        let local_app_data = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("C:\\ProgramData"));
        let apps_dir = local_app_data.join("T3Hub").join("Apps");
        Self::new(db, apps_dir)
    }

    /// Safely split a Windows command line string into (executable_path, args)
    pub fn parse_command_line(cmd_line: &str) -> Option<(String, Vec<String>)> {
        let trimmed = cmd_line.trim();
        if trimmed.is_empty() {
            return None;
        }

        if trimmed.starts_with('"') {
            // Quoted executable: find closing quote
            if let Some(end_quote) = trimmed[1..].find('"') {
                let exe = &trimmed[1..=end_quote];
                let rest = trimmed[end_quote + 2..].trim();
                let args = if rest.is_empty() {
                    Vec::new()
                } else {
                    rest.split_whitespace().map(|s| s.to_string()).collect()
                };
                return Some((exe.to_string(), args));
            }
        }

        // Space-separated
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            None
        } else {
            let exe = parts[0].to_string();
            let args = parts[1..].iter().map(|s| s.to_string()).collect();
            Some((exe, args))
        }
    }

    /// Orchestrates safe uninstallation of a package
    pub fn uninstall_package(&self, package_id: &str) -> Result<UninstallResult, String> {
        info!("Starting uninstallation workflow for package: {}", package_id);
        let op_id = format!("op-uninst-{}", Utc::now().timestamp_millis());

        // 1. Check local installation record in SQLite
        let install_record = self.db.get_installation_by_package_id(package_id)
            .map_err(|e| format!("Database query error: {}", e))?;

        let uninstall_type = self.db.get_installation_uninstall_type(package_id)
            .unwrap_or(None)
            .unwrap_or_else(|| "zip_directory".to_string());

        let target_version = install_record
            .as_ref()
            .map(|r| r.version.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let install_path = install_record
            .as_ref()
            .and_then(|r| r.install_path.clone());

        // Log operation start
        let mut log_lines = vec![format!("Uninstallation started for {} v{}", package_id, target_version)];
        let _ = self.db.record_operation_start(&op_id, package_id, "uninstall", &target_version, &log_lines.join("\n"));

        // 2. Dispatch uninstallation logic based on uninstall_type
        match uninstall_type.as_str() {
            "zip" | "zip_directory" => {
                let target_dir = if let Some(ref p) = install_path {
                    PathBuf::from(p)
                } else {
                    self.apps_dir.join(package_id)
                };

                log_lines.push(format!("Executing ZIP portable cleanup on directory: {:?}", target_dir));

                // SECURITY CHECK: Verify target directory does not escape apps_dir
                if target_dir.exists() {
                    let canonical_target = match target_dir.canonicalize() {
                        Ok(c) => c,
                        Err(e) => {
                            let err = format!("Failed to canonicalize target uninstall directory {:?}: {}", target_dir, e);
                            log_lines.push(err.clone());
                            let _ = self.db.record_operation_complete(&op_id, "failed", &log_lines.join("\n"));
                            return Err(err);
                        }
                    };

                    let canonical_apps = self.apps_dir.canonicalize().unwrap_or_else(|_| self.apps_dir.clone());

                    if !canonical_target.starts_with(&canonical_apps) {
                        let err = format!(
                            "Security Violation: Target path {:?} is outside the managed apps directory {:?}. Aborting uninstallation.",
                            canonical_target, canonical_apps
                        );
                        error!("{}", err);
                        log_lines.push(err.clone());
                        let _ = self.db.record_operation_complete(&op_id, "failed", &log_lines.join("\n"));
                        return Err(err);
                    }

                    if let Err(e) = std::fs::remove_dir_all(&canonical_target) {
                        let err = format!("Failed to remove application directory {:?}: {}", canonical_target, e);
                        error!("{}", err);
                        log_lines.push(err.clone());
                        let _ = self.db.record_operation_complete(&op_id, "failed", &log_lines.join("\n"));
                        return Err(err);
                    }

                    log_lines.push(format!("Successfully deleted managed directory {:?}", canonical_target));
                } else {
                    log_lines.push(format!("Target directory {:?} does not exist on disk, cleaning up database record.", target_dir));
                }
            }

            "msi" => {
                log_lines.push(format!("Executing MSI uninstallation for {}", package_id));
                // If install_path is a product code GUID (e.g. {12345678-....})
                let target = install_path.as_deref().unwrap_or(package_id);
                match MsiInstaller::uninstall(target, None) {
                    Ok(outcome) => {
                        log_lines.push(format!("MSI uninstaller finished: {}", outcome.message));
                    }
                    Err(e) => {
                        warn!("MSI uninstallation reported error: {}. Cleaning inventory.", e);
                        log_lines.push(format!("Warning: MSI uninstaller output: {}", e));
                    }
                }
            }

            "registry" | "exe" => {
                log_lines.push(format!("Executing registry/executable uninstaller for {}", package_id));

                // 1. Look up manifest to find declared uninstaller registry path
                let mut uninstaller_cmd: Option<String> = None;
                if let Ok(Some(details)) = self.db.get_package_details(package_id) {
                    if let Some(ver) = details.versions.first() {
                        if let Ok(manifest) = parse_manifest_from_str(&ver.manifest_raw) {
                            if let Some(ref uninst) = manifest.uninstallation {
                                if let Some(ref detect) = uninst.detect {
                                    if let Some(ref reg_path) = detect.registry_path {
                                        if let Some(detected_info) = AppDetector::find_by_registry_path(reg_path) {
                                            uninstaller_cmd = detected_info.quiet_uninstall_string
                                                .or(detected_info.uninstall_string);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Fallback: search system by package name or ID
                if uninstaller_cmd.is_none() {
                    if let Some(detected) = AppDetector::find_by_name(package_id) {
                        uninstaller_cmd = detected.quiet_uninstall_string.or(detected.uninstall_string);
                    }
                }

                if let Some(cmd_str) = uninstaller_cmd {
                    log_lines.push(format!("Found uninstall command: {}", cmd_str));
                    if let Some((exe, mut args)) = Self::parse_command_line(&cmd_str) {
                        // Append silent arguments if standard uninstaller
                        if !args.iter().any(|a| a.eq_ignore_ascii_case("/silent") || a.eq_ignore_ascii_case("/verysilent") || a.eq_ignore_ascii_case("/s")) {
                            args.push("/VERYSILENT".to_string());
                            args.push("/NORESTART".to_string());
                        }

                        let mut cmd = Command::new(&exe);
                        for a in &args {
                            cmd.arg(a);
                        }

                        match cmd.output() {
                            Ok(output) => {
                                log_lines.push(format!("Uninstaller exited with code: {:?}", output.status.code()));
                            }
                            Err(e) => {
                                warn!("Could not execute uninstaller: {}. Proceeding with DB cleanup.", e);
                                log_lines.push(format!("Warning: Could not launch uninstaller: {}", e));
                            }
                        }
                    }
                } else {
                    log_lines.push("No uninstaller string found in registry; removing local registry inventory.".to_string());
                }
            }

            other => {
                log_lines.push(format!("Unknown uninstall type '{}', proceeding with DB removal.", other));
            }
        }

        // 3. Delete from installations table
        let _ = self.db.remove_installation(package_id);

        // 4. Record operation complete
        log_lines.push(format!("Uninstallation of '{}' completed successfully.", package_id));
        let _ = self.db.record_operation_complete(&op_id, "success", &log_lines.join("\n"));

        Ok(UninstallResult {
            success: true,
            error_message: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_line_quoted_exe() {
        let cmd = r#""C:\Program Files\MyApp\unins000.exe" /SILENT /NORESTART"#;
        let (exe, args) = UninstallEngine::parse_command_line(cmd).unwrap();
        assert_eq!(exe, r#"C:\Program Files\MyApp\unins000.exe"#);
        assert_eq!(args, vec!["/SILENT", "/NORESTART"]);
    }

    #[test]
    fn test_parse_command_line_unquoted_exe() {
        let cmd = r#"C:\Apps\tool\uninstall.exe -q"#;
        let (exe, args) = UninstallEngine::parse_command_line(cmd).unwrap();
        assert_eq!(exe, r#"C:\Apps\tool\uninstall.exe"#);
        assert_eq!(args, vec!["-q"]);
    }

    #[test]
    fn test_zip_uninstall_path_traversal_rejection() {
        let temp_dir = std::env::temp_dir().join("t3hub_test_uninstall");
        let _ = std::fs::remove_dir_all(&temp_dir);
        let _ = std::fs::create_dir_all(&temp_dir);

        let db_path = temp_dir.join("test.db");
        let db = Arc::new(DatabaseManager::new(&db_path).unwrap());
        let apps_dir = temp_dir.join("Apps");
        let _ = std::fs::create_dir_all(&apps_dir);

        let engine = UninstallEngine::new(db.clone(), apps_dir);

        // Record a malicious escape install_path
        let outside_dir = temp_dir.join("outside_target");
        let _ = std::fs::create_dir_all(&outside_dir);

        db.record_installation(
            "malicious.escape",
            "1.0.0",
            "user",
            &outside_dir.to_string_lossy(),
            "zip_directory",
        ).unwrap();

        // Attempt uninstall
        let result = engine.uninstall_package("malicious.escape");
        assert!(result.is_err(), "Uninstalling directory outside apps_dir must be rejected");
        assert!(result.unwrap_err().contains("Security Violation"));

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_zip_uninstall_successful_directory_cleanup() {
        let temp_dir = std::env::temp_dir().join("t3hub_test_uninstall_success");
        let _ = std::fs::remove_dir_all(&temp_dir);
        let _ = std::fs::create_dir_all(&temp_dir);

        let db_path = temp_dir.join("test.db");
        let db = Arc::new(DatabaseManager::new(&db_path).unwrap());
        let apps_dir = temp_dir.join("Apps");
        let target_app_dir = apps_dir.join("developer.testapp");
        let _ = std::fs::create_dir_all(&target_app_dir);

        let dummy_file = target_app_dir.join("testapp.exe");
        std::fs::write(&dummy_file, b"binary content").unwrap();

        db.record_installation(
            "developer.testapp",
            "1.0.0",
            "user",
            &target_app_dir.to_string_lossy(),
            "zip_directory",
        ).unwrap();

        let engine = UninstallEngine::new(db.clone(), apps_dir);

        let res = engine.uninstall_package("developer.testapp").unwrap();
        assert!(res.success);
        assert!(!target_app_dir.exists(), "App directory must be deleted");

        // Verify database state
        let record = db.get_installation_by_package_id("developer.testapp").unwrap();
        assert!(record.is_none(), "Database record must be deleted");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
