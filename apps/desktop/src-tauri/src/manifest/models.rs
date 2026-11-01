use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspPackageMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub publisher: String,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspPlatform {
    pub os: String, // Must be "windows"
    pub architectures: Vec<String>, // ["x64", "x86", "arm64"]
    pub minimum_windows_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspRelease {
    #[serde(default = "default_channel")]
    pub channel: String, // "stable", "beta", "nightly"
    pub release_date: Option<String>,
}

fn default_channel() -> String {
    "stable".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspArtifact {
    pub id: String,
    #[serde(rename = "type")]
    pub artifact_type: String, // "exe", "msi", "zip"
    pub url: String,
    pub sha256: String,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspInstallation {
    #[serde(rename = "type")]
    pub install_type: String, // "exe", "msi", "zip"
    pub artifact: String,
    #[serde(default = "default_scope")]
    pub scope: String, // "user", "machine"
    pub installer_profile: Option<String>, // "inno", "nsis", "wix", "custom"
    #[serde(default)]
    pub arguments: Vec<String>,
    pub timeout_seconds: Option<u64>,
}

fn default_scope() -> String {
    "user".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspUninstallation {
    #[serde(rename = "type")]
    pub uninstall_type: String, // "registry", "zip_directory", "msi"
    pub detect: Option<WspDetect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspDetect {
    pub registry_path: Option<String>,
    pub directory_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WspUpdates {
    #[serde(default = "default_update_strategy")]
    pub strategy: String, // "version", "git_tag", "manifest"
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
    #[serde(default)]
    pub release: Option<WspRelease>,
    pub artifacts: Vec<WspArtifact>,
    pub installation: WspInstallation,
    pub uninstallation: Option<WspUninstallation>,
    pub updates: Option<WspUpdates>,
}
