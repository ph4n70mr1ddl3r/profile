use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use profile_shared::{Message, LobbyError};

/// Maximum number of users allowed in the lobby at once.
///
/// This limit prevents memory exhaustion DoS attacks where an attacker
/// could rapidly create many connections to exhaust server memory.
/// Value is fixed at 10,000 users for MVP - can be made configurable in future phases.
pub const MAX_LOBBY_SIZE: usize = 10_000;

/// Type alias for public keys for clarity and type safety
/// This is exported for use in routing (Story 3.2)
pub type PublicKey = String;

/// Represents an active WebSocket connection for a user in the lobby
#[derive(Debug, Clone)]
#[must_use]
pub struct ActiveConnection {
    pub public_key: PublicKey,
    pub sender: mpsc::UnboundedSender<Message>,
    /// Unique identifier for this connection instance.
    /// Used to track reconnections and verify connection replacement.
    /// Updated when a user reconnects with a new WebSocket connection.
    pub connection_id: u64,
}

/// Thread-safe lobby that tracks all currently authenticated users
/// Uses Arc<RwLock<T>> pattern for concurrent read/write access:
/// - Arc: allows multiple threads to hold references to lobby
/// - RwLock: multiple readers can access simultaneously, exclusive writer for modifications
/// - HashMap: O(1) lookup for message routing (critical for performance)
/// - Arc<ActiveConnection>: Enables efficient shared references without cloning
#[derive(Debug, Clone)]
pub struct Lobby {
    pub users: Arc<RwLock<HashMap<PublicKey, Arc<ActiveConnection>>>>,
}

impl Lobby {
    /// Create a new empty lobby
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a user to lobby (wraps connection in Arc)
    pub async fn add_user(&self, connection: ActiveConnection) -> Result<(), LobbyError> {
        let mut users = self.users.write().await;
        users.insert(connection.public_key.clone(), Arc::new(connection));
        Ok(())
    }

    /// Remove a user from the lobby
    pub async fn remove_user(&self, public_key: &PublicKey) -> Result<(), LobbyError> {
        let mut users = self.users.write().await;
        users.remove(public_key);
        Ok(())
    }

    /// Get full lobby state as public keys
    pub async fn get_full_lobby_state(&self) -> Result<Vec<String>, LobbyError> {
        let users = self.users.read().await;
        let online_users: Vec<String> = users.keys().cloned().collect();
        Ok(online_users)
    }

    /// Check if a user is in lobby
    pub async fn user_exists(&self, public_key: &PublicKey) -> Result<bool, LobbyError> {
        let users = self.users.read().await;
        Ok(users.contains_key(public_key))
    }

    /// Get number of online users
    pub async fn user_count(&self) -> Result<usize, LobbyError> {
        let users = self.users.read().await;
        Ok(users.len())
    }

    /// Get all current connections as Arc wrappers (for broadcasting to all users)
    pub async fn get_all_connections(&self) -> Result<Vec<Arc<ActiveConnection>>, LobbyError> {
        let users = self.users.read().await;
        Ok(users.values().cloned().collect())
    }
}

impl Default for Lobby {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_public_key_type_alias() {
        let key: PublicKey = "test_key".to_string();
        assert_eq!(key, "test_key");
    }

    #[tokio::test]
    async fn test_active_connection_struct_construction() {
        let public_key = "test_key_123".to_string();
        
        // Create mpsc channel for sender
        let (sender, _) = mpsc::unbounded_channel::<Message>();
        
        let connection = ActiveConnection {
            public_key: public_key.clone(),
            sender,
            connection_id: 42,
        };
        
        assert_eq!(connection.public_key, public_key);
        assert_eq!(connection.connection_id, 42);
        // Note: sender field tested in integration tests with actual message sending
    }

    #[tokio::test]
    async fn test_lobby_creation() {
        let lobby = Lobby::new();
        assert_eq!(lobby.user_count().await.unwrap(), 0);
        assert!(lobby.get_full_lobby_state().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_add_and_remove_user() {
        let lobby = Lobby::new();
        let public_key = "test_key".to_string();

        // Test basic lobby state before adding user
        assert_eq!(lobby.user_count().await.unwrap(), 0);
        assert!(!lobby.user_exists(&public_key).await.unwrap());

        // Add user
        let (sender, _) = mpsc::unbounded_channel::<Message>();
        let connection = ActiveConnection {
            public_key: public_key.clone(),
            sender,
            connection_id: 1,
        };

        lobby.add_user(connection).await.unwrap();
        assert_eq!(lobby.user_count().await.unwrap(), 1);
        assert!(lobby.user_exists(&public_key).await.unwrap());

        // Remove user
        lobby.remove_user(&public_key).await.unwrap();
        assert_eq!(lobby.user_count().await.unwrap(), 0);
        assert!(!lobby.user_exists(&public_key).await.unwrap());
    }

    #[tokio::test]
    async fn test_arc_rwlock_thread_safety_pattern() {
        let lobby = Lobby::new();
        
        // Test that we can clone Arc and access from multiple threads
        let lobby_clone = lobby.clone();
        let handle = tokio::spawn(async move {
            lobby_clone.user_count().await.unwrap()
        });
        
        let result = handle.await.unwrap();
        assert_eq!(result, 0); // Empty lobby has 0 users
    }
}