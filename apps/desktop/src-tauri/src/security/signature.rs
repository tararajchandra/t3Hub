use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SignatureError {
    #[error("Invalid public key hex encoding: {0}")]
    InvalidPublicKeyHex(String),

    #[error("Invalid signature hex encoding: {0}")]
    InvalidSignatureHex(String),

    #[error("Public key parsing failed: {0}")]
    PublicKeyFormat(String),

    #[error("Signature parsing failed: {0}")]
    SignatureFormat(String),

    #[error("Cryptographic signature verification failed: signature does not match payload")]
    VerificationMismatch,
}

pub struct SignatureVerifier;

impl SignatureVerifier {
    /// Verify an Ed25519 signature over a payload given a 64-character (32-byte) hex public key
    pub fn verify(
        public_key_hex: &str,
        message: &[u8],
        signature_hex: &str,
    ) -> Result<(), SignatureError> {
        let pubkey_bytes = hex::decode(public_key_hex.trim())
            .map_err(|e| SignatureError::InvalidPublicKeyHex(e.to_string()))?;

        if pubkey_bytes.len() != 32 {
            return Err(SignatureError::PublicKeyFormat(format!(
                "Ed25519 public key must be 32 bytes (64 hex characters), got {} bytes",
                pubkey_bytes.len()
            )));
        }

        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&pubkey_bytes);

        let verifying_key = VerifyingKey::from_bytes(&key_array)
            .map_err(|e| SignatureError::PublicKeyFormat(e.to_string()))?;

        let sig_bytes = hex::decode(signature_hex.trim())
            .map_err(|e| SignatureError::InvalidSignatureHex(e.to_string()))?;

        if sig_bytes.len() != 64 {
            return Err(SignatureError::SignatureFormat(format!(
                "Ed25519 signature must be 64 bytes (128 hex characters), got {} bytes",
                sig_bytes.len()
            )));
        }

        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&sig_bytes);

        let signature = Signature::from_bytes(&sig_array);

        verifying_key
            .verify_strict(message, &signature)
            .map_err(|_| SignatureError::VerificationMismatch)?;

        Ok(())
    }

    /// Sign a message using a 32-byte secret key hex (used for generating signed repository fixtures)
    pub fn sign(secret_key_hex: &str, message: &[u8]) -> Result<String, SignatureError> {
        let secret_bytes = hex::decode(secret_key_hex.trim())
            .map_err(|e| SignatureError::InvalidPublicKeyHex(e.to_string()))?;

        if secret_bytes.len() != 32 {
            return Err(SignatureError::PublicKeyFormat(
                "Ed25519 secret key must be 32 bytes".to_string(),
            ));
        }

        let mut secret_array = [0u8; 32];
        secret_array.copy_from_slice(&secret_bytes);

        let signing_key = SigningKey::from_bytes(&secret_array);
        let signature: Signature = signing_key.sign(message);

        Ok(hex::encode(signature.to_bytes()))
    }

    /// Derive the public key hex from a 32-byte secret key hex
    pub fn get_public_key(secret_key_hex: &str) -> Result<String, SignatureError> {
        let secret_bytes = hex::decode(secret_key_hex.trim())
            .map_err(|e| SignatureError::InvalidPublicKeyHex(e.to_string()))?;

        if secret_bytes.len() != 32 {
            return Err(SignatureError::PublicKeyFormat(
                "Ed25519 secret key must be 32 bytes".to_string(),
            ));
        }

        let mut secret_array = [0u8; 32];
        secret_array.copy_from_slice(&secret_bytes);

        let signing_key = SigningKey::from_bytes(&secret_array);
        let verifying_key = signing_key.verifying_key();

        Ok(hex::encode(verifying_key.to_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_roundtrip_verification() {
        // Deterministic 32-byte test secret key
        let secret_key_hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let public_key_hex = SignatureVerifier::get_public_key(secret_key_hex).unwrap();
        assert_eq!(public_key_hex.len(), 64);

        let message = b"T3 Hub official repository index v1.0";
        let signature_hex = SignatureVerifier::sign(secret_key_hex, message).unwrap();
        assert_eq!(signature_hex.len(), 128);

        // Verification should succeed
        let result = SignatureVerifier::verify(&public_key_hex, message, &signature_hex);
        assert!(result.is_ok(), "Valid signature must verify successfully");
    }

    #[test]
    fn test_signature_tampered_payload_rejected() {
        let secret_key_hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let public_key_hex = SignatureVerifier::get_public_key(secret_key_hex).unwrap();

        let original_message = b"Clean payload data";
        let tampered_message = b"Tampered payload data";

        let signature_hex = SignatureVerifier::sign(secret_key_hex, original_message).unwrap();

        let result = SignatureVerifier::verify(&public_key_hex, tampered_message, &signature_hex);
        assert!(result.is_err(), "Tampered message must be rejected");
        match result.unwrap_err() {
            SignatureError::VerificationMismatch => (),
            e => panic!("Expected VerificationMismatch, got: {:?}", e),
        }
    }

    #[test]
    fn test_signature_invalid_pubkey_format() {
        let bad_pubkey = "too_short";
        let result = SignatureVerifier::verify(bad_pubkey, b"hello", "abcd");
        assert!(result.is_err());
    }
}
