//! Lobby-specific error types

/// Errors that can occur during lobby operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LobbyError {
    /// User not found in lobby
    UserNotFound,
    /// Invalid public key format
    InvalidPublicKey,
    /// Failed to acquire lock (concurrency issue)
    LockFailed,
    /// Network/broadcast failure
    BroadcastFailed,
    /// Lobby has reached maximum capacity
    LobbyFull,
}

impl std::fmt::Display for LobbyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LobbyError::UserNotFound => write!(f, "User not found in lobby"),
            LobbyError::InvalidPublicKey => write!(f, "Invalid public key format"),
            LobbyError::LockFailed => write!(f, "Failed to acquire lobby lock"),
            LobbyError::BroadcastFailed => write!(f, "Failed to broadcast to users"),
            LobbyError::LobbyFull => write!(f, "Lobby is full"),
        }
    }
}

impl std::error::Error for LobbyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lobby_error_display() {
        assert_eq!(LobbyError::UserNotFound.to_string(), "User not found in lobby");
        assert_eq!(LobbyError::InvalidPublicKey.to_string(), "Invalid public key format");
        assert_eq!(LobbyError::LockFailed.to_string(), "Failed to acquire lobby lock");
        assert_eq!(LobbyError::BroadcastFailed.to_string(), "Failed to broadcast to users");
    }

    #[test]
    fn test_lobby_error_equality() {
        assert_eq!(LobbyError::UserNotFound, LobbyError::UserNotFound);
        assert_ne!(LobbyError::UserNotFound, LobbyError::InvalidPublicKey);
    }
}