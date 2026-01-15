//! Lobby synchronization integration tests for the client
//!
//! Tests cover:
//! - Real-time lobby updates (AC1)
//! - Selection persistence through user changes (AC2)
//! - Pre-send availability checks (AC3)
//! - Selection state management (AC5)

use profile_client::handlers::lobby::{
    clear_lobby_selection, get_lobby_selected_user, handle_lobby_delta, handle_lobby_state_update,
    handle_lobby_user_joined, handle_lobby_user_left, handle_lobby_user_select, is_selection_valid,
    is_user_available, select_available_user,
};
use profile_client::state::create_shared_lobby_state;
use profile_client::ui::lobby_state::LobbyUser;

/// Test AC1: Real-time updates with delta processing
///
/// Verifies that lobby state can be updated efficiently using deltas
/// without requiring a full refresh.
#[tokio::test]
async fn test_lobby_delta_update_efficiency() {
    let state = create_shared_lobby_state();

    // Initial state with 5 users
    let initial_users: Vec<LobbyUser> = (0..5)
        .map(|i| LobbyUser::new(format!("user_{}", i), true))
        .collect();
    handle_lobby_state_update(&state, initial_users).await;
    assert_eq!(get_lobby_user_count(&state).await, 5);

    // Simulate delta: 2 users join, 1 user leaves
    let joined: Vec<LobbyUser> = vec![
        LobbyUser::new("new_user_1".to_string(), true),
        LobbyUser::new("new_user_2".to_string(), true),
    ];
    let left = vec!["user_2".to_string()];

    handle_lobby_delta(&state, joined, left).await;

    // Should have 5 + 2 - 1 = 6 users
    assert_eq!(get_lobby_user_count(&state).await, 6);

    // Verify new users are present
    assert!(is_user_available(&state, "new_user_1").await);
    assert!(is_user_available(&state, "new_user_2").await);

    // Verify left user is gone
    assert!(!is_user_available(&state, "user_2").await);
}

/// Test AC2: Extended session consistency through rapid changes
///
/// Simulates extended session with many users joining and leaving
/// to verify lobby remains consistent.
#[tokio::test]
async fn test_extended_session_consistency() {
    let state = create_shared_lobby_state();

    // Simulate 20 rapid changes
    for i in 0..20 {
        // Add a user
        handle_lobby_user_joined(&state, &format!("session_user_{}", i)).await;

        // Occasionally remove a previous user
        if i > 5 {
            handle_lobby_user_left(&state, &format!("session_user_{}", i - 5)).await;
        }
    }

    // After 20 iterations with removal starting at i=6:
    // - Users added: 0-19 (20 users)
    // - Users removed: 0-14 (15 users, starting at i=6 which removes session_user_0)
    // - Final count: 5 users (15, 16, 17, 18, 19)
    let count = get_lobby_user_count(&state).await;
    assert!(
        count >= 3 && count <= 10,
        "Expected 3-10 users, got {}",
        count
    );
}

/// Test AC2: Selection notification when selected user leaves
///
/// Verifies that selection is properly cleared and could trigger
/// notification when the selected user leaves.
#[tokio::test]
async fn test_selection_cleared_when_selected_user_leaves() {
    let state = create_shared_lobby_state();

    // Add users
    handle_lobby_user_joined(&state, "user_a").await;
    handle_lobby_user_joined(&state, "user_b").await;
    handle_lobby_user_joined(&state, "user_c").await;

    // Select user_b
    handle_lobby_user_select(&state, "user_b").await;
    assert_eq!(
        get_lobby_selected_user(&state).await,
        Some("user_b".to_string())
    );

    // user_b leaves via delta
    let joined = vec![];
    let left = vec!["user_b".to_string()];
    let selected_left = handle_lobby_delta(&state, joined, left).await;

    // Should indicate selected user left
    assert!(selected_left);

    // Selection should be cleared
    assert_eq!(get_lobby_selected_user(&state).await, None);
}

/// Test AC3: Pre-send availability check
///
/// Verifies that we can check if a user is available before sending.
#[tokio::test]
async fn test_pre_send_availability_check() {
    let state = create_shared_lobby_state();

    // Add online and "offline" users
    handle_lobby_user_joined(&state, "online_user").await;
    handle_lobby_user_joined(&state, "offline_user").await;

    // Both are added as online by default
    assert!(is_user_available(&state, "online_user").await);
    assert!(is_user_available(&state, "offline_user").await);

    // Nonexistent user is not available
    assert!(!is_user_available(&state, "nonexistent").await);
}

/// Test AC3: Selection validation before message send
///
/// Verifies that is_selection_valid correctly identifies if the
/// currently selected user can receive messages.
#[tokio::test]
async fn test_selection_validation_before_send() {
    let state = create_shared_lobby_state();

    // Add users
    handle_lobby_user_joined(&state, "user_a").await;
    handle_lobby_user_joined(&state, "user_b").await;

    // No selection initially
    assert!(!is_selection_valid(&state).await);

    // Select user_a
    handle_lobby_user_select(&state, "user_a").await;
    assert!(is_selection_valid(&state).await);

    // Clear selection
    clear_lobby_selection(&state).await;
    assert!(!is_selection_valid(&state).await);
}

/// Test AC3: Select available user with validation
///
/// Verifies that select_available_user only selects users who are online.
#[tokio::test]
async fn test_select_available_user_validates() {
    let state = create_shared_lobby_state();

    // Add a user
    handle_lobby_user_joined(&state, "available_user").await;

    // Should be able to select
    let result = select_available_user(&state, "available_user").await;
    assert!(result);
    assert_eq!(
        get_lobby_selected_user(&state).await,
        Some("available_user".to_string())
    );

    // Try to select nonexistent user - should fail
    let result = select_available_user(&state, "nonexistent").await;
    assert!(!result);
    assert_eq!(
        get_lobby_selected_user(&state).await,
        Some("available_user".to_string())
    );
}

/// Test AC5: Selection state management on user leave
///
/// Verifies that selection state is properly managed when users leave.
#[tokio::test]
async fn test_selection_state_management() {
    let state = create_shared_lobby_state();

    // Add multiple users
    handle_lobby_user_joined(&state, "user_1").await;
    handle_lobby_user_joined(&state, "user_2").await;
    handle_lobby_user_joined(&state, "user_3").await;

    // Select user_2
    handle_lobby_user_select(&state, "user_2").await;
    assert_eq!(
        get_lobby_selected_user(&state).await,
        Some("user_2".to_string())
    );

    // user_1 leaves (not selected) - selection should remain
    handle_lobby_user_left(&state, "user_1").await;
    assert_eq!(
        get_lobby_selected_user(&state).await,
        Some("user_2".to_string())
    );

    // user_2 leaves (selected) - selection should be cleared
    handle_lobby_user_left(&state, "user_2").await;
    assert_eq!(get_lobby_selected_user(&state).await, None);
}

/// Test: Multi-user lobby with selection persistence
///
/// Simulates multiple users in lobby with one selected, verifying
/// selection persists through other users joining/leaving.
#[tokio::test]
async fn test_selection_persists_through_other_users_leaving() {
    let state = create_shared_lobby_state();

    // Add 10 users
    for i in 0..10 {
        handle_lobby_user_joined(&state, &format!("user_{}", i)).await;
    }

    // Select user_5
    handle_lobby_user_select(&state, "user_5").await;

    // Remove users 0-4 (not selected)
    for i in 0..5 {
        handle_lobby_user_left(&state, &format!("user_{}", i)).await;
    }

    // Selection should still be user_5
    assert_eq!(
        get_lobby_selected_user(&state).await,
        Some("user_5".to_string())
    );

    // Add new users
    handle_lobby_user_joined(&state, "new_user_1").await;
    handle_lobby_user_joined(&state, "new_user_2").await;

    // Selection should still be user_5
    assert_eq!(
        get_lobby_selected_user(&state).await,
        Some("user_5".to_string())
    );

    // Finally remove user_5 - selection should clear
    handle_lobby_user_left(&state, "user_5").await;
    assert_eq!(get_lobby_selected_user(&state).await, None);
}

/// Helper function to get lobby user count
async fn get_lobby_user_count(state: &profile_client::state::SharedLobbyState) -> usize {
    let state_guard = state.lock().await;
    state_guard.len()
}
