pub mod engine;

pub use engine::UninstallEngine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallRequest {
    pub package_id: String,
    pub installed_version: String,
    pub uninstall_type: String, // "registry" | "zip_directory" | "msi"
    pub uninstall_string: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallResult {
    pub success: bool,
    pub error_message: Option<String>,
}
