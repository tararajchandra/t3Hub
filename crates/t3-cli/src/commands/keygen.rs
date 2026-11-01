use crate::crypto::ed25519::generate_keypair;
use chrono::Utc;
use serde_json::json;
use std::fs;
use std::path::Path;

pub fn execute_keygen(output_file: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let (secret_key_hex, public_key_hex) = generate_keypair()?;

    println!("============================================================");
    println!("          T3 Hub Cryptographic Keypair Generated            ");
    println!("============================================================");
    println!("\nPublic Key (Embed in repository.json):");
    println!("  {}", public_key_hex);
    println!("\nSecret Signing Key (Keep private! Never commit to git!):");
    println!("  {}", secret_key_hex);
    println!("\nAlgorithm: Ed25519 (32-byte public key / 64-byte signature)");

    if let Some(out) = output_file {
        let key_data = json!({
            "algorithm": "Ed25519",
            "public_key": public_key_hex,
            "secret_key": secret_key_hex,
            "generated_at": Utc::now().to_rfc3339(),
            "warning": "DO NOT SHARE OR COMMIT THE SECRET KEY. Used to sign repository index and manifests."
        });

        let out_path = Path::new(out);
        if let Some(parent) = out_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::write(out_path, serde_json::to_string_pretty(&key_data)?)?;
        println!("\nKeypair safely saved to: {}", out);
    }

    Ok(())
}
