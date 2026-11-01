pub mod hash;
pub mod signature;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityCheckResult {
    pub matched: bool,
    pub expected_hash: String,
    pub calculated_hash: String,
}

pub use hash::{compute_file_sha256, verify_file_sha256, SecurityError};
pub use signature::{SignatureError, SignatureVerifier};
