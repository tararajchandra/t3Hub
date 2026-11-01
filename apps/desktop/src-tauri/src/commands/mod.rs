use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Emitter, State};

use crate::database::models::{
    CatalogPackageDetails, CatalogPackageSummary, DatabaseStats, InstalledAppRecord,
    OperationRecord, RepositoryRecord, RepositorySyncResult,
};
use crate::database::DatabaseManager;
use crate::installer::{InstallationEngine, InstallationResult};
use crate::manifest::{parse_manifest_from_str, WspManifest};
use crate::repository::RepositorySynchronizer;

pub struct AppState {
    pub db: Arc<DatabaseManager>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub app_name: String,
    pub app_version: String,
    pub tauri_version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgressPayload {
    pub package_id: String,
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub percentage: f32,
}

#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        app_name: "T3 Hub".to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        tauri_version: tauri::VERSION.to_string(),
    }
}

#[tauri::command]
pub fn get_database_status(state: State<'_, AppState>) -> Result<DatabaseStats, String> {
    state
        .db
        .get_database_stats()
        .map_err(|e| format!("Failed to retrieve database status: {e}"))
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    state
        .db
        .get_settings()
        .map_err(|e| format!("Failed to retrieve settings: {e}"))
}

#[tauri::command]
pub fn update_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    state
        .db
        .set_setting(&key, &value)
        .map_err(|e| format!("Failed to update setting '{key}': {e}"))
}

#[tauri::command]
pub fn get_repositories(state: State<'_, AppState>) -> Result<Vec<RepositoryRecord>, String> {
    state
        .db
        .get_repositories()
        .map_err(|e| format!("Failed to retrieve repositories: {e}"))
}

#[tauri::command]
pub fn add_repository(
    state: State<'_, AppState>,
    id: String,
    name: String,
    url: String,
    repo_type: Option<String>,
) -> Result<(), String> {
    let t = repo_type.unwrap_or_else(|| "community".to_string());
    state
        .db
        .add_or_update_repository(&id, &name, &url, &t, true, "user-approved")
        .map_err(|e| format!("Failed to add repository: {e}"))
}

#[tauri::command]
pub fn get_installed_applications(
    state: State<'_, AppState>,
) -> Result<Vec<InstalledAppRecord>, String> {
    state
        .db
        .get_installed_applications()
        .map_err(|e| format!("Failed to retrieve installed applications: {e}"))
}

#[tauri::command]
pub fn get_operations_log(state: State<'_, AppState>) -> Result<Vec<OperationRecord>, String> {
    state
        .db
        .get_operations()
        .map_err(|e| format!("Failed to retrieve operations log: {e}"))
}

#[tauri::command]
pub fn search_catalog(
    state: State<'_, AppState>,
    query: Option<String>,
    category: Option<String>,
    repository_id: Option<String>,
) -> Result<Vec<CatalogPackageSummary>, String> {
    state
        .db
        .search_packages(
            query.as_deref(),
            category.as_deref(),
            repository_id.as_deref(),
        )
        .map_err(|e| format!("Catalog search failed: {e}"))
}

#[tauri::command]
pub fn get_package_details(
    state: State<'_, AppState>,
    package_id: String,
) -> Result<Option<CatalogPackageDetails>, String> {
    state
        .db
        .get_package_details(&package_id)
        .map_err(|e| format!("Failed to fetch package details for '{package_id}': {e}"))
}

#[tauri::command]
pub async fn sync_repositories(
    state: State<'_, AppState>,
) -> Result<Vec<RepositorySyncResult>, String> {
    let synchronizer = RepositorySynchronizer::new(&state.db);
    Ok(synchronizer.sync_all().await)
}

#[tauri::command]
pub fn validate_manifest_string(yaml_content: String) -> Result<WspManifest, Vec<String>> {
    match parse_manifest_from_str(&yaml_content) {
        Ok(m) => Ok(m),
        Err(crate::manifest::ManifestError::Validation(errs)) => Err(errs),
        Err(e) => Err(vec![e.to_string()]),
    }
}

#[tauri::command]
pub async fn install_package(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    package_id: String,
    version: Option<String>,
) -> Result<InstallationResult, String> {
    let engine = InstallationEngine::default_paths(&state.db);
    let pkg_id_clone = package_id.clone();
    let app_clone = app.clone();

    let res = engine
        .install(
            &package_id,
            version.as_deref(),
            None,
            move |prog| {
                let _ = app_clone.emit(
                    "download-progress",
                    DownloadProgressPayload {
                        package_id: pkg_id_clone.clone(),
                        bytes_downloaded: prog.bytes_downloaded,
                        total_bytes: prog.total_bytes,
                        percentage: prog.percentage,
                    },
                );
            },
        )
        .await;

    Ok(res)
}

#[tauri::command]
pub fn uninstall_package(
    state: State<'_, AppState>,
    package_id: String,
) -> Result<crate::uninstall::UninstallResult, String> {
    let engine = crate::uninstall::UninstallEngine::default_engine(state.db.clone());
    engine.uninstall_package(&package_id)
}

#[tauri::command]
pub fn get_detected_installed_apps() -> Result<Vec<crate::installer::InstalledAppInfo>, String> {
    Ok(crate::installer::AppDetector::detect_all_installed_apps())
}

#[tauri::command]
pub fn check_updates(state: State<'_, AppState>) -> Result<Vec<crate::updater::AvailableUpdate>, String> {
    let engine = crate::updater::UpdateEngine::new(state.db.clone());
    engine.check_for_updates()
}

#[tauri::command]
pub fn toggle_repository(
    state: State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    state
        .db
        .toggle_repository(&id, enabled)
        .map_err(|e| format!("Failed to update repository status: {}", e))
}

#[tauri::command]
pub fn remove_repository(
    state: State<'_, AppState>,
    id: String,
) -> Result<bool, String> {
    state
        .db
        .remove_repository(&id)
        .map_err(|e| format!("Failed to remove repository: {}", e))
}

#[tauri::command]
pub fn update_repository_trust(
    state: State<'_, AppState>,
    id: String,
    trust_policy: String,
) -> Result<(), String> {
    state
        .db
        .update_repository_trust_policy(&id, &trust_policy)
        .map_err(|e| format!("Failed to update repository trust policy: {}", e))
}

#[tauri::command]
pub fn verify_signature(
    public_key_hex: String,
    message: String,
    signature_hex: String,
) -> Result<bool, String> {
    match crate::security::SignatureVerifier::verify(&public_key_hex, message.as_bytes(), &signature_hex) {
        Ok(()) => Ok(true),
        Err(e) => Err(format!("Signature verification error: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_system_info() {
        let info = get_system_info();
        assert_eq!(info.app_name, "T3 Hub");
        assert_eq!(info.app_version, "0.1.0");
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
    }
}
