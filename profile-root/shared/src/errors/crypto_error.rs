use std::fmt;

#[derive(Debug, Clone)]
pub enum CryptoError {
    KeyGenerationFailed(String),
    InvalidKeyFormat(String),
    DerivationFailed(String),
    SigningFailed(String),
    VerificationFailed(String),
    InvalidKey(String),
    InvalidSignature(String),
    SerializationError(String),
    ConnectionLost(String),
    ServerDisconnected(String),
    TimeoutError(String),
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::KeyGenerationFailed(msg) => write!(f, "Key generation failed: {}", msg),
            CryptoError::InvalidKeyFormat(msg) => write!(f, "Invalid key format: {}", msg),
            CryptoError::DerivationFailed(msg) => write!(f, "Key derivation failed: {}", msg),
            CryptoError::SigningFailed(msg) => write!(f, "Message signing failed: {}", msg),
            CryptoError::VerificationFailed(msg) => write!(f, "Signature verification failed: {}", msg),
            CryptoError::InvalidKey(msg) => write!(f, "Invalid key: {}", msg),
            CryptoError::InvalidSignature(msg) => write!(f, "Invalid signature: {}", msg),
            CryptoError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            CryptoError::ConnectionLost(msg) => write!(f, "Connection lost: {}", msg),
            CryptoError::ServerDisconnected(msg) => write!(f, "Server disconnected: {}", msg),
            CryptoError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
        }
    }
}

impl std::error::Error for CryptoError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_error_display_messages() {
        // Test ConnectionLost
        let error = CryptoError::ConnectionLost("Network issue".to_string());
        assert_eq!(error.to_string(), "Connection lost: Network issue");

        // Test ServerDisconnected
        let error = CryptoError::ServerDisconnected("Server shutdown".to_string());
        assert_eq!(error.to_string(), "Server disconnected: Server shutdown");

        // Test TimeoutError
        let error = CryptoError::TimeoutError("Connection timed out".to_string());
        assert_eq!(error.to_string(), "Timeout error: Connection timed out");
    }
}