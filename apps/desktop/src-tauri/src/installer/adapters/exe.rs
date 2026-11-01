use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExeInstallerProfile {
    InnoSetup,
    Nsis,
    Wix,
    Custom,
}

impl ExeInstallerProfile {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "inno" | "innosetup" => Self::InnoSetup,
            "nsis" => Self::Nsis,
            "wix" | "burn" => Self::Wix,
            _ => Self::Custom,
        }
    }

    pub fn default_silent_args(&self) -> Vec<String> {
        match self {
            Self::InnoSetup => vec![
                "/VERYSILENT".to_string(),
                "/SUPPRESSMSGBOXES".to_string(),
                "/NORESTART".to_string(),
                "/SP-".to_string(),
            ],
            Self::Nsis => vec!["/S".to_string()],
            Self::Wix => vec!["/quiet".to_string(), "/norestart".to_string()],
            Self::Custom => vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExeInstallOutcome {
    pub exit_code: i32,
    pub success: bool,
    pub message: String,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ExeInstallError {
    #[error("Installer executable not found at: {0}")]
    FileNotFound(String),

    #[error("Security violation in installer arguments: {0}")]
    SecurityViolation(String),

    #[error("Failed to execute installer process: {0}")]
    ProcessSpawn(String),

    #[error("Installation failed with exit code {0}: {1}")]
    ExecutionFailed(i32, String),
}

pub struct ExeInstaller;

impl ExeInstaller {
    /// Validates arguments against prohibited shell meta-characters
    pub fn validate_arguments(args: &[String]) -> Result<(), ExeInstallError> {
        let prohibited = [";", "&", "|", "<", ">", "`", "$", "%", "\n", "\r"];
        for arg in args {
            for bad in &prohibited {
                if arg.contains(bad) {
                    return Err(ExeInstallError::SecurityViolation(format!(
                        "Argument '{}' contains prohibited shell operator '{}'",
                        arg, bad
                    )));
                }
            }
        }
        Ok(())
    }

    /// Run silent EXE installation with declared or profile arguments
    pub fn install(
        exe_path: &Path,
        profile_name: Option<&str>,
        manifest_args: Option<&[String]>,
    ) -> Result<ExeInstallOutcome, ExeInstallError> {
        if !exe_path.exists() {
            return Err(ExeInstallError::FileNotFound(exe_path.to_string_lossy().to_string()));
        }

        // Determine final arguments
        let final_args: Vec<String> = if let Some(m_args) = manifest_args {
            if !m_args.is_empty() {
                Self::validate_arguments(m_args)?;
                m_args.to_vec()
            } else if let Some(prof) = profile_name {
                let profile = ExeInstallerProfile::from_str(prof);
                profile.default_silent_args()
            } else {
                vec![]
            }
        } else if let Some(prof) = profile_name {
            let profile = ExeInstallerProfile::from_str(prof);
            profile.default_silent_args()
        } else {
            vec![]
        };

        let mut cmd = Command::new(exe_path);
        for arg in &final_args {
            cmd.arg(arg);
        }

        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => return Err(ExeInstallError::ProcessSpawn(e.to_string())),
        };

        let exit_code = output.status.code().unwrap_or(-1);
        let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

        if exit_code == 0 {
            Ok(ExeInstallOutcome {
                exit_code: 0,
                success: true,
                message: "Installer executed successfully.".to_string(),
                stdout: stdout_str,
                stderr: stderr_str,
            })
        } else {
            Err(ExeInstallError::ExecutionFailed(
                exit_code,
                format!("Installer exited with error code {}. Stderr: {}", exit_code, stderr_str.trim()),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_resolution_and_defaults() {
        let inno = ExeInstallerProfile::from_str("innosetup");
        assert_eq!(inno, ExeInstallerProfile::InnoSetup);
        let inno_args = inno.default_silent_args();
        assert!(inno_args.contains(&"/VERYSILENT".to_string()));
        assert!(inno_args.contains(&"/NORESTART".to_string()));

        let nsis = ExeInstallerProfile::from_str("nsis");
        assert_eq!(nsis, ExeInstallerProfile::Nsis);
        assert_eq!(nsis.default_silent_args(), vec!["/S"]);

        let wix = ExeInstallerProfile::from_str("wix");
        assert_eq!(wix, ExeInstallerProfile::Wix);
        assert_eq!(wix.default_silent_args(), vec!["/quiet", "/norestart"]);
    }

    #[test]
    fn test_exe_arguments_reject_injection() {
        let safe = vec!["/DIR=C:\\Apps".to_string(), "/VERYSILENT".to_string()];
        assert!(ExeInstaller::validate_arguments(&safe).is_ok());

        let malicious = vec!["/DIR=C:\\Apps & del /f /q C:\\Windows".to_string()];
        assert!(ExeInstaller::validate_arguments(&malicious).is_err());
    }

    #[test]
    fn test_missing_exe_returns_file_not_found() {
        let fake_path = Path::new("C:\\non_existent_folder_abc\\setup.exe");
        let result = ExeInstaller::install(fake_path, Some("inno"), None);
        assert!(result.is_err());
        match result.unwrap_err() {
            ExeInstallError::FileNotFound(p) => assert!(p.contains("setup.exe")),
            _ => panic!("Expected FileNotFound"),
        }
    }
}
