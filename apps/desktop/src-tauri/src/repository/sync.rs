use super::client::RepositoryClient;
use crate::database::{DatabaseManager, RepositorySyncResult};
use chrono::Utc;
use log::{error, info, warn};

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

    /// Synchronizes a single repository by fetching index.json and latest package manifests.
    pub async fn sync_single_repo(&self, repo_id: &str, repo_url: &str) -> RepositorySyncResult {
        info!("Syncing repository '{}' from '{}'...", repo_id, repo_url);

        // 1. Fetch index.json
        let index = match self.client.fetch_index(repo_url).await {
            Ok(idx) => idx,
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

            // Attempt to fetch and parse the manifest for the latest package version
            if !pkg.manifest_path.is_empty() {
                match self.client.fetch_relative_text(repo_url, &pkg.manifest_path).await {
                    Ok(raw_yaml) => {
                        match crate::manifest::parse_manifest_from_str(&raw_yaml) {
                            Ok(manifest) => {
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
                                warn!("Invalid manifest for package '{}' at '{}': {}", pkg.id, pkg.manifest_path, e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Could not fetch manifest for package '{}' at '{}': {}", pkg.id, pkg.manifest_path, e);
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
