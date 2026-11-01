use crate::crypto::ed25519::sign_bytes;
use crate::manifest::models::{IndexPackage, IndexVersion, RepositoryIndex, WspManifest};
use crate::manifest::validator::validate_manifest;
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct IndexGenOptions {
    pub repo_dir: String,
    pub repo_id: Option<String>,
    pub output: Option<String>,
    pub sign_key: Option<String>,
}

pub fn execute_index_generate(opts: IndexGenOptions) -> Result<(), Box<dyn std::error::Error>> {
    let repo_path = Path::new(&opts.repo_dir);
    if !repo_path.exists() {
        return Err(format!("Repository directory '{}' does not exist.", opts.repo_dir).into());
    }

    let manifests_dir = repo_path.join("manifests");
    if !manifests_dir.exists() {
        return Err(format!(
            "Repository directory '{}' is missing a 'manifests' subfolder.",
            opts.repo_dir
        )
        .into());
    }

    // Try reading repository.json to detect default repo_id
    let repo_id = if let Some(id) = opts.repo_id {
        id
    } else {
        let repo_json_path = repo_path.join("repository.json");
        if repo_json_path.exists() {
            let content = fs::read_to_string(&repo_json_path)?;
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                v.get("id")
                    .and_then(|i| i.as_str())
                    .unwrap_or("community-repo")
                    .to_string()
            } else {
                "community-repo".to_string()
            }
        } else {
            "community-repo".to_string()
        }
    };

    println!("Scanning manifests in: {}", manifests_dir.display());

    // Map: package_id -> (Vec<WspManifest>, Vec<String relative_paths>)
    let mut packages_map: HashMap<String, Vec<(WspManifest, String)>> = HashMap::new();
    let mut count = 0;

    for entry in WalkDir::new(&manifests_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "yaml" || ext == "yml" {
                    let content = fs::read_to_string(path)?;
                    let manifest: WspManifest = match serde_yaml_ng::from_str(&content) {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("[WARN] Skipping malformed YAML {}: {}", path.display(), e);
                            continue;
                        }
                    };

                    if let Err(errors) = validate_manifest(&manifest) {
                        eprintln!(
                            "[WARN] Skipping invalid manifest {}: {} validation error(s)",
                            path.display(),
                            errors.len()
                        );
                        continue;
                    }

                    // Compute relative path from repo_dir, normalized with forward slashes
                    let rel_path = path
                        .strip_prefix(repo_path)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .replace('\\', "/");

                    packages_map
                        .entry(manifest.package.id.clone())
                        .or_default()
                        .push((manifest, rel_path));
                    count += 1;
                }
            }
        }
    }

    println!("Found {} valid manifests across {} package(s).", count, packages_map.len());

    let mut index_packages = Vec::new();

    for (pkg_id, mut manifests_list) in packages_map {
        // Sort versions by SemVer
        manifests_list.sort_by(|a, b| {
            let va = semver::Version::parse(&a.0.package.version).unwrap_or(semver::Version::new(0, 0, 0));
            let vb = semver::Version::parse(&b.0.package.version).unwrap_or(semver::Version::new(0, 0, 0));
            va.cmp(&vb)
        });

        // The latest version is the last in ascending order
        let (latest_manifest, latest_rel_path) = manifests_list.last().unwrap().clone();

        let mut versions = Vec::new();
        for (m, rel_path) in &manifests_list {
            versions.push(IndexVersion {
                version: m.package.version.clone(),
                manifest_path: rel_path.clone(),
                release_date: m.release.as_ref().and_then(|r| r.release_date.clone()),
            });
        }

        // Guess icon URL if not explicitly set
        let icon_url = None;

        index_packages.push(IndexPackage {
            id: pkg_id,
            name: latest_manifest.package.name.clone(),
            publisher: latest_manifest.package.publisher.clone(),
            latest_version: latest_manifest.package.version.clone(),
            description: latest_manifest
                .package
                .description
                .clone()
                .unwrap_or_default(),
            categories: latest_manifest.package.categories.clone(),
            tags: latest_manifest.package.tags.clone(),
            icon_url,
            manifest_path: latest_rel_path,
            versions,
        });
    }

    // Sort packages alphabetically by ID
    index_packages.sort_by(|a, b| a.id.cmp(&b.id));

    let index = RepositoryIndex {
        repository_id: repo_id,
        version: 1,
        generated_at: Utc::now().to_rfc3339(),
        packages: index_packages,
    };

    let index_json = serde_json::to_string_pretty(&index)?;

    let output_path = match opts.output {
        Some(ref out) => PathBuf::from(out),
        None => repo_path.join("index.json"),
    };

    fs::write(&output_path, &index_json)?;
    println!("[SUCCESS] Generated repository index: {}", output_path.display());

    // Optional signing
    if let Some(ref key) = opts.sign_key {
        let sig = sign_bytes(key, index_json.as_bytes())?;
        let sig_path = output_path.with_extension("json.sig");
        fs::write(&sig_path, &sig)?;
        println!("[SUCCESS] Signed index generated: {}", sig_path.display());
    }

    Ok(())
}
