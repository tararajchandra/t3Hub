use crate::crypto::ed25519::sign_bytes;
use std::fs;
use std::path::Path;

pub fn execute_sign(
    file_path: &str,
    secret_key: &str,
    output_file: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File '{}' not found.", file_path).into());
    }

    let file_bytes = fs::read(path)?;
    let signature_hex = sign_bytes(secret_key, &file_bytes)?;

    println!("Signed file: {}", file_path);
    println!("Detached Ed25519 Signature (128 hex chars):");
    println!("{}", signature_hex);

    if let Some(out) = output_file {
        let out_path = Path::new(out);
        if let Some(parent) = out_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(out_path, &signature_hex)?;
        println!("Signature saved to: {}", out);
    }

    Ok(())
}
