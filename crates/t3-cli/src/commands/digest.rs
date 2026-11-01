use crate::crypto::hashing::compute_file_sha256;
use std::path::Path;

pub fn execute_digest(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File '{}' not found.", file_path).into());
    }
    if !path.is_file() {
        return Err(format!("'{}' is not a file.", file_path).into());
    }

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("artifact");

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("exe")
        .to_lowercase();

    let art_type = match ext.as_str() {
        "msi" => "msi",
        "zip" => "zip",
        _ => "exe",
    };

    println!("Computing SHA-256 digest for: {}", file_path);
    let (sha256, size_bytes) = compute_file_sha256(path)?;
    let size_mb = (size_bytes as f64) / (1024.0 * 1024.0);

    println!("\nFile Details:");
    println!("  Path:        {}", path.display());
    println!("  Size:        {} bytes ({:.2} MB)", size_bytes, size_mb);
    println!("  SHA-256:     {}", sha256);
    println!("  Detected:    {}", art_type);

    println!("\nYAML Artifact Snippet (paste into manifest):");
    println!("------------------------------------------------------------");
    println!("artifacts:");
    println!("  - id: \"windows-x64-{}\"", art_type);
    println!("    type: \"{}\"", art_type);
    println!("    url: \"https://example.com/releases/{}\"", file_name);
    println!("    sha256: \"{}\"", sha256);
    println!("    size: {}", size_bytes);
    println!("------------------------------------------------------------");

    Ok(())
}
