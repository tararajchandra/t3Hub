use super::client::RepositoryClient;
use super::models::{RepositoryIndex, RepositoryMetadata};
use crate::database::{DatabaseManager, RepositorySyncResult};
use crate::security::SignatureVerifier;
use chrono::Utc;
use log::{error, info, warn};
use std::collections::HashSet;

pub struct RepositorySynchronizer<'a> {
    db: &'a DatabaseManager,
    client: RepositoryClient,
}

impl<'a> RepositorySynchronizer<'a> {
    pub fn new(db: &'a DatabaseManager) -> Self {
        Self {
            db,
            client: RepositoryClient::new(),
        }
    }

    /// Synchronizes all enabled repositories from the database.
    pub async fn sync_all(&self) -> Vec<RepositorySyncResult> {
        let repos = match self.db.get_repositories() {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to fetch repositories from database: {}", e);
                return vec![];
            }
        };

        let mut results = Vec::new();
        for repo in repos {
            if !repo.enabled {
                continue;
            }
            let res = self.sync_single_repo(&repo.id, &repo.url).await;
            results.push(res);
        }

        results
    }

    /// Synchronizes a single repository by fetching repository metadata, verifying optional
    /// Ed25519 detached signatures over index.json, and syncing package manifests.
    pub async fn sync_single_repo(&self, repo_id: &str, repo_url: &str) -> RepositorySyncResult {
        info!("Syncing repository '{}' from '{}'...", repo_id, repo_url);

        // 1. Fetch raw index.json text so we can both verify its signature and deserialize it
        let index_raw = match self.client.fetch_relative_text(repo_url, "index.json").await {
            Ok(txt) => txt,
            Err(e) => {
                let err_msg = format!("Failed to fetch index.json from {}: {}", repo_url, e);
                error!("{}", err_msg);
                return RepositorySyncResult {
                    repository_id: repo_id.to_string(),
                    packages_synced: 0,
                    success: false,
                    error_message: Some(err_msg),
                };
            }
        };

        // 2. Check repository.json for public_key and verify index.json.sig / index.sig if present
        if let Ok(meta_raw) = self.client.fetch_relative_text(repo_url, "repository.json").await {
            if let Ok(meta) = serde_json::from_str::<RepositoryMetadata>(&meta_raw) {
                if let Some(ref pubkey) = meta.public_key {
                    let sig_text = match self.client.fetch_relative_text(repo_url, "index.json.sig").await {
                        Ok(s) => Some(s),
                        Err(_) => self.client.fetch_relative_text(repo_url, "index.sig").await.ok(),
                    };

                    if let Some(sig_hex) = sig_text {
                        if let Err(e) = SignatureVerifier::verify(pubkey, index_raw.as_bytes(), sig_hex.trim()) {
                            let err_msg = format!(
                                "Cryptographic Ed25519 signature verification failed for repository '{}': {}",
                                repo_id, e
                            );
                            error!("{}", err_msg);
                            return RepositorySyncResult {
                                repository_id: repo_id.to_string(),
                                packages_synced: 0,
                                success: false,
                                error_message: Some(err_msg),
                            };
                        }
                        info!("Verified Ed25519 detached signature for repository '{}'", repo_id);
                    }
                }
            }
        }

        let index: RepositoryIndex = match serde_json::from_str(&index_raw) {
            Ok(idx) => idx,
            Err(e) => {
                let err_msg = format!("Failed to parse index.json from {}: {}", repo_url, e);
                error!("{}", err_msg);
                return RepositorySyncResult {
                    repository_id: repo_id.to_string(),
                    packages_synced: 0,
                    success: false,
                    error_message: Some(err_msg),
                };
            }
        };

        let mut count = 0;
        let now = Utc::now().to_rfc3339();

        for pkg in &index.packages {
            let categories_json = serde_json::to_string(&pkg.categories).unwrap_or_else(|_| "[]".to_string());
            let tags_json = serde_json::to_string(&pkg.tags).unwrap_or_else(|_| "[]".to_string());

            // Upsert package summary in DB
            if let Err(e) = self.db.upsert_package(
                &pkg.id,
                repo_id,
                &pkg.name,
                pkg.description.as_deref(),
                &pkg.publisher,
                &categories_json,
                &tags_json,
                &pkg.latest_version,
                pkg.icon_url.as_deref(),
                &now,
            ) {
                warn!("Failed to upsert package '{}': {}", pkg.id, e);
                continue;
            }

            // Collect all unique manifest paths for this package (latest + historical versions)
            let mut manifest_paths = Vec::new();
            let mut seen_paths = HashSet::new();
            if !pkg.manifest_path.is_empty() && seen_paths.insert(pkg.manifest_path.clone()) {
                manifest_paths.push(pkg.manifest_path.clone());
            }
            for v in &pkg.versions {
                if !v.manifest_path.is_empty() && seen_paths.insert(v.manifest_path.clone()) {
                    manifest_paths.push(v.manifest_path.clone());
                }
            }

            for m_path in manifest_paths {
                match self.client.fetch_relative_text(repo_url, &m_path).await {
                    Ok(raw_yaml) => {
                        match crate::manifest::parse_manifest_from_str(&raw_yaml) {
                            Ok(manifest) => {
                                // Update license, homepage, and repository_url from the latest manifest
                                if manifest.package.version == pkg.latest_version {
                                    let _ = self.db.update_package_extended_metadata(
                                        &pkg.id,
                                        manifest.package.license.as_deref(),
                                        manifest.package.homepage.as_deref(),
                                        manifest.package.repository.as_deref(),
                                    );
                                }

                                let version_id = format!("{}:{}", pkg.id, manifest.package.version);
                                let channel = manifest.release.as_ref()
                                    .map(|r| r.channel.as_str())
                                    .unwrap_or("stable");
                                let release_date = manifest.release.as_ref()
                                    .and_then(|r| r.release_date.as_deref());

                                if let Err(e) = self.db.upsert_package_version(
                                    &version_id,
                                    &pkg.id,
                                    &manifest.package.version,
                                    channel,
                                    release_date,
                                    &raw_yaml,
                                    &now,
                                ) {
                                    warn!("Failed to save package version for {}: {}", pkg.id, e);
                                }

                                // Cache artifacts metadata
                                for art in &manifest.artifacts {
                                    let art_db_id = format!("{}:{}", version_id, art.id);
                                    let _ = self.db.upsert_artifact(
                                        &art_db_id,
                                        &version_id,
                                        &art.id,
                                        &art.artifact_type,
                                        &art.url,
                                        &art.sha256,
                                        art.size.map(|s| s as i64),
                                    );
                                }
                            }
                            Err(e) => {
                                warn!("Invalid manifest for package '{}' at '{}': {}", pkg.id, m_path, e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Could not fetch manifest for package '{}' at '{}': {}", pkg.id, m_path, e);
                    }
                }
            }

            count += 1;
        }

        info!("Successfully synced {} packages from repository '{}'", count, repo_id);

        RepositorySyncResult {
            repository_id: repo_id.to_string(),
            packages_synced: count,
            success: true,
            error_message: None,
        }
    }
}
