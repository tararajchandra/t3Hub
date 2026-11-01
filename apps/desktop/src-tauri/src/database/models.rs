use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryRecord {
    pub id: String,
    pub name: String,
    pub url: String,
    pub repo_type: String,
    pub enabled: bool,
    pub trust_policy: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingRecord {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub schema_version: u32,
    pub db_path: String,
    pub repositories_count: i64,
    pub packages_count: i64,
    pub installations_count: i64,
    pub operations_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledAppRecord {
    pub id: String,
    pub package_id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub install_scope: String,
    pub install_path: Option<String>,
    pub installed_at: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRecord {
    pub id: String,
    pub package_id: String,
    pub operation_type: String,
    pub target_version: String,
    pub status: String,
    pub logs: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogPackageSummary {
    pub id: String,
    pub repository_id: String,
    pub repository_name: String,
    pub name: String,
    pub description: Option<String>,
    pub publisher: String,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub latest_version: String,
    pub icon_url: Option<String>,
    pub installed_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageVersionDetail {
    pub version: String,
    pub release_channel: String,
    pub release_date: Option<String>,
    pub manifest_raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogPackageDetails {
    pub summary: CatalogPackageSummary,
    pub versions: Vec<PackageVersionDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySyncResult {
    pub repository_id: String,
    pub packages_synced: usize,
    pub success: bool,
    pub error_message: Option<String>,
}
