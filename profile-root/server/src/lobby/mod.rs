//! User lobby management for tracking online users
//!
//! This module provides thread-safe lobby management for tracking
//! authenticated users and their WebSocket connections.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use hex;

/// WebSocket connection wrapper for lobby management
#[derive(Debug, Clone)]
pub struct Connection {
    pub public_key: Vec<u8>,
    pub connected_at: std::time::Instant,
    // TODO: Add WebSocket stream reference when implementing actual connections
}

/// Thread-safe lobby manager for tracking online users
///
/// Uses Arc<RwLock<HashMap<PublicKey, Connection>>> for concurrent access
#[derive(Debug, Clone)]
pub struct Lobby {
    users: Arc<RwLock<HashMap<Vec<u8>, Connection>>>,
}

impl Lobby {
    /// Create a new empty lobby
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a user to the lobby
    pub fn add_user(&self, connection: Connection) -> Result<(), String> {
        let mut users = self.users.write().map_err(|_| "Failed to acquire write lock")?;
        users.insert(connection.public_key.clone(), connection);
        Ok(())
    }

    /// Remove a user from the lobby
    pub fn remove_user(&self, public_key: &[u8]) -> Result<(), String> {
        let mut users = self.users.write().map_err(|_| "Failed to acquire write lock")?;
        users.remove(public_key);
        Ok(())
    }

    /// Get the full lobby state as hex-encoded public keys
    pub fn get_full_lobby_state(&self) -> Result<Vec<String>, String> {
        let users = self.users.read().map_err(|_| "Failed to acquire read lock")?;
        let online_users: Vec<String> = users.keys()
            .map(|key| hex::encode(key))
            .collect();
        Ok(online_users)
    }

    /// Check if a user is in the lobby
    pub fn user_exists(&self, public_key: &[u8]) -> Result<bool, String> {
        let users = self.users.read().map_err(|_| "Failed to acquire read lock")?;
        Ok(users.contains_key(public_key))
    }

    /// Get the number of online users
    pub fn user_count(&self) -> Result<usize, String> {
        let users = self.users.read().map_err(|_| "Failed to acquire read lock")?;
        Ok(users.len())
    }

    /// Get a specific user's connection (for broadcasting)
    pub fn get_connection(&self, public_key: &[u8]) -> Result<Option<Connection>, String> {
        let users = self.users.read().map_err(|_| "Failed to acquire read lock")?;
        Ok(users.get(public_key).cloned())
    }

    /// Get all current connections (for broadcasting)
    pub fn get_all_connections(&self) -> Result<Vec<Connection>, String> {
        let users = self.users.read().map_err(|_| "Failed to acquire read lock")?;
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

    #[test]
    fn test_lobby_creation() {
        let lobby = Lobby::new();
        assert_eq!(lobby.user_count().unwrap(), 0);
        assert!(lobby.get_full_lobby_state().unwrap().is_empty());
    }

    #[test]
    fn test_add_and_remove_user() {
        let lobby = Lobby::new();
        let public_key = vec![1, 2, 3, 4];
        
        // Test basic lobby state before adding user
        assert_eq!(lobby.user_count().unwrap(), 0);
        assert!(!lobby.user_exists(&public_key).unwrap());
        
        // Add user
        let connection = Connection {
            public_key: public_key.clone(),
            connected_at: std::time::Instant::now(),
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
    fn test_full_lobby_state() {
        let lobby = Lobby::new();
        
        // Test empty lobby
        let state = lobby.get_full_lobby_state().unwrap();
        assert!(state.is_empty());
        
        // Add users and test state
        let key1 = vec![1, 2, 3, 4];
        let key2 = vec![5, 6, 7, 8];
        
        lobby.add_user(Connection {
            public_key: key1.clone(),
            connected_at: std::time::Instant::now(),
        }).unwrap();
        
        lobby.add_user(Connection {
            public_key: key2.clone(),
            connected_at: std::time::Instant::now(),
        }).unwrap();
        
        let state = lobby.get_full_lobby_state().unwrap();
        assert_eq!(state.len(), 2);
        assert!(state.contains(&hex::encode(key1)));
        assert!(state.contains(&hex::encode(key2)));
    }

    #[test]
    fn test_arc_rwlock_thread_safety() {
        let lobby = Lobby::new();
        
        // Test that multiple threads can read concurrently
        let handles: Vec<_> = (0..10).map(|_| {
            let lobby_clone = lobby.clone();
            std::thread::spawn(move || {
                lobby_clone.user_count().unwrap();
                lobby_clone.get_full_lobby_state().unwrap();
            })
        }).collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Test concurrent write access
        let lobby_clone = lobby.clone();
        let handle = std::thread::spawn(move || {
            let key = vec![42u8; 4];
            lobby_clone.add_user(Connection {
                public_key: key,
                connected_at: std::time::Instant::now(),
            })
        });
        
        let result = handle.join().unwrap();
        assert!(result.is_ok());
        assert_eq!(lobby.user_count().unwrap(), 1);
    }
}