pub mod exe;
pub mod msi;

pub use exe::{ExeInstallError, ExeInstallOutcome, ExeInstaller, ExeInstallerProfile};
pub use msi::{MsiInstallError, MsiInstallOutcome, MsiInstaller};
