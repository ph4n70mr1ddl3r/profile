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
///
/// Moves selection to the previous user in the list.
/// Wraps around to the last user if at the top.
pub async fn handle_lobby_navigate_up(lobby_state: &SharedLobbyState) -> Option<String> {
    let mut state = lobby_state.lock().await;
    let user_count = state.len();

    if user_count == 0 {
        return None;
    }

    let current_index = state.selected_index().unwrap_or(user_count);
    let new_index = if current_index == 0 {
        user_count - 1  // Wrap to last user
    } else {
        current_index - 1  // Move up one
    };

    if state.select_by_index(new_index) {
        state.get_user_at(new_index).map(|u| u.public_key.clone())
    } else {
        None
    }
}

/// Handle keyboard navigation in the lobby (arrow down)
///
/// Moves selection to the next user in the list.
/// Wraps around to the first user if at the bottom.
pub async fn handle_lobby_navigate_down(lobby_state: &SharedLobbyState) -> Option<String> {
    let mut state = lobby_state.lock().await;
    let user_count = state.len();

    if user_count == 0 {
        return None;
    }

    let current_index = state.selected_index().unwrap_or(0usize.saturating_sub(1));
    let new_index = if current_index >= user_count - 1 {
        0  // Wrap to first user
    } else {
        current_index + 1  // Move down one
    };

    if state.select_by_index(new_index) {
        state.get_user_at(new_index).map(|u| u.public_key.clone())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::create_shared_lobby_state;

    #[tokio::test]
    async fn test_handle_lobby_user_select() {
        let state = create_shared_lobby_state();

        // Add a user
        handle_lobby_user_joined(&state, "test_key").await;

        // Select the user
        let result = handle_lobby_user_select(&state, "test_key").await;
        assert!(result);

        // Verify selection
        let selected = get_lobby_selected_user(&state).await;
        assert_eq!(selected, Some("test_key".to_string()));
    }

    #[tokio::test]
    async fn test_handle_lobby_navigate_up() {
        let state = create_shared_lobby_state();

        // Add 3 users
        handle_lobby_user_joined(&state, "user_1").await;
        handle_lobby_user_joined(&state, "user_2").await;
        handle_lobby_user_joined(&state, "user_3").await;

        // Select middle user
        handle_lobby_user_select(&state, "user_2").await;

        // Navigate up should select user_1
        let result = handle_lobby_navigate_up(&state).await;
        assert_eq!(result, Some("user_1".to_string()));

        // Navigate up from first should wrap to last
        let result = handle_lobby_navigate_up(&state).await;
        assert_eq!(result, Some("user_3".to_string()));
    }

    #[tokio::test]
    async fn test_handle_lobby_navigate_down() {
        let state = create_shared_lobby_state();

        // Add 3 users
        handle_lobby_user_joined(&state, "user_1").await;
        handle_lobby_user_joined(&state, "user_2").await;
        handle_lobby_user_joined(&state, "user_3").await;

        // Select first user
        handle_lobby_user_select(&state, "user_1").await;

        // Navigate down should select user_2
        let result = handle_lobby_navigate_down(&state).await;
        assert_eq!(result, Some("user_2".to_string()));

        // Navigate down to last, then wrap to first
        let _ = handle_lobby_navigate_down(&state).await;
        let result = handle_lobby_navigate_down(&state).await;
        assert_eq!(result, Some("user_1".to_string()));
    }

    #[tokio::test]
    async fn test_navigation_deterministic() {
        let state = create_shared_lobby_state();

        // Add users in specific order
        handle_lobby_user_joined(&state, "alpha").await;
        handle_lobby_user_joined(&state, "beta").await;
        handle_lobby_user_joined(&state, "gamma").await;

        // Select first
        handle_lobby_user_select(&state, "alpha").await;

        // Navigate down twice - should be gamma (deterministic)
        let _ = handle_lobby_navigate_down(&state).await;  // alpha -> beta
        let result = handle_lobby_navigate_down(&state).await;  // beta -> gamma
        assert_eq!(result, Some("gamma".to_string()));
    }

    #[tokio::test]
    async fn test_handle_lobby_user_left() {
        let state = create_shared_lobby_state();

        // Add and select a user
        handle_lobby_user_joined(&state, "leave_me").await;
        handle_lobby_user_select(&state, "leave_me").await;

        // Remove user
        handle_lobby_user_left(&state, "leave_me").await;

        // Verify user is removed and selection is cleared
        let count = get_lobby_user_count(&state).await;
        assert_eq!(count, 0);

        let selected = get_lobby_selected_user(&state).await;
        assert_eq!(selected, None);
    }

    #[tokio::test]
    async fn test_handle_lobby_state_update() {
        let state = create_shared_lobby_state();

        let users = vec![
            LobbyUser::new("user_a".to_string(), true),
            LobbyUser::new("user_b".to_string(), false),
        ];

        handle_lobby_state_update(&state, users.clone()).await;

        let count = get_lobby_user_count(&state).await;
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_clear_lobby_selection() {
        let state = create_shared_lobby_state();

        // Add and select a user
        handle_lobby_user_joined(&state, "select_me").await;
        handle_lobby_user_select(&state, "select_me").await;

        assert_eq!(
            get_lobby_selected_user(&state).await,
            Some("select_me".to_string())
        );

        // Clear selection
        clear_lobby_selection(&state).await;

        assert_eq!(get_lobby_selected_user(&state).await, None);
    }
}
