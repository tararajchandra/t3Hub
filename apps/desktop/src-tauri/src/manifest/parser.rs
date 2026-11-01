use super::models::WspManifest;
use super::validation::validate_manifest;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("Failed to read manifest file: {0}")]
    Io(String),

    #[error("Failed to parse manifest YAML: {0}")]
    Yaml(String),

    #[error("Manifest validation failed:\n{}", .0.join("\n"))]
    Validation(Vec<String>),
}

/// Parses a WSP YAML manifest string and validates its schema and integrity constraints.
pub fn parse_manifest_from_str(yaml_str: &str) -> Result<WspManifest, ManifestError> {
    let manifest: WspManifest = serde_yaml_ng::from_str(yaml_str)
        .map_err(|e| ManifestError::Yaml(e.to_string()))?;

    validate_manifest(&manifest)
        .map_err(ManifestError::Validation)?;

    Ok(manifest)
}

/// Reads a WSP YAML manifest from a file path and validates it.
pub fn parse_manifest_from_file<P: AsRef<Path>>(path: P) -> Result<WspManifest, ManifestError> {
    let path_ref = path.as_ref();
    let content = fs::read_to_string(path_ref)
        .map_err(|e| ManifestError::Io(format!("Could not read {}: {}", path_ref.display(), e)))?;

    parse_manifest_from_str(&content)
}
