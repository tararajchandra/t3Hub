use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryMetadata {
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(rename = "type")]
    pub repo_type: String, // "official" | "community"
    #[serde(default = "default_repo_version")]
    pub version: u32,
    pub description: Option<String>,
    pub public_key: Option<String>,
    #[serde(default = "default_trust_policy")]
    pub trust_policy: String, // "verified" | "user-approved" | "untrusted"
    pub updated_at: Option<String>,
}

fn default_repo_version() -> u32 {
    1
}

fn default_trust_policy() -> String {
    "user-approved".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryIndexVersion {
    pub version: String,
    pub manifest_path: String,
    pub release_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryIndexPackage {
    pub id: String,
    pub name: String,
    pub publisher: String,
    pub latest_version: String,
    pub description: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub icon_url: Option<String>,
    pub manifest_path: String,
    #[serde(default)]
    pub versions: Vec<RepositoryIndexVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryIndex {
    pub repository_id: String,
    #[serde(default = "default_repo_version")]
    pub version: u32,
    pub generated_at: Option<String>,
    #[serde(default)]
    pub packages: Vec<RepositoryIndexPackage>,
}
