pub mod models;

use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub use self::models::{
    CatalogPackageDetails, CatalogPackageSummary, DatabaseStats, InstalledAppRecord,
    OperationRecord, PackageVersionDetail, RepositoryRecord, RepositorySyncResult, SettingRecord,
};

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Migration error: {0}")]
    Migration(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub struct DatabaseManager {
    conn: Mutex<Connection>,
    db_path: PathBuf,
}

impl DatabaseManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, DatabaseError> {
        let db_path = path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             PRAGMA synchronous = NORMAL;",
        )?;

        let manager = Self {
            conn: Mutex::new(conn),
            db_path,
        };

        manager.migrate()?;
        Ok(manager)
    }

    fn migrate(&self) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();

        let mut version: u32 = conn.query_row("PRAGMA user_version;", [], |row| row.get(0))?;

        if version < 1 {
            log::info!("Applying SQLite migration 1: Initial schema");

            conn.execute_batch(
                "BEGIN TRANSACTION;

                -- Repositories table
                CREATE TABLE IF NOT EXISTS repositories (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    url TEXT NOT NULL UNIQUE,
                    repo_type TEXT NOT NULL DEFAULT 'community',
                    enabled INTEGER NOT NULL DEFAULT 1,
                    trust_policy TEXT NOT NULL DEFAULT 'user-approved',
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                -- Packages cached metadata
                CREATE TABLE IF NOT EXISTS packages (
                    id TEXT PRIMARY KEY,
                    repository_id TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                    name TEXT NOT NULL,
                    description TEXT,
                    publisher TEXT NOT NULL,
                    license TEXT,
                    homepage TEXT,
                    repository_url TEXT,
                    categories TEXT, -- JSON array
                    tags TEXT,       -- JSON array
                    latest_version TEXT NOT NULL DEFAULT '1.0.0',
                    icon_url TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                -- Package versions
                CREATE TABLE IF NOT EXISTS package_versions (
                    id TEXT PRIMARY KEY,
                    package_id TEXT NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
                    version TEXT NOT NULL,
                    release_channel TEXT NOT NULL DEFAULT 'stable',
                    release_date TEXT,
                    manifest_raw TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    UNIQUE(package_id, version)
                );

                -- Installations (local inventory)
                CREATE TABLE IF NOT EXISTS installations (
                    id TEXT PRIMARY KEY,
                    package_id TEXT NOT NULL,
                    version TEXT NOT NULL,
                    install_scope TEXT NOT NULL DEFAULT 'user',
                    install_path TEXT,
                    installed_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    uninstall_type TEXT,
                    uninstall_string TEXT,
                    status TEXT NOT NULL DEFAULT 'installed'
                );

                -- Artifacts metadata & cache
                CREATE TABLE IF NOT EXISTS artifacts (
                    id TEXT PRIMARY KEY,
                    package_version_id TEXT NOT NULL REFERENCES package_versions(id) ON DELETE CASCADE,
                    artifact_id TEXT NOT NULL,
                    artifact_type TEXT NOT NULL, -- exe, msi, zip
                    url TEXT NOT NULL,
                    sha256 TEXT NOT NULL,
                    size_bytes INTEGER,
                    cached_path TEXT
                );

                -- Downloads history & active tasks
                CREATE TABLE IF NOT EXISTS downloads (
                    id TEXT PRIMARY KEY,
                    artifact_id TEXT NOT NULL,
                    status TEXT NOT NULL, -- queued, downloading, completed, failed
                    bytes_downloaded INTEGER NOT NULL DEFAULT 0,
                    total_bytes INTEGER,
                    destination_path TEXT,
                    started_at TEXT NOT NULL,
                    completed_at TEXT,
                    error_message TEXT
                );

                -- Operations history (install, update, uninstall)
                CREATE TABLE IF NOT EXISTS operations (
                    id TEXT PRIMARY KEY,
                    package_id TEXT NOT NULL,
                    operation_type TEXT NOT NULL, -- install, update, uninstall
                    target_version TEXT NOT NULL,
                    status TEXT NOT NULL, -- pending, in_progress, success, failed
                    logs TEXT,
                    created_at TEXT NOT NULL,
                    completed_at TEXT
                );

                -- Trusted publishers
                CREATE TABLE IF NOT EXISTS trusted_publishers (
                    id TEXT PRIMARY KEY,
                    publisher_name TEXT NOT NULL UNIQUE,
                    fingerprint TEXT,
                    trust_level TEXT NOT NULL DEFAULT 'standard',
                    notes TEXT,
                    created_at TEXT NOT NULL
                );

                -- Trusted repository public keys
                CREATE TABLE IF NOT EXISTS trusted_repository_keys (
                    id TEXT PRIMARY KEY,
                    repository_id TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                    public_key TEXT NOT NULL,
                    algorithm TEXT NOT NULL DEFAULT 'ed25519',
                    valid_until TEXT,
                    created_at TEXT NOT NULL
                );

                -- Application settings
                CREATE TABLE IF NOT EXISTS settings (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                PRAGMA user_version = 1;

                COMMIT;",
            )?;

            // Seed default configuration
            let now = Utc::now().to_rfc3339();

            let default_settings = [
                ("theme", "system"),
                ("cache_limit_gb", "10"),
                ("auto_check_updates", "true"),
                ("require_signatures", "true"),
                ("default_install_scope", "user"),
                ("download_directory", ""),
                ("telemetry", "false"),
            ];

            for (k, v) in default_settings {
                conn.execute(
                    "INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3);",
                    params![k, v, now],
                )?;
            }

            // Seed official repository definition
            conn.execute(
                "INSERT OR IGNORE INTO repositories (id, name, url, repo_type, enabled, trust_policy, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
                params![
                    "t3-official",
                    "T3 Official Repository",
                    "https://repo.t3hub.dev/official/",
                    "official",
                    1,
                    "verified",
                    now,
                    now
                ],
            )?;

            version = 1;
        }

        if version < 2 {
            log::info!("Applying SQLite migration 2: Ensuring packages.latest_version exists");
            // Check if column exists to avoid duplicate column error if created fresh in v1
            let has_col: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM pragma_table_info('packages') WHERE name = 'latest_version';",
                    [],
                    |row| Ok(row.get::<_, i64>(0)? > 0),
                )
                .unwrap_or(false);

            if !has_col {
                conn.execute(
                    "ALTER TABLE packages ADD COLUMN latest_version TEXT NOT NULL DEFAULT '1.0.0';",
                    [],
                )?;
            }

            conn.execute("PRAGMA user_version = 2;", [])?;
            version = 2;
        }

        log::info!("Database schema is up to date (version {})", version);
        Ok(())
    }

    pub fn get_settings(&self) -> Result<HashMap<String, String>, DatabaseError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT key, value FROM settings;")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut map = HashMap::new();
        for item in rows {
            let (k, v) = item?;
            map.insert(k, v);
        }
        Ok(map)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at;",
            params![key, value, now],
        )?;
        Ok(())
    }

    pub fn get_repositories(&self) -> Result<Vec<RepositoryRecord>, DatabaseError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, url, repo_type, enabled, trust_policy, created_at, updated_at FROM repositories ORDER BY repo_type ASC, name ASC;"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(RepositoryRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                repo_type: row.get(3)?,
                enabled: row.get::<_, i64>(4)? == 1,
                trust_policy: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        let mut list = Vec::new();
        for item in rows {
            list.push(item?);
        }
        Ok(list)
    }

    pub fn add_or_update_repository(
        &self,
        id: &str,
        name: &str,
        url: &str,
        repo_type: &str,
        enabled: bool,
        trust_policy: &str,
    ) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO repositories (id, name, url, repo_type, enabled, trust_policy, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                url = excluded.url,
                repo_type = excluded.repo_type,
                enabled = excluded.enabled,
                trust_policy = excluded.trust_policy,
                updated_at = excluded.updated_at;",
            params![id, name, url, repo_type, if enabled { 1 } else { 0 }, trust_policy, now, now],
        )?;
        Ok(())
    }

    pub fn toggle_repository(&self, id: &str, enabled: bool) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE repositories SET enabled = ?1, updated_at = ?2 WHERE id = ?3;",
            params![if enabled { 1 } else { 0 }, now, id],
        )?;
        Ok(())
    }

    pub fn update_repository_trust_policy(&self, id: &str, trust_policy: &str) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE repositories SET trust_policy = ?1, updated_at = ?2 WHERE id = ?3;",
            params![trust_policy, now, id],
        )?;
        Ok(())
    }

    pub fn remove_repository(&self, id: &str) -> Result<bool, DatabaseError> {
        let conn = self.conn.lock();
        let affected = conn.execute(
            "DELETE FROM repositories WHERE id = ?1;",
            params![id],
        )?;
        Ok(affected > 0)
    }

    pub fn upsert_package(
        &self,
        id: &str,
        repository_id: &str,
        name: &str,
        description: Option<&str>,
        publisher: &str,
        categories_json: &str,
        tags_json: &str,
        latest_version: &str,
        icon_url: Option<&str>,
        timestamp: &str,
    ) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO packages (id, repository_id, name, description, publisher, categories, tags, latest_version, icon_url, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
                repository_id = excluded.repository_id,
                name = excluded.name,
                description = excluded.description,
                publisher = excluded.publisher,
                categories = excluded.categories,
                tags = excluded.tags,
                latest_version = excluded.latest_version,
                icon_url = excluded.icon_url,
                updated_at = excluded.updated_at;",
            params![
                id,
                repository_id,
                name,
                description,
                publisher,
                categories_json,
                tags_json,
                latest_version,
                icon_url,
                timestamp,
                timestamp
            ],
        )?;
        Ok(())
    }

    pub fn upsert_package_version(
        &self,
        id: &str,
        package_id: &str,
        version: &str,
        channel: &str,
        release_date: Option<&str>,
        manifest_raw: &str,
        timestamp: &str,
    ) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO package_versions (id, package_id, version, release_channel, release_date, manifest_raw, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(package_id, version) DO UPDATE SET
                release_channel = excluded.release_channel,
                release_date = excluded.release_date,
                manifest_raw = excluded.manifest_raw;",
            params![id, package_id, version, channel, release_date, manifest_raw, timestamp],
        )?;
        Ok(())
    }

    pub fn upsert_artifact(
        &self,
        id: &str,
        package_version_id: &str,
        artifact_id: &str,
        artifact_type: &str,
        url: &str,
        sha256: &str,
        size_bytes: Option<i64>,
    ) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO artifacts (id, package_version_id, artifact_id, artifact_type, url, sha256, size_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                url = excluded.url,
                sha256 = excluded.sha256,
                size_bytes = excluded.size_bytes;",
            params![id, package_version_id, artifact_id, artifact_type, url, sha256, size_bytes],
        )?;
        Ok(())
    }

    pub fn search_packages(
        &self,
        query: Option<&str>,
        category: Option<&str>,
        repository_id: Option<&str>,
    ) -> Result<Vec<CatalogPackageSummary>, DatabaseError> {
        let conn = self.conn.lock();

        let query_pattern = query
            .map(|q| q.trim())
            .filter(|q| !q.is_empty())
            .map(|q| format!("%{}%", q));

        let category_pattern = category
            .map(|c| c.trim())
            .filter(|c| !c.is_empty() && *c != "All")
            .map(|c| format!("%\"{}\"%", c));

        let repo_filter = repository_id
            .map(|r| r.trim())
            .filter(|r| !r.is_empty() && *r != "all");

        let sql = "
            SELECT 
                p.id, p.repository_id, r.name as repository_name, p.name, p.description, 
                p.publisher, p.license, p.homepage, p.categories, p.tags, 
                p.latest_version, p.icon_url,
                i.version as installed_version
            FROM packages p
            JOIN repositories r ON p.repository_id = r.id
            LEFT JOIN installations i ON p.id = i.package_id
            WHERE r.enabled = 1
              AND (?1 IS NULL OR p.name LIKE ?1 OR p.id LIKE ?1 OR p.description LIKE ?1 OR p.publisher LIKE ?1 OR p.tags LIKE ?1)
              AND (?2 IS NULL OR p.categories LIKE ?2)
              AND (?3 IS NULL OR p.repository_id = ?3)
            ORDER BY p.name ASC;";

        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(
            params![query_pattern, category_pattern, repo_filter],
            |row| {
                let categories_str: Option<String> = row.get(8)?;
                let tags_str: Option<String> = row.get(9)?;

                let categories: Vec<String> = categories_str
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or_default();

                let tags: Vec<String> = tags_str
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or_default();

                Ok(CatalogPackageSummary {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    repository_name: row.get(2)?,
                    name: row.get(3)?,
                    description: row.get(4)?,
                    publisher: row.get(5)?,
                    license: row.get(6)?,
                    homepage: row.get(7)?,
                    categories,
                    tags,
                    latest_version: row.get(10)?,
                    icon_url: row.get(11)?,
                    installed_version: row.get(12)?,
                })
            },
        )?;

        let mut results = Vec::new();
        for item in rows {
            results.push(item?);
        }
        Ok(results)
    }

    pub fn get_package_details(&self, package_id: &str) -> Result<Option<CatalogPackageDetails>, DatabaseError> {
        let conn = self.conn.lock();

        let summary_sql = "
            SELECT 
                p.id, p.repository_id, r.name as repository_name, p.name, p.description, 
                p.publisher, p.license, p.homepage, p.categories, p.tags, 
                p.latest_version, p.icon_url,
                i.version as installed_version
            FROM packages p
            JOIN repositories r ON p.repository_id = r.id
            LEFT JOIN installations i ON p.id = i.package_id
            WHERE p.id = ?1;";

        let summary = conn.query_row(summary_sql, params![package_id], |row| {
            let categories_str: Option<String> = row.get(8)?;
            let tags_str: Option<String> = row.get(9)?;

            let categories: Vec<String> = categories_str
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();

            let tags: Vec<String> = tags_str
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();

            Ok(CatalogPackageSummary {
                id: row.get(0)?,
                repository_id: row.get(1)?,
                repository_name: row.get(2)?,
                name: row.get(3)?,
                description: row.get(4)?,
                publisher: row.get(5)?,
                license: row.get(6)?,
                homepage: row.get(7)?,
                categories,
                tags,
                latest_version: row.get(10)?,
                icon_url: row.get(11)?,
                installed_version: row.get(12)?,
            })
        }).optional()?;

        let summary = match summary {
            Some(s) => s,
            None => return Ok(None),
        };

        // Fetch versions
        let mut ver_stmt = conn.prepare(
            "SELECT version, release_channel, release_date, manifest_raw
             FROM package_versions WHERE package_id = ?1 ORDER BY created_at DESC;"
        )?;

        let version_rows = ver_stmt.query_map(params![package_id], |row| {
            Ok(PackageVersionDetail {
                version: row.get(0)?,
                release_channel: row.get(1)?,
                release_date: row.get(2)?,
                manifest_raw: row.get(3)?,
            })
        })?;

        let mut versions = Vec::new();
        for v in version_rows {
            versions.push(v?);
        }

        Ok(Some(CatalogPackageDetails {
            summary,
            versions,
        }))
    }

    pub fn get_installed_applications(&self) -> Result<Vec<InstalledAppRecord>, DatabaseError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT i.id, i.package_id, COALESCE(p.name, i.package_id), i.version,
                    COALESCE(p.publisher, 'Unknown'), i.install_scope, i.install_path, i.installed_at, i.status
             FROM installations i
             LEFT JOIN packages p ON i.package_id = p.id
             ORDER BY i.installed_at DESC;"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(InstalledAppRecord {
                id: row.get(0)?,
                package_id: row.get(1)?,
                name: row.get(2)?,
                version: row.get(3)?,
                publisher: row.get(4)?,
                install_scope: row.get(5)?,
                install_path: row.get(6)?,
                installed_at: row.get(7)?,
                status: row.get(8)?,
            })
        })?;

        let mut list = Vec::new();
        for item in rows {
            list.push(item?);
        }
        Ok(list)
    }

    pub fn get_operations(&self) -> Result<Vec<OperationRecord>, DatabaseError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, package_id, operation_type, target_version, status, logs, created_at, completed_at
             FROM operations ORDER BY created_at DESC LIMIT 50;"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(OperationRecord {
                id: row.get(0)?,
                package_id: row.get(1)?,
                operation_type: row.get(2)?,
                target_version: row.get(3)?,
                status: row.get(4)?,
                logs: row.get(5)?,
                created_at: row.get(6)?,
                completed_at: row.get(7)?,
            })
        })?;

        let mut list = Vec::new();
        for item in rows {
            list.push(item?);
        }
        Ok(list)
    }

    pub fn get_database_stats(&self) -> Result<DatabaseStats, DatabaseError> {
        let conn = self.conn.lock();
        let version: u32 = conn.query_row("PRAGMA user_version;", [], |row| row.get(0))?;
        let repo_count: i64 = conn.query_row("SELECT COUNT(*) FROM repositories;", [], |row| row.get(0))?;
        let package_count: i64 = conn.query_row("SELECT COUNT(*) FROM packages;", [], |row| row.get(0))?;
        let install_count: i64 = conn.query_row("SELECT COUNT(*) FROM installations;", [], |row| row.get(0))?;
        let op_count: i64 = conn.query_row("SELECT COUNT(*) FROM operations;", [], |row| row.get(0))?;

        Ok(DatabaseStats {
            schema_version: version,
            db_path: self.db_path.to_string_lossy().to_string(),
            repositories_count: repo_count,
            packages_count: package_count,
            installations_count: install_count,
            operations_count: op_count,
        })
    }

    pub fn record_operation_start(
        &self,
        op_id: &str,
        package_id: &str,
        op_type: &str,
        target_version: &str,
        initial_log: &str,
    ) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO operations (id, package_id, operation_type, target_version, status, logs, created_at)
             VALUES (?1, ?2, ?3, ?4, 'in_progress', ?5, ?6);",
            params![op_id, package_id, op_type, target_version, initial_log, now],
        )?;
        Ok(())
    }

    pub fn record_operation_complete(
        &self,
        op_id: &str,
        status: &str,
        logs: &str,
    ) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE operations SET status = ?1, logs = ?2, completed_at = ?3 WHERE id = ?4;",
            params![status, logs, now, op_id],
        )?;
        Ok(())
    }

    pub fn record_installation(
        &self,
        package_id: &str,
        version: &str,
        install_scope: &str,
        install_path: &str,
        uninstall_type: &str,
    ) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO installations (id, package_id, version, install_scope, install_path, installed_at, updated_at, uninstall_type, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'installed')
             ON CONFLICT(id) DO UPDATE SET
                version = excluded.version,
                install_scope = excluded.install_scope,
                install_path = excluded.install_path,
                updated_at = excluded.updated_at,
                uninstall_type = excluded.uninstall_type,
                status = 'installed';",
            params![package_id, package_id, version, install_scope, install_path, now, now, uninstall_type],
        )?;
        Ok(())
    }

    pub fn get_installation_by_package_id(&self, package_id: &str) -> Result<Option<InstalledAppRecord>, DatabaseError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT i.id, i.package_id, COALESCE(p.name, i.package_id), i.version,
                    COALESCE(p.publisher, 'Unknown'), i.install_scope, i.install_path, i.installed_at, i.status
             FROM installations i
             LEFT JOIN packages p ON i.package_id = p.id
             WHERE i.package_id = ?1;",
        )?;

        let res = stmt.query_row(params![package_id], |row| {
            Ok(InstalledAppRecord {
                id: row.get(0)?,
                package_id: row.get(1)?,
                name: row.get(2)?,
                version: row.get(3)?,
                publisher: row.get(4)?,
                install_scope: row.get(5)?,
                install_path: row.get(6)?,
                installed_at: row.get(7)?,
                status: row.get(8)?,
            })
        }).optional()?;

        Ok(res)
    }

    pub fn get_installation_uninstall_type(&self, package_id: &str) -> Result<Option<String>, DatabaseError> {
        let conn = self.conn.lock();
        let res = conn.query_row(
            "SELECT uninstall_type FROM installations WHERE package_id = ?1;",
            params![package_id],
            |row| row.get(0),
        ).optional()?;
        Ok(res)
    }

    pub fn remove_installation(&self, package_id: &str) -> Result<bool, DatabaseError> {
        let conn = self.conn.lock();
        let affected = conn.execute(
            "DELETE FROM installations WHERE package_id = ?1;",
            params![package_id],
        )?;
        Ok(affected > 0)
    }

    pub fn record_download(
        &self,
        download_id: &str,
        artifact_id: &str,
        status: &str,
        bytes_downloaded: i64,
        total_bytes: Option<i64>,
        destination_path: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO downloads (id, artifact_id, status, bytes_downloaded, total_bytes, destination_path, started_at, completed_at, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                bytes_downloaded = excluded.bytes_downloaded,
                total_bytes = excluded.total_bytes,
                destination_path = excluded.destination_path,
                completed_at = excluded.completed_at,
                error_message = excluded.error_message;",
            params![download_id, artifact_id, status, bytes_downloaded, total_bytes, destination_path, now, now, error_message],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_sqlite_initialization_and_migration() {
        let temp_dir = std::env::temp_dir().join(format!("t3hub_test_{}", uuid::Uuid::new_v4()));
        let db_path = temp_dir.join("test_hub.db");

        let db = DatabaseManager::new(&db_path).expect("Failed to initialize test DB");
        let stats = db.get_database_stats().expect("Failed to retrieve DB stats");

        assert_eq!(stats.schema_version, 2);
        assert_eq!(stats.repositories_count, 1); // Seeded official repo

        // Check seeded settings
        let settings = db.get_settings().expect("Failed to get settings");
        assert_eq!(settings.get("theme").map(|s| s.as_str()), Some("system"));
        assert_eq!(settings.get("require_signatures").map(|s| s.as_str()), Some("true"));

        // Check setting update
        db.set_setting("theme", "dark").expect("Failed to set setting");
        let updated = db.get_settings().expect("Failed to get updated settings");
        assert_eq!(updated.get("theme").map(|s| s.as_str()), Some("dark"));

        // Check official repository
        let repos = db.get_repositories().expect("Failed to get repositories");
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].id, "t3-official");
        assert_eq!(repos[0].trust_policy, "verified");

        // Test package upsert and search
        db.upsert_package(
            "developer.devstudio",
            "t3-official",
            "DevStudio",
            Some("Lightweight code editor"),
            "T3 OpenSource",
            "[\"Developer Tools\", \"Utilities\"]",
            "[\"code-editor\"]",
            "1.2.0",
            None,
            "2026-10-31T12:00:00Z",
        ).expect("Upsert package failed");

        let search_res = db.search_packages(Some("devstudio"), None, None).expect("Search failed");
        assert_eq!(search_res.len(), 1);
        assert_eq!(search_res[0].id, "developer.devstudio");
        assert_eq!(search_res[0].name, "DevStudio");

        // Clean up
        let _ = fs::remove_dir_all(temp_dir);
    }
}
