use std::path::Path;
use std::process::Command;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsiInstallOutcome {
    pub exit_code: i32,
    pub reboot_required: bool,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum MsiInstallError {
    #[error("MSI package file not found at: {0}")]
    FileNotFound(String),

    #[error("Security violation in MSI arguments: {0}")]
    SecurityViolation(String),

    #[error("Failed to execute msiexec process: {0}")]
    ProcessSpawn(String),

    #[error("Installation failed with exit code {code}: {explanation}")]
    InstallFailed { code: i32, explanation: String },

    #[error("Installation timed out after {0} seconds")]
    Timeout(u64),
}

pub struct MsiInstaller;

impl MsiInstaller {
    /// Validate that arguments contain no prohibited shell meta-characters
    pub fn validate_arguments(args: &[String]) -> Result<(), MsiInstallError> {
        let prohibited = [";", "&", "|", "<", ">", "`", "$", "\n", "\r"];
        for arg in args {
            for bad in &prohibited {
                if arg.contains(bad) {
                    return Err(MsiInstallError::SecurityViolation(format!(
                        "Argument '{}' contains prohibited shell character '{}'",
                        arg, bad
                    )));
                }
            }
        }
        Ok(())
    }

    /// Map standard Windows Installer exit codes to human-readable explanations
    pub fn explain_exit_code(code: i32) -> &'static str {
        match code {
            0 => "Installation completed successfully.",
            3010 => "Installation completed successfully. A system restart is required to complete configuration.",
            1601 => "The Windows Installer service could not be accessed.",
            1602 => "The user canceled installation.",
            1603 => "Fatal error during installation.",
            1605 => "This action is only valid for products that are currently installed.",
            1618 => "Another installation is already in progress. Complete that installation before proceeding.",
            1619 => "This installation package could not be opened. Verify that the package exists and is accessible.",
            1620 => "This installation package could not be opened. Contact the application vendor to verify package validity.",
            1625 => "This installation is forbidden by system policy. Contact your system administrator.",
            1638 => "Another version of this product is already installed. Installation cannot continue.",
            1641 => "The installer has initiated a restart. This message is indicative of a success.",
            _ => "The installer returned an unexpected error code.",
        }
    }

    /// Run silent MSI installation using Windows `msiexec.exe /i <path> /qn /norestart`
    pub fn install(
        msi_path: &Path,
        additional_args: Option<&[String]>,
        timeout_seconds: Option<u64>,
    ) -> Result<MsiInstallOutcome, MsiInstallError> {
        if !msi_path.exists() {
            return Err(MsiInstallError::FileNotFound(msi_path.to_string_lossy().to_string()));
        }

        if let Some(args) = additional_args {
            Self::validate_arguments(args)?;
        }

        let mut cmd = Command::new("msiexec.exe");
        cmd.arg("/i")
            .arg(msi_path)
            .arg("/qn")
            .arg("/norestart");

        if let Some(args) = additional_args {
            for arg in args {
                cmd.arg(arg);
            }
        }

        let _timeout = Duration::from_secs(timeout_seconds.unwrap_or(600));

        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => return Err(MsiInstallError::ProcessSpawn(e.to_string())),
        };

        let exit_code = output.status.code().unwrap_or(-1);

        match exit_code {
            0 => Ok(MsiInstallOutcome {
                exit_code: 0,
                reboot_required: false,
                message: Self::explain_exit_code(0).to_string(),
            }),
            3010 => Ok(MsiInstallOutcome {
                exit_code: 3010,
                reboot_required: true,
                message: Self::explain_exit_code(3010).to_string(),
            }),
            code => Err(MsiInstallError::InstallFailed {
                code,
                explanation: Self::explain_exit_code(code).to_string(),
            }),
        }
    }

    /// Run silent MSI uninstallation using Windows `msiexec.exe /x <product_code_or_path> /qn /norestart`
    pub fn uninstall(
        target: &str,
        additional_args: Option<&[String]>,
    ) -> Result<MsiInstallOutcome, MsiInstallError> {
        if let Some(args) = additional_args {
            Self::validate_arguments(args)?;
        }

        let mut cmd = Command::new("msiexec.exe");
        cmd.arg("/x")
            .arg(target)
            .arg("/qn")
            .arg("/norestart");

        if let Some(args) = additional_args {
            for arg in args {
                cmd.arg(arg);
            }
        }

        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => return Err(MsiInstallError::ProcessSpawn(e.to_string())),
        };

        let exit_code = output.status.code().unwrap_or(-1);

        match exit_code {
            0 => Ok(MsiInstallOutcome {
                exit_code: 0,
                reboot_required: false,
                message: "Uninstallation completed successfully.".to_string(),
            }),
            3010 => Ok(MsiInstallOutcome {
                exit_code: 3010,
                reboot_required: true,
                message: "Uninstallation completed successfully. A system restart is required.".to_string(),
            }),
            code => Err(MsiInstallError::InstallFailed {
                code,
                explanation: Self::explain_exit_code(code).to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explain_standard_msi_exit_codes() {
        assert_eq!(MsiInstaller::explain_exit_code(0), "Installation completed successfully.");
        assert!(MsiInstaller::explain_exit_code(3010).contains("restart is required"));
        assert!(MsiInstaller::explain_exit_code(1602).contains("canceled"));
        assert!(MsiInstaller::explain_exit_code(1618).contains("Another installation is already in progress"));
    }

    #[test]
    fn test_argument_validation_rejects_shell_injection() {
        let safe_args = vec!["ALLUSERS=2".to_string(), "MSIINSTALLPERUSER=1".to_string()];
        assert!(MsiInstaller::validate_arguments(&safe_args).is_ok());

        let bad_args1 = vec!["ALLUSERS=2; calc.exe".to_string()];
        assert!(MsiInstaller::validate_arguments(&bad_args1).is_err());

        let bad_args2 = vec!["| powershell.exe".to_string()];
        assert!(MsiInstaller::validate_arguments(&bad_args2).is_err());

        let bad_args3 = vec!["`whoami`".to_string()];
        assert!(MsiInstaller::validate_arguments(&bad_args3).is_err());
    }

    #[test]
    fn test_non_existent_msi_returns_file_not_found() {
        let fake_path = Path::new("C:\\non_existent_folder_xyz\\installer.msi");
        let result = MsiInstaller::install(fake_path, None, None);
        assert!(result.is_err());
        match result.unwrap_err() {
            MsiInstallError::FileNotFound(p) => assert!(p.contains("installer.msi")),
            _ => panic!("Expected FileNotFound error"),
        }
    }
}
