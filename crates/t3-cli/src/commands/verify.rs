use crate::crypto::ed25519::verify_bytes;
use std::fs;
use std::path::Path;

pub fn execute_verify(
    file_path: &str,
    signature_or_sigfile: &str,
    public_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File '{}' not found.", file_path).into());
    }

    let file_bytes = fs::read(path)?;

    // Check if signature argument is a file path
    let signature_hex = if Path::new(signature_or_sigfile).exists() {
        fs::read_to_string(signature_or_sigfile)?
            .trim()
            .to_string()
    } else {
        signature_or_sigfile.trim().to_string()
    };

    match verify_bytes(public_key, &file_bytes, &signature_hex) {
        Ok(()) => {
            println!("[VERIFIED] Cryptographic Ed25519 signature is VALID.");
            println!("  File:       {}", file_path);
            println!("  Public Key: {}", public_key);
            Ok(())
        }
        Err(e) => {
            eprintln!("[REJECTED] Cryptographic signature verification FAILED.");
            eprintln!("  File:       {}", file_path);
            eprintln!("  Error:      {}", e);
            Err("Signature verification failed.".into())
        }
    }
}
