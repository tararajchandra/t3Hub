use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspPackageMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub publisher: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspPlatform {
    pub os: String,
    pub architectures: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_windows_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspRelease {
    #[serde(default = "default_channel")]
    pub channel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
}

fn default_channel() -> String {
    "stable".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspArtifact {
    pub id: String,
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub url: String,
    pub sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspInstallation {
    #[serde(rename = "type")]
    pub install_type: String,
    pub artifact: String,
    #[serde(default = "default_scope")]
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installer_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
}

fn default_scope() -> String {
    "user".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspUninstallation {
    #[serde(rename = "type")]
    pub uninstall_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detect: Option<WspDetect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspDetect {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspUpdates {
    #[serde(default = "default_update_strategy")]
    pub strategy: String,
    #[serde(default = "default_channel")]
    pub channel: String,
}

fn default_update_strategy() -> String {
    "version".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspManifest {
    pub schema_version: String,
    pub package: WspPackageMetadata,
    pub platform: WspPlatform,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<WspRelease>,
    pub artifacts: Vec<WspArtifact>,
    pub installation: WspInstallation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uninstallation: Option<WspUninstallation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updates: Option<WspUpdates>,
}

/// Catalog index models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryIndex {
    pub repository_id: String,
    pub version: u32,
    pub generated_at: String,
    pub packages: Vec<IndexPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexPackage {
    pub id: String,
    pub name: String,
    pub publisher: String,
    pub latest_version: String,
    pub description: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub icon_url: Option<String>,
    pub manifest_path: String,
    pub versions: Vec<IndexVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexVersion {
    pub version: String,
    pub manifest_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
}
