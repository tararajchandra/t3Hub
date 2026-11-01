pub mod adapters;
pub mod detection;
pub mod engine;
pub mod zip_adapter;

pub use adapters::{
    ExeInstallError, ExeInstallOutcome, ExeInstaller, ExeInstallerProfile,
    MsiInstallError, MsiInstallOutcome, MsiInstaller,
};
pub use detection::{AppDetector, InstalledAppInfo};
pub use engine::{InstallationEngine, InstallationResult};
pub use zip_adapter::{ZipInstallError, ZipInstallOutcome, ZipInstaller};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallerType {
    Exe,
    Msi,
    Zip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationRequest {
    pub package_id: String,
    pub version: Option<String>,
}
