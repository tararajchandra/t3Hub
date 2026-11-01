use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use thiserror::Error;
use zip::ZipArchive;

#[derive(Debug, Error)]
pub enum ZipInstallError {
    #[error("I/O error during ZIP installation: {0}")]
    Io(#[from] io::Error),

    #[error("ZIP format error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Path traversal attack detected in ZIP entry: '{0}'. Extraction aborted for security.")]
    PathTraversal(String),

    #[error("No valid destination directory could be established")]
    InvalidDestination,
}

#[derive(Debug, Clone)]
pub struct ZipInstallOutcome {
    pub install_dir: PathBuf,
    pub executable_path: Option<PathBuf>,
    pub files_extracted: usize,
    pub total_bytes: u64,
}

pub struct ZipInstaller;

impl ZipInstaller {
    /// Safely extracts a ZIP package into the target directory with strict
    /// path traversal (Zip Slip) prevention.
    pub fn install<P: AsRef<Path>, D: AsRef<Path>>(
        zip_archive_path: P,
        destination_dir: D,
    ) -> Result<ZipInstallOutcome, ZipInstallError> {
        let zip_path = zip_archive_path.as_ref();
        let target_dir = destination_dir.as_ref().to_path_buf();

        fs::create_dir_all(&target_dir)?;
        let canonical_target = target_dir.canonicalize().map_err(ZipInstallError::Io)?;

        let file = File::open(zip_path)?;
        let reader = BufReader::new(file);
        let mut archive = ZipArchive::new(reader)?;

        let mut files_extracted = 0;
        let mut total_bytes: u64 = 0;
        let mut candidate_exes: Vec<PathBuf> = Vec::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;

            // 1. Strict Zip Slip check: enclosed_name ensures no path escapes root
            let enclosed_path = match file.enclosed_name() {
                Some(p) => p.to_path_buf(),
                None => {
                    return Err(ZipInstallError::PathTraversal(file.name().to_string()));
                }
            };

            let outpath = canonical_target.join(&enclosed_path);

            // Double check starts_with on normalized path
            if !outpath.starts_with(&canonical_target) {
                return Err(ZipInstallError::PathTraversal(file.name().to_string()));
            }

            if file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }

                let mut outfile = File::create(&outpath)?;
                let copied = io::copy(&mut file, &mut outfile)?;
                total_bytes += copied;
                files_extracted += 1;

                // Track executable candidates
                if let Some(ext) = outpath.extension() {
                    if ext.eq_ignore_ascii_case("exe") {
                        candidate_exes.push(outpath);
                    }
                }
            }
        }

        // Determine main executable
        let main_exe = Self::detect_main_executable(&candidate_exes, &canonical_target);

        log::info!(
            "ZIP installation complete: {} files extracted to {:?}",
            files_extracted,
            canonical_target
        );

        Ok(ZipInstallOutcome {
            install_dir: canonical_target,
            executable_path: main_exe,
            files_extracted,
            total_bytes,
        })
    }

    /// Picks the best executable from candidates:
    /// 1. Directly in target dir
    /// 2. In bin/ subdir
    /// 3. First available .exe
    fn detect_main_executable(candidates: &[PathBuf], target_dir: &Path) -> Option<PathBuf> {
        if candidates.is_empty() {
            return None;
        }

        // Check if there is an .exe directly in root
        for c in candidates {
            if c.parent() == Some(target_dir) {
                return Some(c.clone());
            }
        }

        // Check if there is an .exe in bin/
        let bin_dir = target_dir.join("bin");
        for c in candidates {
            if c.parent() == Some(&bin_dir) {
                return Some(c.clone());
            }
        }

        // Default to first found
        candidates.first().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::{ExtendedFileOptions, FileOptions};
    use zip::ZipWriter;

    #[test]
    fn test_zip_safe_extraction() {
        let temp_dir = std::env::temp_dir().join(format!("t3_zip_test_{}", uuid::Uuid::new_v4()));
        let zip_file_path = temp_dir.join("test_app.zip");
        let extract_dir = temp_dir.join("extracted");

        fs::create_dir_all(&temp_dir).unwrap();

        // Create a valid test zip archive with an executable and readme
        {
            let file = File::create(&zip_file_path).unwrap();
            let mut zip = ZipWriter::new(file);

            let options = FileOptions::<ExtendedFileOptions>::default();

            zip.start_file("app.exe", options.clone()).unwrap();
            zip.write_all(b"MOCK_EXE_BINARY_DATA").unwrap();

            zip.start_file("config/settings.json", options).unwrap();
            zip.write_all(b"{\"theme\": \"dark\"}").unwrap();

            zip.finish().unwrap();
        }

        let outcome = ZipInstaller::install(&zip_file_path, &extract_dir)
            .expect("Safe extraction should succeed");

        assert_eq!(outcome.files_extracted, 2);
        assert!(outcome.install_dir.exists());
        assert!(outcome.install_dir.join("app.exe").exists());
        assert!(outcome.install_dir.join("config").join("settings.json").exists());
        assert_eq!(
            outcome.executable_path,
            Some(outcome.install_dir.join("app.exe"))
        );

        // Clean up
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_zip_path_traversal_rejection() {
        let temp_dir = std::env::temp_dir().join(format!("t3_zip_slip_test_{}", uuid::Uuid::new_v4()));
        let zip_file_path = temp_dir.join("malicious.zip");
        let extract_dir = temp_dir.join("safe_dest");

        fs::create_dir_all(&temp_dir).unwrap();

        // Create a malicious zip attempting path traversal
        {
            let file = File::create(&zip_file_path).unwrap();
            let mut zip = ZipWriter::new(file);
            let options = FileOptions::<ExtendedFileOptions>::default();

            // Attempt to write outside target dir with "../escaped.txt"
            zip.start_file("../escaped.txt", options).unwrap();
            zip.write_all(b"EVIL_DATA").unwrap();
            zip.finish().unwrap();
        }

        let err = ZipInstaller::install(&zip_file_path, &extract_dir).unwrap_err();
        match err {
            ZipInstallError::PathTraversal(path) => {
                assert!(path.contains("escaped.txt"));
            }
            _ => panic!("Expected PathTraversal error, got {:?}", err),
        }

        // Clean up
        let _ = fs::remove_dir_all(temp_dir);
    }
}
