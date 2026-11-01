use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use uuid::Uuid;

use crate::database::DatabaseManager;
use crate::download::{DownloadManager, ProgressUpdate};
use crate::installer::adapters::{ExeInstaller, MsiInstaller};
use crate::installer::zip_adapter::ZipInstaller;
use crate::manifest::parse_manifest_from_str;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstallationResult {
    pub success: bool,
    pub install_path: Option<String>,
    pub logs: String,
    pub error_message: Option<String>,
}

pub struct InstallationEngine<'a> {
    db: &'a DatabaseManager,
    download_manager: DownloadManager,
    apps_dir: PathBuf,
}

impl<'a> InstallationEngine<'a> {
    pub fn new<P: AsRef<Path>>(db: &'a DatabaseManager, cache_dir: P, apps_dir: P) -> Self {
        Self {
            db,
            download_manager: DownloadManager::new(cache_dir),
            apps_dir: apps_dir.as_ref().to_path_buf(),
        }
    }

    /// Default constructor utilizing standard Windows per-user directories (%LOCALAPPDATA%\T3Hub)
    pub fn default_paths(db: &'a DatabaseManager) -> Self {
        let base_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("T3Hub");

        let cache_dir = base_dir.join("Cache");
        let apps_dir = base_dir.join("Apps");

        Self::new(db, cache_dir, apps_dir)
    }

    /// Orchestrates end-to-end installation of a package:
    /// 1. Manifest resolution from SQLite
    /// 2. Verification of architecture & integrity
    /// 3. Download & SHA-256 digest verification
    /// 4. Adapter execution (safe ZIP portable extraction)
    /// 5. Local inventory & operations history persistence
    pub async fn install<F>(
        &self,
        package_id: &str,
        target_version: Option<&str>,
        cancel_token: Option<Arc<AtomicBool>>,
        mut on_progress: F,
    ) -> InstallationResult
    where
        F: FnMut(ProgressUpdate),
    {
        let op_id = Uuid::new_v4().to_string();
        let mut logs = Vec::new();

        logs.push(format!("Initiating installation for package '{}'", package_id));

        // 1. Retrieve package details and manifest
        let details = match self.db.get_package_details(package_id) {
            Ok(Some(d)) => d,
            Ok(None) => {
                let err = format!("Package '{}' not found in local catalog index.", package_id);
                return InstallationResult {
                    success: false,
                    install_path: None,
                    logs: err.clone(),
                    error_message: Some(err),
                };
            }
            Err(e) => {
                let err = format!("Database error while resolving package '{}': {}", package_id, e);
                return InstallationResult {
                    success: false,
                    install_path: None,
                    logs: err.clone(),
                    error_message: Some(err),
                };
            }
        };

        // Determine target version
        let version_detail = if let Some(req_ver) = target_version {
            details.versions.iter().find(|v| v.version == req_ver)
        } else {
            details.versions.first()
        };

        let ver_info = match version_detail {
            Some(v) => v,
            None => {
                let err = format!("No available manifest version found for package '{}'", package_id);
                return InstallationResult {
                    success: false,
                    install_path: None,
                    logs: err.clone(),
                    error_message: Some(err),
                };
            }
        };

        // Parse manifest
        let manifest = match parse_manifest_from_str(&ver_info.manifest_raw) {
            Ok(m) => m,
            Err(e) => {
                let err = format!("Manifest parsing failed for '{}': {}", package_id, e);
                return InstallationResult {
                    success: false,
                    install_path: None,
                    logs: err.clone(),
                    error_message: Some(err),
                };
            }
        };

        let ver_str = manifest.package.version.clone();
        logs.push(format!("Resolved manifest for {} v{}", manifest.package.name, ver_str));

        // Record operation started in SQLite
        let _ = self.db.record_operation_start(
            &op_id,
            package_id,
            "install",
            &ver_str,
            &logs.join("\n"),
        );

        // 2. Resolve target artifact
        let artifact = match manifest
            .artifacts
            .iter()
            .find(|a| a.id == manifest.installation.artifact)
        {
            Some(a) => a,
            None => {
                let err = format!(
                    "Referenced artifact '{}' not found in manifest artifacts list.",
                    manifest.installation.artifact
                );
                logs.push(err.clone());
                let _ = self.db.record_operation_complete(&op_id, "failed", &logs.join("\n"));
                return InstallationResult {
                    success: false,
                    install_path: None,
                    logs: logs.join("\n"),
                    error_message: Some(err),
                };
            }
        };

        logs.push(format!(
            "Downloading artifact '{}' ({}) with expected SHA-256 {}",
            artifact.id, artifact.artifact_type, artifact.sha256
        ));

        // 3. Download and verify SHA-256
        let cached_path = match self
            .download_manager
            .download_and_verify(
                &artifact.url,
                &artifact.sha256,
                &artifact.artifact_type,
                cancel_token,
                &mut on_progress,
            )
            .await
        {
            Ok(p) => {
                logs.push(format!("Artifact downloaded and verified. SHA-256 matched: {:?}", p));
                p
            }
            Err(e) => {
                let err = format!("Download/verification failed: {}", e);
                logs.push(err.clone());
                let _ = self.db.record_operation_complete(&op_id, "failed", &logs.join("\n"));
                return InstallationResult {
                    success: false,
                    install_path: None,
                    logs: logs.join("\n"),
                    error_message: Some(err),
                };
            }
        };

        // 4. Dispatch to installer adapter
        let install_type = manifest.installation.install_type.as_str();
        match install_type {
            "zip" => {
                let destination = self.apps_dir.join(package_id);
                logs.push(format!("Extracting ZIP archive safely into {:?}", destination));

                match ZipInstaller::install(&cached_path, &destination) {
                    Ok(outcome) => {
                        let path_str = outcome.install_dir.to_string_lossy().to_string();
                        logs.push(format!(
                            "Extraction successful! Extracted {} files ({} bytes). Main executable: {:?}",
                            outcome.files_extracted, outcome.total_bytes, outcome.executable_path
                        ));

                        // Persist to local installations inventory
                        if let Err(e) = self.db.record_installation(
                            package_id,
                            &ver_str,
                            &manifest.installation.scope,
                            &path_str,
                            "zip_directory",
                        ) {
                            logs.push(format!("Warning: Failed to record installation in DB: {}", e));
                        }

                        let _ = self.db.record_operation_complete(&op_id, "success", &logs.join("\n"));

                        InstallationResult {
                            success: true,
                            install_path: Some(path_str),
                            logs: logs.join("\n"),
                            error_message: None,
                        }
                    }
                    Err(e) => {
                        let err = format!("ZIP extraction failed: {}", e);
                        logs.push(err.clone());
                        let _ = self.db.record_operation_complete(&op_id, "failed", &logs.join("\n"));
                        InstallationResult {
                            success: false,
                            install_path: None,
                            logs: logs.join("\n"),
                            error_message: Some(err),
                        }
                    }
                }
            }
            "msi" => {
                logs.push(format!("Executing silent Windows MSI installer for: {:?}", cached_path));
                match MsiInstaller::install(
                    &cached_path,
                    Some(&manifest.installation.arguments),
                    manifest.installation.timeout_seconds,
                ) {
                    Ok(outcome) => {
                        let path_str = cached_path.to_string_lossy().to_string();
                        logs.push(format!("MSI installation completed. Outcome: {}", outcome.message));
                        if outcome.reboot_required {
                            logs.push("Note: A system reboot is recommended to finalize installation.".to_string());
                        }

                        let _ = self.db.record_installation(
                            package_id,
                            &ver_str,
                            &manifest.installation.scope,
                            &path_str,
                            "msi",
                        );
                        let _ = self.db.record_operation_complete(&op_id, "success", &logs.join("\n"));

                        InstallationResult {
                            success: true,
                            install_path: Some(path_str),
                            logs: logs.join("\n"),
                            error_message: None,
                        }
                    }
                    Err(e) => {
                        let err = format!("MSI installation error: {}", e);
                        logs.push(err.clone());
                        let _ = self.db.record_operation_complete(&op_id, "failed", &logs.join("\n"));
                        InstallationResult {
                            success: false,
                            install_path: None,
                            logs: logs.join("\n"),
                            error_message: Some(err),
                        }
                    }
                }
            }
            "exe" => {
                logs.push(format!("Executing silent Windows EXE installer for: {:?}", cached_path));
                match ExeInstaller::install(
                    &cached_path,
                    manifest.installation.installer_profile.as_deref(),
                    Some(&manifest.installation.arguments),
                ) {
                    Ok(outcome) => {
                        let path_str = cached_path.to_string_lossy().to_string();
                        logs.push(format!("EXE installation completed successfully: {}", outcome.message));

                        let _ = self.db.record_installation(
                            package_id,
                            &ver_str,
                            &manifest.installation.scope,
                            &path_str,
                            "registry",
                        );
                        let _ = self.db.record_operation_complete(&op_id, "success", &logs.join("\n"));

                        InstallationResult {
                            success: true,
                            install_path: Some(path_str),
                            logs: logs.join("\n"),
                            error_message: None,
                        }
                    }
                    Err(e) => {
                        let err = format!("EXE installation error: {}", e);
                        logs.push(err.clone());
                        let _ = self.db.record_operation_complete(&op_id, "failed", &logs.join("\n"));
                        InstallationResult {
                            success: false,
                            install_path: None,
                            logs: logs.join("\n"),
                            error_message: Some(err),
                        }
                    }
                }
            }
            other => {
                let err = format!("Unsupported installation adapter: '{}'", other);
                logs.push(err.clone());
                let _ = self.db.record_operation_complete(&op_id, "failed", &logs.join("\n"));
                InstallationResult {
                    success: false,
                    install_path: None,
                    logs: logs.join("\n"),
                    error_message: Some(err),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use zip::write::{ExtendedFileOptions, FileOptions};
    use zip::ZipWriter;

    #[tokio::test]
    async fn test_full_installation_engine_pipeline() {
        let temp_dir = std::env::temp_dir().join(format!("t3_engine_test_{}", Uuid::new_v4()));
        let db_path = temp_dir.join("test.db");
        let cache_dir = temp_dir.join("cache");
        let apps_dir = temp_dir.join("apps");
        let repo_artifact = temp_dir.join("tools.terminal-x-v2.0.1.zip");

        fs::create_dir_all(&temp_dir).unwrap();

        // 1. Create a mock zip artifact
        {
            let file = File::create(&repo_artifact).unwrap();
            let mut zip = ZipWriter::new(file);
            let options = FileOptions::<ExtendedFileOptions>::default();

            zip.start_file("terminalx.exe", options.clone()).unwrap();
            zip.write_all(b"BINARY_CONTENT").unwrap();

            zip.start_file("README.md", options).unwrap();
            zip.write_all(b"# Terminal-X Modern Terminal").unwrap();

            zip.finish().unwrap();
        }

        let sha256 = crate::security::compute_file_sha256(&repo_artifact).unwrap();

        // 2. Setup SQLite DB
        let db = DatabaseManager::new(&db_path).unwrap();

        // Add package and manifest
        let manifest_yaml = format!(
            r#"
schema_version: "1.0"
package:
  id: "tools.terminal-x"
  name: "Terminal-X"
  version: "2.0.1"
  publisher: "Terminal Lab"
platform:
  os: "windows"
  architectures:
    - "x64"
artifacts:
  - id: "win-x64"
    type: "zip"
    url: "file:///{}"
    sha256: "{}"
installation:
  type: "zip"
  artifact: "win-x64"
  scope: "user"
"#,
            repo_artifact.to_str().unwrap().replace('\\', "/"),
            sha256
        );

        db.upsert_package(
            "tools.terminal-x",
            "t3-official",
            "Terminal-X",
            Some("GPU Terminal"),
            "Terminal Lab",
            "[\"Developer Tools\"]",
            "[\"cli\"]",
            "2.0.1",
            None,
            "2026-10-31T00:00:00Z",
        ).unwrap();

        db.upsert_package_version(
            "tools.terminal-x:2.0.1",
            "tools.terminal-x",
            "2.0.1",
            "stable",
            Some("2026-10-31"),
            &manifest_yaml,
            "2026-10-31T00:00:00Z",
        ).unwrap();

        // 3. Run installation engine
        let engine = InstallationEngine::new(&db, cache_dir, apps_dir);
        let result = engine.install("tools.terminal-x", Some("2.0.1"), None, |_| {}).await;

        assert!(result.success, "Installation should succeed: {:?}", result.error_message);
        assert!(result.install_path.is_some());

        let installed_path = PathBuf::from(result.install_path.unwrap());
        assert!(installed_path.exists());
        assert!(installed_path.join("terminalx.exe").exists());
        assert!(installed_path.join("README.md").exists());

        // 4. Verify SQLite recorded inventory and operation
        let installed_apps = db.get_installed_applications().unwrap();
        assert_eq!(installed_apps.len(), 1);
        assert_eq!(installed_apps[0].package_id, "tools.terminal-x");
        assert_eq!(installed_apps[0].version, "2.0.1");
        assert_eq!(installed_apps[0].status, "installed");

        let operations = db.get_operations().unwrap();
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].status, "success");
        assert_eq!(operations[0].operation_type, "install");

        // Clean up
        let _ = fs::remove_dir_all(temp_dir);
    }
}
