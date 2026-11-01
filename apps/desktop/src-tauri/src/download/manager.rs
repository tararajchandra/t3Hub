use futures_util::StreamExt;
use reqwest::Client;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use url::Url;

use crate::security::{verify_file_sha256, SecurityError};

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("URL error: {0}")]
    Url(String),

    #[error("Security integrity error: {0}")]
    Security(#[from] SecurityError),

    #[error("Download was cancelled")]
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub percentage: f32,
}

pub struct DownloadManager {
    client: Client,
    cache_dir: PathBuf,
}

impl DownloadManager {
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Self {
        let dir = cache_dir.as_ref().to_path_buf();
        let _ = fs::create_dir_all(&dir);

        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .user_agent("T3Hub-Downloader/1.0 (Windows NT)")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            cache_dir: dir,
        }
    }

    /// Returns the cache path for a given SHA-256 and extension.
    pub fn get_cached_artifact_path(&self, sha256: &str, extension: &str) -> PathBuf {
        let clean_ext = extension.trim_start_matches('.');
        let filename = format!("{}.{}", sha256.to_lowercase(), clean_ext);
        self.cache_dir.join(filename)
    }

    /// Checks if a valid cached artifact already exists and matches expected SHA-256.
    pub fn get_verified_cache(&self, sha256: &str, extension: &str) -> Option<PathBuf> {
        let path = self.get_cached_artifact_path(sha256, extension);
        if path.exists() {
            if let Ok(res) = verify_file_sha256(&path, sha256) {
                if res.matched {
                    return Some(path);
                }
            }
            // Invalid/corrupted file in cache, delete it
            let _ = fs::remove_file(&path);
        }
        None
    }

    /// Downloads or copies an artifact from a URL or local file, verifies its SHA-256,
    /// and stores it in the cache directory.
    pub async fn download_and_verify<F>(
        &self,
        url_or_path: &str,
        expected_sha256: &str,
        extension: &str,
        cancel_token: Option<Arc<AtomicBool>>,
        mut on_progress: F,
    ) -> Result<PathBuf, DownloadError>
    where
        F: FnMut(ProgressUpdate),
    {
        // 1. Check if already in cache and verified
        if let Some(cached) = self.get_verified_cache(expected_sha256, extension) {
            log::info!("Artifact '{}' already cached at {:?}", expected_sha256, cached);
            on_progress(ProgressUpdate {
                bytes_downloaded: 1,
                total_bytes: Some(1),
                percentage: 100.0,
            });
            return Ok(cached);
        }

        let target_path = self.get_cached_artifact_path(expected_sha256, extension);
        let part_path = target_path.with_extension("part");

        // Ensure parent exists
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let url_clean = url_or_path.trim();

        if url_clean.starts_with("http://") || url_clean.starts_with("https://") {
            // Remote HTTP(S) stream download
            let response = self.client.get(url_clean).send().await?;
            if !response.status().is_success() {
                return Err(DownloadError::Network(response.error_for_status().unwrap_err()));
            }

            let total_bytes = response.content_length();
            let mut downloaded: u64 = 0;
            let mut file = File::create(&part_path)?;

            let mut stream = response.bytes_stream();
            while let Some(chunk_res) = stream.next().await {
                if let Some(ref cancel) = cancel_token {
                    if cancel.load(Ordering::Relaxed) {
                        let _ = fs::remove_file(&part_path);
                        return Err(DownloadError::Cancelled);
                    }
                }

                let chunk = chunk_res?;
                file.write_all(&chunk)?;
                downloaded += chunk.len() as u64;

                let percentage = match total_bytes {
                    Some(total) if total > 0 => ((downloaded as f64 / total as f64) * 100.0) as f32,
                    _ => 0.0,
                };

                on_progress(ProgressUpdate {
                    bytes_downloaded: downloaded,
                    total_bytes,
                    percentage,
                });
            }

            file.flush()?;
            drop(file);
        } else {
            // Local file copy (file:// scheme or filesystem path)
            let src_path = if url_clean.starts_with("file://") {
                let parsed = Url::parse(url_clean).map_err(|e| DownloadError::Url(e.to_string()))?;
                parsed.to_file_path().map_err(|_| DownloadError::Url("Invalid file:// URL path".to_string()))?
            } else {
                PathBuf::from(url_clean)
            };

            if !src_path.exists() {
                return Err(DownloadError::Io(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Local artifact not found: {}", src_path.display()),
                )));
            }

            let meta = fs::metadata(&src_path)?;
            let total_bytes = Some(meta.len());

            fs::copy(&src_path, &part_path)?;

            on_progress(ProgressUpdate {
                bytes_downloaded: meta.len(),
                total_bytes,
                percentage: 100.0,
            });
        }

        // 2. Perform SHA-256 verification on the downloaded temporary file
        match verify_file_sha256(&part_path, expected_sha256) {
            Ok(check) if check.matched => {
                // Rename part file to final cached artifact file
                fs::rename(&part_path, &target_path)?;
                log::info!(
                    "Artifact successfully verified and cached at: {:?}",
                    target_path
                );
                Ok(target_path)
            }
            Ok(_) => {
                let _ = fs::remove_file(&part_path);
                Err(DownloadError::Security(SecurityError::ChecksumMismatch {
                    expected: expected_sha256.to_string(),
                    calculated: "mismatch".to_string(),
                }))
            }
            Err(e) => {
                let _ = fs::remove_file(&part_path);
                Err(DownloadError::Security(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download_and_verify_local_file() {
        let temp_dir = std::env::temp_dir().join(format!("t3_download_test_{}", uuid::Uuid::new_v4()));
        let cache_dir = temp_dir.join("cache");
        let source_file = temp_dir.join("dummy_artifact.zip");

        fs::create_dir_all(&temp_dir).unwrap();
        // Sample content
        fs::write(&source_file, b"T3 Hub Portable Zip Package Sample").unwrap();

        // Calculate expected SHA-256
        let expected_hash = crate::security::compute_file_sha256(&source_file).unwrap();

        let manager = DownloadManager::new(&cache_dir);
        let mut progress_called = false;

        let cached_file = manager
            .download_and_verify(
                source_file.to_str().unwrap(),
                &expected_hash,
                "zip",
                None,
                |p| {
                    if p.percentage >= 100.0 {
                        progress_called = true;
                    }
                },
            )
            .await
            .expect("Download and verification should succeed");

        assert!(cached_file.exists());
        assert!(progress_called);

        // Test second call uses cache
        let cached_second = manager
            .download_and_verify(
                "invalid_nonexistent_url",
                &expected_hash,
                "zip",
                None,
                |_| {},
            )
            .await
            .expect("Should reuse valid cached file without network fetch");

        assert_eq!(cached_file, cached_second);

        // Clean up
        let _ = fs::remove_dir_all(temp_dir);
    }
}
