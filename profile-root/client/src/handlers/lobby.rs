//! Lobby event handlers
//!
//! This module provides handlers for lobby UI events including
//! user selection, keyboard navigation, and chat activation.

use crate::state::SharedLobbyState;
use crate::ui::lobby_state::LobbyUser;

/// Handler for lobby user selection events
///
/// When a user is selected from the lobby:
/// 1. Updates the lobby state to reflect selection
/// 2. Triggers chat composer activation
/// 3. Notifies the UI of the selection change
pub async fn handle_lobby_user_select(
    lobby_state: &SharedLobbyState,
    public_key: &str,
) -> bool {
    let mut state = lobby_state.lock().await;
    state.select(public_key)
}

/// Handle keyboard navigation in the lobby (arrow up)
pub async fn handle_lobby_navigate_up(lobby_state: &SharedLobbyState) -> Option<String> {
    let mut state = lobby_state.lock().await;
    let users: Vec<LobbyUser> = state.users().into_iter().cloned().collect();
    let user_count = users.len();

    if user_count == 0 {
        return None;
    }

    let current_index = state.selected_index().unwrap_or(user_count);
    let new_index = if current_index == 0 {
        user_count - 1
    } else {
        current_index - 1
    };

    if let Some(user) = users.get(new_index) {
        state.select(&user.public_key);
        Some(user.public_key.clone())
    } else {
        None
    }
}

/// Handle keyboard navigation in the lobby (arrow down)
pub async fn handle_lobby_navigate_down(lobby_state: &SharedLobbyState) -> Option<String> {
    let mut state = lobby_state.lock().await;
    let users: Vec<LobbyUser> = state.users().into_iter().cloned().collect();
    let user_count = users.len();

    if user_count == 0 {
        return None;
    }

    let current_index = state.selected_index().unwrap_or(0usize.saturating_sub(1));
    let new_index = if current_index >= user_count - 1 {
        0
    } else {
        current_index + 1
    };

    if let Some(user) = users.get(new_index) {
        state.select(&user.public_key);
        Some(user.public_key.clone())
    } else {
        None
    }
}

/// Handle lobby user join event
pub async fn handle_lobby_user_joined(
    lobby_state: &SharedLobbyState,
    public_key: &str,
) {
    let mut state = lobby_state.lock().await;
    state.add_user(LobbyUser::new(public_key.to_string(), true));
}

/// Handle lobby user leave event
pub async fn handle_lobby_user_left(
    lobby_state: &SharedLobbyState,
    public_key: &str,
) {
    let mut state = lobby_state.lock().await;
    state.remove_user(public_key);
}

/// Handle lobby state update (initial load)
pub async fn handle_lobby_state_update(
    lobby_state: &SharedLobbyState,
    users: Vec<LobbyUser>,
) {
    let mut state = lobby_state.lock().await;
    state.set_users(users);
}

/// Clear lobby selection
pub async fn clear_lobby_selection(lobby_state: &SharedLobbyState) {
    let mut state = lobby_state.lock().await;
    state.clear_selection();
}

/// Get current lobby user count
pub async fn get_lobby_user_count(lobby_state: &SharedLobbyState) -> usize {
    let state = lobby_state.lock().await;
    state.len()
}

/// Get currently selected user
pub async fn get_lobby_selected_user(lobby_state: &SharedLobbyState) -> Option<String> {
    let state = lobby_state.lock().await;
    state.selected_user().map(|s| s.to_string())
}
