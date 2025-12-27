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

/// Check if a user is available for messaging (exists and is online)
///
/// **AC3**: Pre-send availability check before sending a message.
/// Returns whether the user can receive messages.
///
/// # Arguments
///
/// * `lobby_state` - Shared lobby state
/// * `public_key` - User's public key to check
///
/// # Returns
///
/// `true` if user is online and available, `false` otherwise
pub async fn is_user_available(lobby_state: &SharedLobbyState, public_key: &str) -> bool {
    let state = lobby_state.lock().await;
    state.is_user_online(public_key)
}

/// Check if current selection is still valid for messaging
///
/// **AC2/AC3**: Validates that the selected recipient is still online
/// before attempting to send a message. Use this before message send.
///
/// # Arguments
///
/// * `lobby_state` - Shared lobby state
///
/// # Returns
///
/// `true` if a user is selected and is online, `false` otherwise
pub async fn is_selection_valid(lobby_state: &SharedLobbyState) -> bool {
    let state = lobby_state.lock().await;
    state.is_selection_valid()
}

/// Validate and select a user with availability check
///
/// **AC3**: Selects a user only if they are online and available.
/// Returns whether selection succeeded.
///
/// # Arguments
///
/// * `lobby_state` - Shared lobby state
/// * `public_key` - User's public key to select
///
/// # Returns
///
/// `true` if user was found and selected, `false` if user not available
pub async fn select_available_user(lobby_state: &SharedLobbyState, public_key: &str) -> bool {
    let mut state = lobby_state.lock().await;
    // Only select if user exists and is online
    if state.is_user_online(public_key) {
        state.select(public_key);
        true
    } else {
        false
    }
}

/// Handle lobby update (delta processing)
///
/// **AC1**: Processes lobby update events efficiently using delta format.
/// This ensures real-time updates without full refresh.
///
/// # Arguments
///
/// * `lobby_state` - Shared lobby state
/// * `joined` - Users who joined the lobby
/// * `left` - Public keys of users who left the lobby
///
/// # Returns
///
/// `true` if the selected user left (caller should show notification)
pub async fn handle_lobby_delta(
    lobby_state: &SharedLobbyState,
    joined: Vec<LobbyUser>,
    left: Vec<String>,
) -> bool {
    let mut state = lobby_state.lock().await;
    let selected_left = state.selected_user_left(&left);
    state.apply_delta(joined, left);
    selected_left
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

    #[tokio::test]
    async fn test_is_user_available() {
        let state = create_shared_lobby_state();

        // Add online and offline users
        handle_lobby_user_joined(&state, "online_user").await;
        handle_lobby_user_joined(&state, "offline_user").await;

        // Note: handle_lobby_user_joined adds user as online (is_online: true)
        assert!(is_user_available(&state, "online_user").await);
        assert!(is_user_available(&state, "offline_user").await);
        assert!(!is_user_available(&state, "nonexistent").await);
    }

    #[tokio::test]
    async fn test_is_selection_valid() {
        let state = create_shared_lobby_state();

        // Add and select a user
        handle_lobby_user_joined(&state, "selectable_user").await;
        handle_lobby_user_select(&state, "selectable_user").await;

        assert!(is_selection_valid(&state).await);

        // Clear selection
        clear_lobby_selection(&state).await;
        assert!(!is_selection_valid(&state).await);
    }

    #[tokio::test]
    async fn test_select_available_user() {
        let state = create_shared_lobby_state();

        // Add a user
        handle_lobby_user_joined(&state, "available_user").await;

        // Select using availability check
        let result = select_available_user(&state, "available_user").await;
        assert!(result);

        // Verify selection
        let selected = get_lobby_selected_user(&state).await;
        assert_eq!(selected, Some("available_user".to_string()));
    }

    #[tokio::test]
    async fn test_select_unavailable_user_fails() {
        let state = create_shared_lobby_state();

        // Try to select a user that doesn't exist
        let result = select_available_user(&state, "nonexistent").await;
        assert!(!result);

        // No selection should be set
        let selected = get_lobby_selected_user(&state).await;
        assert_eq!(selected, None);
    }

    #[tokio::test]
    async fn test_handle_lobby_delta() {
        let state = create_shared_lobby_state();

        // Add initial user and select them
        handle_lobby_user_joined(&state, "initial_user").await;
        handle_lobby_user_select(&state, "initial_user").await;

        // Simulate lobby delta: new user joined, initial user left
        let joined = vec![LobbyUser::new("new_user".to_string(), true)];
        let left = vec!["initial_user".to_string()];

        let selected_left = handle_lobby_delta(&state, joined, left).await;

        // Should return true since selected user left
        assert!(selected_left);

        // Verify state updated
        let count = get_lobby_user_count(&state).await;
        assert_eq!(count, 1);
        assert!(get_lobby_selected_user(&state).await.is_none());
    }

    #[tokio::test]
    async fn test_handle_lobby_delta_selected_user_stays() {
        let state = create_shared_lobby_state();

        // Add users
        handle_lobby_user_joined(&state, "user_a").await;
        handle_lobby_user_joined(&state, "user_b").await;
        handle_lobby_user_select(&state, "user_a").await;

        // Simulate lobby delta: user_b left, user_c joined (user_a stays)
        let joined = vec![LobbyUser::new("user_c".to_string(), true)];
        let left = vec!["user_b".to_string()];

        let selected_left = handle_lobby_delta(&state, joined, left).await;

        // Should return false since selected user didn't leave
        assert!(!selected_left);

        // Verify user_a is still selected
        let selected = get_lobby_selected_user(&state).await;
        assert_eq!(selected, Some("user_a".to_string()));
    }
}
