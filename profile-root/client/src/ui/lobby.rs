//! Lobby UI Component
//!
//! This module provides the lobby component for displaying online users.
//! The lobby shows a scrollable list of users with their public keys and
//! online status, allowing users to select recipients for messaging.
//!
//! # Architecture
//!
//! - `LobbyItemData`: Data struct for individual lobby items
//! - `LobbyStateWrapper`: Wraps LobbyState for UI integration
//! - `LobbyEventHandler`: Handles user interactions (clicks, keyboard navigation)
//!
//! # Integration
//!
//! The lobby integrates with:
//! - `LobbyState`: For state management (users, selection)
//! - `WebSocketClient`: For receiving lobby updates from server
//! - Chat composer: For activating messaging when user is selected

use crate::ui::lobby_state::{LobbyState, LobbyUser};
use std::cell::RefCell;
use std::rc::Rc;

/// Color constants for lobby styling (from UX Design)
pub mod lobby_colors {
    /// Online indicator - green #22c55e
    pub const ONLINE_INDICATOR: u32 = 0x22c55e;
    /// Offline indicator - gray #6b7280
    pub const OFFLINE_INDICATOR: u32 = 0x6b7280;
    /// Selected background - blue #0066CC
    pub const SELECTED_BG: u32 = 0x0066CC;
    /// Identity blue for public keys #0066CC
    pub const KEY_COLOR: u32 = 0x0066CC;
    /// Surface dark background #111827
    pub const DEFAULT_BG: u32 = 0x111827;
    /// Surface lighter for hover #374151
    pub const HOVER_BG: u32 = 0x374151;
}

/// Individual lobby item for the UI model
#[derive(Debug, Clone, PartialEq)]
pub struct LobbyItemData {
    /// The user's public key (64 hex characters)
    pub public_key: String,
    /// Whether the user is currently online
    pub is_online: bool,
    /// Whether this item is currently selected
    pub is_selected: bool,
}

impl LobbyItemData {
    /// Create a new lobby item
    #[inline]
    pub fn new(public_key: String, is_online: bool, is_selected: bool) -> Self {
        Self {
            public_key,
            is_online,
            is_selected,
        }
    }
}

/// Callback handler for lobby events
#[derive(Clone)]
pub struct LobbyEventHandler {
    /// Called when a user is selected (click or keyboard)
    pub on_user_selected: Rc<RefCell<dyn Fn(String)>>,
    /// Called when selection changes (for visual updates)
    pub on_selection_changed: Rc<RefCell<dyn Fn(Option<String>)>>,
}

impl LobbyEventHandler {
    /// Create a new event handler
    #[inline]
    pub fn new(
        on_user_selected: impl Fn(String) + 'static,
        on_selection_changed: impl Fn(Option<String>) + 'static,
    ) -> Self {
        Self {
            on_user_selected: Rc::new(RefCell::new(on_user_selected)),
            on_selection_changed: Rc::new(RefCell::new(on_selection_changed)),
        }
    }

    /// Emit user selected event
    #[inline]
    pub fn select_user(&self, public_key: String) {
        (self.on_user_selected.borrow())(public_key.clone());
    }

    /// Emit selection changed event
    #[inline]
    pub fn selection_changed(&self, public_key: Option<String>) {
        (self.on_selection_changed.borrow())(public_key);
    }
}

/// Wrapper for LobbyState with cell for interior mutability
#[derive(Clone)]
pub struct LobbyStateWrapper {
    /// The actual lobby state (interior mutability)
    state: Rc<RefCell<LobbyState>>,
}

impl LobbyStateWrapper {
    /// Create a new empty lobby state wrapper
    #[inline]
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(LobbyState::new())),
        }
    }

    /// Create from existing lobby state
    #[inline]
    pub fn from_state(state: LobbyState) -> Self {
        Self {
            state: Rc::new(RefCell::new(state)),
        }
    }

    /// Get reference to the internal state
    #[inline]
    pub fn state(&self) -> &Rc<RefCell<LobbyState>> {
        &self.state
    }

    /// Get a snapshot of the current items
    #[inline]
    pub fn snapshot_items(&self) -> Vec<LobbyItemData> {
        let state = self.state.borrow();
        let selected_key = state.selected_user();
        state
            .users()
            .iter()
            .map(|user| {
                LobbyItemData::new(
                    user.public_key.clone(),
                    user.is_online,
                    Some(user.public_key.as_str()) == selected_key,
                )
            })
            .collect()
    }

    /// Set users
    #[inline]
    pub fn set_users(&mut self, users: Vec<LobbyUser>) {
        self.state.borrow_mut().set_users(users);
    }

    /// Add a user
    #[inline]
    pub fn add_user(&mut self, user: LobbyUser) {
        self.state.borrow_mut().add_user(user);
    }

    /// Remove a user
    #[inline]
    pub fn remove_user(&mut self, public_key: &str) {
        self.state.borrow_mut().remove_user(public_key);
    }

    /// Select a user
    #[inline]
    pub fn select_user(&mut self, public_key: &str) -> bool {
        self.state.borrow_mut().select(public_key)
    }

    /// Clear selection
    #[inline]
    pub fn clear_selection(&mut self) {
        self.state.borrow_mut().clear_selection();
    }

    /// Get current selection
    #[inline]
    pub fn selected_user(&self) -> Option<String> {
        self.state.borrow().selected_user().map(|s| s.to_string())
    }

    /// Check if lobby is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.state.borrow().is_empty()
    }

    /// Get user count
    #[inline]
    pub fn len(&self) -> usize {
        self.state.borrow().len()
    }

    /// Find index of a user by public key
    #[inline]
    pub fn find_index(&self, public_key: &str) -> Option<usize> {
        self.state.borrow().index_of(public_key)
    }
}

impl Default for LobbyStateWrapper {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Keyboard navigation handler for the lobby
#[derive(Clone, Default)]
pub struct LobbyKeyboardHandler {
    /// Current selection index
    selection_index: Option<usize>,
    /// Total number of items
    item_count: usize,
}

impl LobbyKeyboardHandler {
    /// Create a new keyboard handler
    #[inline]
    pub fn new() -> Self {
        Self {
            selection_index: None,
            item_count: 0,
        }
    }

    /// Set the item count
    #[inline]
    pub fn set_item_count(&mut self, count: usize) {
        self.item_count = count;
        if let Some(idx) = self.selection_index {
            if idx >= count && count > 0 {
                self.selection_index = Some(count - 1);
            }
        }
    }

    /// Handle arrow up key - move selection up (with wrap)
    #[inline]
    pub fn handle_arrow_up(&mut self) -> Option<usize> {
        if self.item_count == 0 {
            return None;
        }
        let new_index = match self.selection_index {
            None => self.item_count - 1,
            Some(0) => self.item_count - 1,
            Some(i) => i - 1,
        };
        self.selection_index = Some(new_index);
        Some(new_index)
    }

    /// Handle arrow down key - move selection down (with wrap)
    #[inline]
    pub fn handle_arrow_down(&mut self) -> Option<usize> {
        if self.item_count == 0 {
            return None;
        }
        let new_index = match self.selection_index {
            None => 0,
            Some(i) if i + 1 >= self.item_count => 0,
            Some(i) => i + 1,
        };
        self.selection_index = Some(new_index);
        Some(new_index)
    }

    /// Handle home key - move to first item
    #[inline]
    pub fn handle_home(&mut self) -> Option<usize> {
        if self.item_count == 0 {
            return None;
        }
        self.selection_index = Some(0);
        Some(0)
    }

    /// Handle end key - move to last item
    #[inline]
    pub fn handle_end(&mut self) -> Option<usize> {
        if self.item_count == 0 {
            return None;
        }
        self.selection_index = Some(self.item_count - 1);
        Some(self.item_count - 1)
    }

    /// Get current selection index
    #[inline]
    pub fn selection(&self) -> Option<usize> {
        self.selection_index
    }

    /// Clear selection
    #[inline]
    pub fn clear_selection(&mut self) {
        self.selection_index = None;
    }

    /// Set selection explicitly
    #[inline]
    pub fn set_selection(&mut self, index: usize) {
        if index < self.item_count {
            self.selection_index = Some(index);
        }
    }
}

/// Configuration for lobby display
#[derive(Clone, Debug)]
pub struct LobbyConfig {
    /// Maximum users to display before showing scrollbar
    pub max_visible_users: usize,
    /// Height of each lobby item in pixels
    pub item_height: i32,
    /// Whether to show online indicators
    pub show_online_indicators: bool,
    /// Whether to allow keyboard navigation
    pub enable_keyboard_nav: bool,
}

impl Default for LobbyConfig {
    fn default() -> Self {
        Self {
            max_visible_users: 50,
            item_height: 36,
            show_online_indicators: true,
            enable_keyboard_nav: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lobby_item_data_creation() {
        let item = LobbyItemData::new("test_key_123".to_string(), true, false);
        assert_eq!(item.public_key, "test_key_123");
        assert!(item.is_online);
        assert!(!item.is_selected);
    }

    #[test]
    fn test_lobby_state_wrapper_empty() {
        let wrapper = LobbyStateWrapper::new();
        assert!(wrapper.is_empty());
        assert_eq!(wrapper.len(), 0);
    }

    #[test]
    fn test_lobby_state_wrapper_set_users() {
        let mut wrapper = LobbyStateWrapper::new();
        let users = vec![
            LobbyUser::new("key_1".to_string(), true),
            LobbyUser::new("key_2".to_string(), false),
        ];
        wrapper.set_users(users);

        assert_eq!(wrapper.len(), 2);
        assert!(!wrapper.is_empty());
    }

    #[test]
    fn test_lobby_state_wrapper_select() {
        let mut wrapper = LobbyStateWrapper::new();
        let users = vec![
            LobbyUser::new("key_1".to_string(), true),
            LobbyUser::new("key_2".to_string(), true),
        ];
        wrapper.set_users(users);

        assert!(wrapper.select_user("key_1"));
        assert_eq!(wrapper.selected_user(), Some("key_1".to_string()));

        assert!(!wrapper.select_user("nonexistent"));
    }

    #[test]
    fn test_lobby_keyboard_handler_arrow_up_down() {
        let mut handler = LobbyKeyboardHandler::new();
        handler.set_item_count(5);

        assert_eq!(handler.handle_arrow_down(), Some(0));
        assert_eq!(handler.selection(), Some(0));

        assert_eq!(handler.handle_arrow_down(), Some(1));

        assert_eq!(handler.handle_arrow_up(), Some(0));

        handler.clear_selection();
        handler.set_item_count(5);
        assert_eq!(handler.handle_arrow_up(), Some(4));
    }

    #[test]
    fn test_lobby_keyboard_handler_home_end() {
        let mut handler = LobbyKeyboardHandler::new();
        handler.set_item_count(10);

        assert_eq!(handler.handle_end(), Some(9));
        assert_eq!(handler.handle_home(), Some(0));
    }

    #[test]
    fn test_lobby_config_defaults() {
        let config = LobbyConfig::default();
        assert_eq!(config.max_visible_users, 50);
        assert_eq!(config.item_height, 36);
        assert!(config.show_online_indicators);
        assert!(config.enable_keyboard_nav);
    }
}
