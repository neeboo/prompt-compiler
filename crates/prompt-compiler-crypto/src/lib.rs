//! Cryptographic module - Hashing and signatures

use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Hash value
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash(pub String);

/// Signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

/// Cryptographic errors
#[derive(thiserror::Error, Debug)]
pub enum CryptoError {
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, CryptoError>;

impl Hash {
    /// Compute hash from data
    pub fn from_data(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        Hash(hex::encode(result))
    }

    /// Create hash from string
    pub fn from_string(data: &str) -> Self {
        Self::from_data(data.as_bytes())
    }

    /// Get hash string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get short hash (first 8 characters)
    pub fn short(&self) -> String {
        self.0.chars().take(8).collect()
    }
}

impl Signature {
    /// Create new signature
    pub fn sign(data: &[u8], signing_key: &SigningKey) -> Self {
        let signature = signing_key.sign(data);
        let verifying_key = signing_key.verifying_key();
        Self {
            signature: signature.to_bytes().to_vec(),
            public_key: verifying_key.to_bytes().to_vec(),
        }
    }

    /// Verify signature
    pub fn verify(&self, data: &[u8]) -> Result<bool> {
        let public_key_bytes: [u8; 32] =
            self.public_key.as_slice().try_into().map_err(|_| {
                CryptoError::InvalidPublicKey("Invalid public key length".to_string())
            })?;

        let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
            .map_err(|e| CryptoError::InvalidPublicKey(format!("{}", e)))?;

        let signature_bytes: [u8; 64] =
            self.signature.as_slice().try_into().map_err(|_| {
                CryptoError::InvalidSignature("Invalid signature length".to_string())
            })?;

        let signature = Ed25519Signature::from_bytes(&signature_bytes);

        match verifying_key.verify(data, &signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// Generate new signing key
pub fn generate_signing_key() -> SigningKey {
    use rand::rngs::OsRng;
    SigningKey::generate(&mut OsRng)
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
