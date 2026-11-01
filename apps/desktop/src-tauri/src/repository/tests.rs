#[cfg(test)]
mod tests {
    use crate::database::DatabaseManager;
    use crate::repository::client::RepositoryClient;
    use crate::repository::sync::RepositorySynchronizer;
    use std::path::PathBuf;

    fn get_official_repo_dir() -> PathBuf {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        // manifest_dir is <project_root>/apps/desktop/src-tauri
        // Repo is located at <project_root>/repositories/official
        manifest_dir
            .parent().unwrap() // apps/desktop
            .parent().unwrap() // apps
            .parent().unwrap() // <project_root>
            .join("repositories")
            .join("official")
    }

    #[tokio::test]
    async fn test_repository_local_read() {
        let repo_dir = get_official_repo_dir();
        assert!(repo_dir.exists(), "Official repository directory should exist at {:?}", repo_dir);

        let client = RepositoryClient::new();
        let base_url = repo_dir.to_str().unwrap();

        // 1. Test metadata read
        let meta = client.fetch_metadata(base_url).await.expect("Failed to read repository.json");
        assert_eq!(meta.id, "t3-official");
        assert_eq!(meta.repo_type, "official");

        // 2. Test index read
        let index = client.fetch_index(base_url).await.expect("Failed to read index.json");
        assert_eq!(index.repository_id, "t3-official");
        assert!(index.packages.len() >= 4, "Should have at least 4 test packages");

        // 3. Test manifest fetch and validation
        let devstudio_pkg = index.packages.iter()
            .find(|p| p.id == "developer.devstudio")
            .expect("developer.devstudio should exist in index");
        let manifest = client.fetch_manifest(base_url, &devstudio_pkg.manifest_path).await
            .expect("Failed to fetch and parse manifest");

        assert_eq!(manifest.package.id, devstudio_pkg.id);
        assert_eq!(manifest.schema_version, "1.0");
        assert_eq!(manifest.platform.os, "windows");
        assert_eq!(manifest.installation.artifact, "windows-x64-installer");
    }

    #[tokio::test]
    async fn test_repository_sync_into_sqlite() {
        let temp_dir = std::env::temp_dir().join(format!("t3hub_sync_test_{}", uuid::Uuid::new_v4()));
        let db_path = temp_dir.join("test_sync.db");

        let db = DatabaseManager::new(&db_path).expect("Failed to create test DB");

        // Point official repo in SQLite to the local filesystem mock directory
        let repo_dir = get_official_repo_dir();
        db.add_or_update_repository(
            "t3-official",
            "T3 Official Repository",
            repo_dir.to_str().unwrap(),
            "official",
            true,
            "verified",
        ).expect("Failed to update official repo URL");

        let synchronizer = RepositorySynchronizer::new(&db);
        let results = synchronizer.sync_all().await;

        assert_eq!(results.len(), 1);
        assert!(results[0].success, "Sync should succeed: {:?}", results[0].error_message);
        assert!(results[0].packages_synced >= 4);

        // Search catalog from SQLite
        let devstudio_results = db.search_packages(Some("DevStudio"), None, None)
            .expect("Catalog search should succeed");
        assert_eq!(devstudio_results.len(), 1);
        assert_eq!(devstudio_results[0].name, "DevStudio");
        assert_eq!(devstudio_results[0].publisher, "T3 Community");

        // Category filter search
        let dev_results = db.search_packages(None, Some("Developer Tools"), None)
            .expect("Category filter should succeed");
        assert!(dev_results.iter().any(|p| p.id == "tools.terminal-x"));

        // Get full details
        let details = db.get_package_details("developer.devstudio")
            .expect("Package details query failed")
            .expect("Details should exist");
        assert_eq!(details.versions.len(), 1);
        assert_eq!(details.versions[0].version, "1.2.0");

        // Clean up
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
