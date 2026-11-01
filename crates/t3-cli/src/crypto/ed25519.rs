use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Failed to generate secure random bytes: {0}")]
    EntropyError(String),

    #[error("Hex decode error: {0}")]
    HexError(String),

    #[error("Invalid key format: {0}")]
    KeyFormatError(String),

    #[error("Invalid signature format: {0}")]
    SignatureFormatError(String),

    #[error("Cryptographic signature verification failed")]
    VerificationFailed,
}

/// Generates a new cryptographically secure Ed25519 keypair.
/// Returns (secret_key_hex, public_key_hex) as 64-character hex strings.
pub fn generate_keypair() -> Result<(String, String), CryptoError> {
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed)
        .map_err(|e| CryptoError::EntropyError(e.to_string()))?;

    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();

    let secret_hex = hex::encode(signing_key.to_bytes());
    let public_hex = hex::encode(verifying_key.to_bytes());

    Ok((secret_hex, public_hex))
}

/// Derives the 32-byte public key hex from a 32-byte secret key hex.
#[allow(dead_code)]
pub fn derive_public_key(secret_key_hex: &str) -> Result<String, CryptoError> {
    let secret_bytes = hex::decode(secret_key_hex.trim())
        .map_err(|e| CryptoError::HexError(e.to_string()))?;

    if secret_bytes.len() != 32 {
        return Err(CryptoError::KeyFormatError(format!(
            "Secret key must be 32 bytes (64 hex characters), got {} bytes",
            secret_bytes.len()
        )));
    }

    let mut secret_array = [0u8; 32];
    secret_array.copy_from_slice(&secret_bytes);

    let signing_key = SigningKey::from_bytes(&secret_array);
    let verifying_key = signing_key.verifying_key();

    Ok(hex::encode(verifying_key.to_bytes()))
}

/// Signs a message using a 32-byte secret key hex.
/// Returns a 64-byte Ed25519 signature as a 128-character hex string.
pub fn sign_bytes(secret_key_hex: &str, message: &[u8]) -> Result<String, CryptoError> {
    let secret_bytes = hex::decode(secret_key_hex.trim())
        .map_err(|e| CryptoError::HexError(e.to_string()))?;

    if secret_bytes.len() != 32 {
        return Err(CryptoError::KeyFormatError(format!(
            "Secret key must be 32 bytes (64 hex characters), got {} bytes",
            secret_bytes.len()
        )));
    }

    let mut secret_array = [0u8; 32];
    secret_array.copy_from_slice(&secret_bytes);

    let signing_key = SigningKey::from_bytes(&secret_array);
    let signature: Signature = signing_key.sign(message);

    Ok(hex::encode(signature.to_bytes()))
}

/// Verifies an Ed25519 detached signature against a message and 32-byte public key hex.
pub fn verify_bytes(
    public_key_hex: &str,
    message: &[u8],
    signature_hex: &str,
) -> Result<(), CryptoError> {
    let pubkey_bytes = hex::decode(public_key_hex.trim())
        .map_err(|e| CryptoError::HexError(e.to_string()))?;

    if pubkey_bytes.len() != 32 {
        return Err(CryptoError::KeyFormatError(format!(
            "Public key must be 32 bytes (64 hex characters), got {} bytes",
            pubkey_bytes.len()
        )));
    }

    let mut pub_array = [0u8; 32];
    pub_array.copy_from_slice(&pubkey_bytes);

    let verifying_key = VerifyingKey::from_bytes(&pub_array)
        .map_err(|e| CryptoError::KeyFormatError(e.to_string()))?;

    let sig_bytes = hex::decode(signature_hex.trim())
        .map_err(|e| CryptoError::HexError(e.to_string()))?;

    if sig_bytes.len() != 64 {
        return Err(CryptoError::SignatureFormatError(format!(
            "Signature must be 64 bytes (128 hex characters), got {} bytes",
            sig_bytes.len()
        )));
    }

    let mut sig_array = [0u8; 64];
    sig_array.copy_from_slice(&sig_bytes);

    let signature = Signature::from_bytes(&sig_array);

    verifying_key
        .verify_strict(message, &signature)
        .map_err(|_| CryptoError::VerificationFailed)?;

    Ok(())
}
