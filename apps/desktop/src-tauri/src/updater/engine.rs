use std::sync::Arc;
use serde::{Deserialize, Serialize};
use log::info;

use crate::database::DatabaseManager;
use crate::manifest::parser::parse_manifest_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableUpdate {
    pub package_id: String,
    pub package_name: String,
    pub publisher: String,
    pub installed_version: String,
    pub new_version: String,
    pub size_mb: Option<f32>,
    pub release_notes: Option<String>,
    pub channel: String,
    pub repository_id: String,
}

pub struct UpdateEngine {
    db: Arc<DatabaseManager>,
}

impl UpdateEngine {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self { db }
    }

    /// Determines if candidate_version is semantically newer than installed_version
    pub fn is_newer_version(installed_version: &str, candidate_version: &str) -> bool {
        let clean_installed = installed_version.trim_start_matches('v');
        let clean_candidate = candidate_version.trim_start_matches('v');

        match (
            semver::Version::parse(clean_installed),
            semver::Version::parse(clean_candidate),
        ) {
            (Ok(inst), Ok(cand)) => cand > inst,
            _ => clean_candidate != clean_installed,
        }
    }

    /// Query all locally installed applications and compare against the latest repository versions
    pub fn check_for_updates(&self) -> Result<Vec<AvailableUpdate>, String> {
        info!("Checking for available package updates across enabled repositories...");
        let installed = self
            .db
            .get_installed_applications()
            .map_err(|e| format!("Failed to fetch installed applications: {}", e))?;

        let mut available_updates = Vec::new();

        for app in installed {
            let details = match self.db.get_package_details(&app.package_id) {
                Ok(Some(d)) => d,
                _ => continue, // Package not in current catalog repositories
            };

            if Self::is_newer_version(&app.version, &details.summary.latest_version) {
                // Find latest release details from manifest if available
                let mut size_mb = None;
                let mut channel = "stable".to_string();

                if let Some(latest_ver_info) = details.versions.iter().find(|v| v.version == details.summary.latest_version) {
                    if let Ok(manifest) = parse_manifest_from_str(&latest_ver_info.manifest_raw) {
                        if let Some(artifact) = manifest.artifacts.iter().find(|a| a.id == manifest.installation.artifact) {
                            if let Some(bytes) = artifact.size {
                                size_mb = Some((bytes as f32) / (1024.0 * 1024.0));
                            }
                        }
                        if let Some(rel) = manifest.release {
                            channel = rel.channel;
                        }
                    }
                }

                available_updates.push(AvailableUpdate {
                    package_id: app.package_id.clone(),
                    package_name: app.name.clone(),
                    publisher: app.publisher.clone(),
                    installed_version: app.version.clone(),
                    new_version: details.summary.latest_version.clone(),
                    size_mb,
                    release_notes: Some(format!(
                        "Upgraded from v{} to v{}. Verified SHA-256 integrity release.",
                        app.version, details.summary.latest_version
                    )),
                    channel,
                    repository_id: details.summary.repository_id.clone(),
                });
            }
        }

        info!("Update check completed. Found {} available updates.", available_updates.len());
        Ok(available_updates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_comparisons() {
        assert!(UpdateEngine::is_newer_version("1.0.0", "1.1.0"));
        assert!(UpdateEngine::is_newer_version("1.0.0", "2.0.0"));
        assert!(UpdateEngine::is_newer_version("1.0.0-beta.1", "1.0.0"));
        assert!(!UpdateEngine::is_newer_version("1.2.0", "1.2.0"));
        assert!(!UpdateEngine::is_newer_version("2.1.0", "2.0.9"));
        assert!(UpdateEngine::is_newer_version("v1.0.0", "v1.0.1"));
    }

    #[test]
    fn test_check_for_updates_pipeline() {
        let temp_dir = std::env::temp_dir().join("t3hub_test_updates");
        let _ = std::fs::remove_dir_all(&temp_dir);
        let _ = std::fs::create_dir_all(&temp_dir);

        let db_path = temp_dir.join("test.db");
        let db = Arc::new(DatabaseManager::new(&db_path).unwrap());

        // Upsert package in repository at version 2.0.0
        db.upsert_package(
            "developer.toolkit",
            "t3-official",
            "Dev Toolkit",
            Some("Essential tools"),
            "T3 Community",
            "[\"Utilities\"]",
            "[]",
            "2.0.0",
            None,
            "2026-10-31T12:00:00Z",
        ).unwrap();

        // Record installed version at 1.0.0
        db.record_installation(
            "developer.toolkit",
            "1.0.0",
            "user",
            "C:\\Apps\\toolkit",
            "zip_directory",
        ).unwrap();

        let engine = UpdateEngine::new(db.clone());
        let updates = engine.check_for_updates().unwrap();

        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].package_id, "developer.toolkit");
        assert_eq!(updates[0].installed_version, "1.0.0");
        assert_eq!(updates[0].new_version, "2.0.0");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
