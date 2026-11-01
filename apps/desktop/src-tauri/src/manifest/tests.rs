#[cfg(test)]
mod tests {
    use crate::manifest::parser::{parse_manifest_from_str, ManifestError};

    const VALID_MANIFEST_YAML: &str = r#"
schema_version: "1.0"

package:
  id: "developer.devstudio"
  name: "DevStudio"
  version: "1.2.0"
  description: "Modern lightweight workspace editor and developer toolkit"
  publisher: "T3 OpenSource"
  license: "MIT"
  homepage: "https://devstudio.tools"
  repository: "https://github.com/developer/devstudio"
  categories:
    - Developer Tools
    - Utilities
  tags:
    - editor
    - ide
    - developer-tools

platform:
  os: "windows"
  architectures:
    - "x64"
  minimum_windows_version: "10"

release:
  channel: "stable"
  release_date: "2026-10-31"

artifacts:
  - id: "windows-x64"
    type: "exe"
    url: "https://github.com/developer/devstudio/releases/download/v1.2.0/devstudio-setup.exe"
    sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    size: 25000000

installation:
  type: "exe"
  artifact: "windows-x64"
  scope: "user"
  arguments:
    - "/VERYSILENT"
    - "/SUPPRESSMSGBOXES"
    - "/NORESTART"
  timeout_seconds: 600

uninstallation:
  type: "registry"
  detect:
    registry_path: "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\DevStudio"

updates:
  strategy: "version"
  channel: "stable"
"#;

    #[test]
    fn test_valid_manifest_parse() {
        let manifest = parse_manifest_from_str(VALID_MANIFEST_YAML).expect("Valid manifest should parse");
        assert_eq!(manifest.schema_version, "1.0");
        assert_eq!(manifest.package.id, "developer.devstudio");
        assert_eq!(manifest.package.version, "1.2.0");
        assert_eq!(manifest.installation.install_type, "exe");
        assert_eq!(manifest.installation.artifact, "windows-x64");
    }

    #[test]
    fn test_invalid_package_id() {
        let invalid_yaml = VALID_MANIFEST_YAML.replace("id: \"developer.devstudio\"", "id: \"devstudio\"");
        let err = parse_manifest_from_str(&invalid_yaml).unwrap_err();
        match err {
            ManifestError::Validation(errs) => {
                assert!(errs.iter().any(|e| e.contains("reverse-DNS style namespace")));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[test]
    fn test_invalid_semver() {
        let invalid_yaml = VALID_MANIFEST_YAML.replace("version: \"1.2.0\"", "version: \"1.2\"");
        let err = parse_manifest_from_str(&invalid_yaml).unwrap_err();
        match err {
            ManifestError::Validation(errs) => {
                assert!(errs.iter().any(|e| e.contains("SemVer 2.0")));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[test]
    fn test_invalid_sha256() {
        let invalid_yaml = VALID_MANIFEST_YAML.replace(
            "sha256: \"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\"",
            "sha256: \"short_invalid_hash\""
        );
        let err = parse_manifest_from_str(&invalid_yaml).unwrap_err();
        match err {
            ManifestError::Validation(errs) => {
                assert!(errs.iter().any(|e| e.contains("exactly 64 hexadecimal characters")));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[test]
    fn test_mismatched_artifact_id() {
        let invalid_yaml = VALID_MANIFEST_YAML.replace("artifact: \"windows-x64\"", "artifact: \"nonexistent-artifact\"");
        let err = parse_manifest_from_str(&invalid_yaml).unwrap_err();
        match err {
            ManifestError::Validation(errs) => {
                assert!(errs.iter().any(|e| e.contains("does not match any declared artifact id")));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[test]
    fn test_prohibited_shell_argument() {
        let invalid_yaml = VALID_MANIFEST_YAML.replace("- \"/NORESTART\"", "- \"/NORESTART; rm -rf /\"");
        let err = parse_manifest_from_str(&invalid_yaml).unwrap_err();
        match err {
            ManifestError::Validation(errs) => {
                assert!(errs.iter().any(|e| e.contains("prohibited shell control characters")));
            }
            _ => panic!("Expected validation error"),
        }
    }
}
