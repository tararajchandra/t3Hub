use super::models::{RepositoryIndex, RepositoryMetadata};
use crate::manifest::{parse_manifest_from_str, ManifestError, WspManifest};
use reqwest::Client;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Network request failed: {0}")]
    Network(String),

    #[error("Local filesystem read failed: {0}")]
    Io(String),

    #[error("JSON deserialization error: {0}")]
    Json(String),

    #[error("Invalid URL: {0}")]
    Url(String),

    #[error("Manifest error: {0}")]
    Manifest(#[from] ManifestError),
}

pub struct RepositoryClient {
    http_client: Client,
}

impl Default for RepositoryClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RepositoryClient {
    pub fn new() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("T3Hub-Desktop/1.0 (Windows NT)")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { http_client }
    }

    /// Fetches the repository.json metadata for a given repository base URL or local directory.
    pub async fn fetch_metadata(&self, base_url: &str) -> Result<RepositoryMetadata, RepositoryError> {
        let content = self.fetch_relative_text(base_url, "repository.json").await?;
        serde_json::from_str(&content).map_err(|e| RepositoryError::Json(e.to_string()))
    }

    /// Fetches the index.json package catalog for a given repository base URL or local directory.
    pub async fn fetch_index(&self, base_url: &str) -> Result<RepositoryIndex, RepositoryError> {
        let content = self.fetch_relative_text(base_url, "index.json").await?;
        serde_json::from_str(&content).map_err(|e| RepositoryError::Json(e.to_string()))
    }

    /// Fetches and parses a package manifest from a relative or absolute manifest path.
    pub async fn fetch_manifest(&self, base_url: &str, relative_path: &str) -> Result<WspManifest, RepositoryError> {
        let yaml_content = self.fetch_relative_text(base_url, relative_path).await?;
        parse_manifest_from_str(&yaml_content).map_err(RepositoryError::Manifest)
    }

    /// Resolves and fetches a text file (JSON or YAML) from either a remote HTTP(S) URL or a local path / file:// URL.
    pub async fn fetch_relative_text(&self, base_url: &str, relative_path: &str) -> Result<String, RepositoryError> {
        let base_clean = base_url.trim();

        // Check if base is a URL or local file path
        if base_clean.starts_with("http://") || base_clean.starts_with("https://") {
            let mut url = Url::parse(base_clean).map_err(|e| RepositoryError::Url(e.to_string()))?;
            // Ensure trailing slash on base URL before joining relative path
            if !url.path().ends_with('/') {
                let new_path = format!("{}/", url.path());
                url.set_path(&new_path);
            }
            let target_url = url.join(relative_path).map_err(|e| RepositoryError::Url(e.to_string()))?;

            let resp = self.http_client.get(target_url)
                .send()
                .await
                .map_err(|e| RepositoryError::Network(e.to_string()))?;

            if !resp.status().is_success() {
                return Err(RepositoryError::Network(format!(
                    "HTTP request returned status {}",
                    resp.status()
                )));
            }

            resp.text().await.map_err(|e| RepositoryError::Network(e.to_string()))
        } else {
            // Local filesystem path or file:// URL
            let local_path = if base_clean.starts_with("file://") {
                let parsed = Url::parse(base_clean).map_err(|e| RepositoryError::Url(e.to_string()))?;
                parsed.to_file_path().map_err(|_| RepositoryError::Io("Invalid file:// URL path".to_string()))?
            } else {
                PathBuf::from(base_clean)
            };

            let joined_path = local_path.join(relative_path.trim_start_matches('/'));
            fs::read_to_string(&joined_path).map_err(|e| {
                RepositoryError::Io(format!("Failed to read {}: {}", joined_path.display(), e))
            })
        }
    }
}
