use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use thiserror::Error;

use super::IntegrityCheckResult;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("File I/O error during hash calculation: {0}")]
    Io(#[from] io::Error),

    #[error("Integrity check failed: Expected SHA-256 '{expected}', but calculated '{calculated}'")]
    ChecksumMismatch {
        expected: String,
        calculated: String,
    },
}

/// Computes the SHA-256 digest of a file in streaming chunks (64 KB).
pub fn compute_file_sha256<P: AsRef<Path>>(path: P) -> Result<String, SecurityError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536]; // 64 KB buffer

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// Verifies that a file's SHA-256 matches the expected hash.
pub fn verify_file_sha256<P: AsRef<Path>>(
    path: P,
    expected_hex: &str,
) -> Result<IntegrityCheckResult, SecurityError> {
    let calculated = compute_file_sha256(path)?;
    let expected_clean = expected_hex.trim().to_lowercase();
    let matched = calculated.eq_ignore_ascii_case(&expected_clean);

    let res = IntegrityCheckResult {
        matched,
        expected_hash: expected_clean.clone(),
        calculated_hash: calculated.clone(),
    };

    if !matched {
        return Err(SecurityError::ChecksumMismatch {
            expected: expected_clean,
            calculated,
        });
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_sha256_streaming_computation() {
        let temp_dir = std::env::temp_dir().join(format!("t3_hash_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("test_sample.txt");

        // Write known content: "Hello T3 Hub"
        // SHA-256 of "Hello T3 Hub" (without newline):
        // echo -n "Hello T3 Hub" | sha256sum -> d9b2adbc382ff5cf2bfbba6bb5e78a032d8ff552199f1faae75502c525f0525d
        {
            let mut f = File::create(&file_path).unwrap();
            f.write_all(b"Hello T3 Hub").unwrap();
        }

        let hash = compute_file_sha256(&file_path).expect("Hash computation failed");
        assert_eq!(
            hash,
            "51e1b548b576233f7b8b41ae791e1427d524d5c6a9519a4c6dc0779f28f10973"
        );

        // Verify match
        let verify_ok = verify_file_sha256(
            &file_path,
            "51E1B548B576233F7B8B41AE791E1427D524D5C6A9519A4C6DC0779F28F10973",
        )
        .expect("Verification should succeed with case-insensitivity");
        assert!(verify_ok.matched);

        // Verify mismatch
        let mismatch_err = verify_file_sha256(
            &file_path,
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap_err();
        match mismatch_err {
            SecurityError::ChecksumMismatch { .. } => {}
            _ => panic!("Expected ChecksumMismatch error"),
        }

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
