use crate::manifest::models::WspManifest;
use crate::manifest::validator::validate_manifest;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn execute_validate(path_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(path_str);
    if !path.exists() {
        return Err(format!("Path '{}' does not exist.", path_str).into());
    }

    let mut files_to_validate = Vec::new();

    if path.is_file() {
        files_to_validate.push(path.to_path_buf());
    } else if path.is_dir() {
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() {
                if let Some(ext) = p.extension() {
                    if ext == "yaml" || ext == "yml" {
                        files_to_validate.push(p.to_path_buf());
                    }
                }
            }
        }
    }

    if files_to_validate.is_empty() {
        return Err(format!("No .yaml or .yml manifest files found in '{}'.", path_str).into());
    }

    println!("Validating {} manifest(s)...", files_to_validate.len());
    let mut total_errors = 0;
    let mut passed_count = 0;

    for file in &files_to_validate {
        let display_path = file.display();
        let content = match fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[FAIL] {}: Failed to read file: {}", display_path, e);
                total_errors += 1;
                continue;
            }
        };

        let manifest: WspManifest = match serde_yaml_ng::from_str(&content) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("[FAIL] {}: YAML syntax/structure error: {}", display_path, e);
                total_errors += 1;
                continue;
            }
        };

        match validate_manifest(&manifest) {
            Ok(()) => {
                println!(
                    "[PASS] {} -> {} v{} ({}) [{}]",
                    display_path,
                    manifest.package.id,
                    manifest.package.version,
                    manifest.package.name,
                    manifest.installation.install_type
                );
                passed_count += 1;
            }
            Err(errors) => {
                eprintln!(
                    "[FAIL] {} ({} errors):",
                    display_path,
                    errors.len()
                );
                for err in errors {
                    eprintln!("  - {}", err);
                }
                total_errors += 1;
            }
        }
    }

    println!("\nValidation Summary: {} passed, {} failed.", passed_count, total_errors);

    if total_errors > 0 {
        Err(format!("Validation failed with {} error(s).", total_errors).into())
    } else {
        Ok(())
    }
}
