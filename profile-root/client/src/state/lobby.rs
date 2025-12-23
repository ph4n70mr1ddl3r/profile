//! Lobby state management for the client
//!
//! This module provides lobby state that integrates with the client's
//! state management system.

use crate::ui::lobby_state::LobbyState;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared lobby state for the client
pub type SharedLobbyState = Arc<Mutex<LobbyState>>;

/// Create a new shared lobby state
#[inline]
pub fn create_shared_lobby_state() -> SharedLobbyState {
    Arc::new(Mutex::new(LobbyState::new()))
}
