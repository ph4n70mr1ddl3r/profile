#[cfg(test)]
mod lobby_state_isolated_tests {
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    use tokio::sync::mpsc;
    use profile_shared::Message;

    // Copy the types here for isolated testing
    pub type PublicKey = String;

    #[derive(Debug, Clone)]
    #[must_use]
    pub struct ActiveConnection {
        pub public_key: PublicKey,
        pub sender: mpsc::UnboundedSender<Message>,
    }

    #[derive(Debug, Clone)]
    pub struct Lobby {
        users: Arc<RwLock<HashMap<PublicKey, ActiveConnection>>>,
    }

    impl Lobby {
        pub fn new() -> Self {
            Self {
                users: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        pub fn add_user(&self, connection: ActiveConnection) -> Result<(), String> {
            let mut users = self.users.write().map_err(|_| "Failed to acquire write lock")?;
            users.insert(connection.public_key.clone(), connection);
            Ok(())
        }

        pub fn remove_user(&self, public_key: &PublicKey) -> Result<(), String> {
            let mut users = self.users.write().map_err(|_| "Failed to acquire write lock")?;
            users.remove(public_key);
            Ok(())
        }

        pub fn get_full_lobby_state(&self) -> Result<Vec<String>, String> {
            let users = self.users.read().map_err(|_| "Failed to acquire read lock")?;
            let online_users: Vec<String> = users.keys().cloned().collect();
            Ok(online_users)
        }

        pub fn user_count(&self) -> Result<usize, String> {
            let users = self.users.read().map_err(|_| "Failed to acquire read lock")?;
            Ok(users.len())
        }
    }

    #[test]
    fn test_lobby_state_implementation() {
        let lobby = Lobby::new();
        
        // Test initial state
        assert_eq!(lobby.user_count().unwrap(), 0);
        assert!(lobby.get_full_lobby_state().unwrap().is_empty());
        
        // Create a test user
        let (sender, _) = mpsc::unbounded_channel::<Message>();
        let user_key = "test_user_123".to_string();
        let connection = ActiveConnection {
            public_key: user_key.clone(),
            sender,
        };
        
        // Add user
        lobby.add_user(connection).unwrap();
        assert_eq!(lobby.user_count().unwrap(), 1);
        
        let state = lobby.get_full_lobby_state().unwrap();
        assert_eq!(state.len(), 1);
        assert!(state.contains(&user_key));
        
        // Remove user
        lobby.remove_user(&user_key).unwrap();
        assert_eq!(lobby.user_count().unwrap(), 0);
        assert!(lobby.get_full_lobby_state().unwrap().is_empty());
    }

    #[test]
    fn test_public_key_type_alias() {
        let key: PublicKey = "test_key".to_string();
        assert_eq!(key, "test_key");
    }
}