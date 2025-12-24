//! Lobby state management - single source of truth for lobby data
//!
//! This module provides the LobbyState struct which manages:
//! - List of users in the lobby (maintains insertion order for deterministic UI)
//! - Selection state (which user is selected for messaging)
//! - User operations (add, remove, select, deselect)
//!
//! All lobby state changes happen through this module to ensure consistency.
//!
//! # Deterministic Order
//!
//! Lobby users are stored in a Vec to preserve insertion order. This ensures:
//! - Keyboard navigation moves predictably (ArrowUp/ArrowDown)
//! - UI displays users in a stable, consistent order
//! - User selection by index is reliable

use serde::{Deserialize, Serialize};

/// Represents a user displayed in the lobby
#[derive(Debug, Clone, PartialEq)]
pub struct LobbyUser {
    /// The user's public key (hex-encoded, full 64-char display)
    pub public_key: String,
    /// Whether the user is currently online
    pub is_online: bool,
}

impl LobbyUser {
    /// Create a new lobby user
    #[inline]
    pub fn new(public_key: String, is_online: bool) -> Self {
        Self { public_key, is_online }
    }
}

/// Serializable lobby user for state persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyUserSerializable {
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub status: String,
}

impl From<LobbyUser> for LobbyUserSerializable {
    fn from(user: LobbyUser) -> Self {
        Self {
            public_key: user.public_key,
            status: if user.is_online { "online".to_string() } else { "offline".to_string() },
        }
    }
}

impl From<LobbyUserSerializable> for LobbyUser {
    fn from(user: LobbyUserSerializable) -> Self {
        Self {
            public_key: user.public_key,
            is_online: user.status == "online",
        }
    }
}

impl From<LobbyStateSerializable> for LobbyState {
    fn from(serializable: LobbyStateSerializable) -> Self {
        let mut state = Self::new();
        let users: Vec<LobbyUser> = serializable.users.into_iter().map(|u| u.into()).collect();
        state.set_users(users);
        state.selected_user = serializable.selected_user;
        state
    }
}

/// Serializable lobby state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyStateSerializable {
    #[serde(rename = "users")]
    pub users: Vec<LobbyUserSerializable>,
    #[serde(rename = "selectedUser")]
    pub selected_user: Option<String>,
}

impl From<LobbyState> for LobbyStateSerializable {
    fn from(state: LobbyState) -> Self {
        let users_serializable: Vec<LobbyUserSerializable> = state.users.into_iter().map(|u| u.into()).collect();
        Self {
            users: users_serializable,
            selected_user: state.selected_user,
        }
    }
}

/// Main lobby state manager
///
/// # Thread Safety
///
/// This struct is not thread-safe on its own. For use across threads,
/// wrap in an `Arc<Mutex<LobbyState>>` or similar synchronization primitive.
///
/// # Deterministic Ordering
///
/// Users are stored in a Vec (not HashMap) to maintain insertion order.
/// This ensures predictable keyboard navigation and consistent UI display.
///
/// # Example
///
/// ```ignore
/// use profile_client::ui::lobby_state::LobbyState;
///
/// let mut state = LobbyState::new();
/// state.add_user(LobbyUser::new("abc123".to_string(), true));
/// state.select("abc123");
/// assert_eq!(state.selected_user(), Some("abc123"));
/// ```
#[derive(Debug, Clone)]
pub struct LobbyState {
    /// Vector of users (maintains insertion order for deterministic UI)
    users: Vec<LobbyUser>,
    /// Currently selected user for messaging (None if no selection)
    selected_user: Option<String>,
}

impl LobbyState {
    /// Create a new empty lobby state
    #[inline]
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            selected_user: None,
        }
    }

    /// Check if a user exists in the lobby
    ///
    /// # Arguments
    ///
    /// * `public_key` - The public key to check (hex-encoded string)
    ///
    /// # Returns
    ///
    /// `true` if user exists, `false` otherwise
    #[inline]
    pub fn has_user(&self, public_key: &str) -> bool {
        self.users.iter().any(|u| u.public_key == public_key)
    }

    /// Set's complete list of users (replaces existing users)
    ///
    /// This performs deduplication - if a user already exists,
    /// they will not be added again.
    ///
    /// # Arguments
    ///
    /// * `users` - Vector of users to set
    pub fn set_users(&mut self, users: Vec<LobbyUser>) {
        self.users.clear();
        for user in users {
            // Deduplicate: only insert if not already present
            if !self.has_user(&user.public_key) {
                self.users.push(user);
            }
        }
    }

    /// Add a single user to lobby
    ///
    /// Performs deduplication - if user already exists,
    /// this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `user` - The user to add
    #[inline]
    pub fn add_user(&mut self, user: LobbyUser) {
        // Deduplicate before adding
        if !self.has_user(&user.public_key) {
            self.users.push(user);
        }
    }

    /// Add multiple users to lobby
    ///
    /// Performs deduplication for each user.
    ///
    /// # Arguments
    ///
    /// * `users` - Iterator of users to add
    #[inline]
    pub fn add_users<I>(&mut self, users: I)
    where
        I: IntoIterator<Item = LobbyUser>,
    {
        for user in users {
            self.add_user(user);
        }
    }

    /// Remove a user from lobby
    ///
    /// If removed user was selected, selection is cleared.
    ///
    /// # Arguments
    ///
    /// * `public_key` - The public key of user to remove
    ///
    /// # Returns
    ///
    /// `true` if user was present and removed, `false` otherwise
    #[inline]
    pub fn remove_user(&mut self, public_key: &str) -> bool {
        let was_present = self.users.iter().position(|u| u.public_key == public_key).is_some();
        self.users.retain(|u| u.public_key != public_key);

        if self.selected_user.as_deref() == Some(public_key) {
            self.selected_user = None;
        }
        was_present
    }

    /// Remove multiple users from lobby
    ///
    /// # Arguments
    ///
    /// * `public_keys` - Iterator of public keys to remove
    ///
    /// # Returns
    ///
    /// Number of users actually removed
    #[inline]
    pub fn remove_users<I>(&mut self, public_keys: I) -> usize
    where
        I: IntoIterator<Item = String>,
    {
        let mut removed = 0;
        for key in public_keys {
            if self.remove_user(&key) {
                removed += 1;
            }
        }
        removed
    }

    /// Get all users in the lobby
    ///
    /// # Returns
    ///
    /// Vector of references to all users (in insertion order)
    #[inline]
    pub fn users(&self) -> Vec<&LobbyUser> {
        self.users.iter().collect()
    }

    /// Get all users as owned values
    ///
    /// # Returns
    ///
    /// Clone of users vector
    #[inline]
    pub fn users_cloned(&self) -> Vec<LobbyUser> {
        self.users.clone()
    }

    /// Get the number of users in the lobby
    ///
    /// # Returns
    ///
    /// Current user count
    #[inline]
    pub fn len(&self) -> usize {
        self.users.len()
    }

    /// Check if lobby is empty
    ///
    /// # Returns
    ///
    /// `true` if there are no users, `false` otherwise
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }

    /// Select a user for messaging
    ///
    /// # Arguments
    ///
    /// * `public_key` - The public key of user to select
    ///
    /// # Returns
    ///
    /// `true` if user was found and selected, `false` otherwise
    #[inline]
    pub fn select(&mut self, public_key: &str) -> bool {
        if self.has_user(public_key) {
            self.selected_user = Some(public_key.to_string());
            true
        } else {
            false
        }
    }

    /// Get the currently selected user's public key
    ///
    /// # Returns
    ///
    /// `Some(public_key)` if a user is selected, `None` otherwise
    #[inline]
    pub fn selected_user(&self) -> Option<&str> {
        self.selected_user.as_deref()
    }

    /// Check if a specific user is selected
    ///
    /// # Arguments
    ///
    /// * `public_key` - The public key to check
    ///
    /// # Returns
    ///
    /// `true` if this user is currently selected
    #[inline]
    pub fn is_selected(&self, public_key: &str) -> bool {
        self.selected_user.as_deref() == Some(public_key)
    }

    /// Clear the current selection
    ///
    /// After calling this, no user will be selected.
    #[inline]
    pub fn clear_selection(&mut self) {
        self.selected_user = None;
    }

    /// Get a specific user by public key
    ///
    /// # Arguments
    ///
    /// * `public_key` - The public key to look up
    ///
    /// # Returns
    ///
    /// `Some(&LobbyUser)` if found, `None` otherwise
    #[inline]
    pub fn get_user(&self, public_key: &str) -> Option<&LobbyUser> {
        self.users.iter().find(|u| u.public_key == public_key)
    }

    /// Get the index of the selected user in the user list
    ///
    /// Used for keyboard navigation (arrow keys move selection up/down).
    ///
    /// # Returns
    ///
    /// `Some(index)` if a user is selected, `None` otherwise
    pub fn selected_index(&self) -> Option<usize> {
        let selected_key = self.selected_user()?;
        self.users.iter().position(|u| u.public_key == selected_key)
    }

    /// Select user by index in the user list
    ///
    /// Used for keyboard navigation.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of user to select
    ///
    /// # Returns
    ///
    /// `true` if a user exists at that index, `false` otherwise
    pub fn select_by_index(&mut self, index: usize) -> bool {
        if index < self.users.len() {
            let key = self.users[index].public_key.clone();
            self.selected_user = Some(key);
            true
        } else {
            false
        }
    }

    /// Get the index of a user by public key
    ///
    /// Used for determining where to scroll when selecting.
    ///
    /// # Arguments
    ///
    /// * `public_key` - The public key to look up
    ///
    /// # Returns
    ///
    /// `Some(index)` if found, `None` otherwise
    pub fn index_of(&self, public_key: &str) -> Option<usize> {
        self.users.iter().position(|u| u.public_key == public_key)
    }

    /// Get user at a specific index
    ///
    /// # Arguments
    ///
    /// * `index` - The index to look up
    ///
    /// # Returns
    ///
    /// `Some(&LobbyUser)` if index is valid, `None` otherwise
    #[inline]
    pub fn get_user_at(&self, index: usize) -> Option<&LobbyUser> {
        self.users.get(index)
    }

    /// Clear all users from the lobby
    ///
    /// Also clears selection.
    #[inline]
    pub fn clear(&mut self) {
        self.users.clear();
        self.selected_user = None;
    }
}

impl Default for LobbyState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lobby_is_empty() {
        let state = LobbyState::new();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
        assert!(state.users().is_empty());
        assert_eq!(state.selected_user(), None);
    }

    #[test]
    fn test_add_single_user() {
        let mut state = LobbyState::new();
        let user = LobbyUser::new("test_key_123".to_string(), true);
        state.add_user(user);

        assert!(!state.is_empty());
        assert_eq!(state.len(), 1);
        assert!(state.has_user("test_key_123"));
    }

    #[test]
    fn test_add_multiple_users_maintains_order() {
        let mut state = LobbyState::new();
        let users = vec![
            LobbyUser::new("key_1".to_string(), true),
            LobbyUser::new("key_2".to_string(), true),
            LobbyUser::new("key_3".to_string(), false),
        ];
        state.add_users(users);

        assert_eq!(state.len(), 3);
        assert!(state.has_user("key_1"));
        assert!(state.has_user("key_2"));
        assert!(state.has_user("key_3"));

        // Verify order is maintained
        let users_vec = state.users();
        assert_eq!(users_vec[0].public_key, "key_1");
        assert_eq!(users_vec[1].public_key, "key_2");
        assert_eq!(users_vec[2].public_key, "key_3");
    }

    #[test]
    fn test_set_users_replaces_existing() {
        let mut state = LobbyState::new();
        state.add_user(LobbyUser::new("old_key".to_string(), true));

        let new_users = vec![
            LobbyUser::new("new_key_1".to_string(), true),
            LobbyUser::new("new_key_2".to_string(), true),
        ];
        state.set_users(new_users);

        assert_eq!(state.len(), 2);
        assert!(state.has_user("new_key_1"));
        assert!(state.has_user("new_key_2"));
        assert!(!state.has_user("old_key"));
    }

    #[test]
    fn test_remove_user() {
        let mut state = LobbyState::new();
        state.add_user(LobbyUser::new("remove_me".to_string(), true));
        assert!(state.has_user("remove_me"));

        let removed = state.remove_user("remove_me");
        assert!(removed);
        assert!(!state.has_user("remove_me"));
    }

    #[test]
    fn test_remove_nonexistent_user() {
        let mut state = LobbyState::new();
        let removed = state.remove_user("nonexistent");
        assert!(!removed);
    }

    #[test]
    fn test_remove_selected_user_clears_selection() {
        let mut state = LobbyState::new();
        state.add_user(LobbyUser::new("selected_key".to_string(), true));
        state.select("selected_key");
        assert_eq!(state.selected_user(), Some("selected_key"));

        state.remove_user("selected_key");
        assert_eq!(state.selected_user(), None);
    }

    #[test]
    fn test_select_user() {
        let mut state = LobbyState::new();
        state.add_user(LobbyUser::new("selectable".to_string(), true));

        assert_eq!(state.selected_user(), None);

        let result = state.select("selectable");
        assert!(result);
        assert_eq!(state.selected_user(), Some("selectable"));
    }

    #[test]
    fn test_select_nonexistent_user() {
        let mut state = LobbyState::new();
        let result = state.select("nonexistent");
        assert!(!result);
        assert_eq!(state.selected_user(), None);
    }

    #[test]
    fn test_is_selected() {
        let mut state = LobbyState::new();
        state.add_user(LobbyUser::new("user_a".to_string(), true));
        state.add_user(LobbyUser::new("user_b".to_string(), true));

        assert!(!state.is_selected("user_a"));
        assert!(!state.is_selected("user_b"));

        state.select("user_a");
        assert!(state.is_selected("user_a"));
        assert!(!state.is_selected("user_b"));
    }

    #[test]
    fn test_clear_selection() {
        let mut state = LobbyState::new();
        state.add_user(LobbyUser::new("key".to_string(), true));
        state.select("key");
        assert_eq!(state.selected_user(), Some("key"));

        state.clear_selection();
        assert_eq!(state.selected_user(), None);
    }

    #[test]
    fn test_deduplication_on_add() {
        let mut state = LobbyState::new();
        let user = LobbyUser::new("dup_key".to_string(), true);

        state.add_user(user.clone());
        state.add_user(user.clone());
        state.add_user(user.clone());

        assert_eq!(state.len(), 1);
    }

    #[test]
    fn test_deduplication_on_set_users() {
        let mut state = LobbyState::new();
        let users = vec![
            LobbyUser::new("key_a".to_string(), true),
            LobbyUser::new("key_a".to_string(), true), // duplicate
            LobbyUser::new("key_b".to_string(), true),
        ];
        state.set_users(users);

        assert_eq!(state.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut state = LobbyState::new();
        state.add_user(LobbyUser::new("key_1".to_string(), true));
        state.add_user(LobbyUser::new("key_2".to_string(), true));
        state.select("key_1");

        state.clear();

        assert!(state.is_empty());
        assert_eq!(state.selected_user(), None);
    }

    #[test]
    fn test_selected_index_deterministic() {
        let mut state = LobbyState::new();
        state.add_user(LobbyUser::new("key_0".to_string(), true));
        state.add_user(LobbyUser::new("key_1".to_string(), true));
        state.add_user(LobbyUser::new("key_2".to_string(), true));

        assert_eq!(state.selected_index(), None);

        state.select("key_1");
        // Index should be 1 (deterministic order)
        assert_eq!(state.selected_index(), Some(1));
    }

    #[test]
    fn test_select_by_index_deterministic() {
        let mut state = LobbyState::new();
        state.add_user(LobbyUser::new("first".to_string(), true));
        state.add_user(LobbyUser::new("second".to_string(), true));
        state.add_user(LobbyUser::new("third".to_string(), true));

        // Select by index 0 should be first
        assert!(state.select_by_index(0));
        assert_eq!(state.selected_user(), Some("first"));

        // Select by index 2 should be third
        assert!(state.select_by_index(2));
        assert_eq!(state.selected_user(), Some("third"));

        // Index out of bounds should fail
        assert!(!state.select_by_index(99));
    }

    #[test]
    fn test_index_of_deterministic() {
        let mut state = LobbyState::new();
        state.add_user(LobbyUser::new("alpha".to_string(), true));
        state.add_user(LobbyUser::new("beta".to_string(), true));
        state.add_user(LobbyUser::new("gamma".to_string(), true));

        // Index should be deterministic
        assert_eq!(state.index_of("alpha"), Some(0));
        assert_eq!(state.index_of("beta"), Some(1));
        assert_eq!(state.index_of("gamma"), Some(2));
        assert_eq!(state.index_of("nonexistent"), None);
    }

    #[test]
    fn test_get_user_at() {
        let mut state = LobbyState::new();
        state.add_user(LobbyUser::new("key_1".to_string(), true));
        state.add_user(LobbyUser::new("key_2".to_string(), true));

        assert_eq!(state.get_user_at(0).map(|u| &u.public_key), Some(&"key_1".to_string()));
        assert_eq!(state.get_user_at(1).map(|u| &u.public_key), Some(&"key_2".to_string()));
        assert!(state.get_user_at(99).is_none());
    }

    #[test]
    fn test_get_user() {
        let mut state = LobbyState::new();
        state.add_user(LobbyUser::new("key".to_string(), true));

        let user = state.get_user("key");
        assert!(user.is_some());
        assert_eq!(user.unwrap().public_key, "key");

        assert!(state.get_user("nonexistent").is_none());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut state = LobbyState::new();
        let users = vec![
            LobbyUser::new("key_1_online".to_string(), true),
            LobbyUser::new("key_2_offline".to_string(), false),
        ];
        state.set_users(users.clone());
        state.select("key_1_online");

        let serializable: LobbyStateSerializable = state.into();
        let deserialized: LobbyState = serializable.into();

        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized.selected_user(), Some("key_1_online"));
    }

    #[test]
    fn test_order_preserved_after_serialization() {
        let mut state = LobbyState::new();
        let users = vec![
            LobbyUser::new("first".to_string(), true),
            LobbyUser::new("second".to_string(), true),
            LobbyUser::new("third".to_string(), true),
        ];
        state.set_users(users);

        let serializable: LobbyStateSerializable = state.into();
        let deserialized: LobbyState = serializable.into();

        // Order should be preserved
        assert_eq!(deserialized.get_user_at(0).map(|u| &u.public_key), Some(&"first".to_string()));
        assert_eq!(deserialized.get_user_at(1).map(|u| &u.public_key), Some(&"second".to_string()));
        assert_eq!(deserialized.get_user_at(2).map(|u| &u.public_key), Some(&"third".to_string()));
    }
}
