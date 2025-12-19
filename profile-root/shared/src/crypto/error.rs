use std::fmt;

#[derive(Debug, Clone)]
pub enum CryptoError {
    KeyGenerationFailed(String),
    InvalidKeyFormat(String),
    DerivationFailed(String),
    SigningFailed(String),
    VerificationFailed(String),
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::KeyGenerationFailed(msg) => write!(f, "Key generation failed: {}", msg),
            CryptoError::InvalidKeyFormat(msg) => write!(f, "Invalid key format: {}", msg),
            CryptoError::DerivationFailed(msg) => write!(f, "Key derivation failed: {}", msg),
            CryptoError::SigningFailed(msg) => write!(f, "Message signing failed: {}", msg),
            CryptoError::VerificationFailed(msg) => write!(f, "Signature verification failed: {}", msg),
        }
    }
}

impl std::error::Error for CryptoError {}
