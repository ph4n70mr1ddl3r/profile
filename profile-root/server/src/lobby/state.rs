use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use profile_shared::Message;
use hex;

/// Type alias for public keys for clarity and type safety
/// This is exported for use in routing (Story 3.2)
pub type PublicKey = String;

/// Represents an active WebSocket connection for a user in the lobby
#[derive(Debug, Clone)]
#[must_use]
pub struct ActiveConnection {
    pub public_key: PublicKey,
    pub sender: mpsc::UnboundedSender<Message>,
    pub connection_id: u64, // For testing reconnection
}

/// Thread-safe lobby that tracks all currently authenticated users
/// Uses Arc<RwLock<T>> pattern for concurrent read/write access:
/// - Arc: allows multiple threads to hold references to the lobby
/// - RwLock: multiple readers can access simultaneously, exclusive writer for modifications  
/// - HashMap: O(1) lookup for message routing (critical for performance)
#[derive(Debug, Clone)]
pub struct Lobby {
    pub users: Arc<RwLock<HashMap<PublicKey, ActiveConnection>>>,
}

impl Lobby {
    /// Create a new empty lobby
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a user to the lobby
    pub fn add_user(&self, connection: ActiveConnection) -> Result<(), String> {
        let mut users = self.users.write().map_err(|_| "Failed to acquire write lock")?;
        users.insert(connection.public_key.clone(), connection);
        Ok(())
    }

    /// Add user with Connection type (compatibility method for existing code)
    pub fn add_user_connection(&self, connection: crate::lobby::Connection) -> Result<(), String> {
        let key_str = hex::encode(&connection.public_key);
        let (sender, _) = mpsc::unbounded_channel::<Message>();
        let active_connection = ActiveConnection {
            public_key: key_str,
            sender,
            connection_id: 0, // Default ID for compatibility
        };
        self.add_user(active_connection)
    }

    /// Remove a user from the lobby
    pub fn remove_user(&self, public_key: &PublicKey) -> Result<(), String> {
        let mut users = self.users.write().map_err(|_| "Failed to acquire write lock")?;
        users.remove(public_key);
        Ok(())
    }

    /// Get the full lobby state as public keys
    pub fn get_full_lobby_state(&self) -> Result<Vec<String>, String> {
        let users = self.users.read().map_err(|_| "Failed to acquire read lock")?;
        let online_users: Vec<String> = users.keys().cloned().collect();
        Ok(online_users)
    }

    /// Check if a user is in the lobby
    pub fn user_exists(&self, public_key: &PublicKey) -> Result<bool, String> {
        let users = self.users.read().map_err(|_| "Failed to acquire read lock")?;
        Ok(users.contains_key(public_key))
    }

    /// Get the number of online users
    pub fn user_count(&self) -> Result<usize, String> {
        let users = self.users.read().map_err(|_| "Failed to acquire read lock")?;
        Ok(users.len())
    }

    /// Get a specific user's connection (for broadcasting)
    pub fn get_connection(&self, public_key: &PublicKey) -> Result<Option<ActiveConnection>, String> {
        let users = self.users.read().map_err(|_| "Failed to acquire read lock")?;
        Ok(users.get(public_key).map(|conn| conn.clone()))
    }

    /// Get all current connections (for broadcasting)
    pub fn get_all_connections(&self) -> Result<Vec<ActiveConnection>, String> {
        let users = self.users.read().map_err(|_| "Failed to acquire read lock")?;
        Ok(users.values().cloned().collect())
    }

    // Compatibility methods for existing code that uses Vec<u8> public keys
    
    /// Remove user by Vec<u8> key (compatibility method)
    pub fn remove_user_vec(&self, public_key: &[u8]) -> Result<(), String> {
        let key_str = hex::encode(public_key);
        self.remove_user(&key_str)
    }

    /// Check if user exists by Vec<u8> key (compatibility method)  
    pub fn user_exists_vec(&self, public_key: &[u8]) -> Result<bool, String> {
        let key_str = hex::encode(public_key);
        self.user_exists(&key_str)
    }

    /// Get full lobby state as Vec<u8> keys (compatibility method)
    pub fn get_full_lobby_state_vec(&self) -> Result<Vec<Vec<u8>>, String> {
        let users = self.users.read().map_err(|_| "Failed to acquire read lock")?;
        let online_users: Vec<Vec<u8>> = users.keys()
            .map(|key| hex::decode(key).map_err(|_| "Invalid hex key".to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(online_users)
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

    #[test]
    fn test_public_key_type_alias() {
        let key: PublicKey = "test_key".to_string();
        assert_eq!(key, "test_key");
    }

    #[test]
    fn test_active_connection_struct_construction() {
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

    #[test]
    fn test_lobby_creation() {
        let lobby = Lobby::new();
        assert_eq!(lobby.user_count().unwrap(), 0);
        assert!(lobby.get_full_lobby_state().unwrap().is_empty());
    }

    #[test]
    fn test_add_and_remove_user() {
        let lobby = Lobby::new();
        let public_key = "test_key".to_string();
        
        // Test basic lobby state before adding user
        assert_eq!(lobby.user_count().unwrap(), 0);
        assert!(!lobby.user_exists(&public_key).unwrap());
        
        // Add user
        let (sender, _) = mpsc::unbounded_channel::<Message>();
        let connection = ActiveConnection {
            public_key: public_key.clone(),
            sender,
            connection_id: 1,
        };
        
        lobby.add_user(connection).unwrap();
        assert_eq!(lobby.user_count().unwrap(), 1);
        assert!(lobby.user_exists(&public_key).unwrap());
        
        // Remove user
        lobby.remove_user(&public_key).unwrap();
        assert_eq!(lobby.user_count().unwrap(), 0);
        assert!(!lobby.user_exists(&public_key).unwrap());
    }

    #[test]
    fn test_arc_rwlock_thread_safety_pattern() {
        let lobby = Lobby::new();
        
        // Test that we can clone Arc and access from multiple threads
        let lobby_clone = lobby.clone();
        let handle = std::thread::spawn(move || {
            lobby_clone.user_count().unwrap()
        });
        
        let result = handle.join().unwrap();
        assert_eq!(result, 0); // Empty lobby has 0 users
    }
}