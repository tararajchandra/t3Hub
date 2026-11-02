use super::models::WspManifest;
use std::collections::HashSet;
use url::Url;

/// Validates a WSP manifest according to the T3P v1.0 specification.
/// Returns Ok(()) if fully valid, or Err(Vec<String>) containing all validation errors.
pub fn validate_manifest(manifest: &WspManifest) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // 1. Schema version
    if manifest.schema_version != "1.0" {
        errors.push(format!(
            "Unsupported schema_version '{}'. Expected '1.0'.",
            manifest.schema_version
        ));
    }

    // 2. Package metadata
    let pkg = &manifest.package;
    if pkg.id.trim().is_empty() {
        errors.push("package.id must not be empty.".to_string());
    } else if !is_valid_package_id(&pkg.id) {
        errors.push(format!(
            "Invalid package.id '{}'. Must be a reverse-DNS style namespace (e.g. 'publisher.package-name').",
            pkg.id
        ));
    }

    if pkg.name.trim().is_empty() {
        errors.push("package.name must not be empty.".to_string());
    }

    if pkg.publisher.trim().is_empty() {
        errors.push("package.publisher must not be empty.".to_string());
    }

    // SemVer validation
    if let Err(e) = semver::Version::parse(&pkg.version) {
        errors.push(format!(
            "Invalid package.version '{}'. Must be valid SemVer 2.0 (e.g. 1.0.0): {}",
            pkg.version, e
        ));
    }

    // Optional URLs validation
    if let Some(ref hp) = pkg.homepage {
        if !is_valid_url(hp) {
            errors.push(format!("package.homepage '{}' is not a valid URL.", hp));
        }
    }

    if let Some(ref repo) = pkg.repository {
        if !is_valid_url(repo) {
            errors.push(format!("package.repository '{}' is not a valid URL.", repo));
        }
    }

    // 3. Platform
    let plat = &manifest.platform;
    if plat.os.to_lowercase() != "windows" {
        errors.push(format!(
            "Unsupported platform.os '{}'. Only 'windows' is supported.",
            plat.os
        ));
    }

    if plat.architectures.is_empty() {
        errors.push("platform.architectures must contain at least one architecture (e.g. ['x64']).".to_string());
    } else {
        let allowed_archs = ["x64", "x86", "arm64"];
        for arch in &plat.architectures {
            if !allowed_archs.contains(&arch.as_str()) {
                errors.push(format!(
                    "Invalid architecture '{}'. Allowed values: x64, x86, arm64.",
                    arch
                ));
            }
        }
    }

    // 4. Artifacts
    if manifest.artifacts.is_empty() {
        errors.push("Manifest must declare at least one artifact in 'artifacts'.".to_string());
    }

    let mut artifact_ids = HashSet::new();
    let mut artifact_type_map = std::collections::HashMap::new();

    for (idx, art) in manifest.artifacts.iter().enumerate() {
        let art_label = format!("artifacts[{}]", idx);

        if art.id.trim().is_empty() {
            errors.push(format!("{}.id must not be empty.", art_label));
        } else if !artifact_ids.insert(art.id.clone()) {
            errors.push(format!("Duplicate artifact id '{}' detected.", art.id));
        }

        let allowed_types = ["exe", "msi", "zip"];
        if !allowed_types.contains(&art.artifact_type.as_str()) {
            errors.push(format!(
                "{}.type '{}' is invalid. Must be 'exe', 'msi', or 'zip'.",
                art_label, art.artifact_type
            ));
        } else {
            artifact_type_map.insert(art.id.clone(), art.artifact_type.clone());
        }

        // SHA-256 validation (strictly 64 hex characters)
        if art.sha256.len() != 64 || !art.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            errors.push(format!(
                "{}.sha256 '{}' is invalid. Must be exactly 64 hexadecimal characters.",
                art_label, art.sha256
            ));
        }

        // URL validation (https or file)
        match Url::parse(&art.url) {
            Ok(parsed) => {
                let scheme = parsed.scheme();
                if scheme != "https" && scheme != "file" {
                    errors.push(format!(
                        "{}.url scheme '{}' is not supported. Must use HTTPS (or file for offline repos).",
                        art_label, scheme
                    ));
                }
            }
            Err(e) => {
                errors.push(format!("{}.url '{}' is malformed: {}", art_label, art.url, e));
            }
        }
    }

    // 5. Installation
    let inst = &manifest.installation;
    let allowed_inst_types = ["exe", "msi", "zip"];
    if !allowed_inst_types.contains(&inst.install_type.as_str()) {
        errors.push(format!(
            "installation.type '{}' is invalid. Must be 'exe', 'msi', or 'zip'.",
            inst.install_type
        ));
    }

    if !artifact_ids.contains(&inst.artifact) {
        errors.push(format!(
            "installation.artifact '{}' does not match any declared artifact id.",
            inst.artifact
        ));
    } else if let Some(art_type) = artifact_type_map.get(&inst.artifact) {
        if *art_type != inst.install_type {
            errors.push(format!(
                "installation.type '{}' does not match referenced artifact type '{}'.",
                inst.install_type, art_type
            ));
        }
    }

    let allowed_scopes = ["user", "machine"];
    if !allowed_scopes.contains(&inst.scope.as_str()) {
        errors.push(format!(
            "installation.scope '{}' is invalid. Allowed: 'user', 'machine'.",
            inst.scope
        ));
    }

    // Security check: arguments must not contain shell pipe/redirection characters
    for arg in &inst.arguments {
        if arg.contains('|')
            || arg.contains('&')
            || arg.contains(';')
            || arg.contains('>')
            || arg.contains('<')
            || arg.contains('`')
            || arg.contains('$')
            || arg.contains('\n')
            || arg.contains('\r')
        {
            errors.push(format!(
                "installation.arguments contains prohibited shell control characters in argument '{}'. Arguments are passed directly to execve without shell expansion.",
                arg
            ));
        }
    }

    // 6. Uninstallation (optional, but if present must be valid)
    if let Some(ref uninst) = manifest.uninstallation {
        let allowed_uninst_types = ["registry", "zip_directory", "msi"];
        if !allowed_uninst_types.contains(&uninst.uninstall_type.as_str()) {
            errors.push(format!(
                "uninstallation.type '{}' is invalid. Allowed: 'registry', 'zip_directory', 'msi'.",
                uninst.uninstall_type
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Helper to validate package ID format: e.g. "developer.devstudio" or "org.my-app.editor"
fn is_valid_package_id(id: &str) -> bool {
    let parts: Vec<&str> = id.split('.').collect();
    if parts.len() < 2 {
        return false;
    }
    for part in parts {
        if part.is_empty() {
            return false;
        }
        if !part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return false;
        }
    }
    true
}

fn is_valid_url(s: &str) -> bool {
    Url::parse(s).is_ok()
}
