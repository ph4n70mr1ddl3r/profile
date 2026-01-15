//! Profile client application (Slint UI + core crypto functionality).

use profile_client::{handlers, state};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

slint::include_modules!();

/// Parse common clipboard error codes into user-friendly messages
fn parse_clipboard_error(error: &str) -> String {
    // Windows HRESULT error codes
    if error.contains("0x80040155") || error.contains("CLIPBRD_E_CANT_OPEN") {
        return "Clipboard is busy. Please try again.".to_string();
    }
    if error.contains("0x800401D0") || error.contains("CLIPBRD_E_CANT_EMPTY") {
        return "Could not clear clipboard. Please try again.".to_string();
    }
    if error.contains("0x800401D1") || error.contains("CLIPBRD_E_CANT_SET") {
        return "Could not write to clipboard. Please try again.".to_string();
    }
    if error.contains("0x80040154") || error.contains("REGDB_E_CLASSNOTREG") {
        return "Clipboard service not available.".to_string();
    }

    // Generic clipboard errors
    if error.contains("clipboard") || error.contains("Clipboard") {
        return "Clipboard operation failed. Please try again.".to_string();
    }

    // Fallback: return simplified version of error
    "Copy operation failed. Please try again.".to_string()
}

/// Copy text to system clipboard using arboard
/// Returns Ok(()) on success, or Err(message) on failure
fn copy_to_clipboard(text: &str) -> Result<(), String> {
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => {
            clipboard
                .set_text(text)
                .map_err(|e| parse_clipboard_error(&e.to_string()))?;
            Ok(())
        }
        Err(e) => Err(parse_clipboard_error(&e.to_string())),
    }
}

/// Update lobby UI properties from lobby state
///
/// This function reads the current lobby state and updates the UI's
/// slot-based properties (lobby_user_1_public_key, etc.) to display
/// the users. The UI uses fixed slots (up to 5 for MVP) since
/// Slint 1.5 doesn't support dynamic for-each loops.
///
/// # Arguments
///
/// * `ui` - Reference to the Slint UI window
/// * `lobby_state` - Reference to the shared lobby state
async fn update_lobby_ui(
    ui: &AppWindow,
    lobby_state: &Arc<tokio::sync::Mutex<profile_client::ui::lobby_state::LobbyState>>,
) {
    let state = lobby_state.lock().await;
    let users = state.users_cloned();
    let selected_user = state.selected_user().map(|s| s.to_string());
    let user_count = users.len();

    // Update user count
    ui.set_lobby_user_count(user_count as i32);

    // Clear all slots first
    ui.set_lobby_user_1_public_key("".into());
    ui.set_lobby_user_1_online(true);
    ui.set_lobby_user_1_selected(false);

    ui.set_lobby_user_2_public_key("".into());
    ui.set_lobby_user_2_online(true);
    ui.set_lobby_user_2_selected(false);

    ui.set_lobby_user_3_public_key("".into());
    ui.set_lobby_user_3_online(true);
    ui.set_lobby_user_3_selected(false);

    ui.set_lobby_user_4_public_key("".into());
    ui.set_lobby_user_4_online(true);
    ui.set_lobby_user_4_selected(false);

    ui.set_lobby_user_5_public_key("".into());
    ui.set_lobby_user_5_online(true);
    ui.set_lobby_user_5_selected(false);

    // Populate slots with user data (up to 5 for MVP)
    for (i, user) in users.iter().enumerate().take(5) {
        let is_selected = selected_user.as_deref() == Some(user.public_key.as_str());

        match i {
            0 => {
                ui.set_lobby_user_1_public_key(user.public_key.clone().into());
                ui.set_lobby_user_1_online(user.is_online);
                ui.set_lobby_user_1_selected(is_selected);
            }
            1 => {
                ui.set_lobby_user_2_public_key(user.public_key.clone().into());
                ui.set_lobby_user_2_online(user.is_online);
                ui.set_lobby_user_2_selected(is_selected);
            }
            2 => {
                ui.set_lobby_user_3_public_key(user.public_key.clone().into());
                ui.set_lobby_user_3_online(user.is_online);
                ui.set_lobby_user_3_selected(is_selected);
            }
            3 => {
                ui.set_lobby_user_4_public_key(user.public_key.clone().into());
                ui.set_lobby_user_4_online(user.is_online);
                ui.set_lobby_user_4_selected(is_selected);
            }
            4 => {
                ui.set_lobby_user_5_public_key(user.public_key.clone().into());
                ui.set_lobby_user_5_online(user.is_online);
                ui.set_lobby_user_5_selected(is_selected);
            }
            _ => break, // More than 5 users (not shown in MVP)
        }
    }

    // Update selected user display text
    if let Some(ref key) = selected_user {
        ui.set_lobby_selected_user(key.clone().into());
    } else {
        ui.set_lobby_selected_user("".into());
    }
}

/// Update chat message UI slots from message history
///
/// This function converts ChatMessages to DisplayMessages and updates the UI slots.
async fn update_chat_messages_ui(
    ui: &AppWindow,
    message_history: &Arc<tokio::sync::Mutex<profile_client::state::MessageHistory>>,
    my_public_key: &str,
) {
    use profile_client::ui::chat::DisplayMessage;

    let history = message_history.lock().await;
    let messages: Vec<_> = history.messages().collect();
    let message_count = messages.len().min(10);

    // Update message count
    ui.set_chat_message_count(message_count as i32);

    // Convert ChatMessages to DisplayMessages and update slots
    for (i, msg) in messages.iter().enumerate().take(10) {
        let index = i + 1;
        let is_self = msg.sender_public_key == my_public_key;
        let display_msg = DisplayMessage::from_chat_message(msg, is_self);

        match index {
            1 => {
                ui.set_chat_msg_1_sender_key(display_msg.sender_key.clone().into());
                ui.set_chat_msg_1_sender_key_short(display_msg.sender_key_short.clone().into());
                ui.set_chat_msg_1_content(display_msg.content.clone().into());
                ui.set_chat_msg_1_timestamp(display_msg.timestamp.clone().into());
                ui.set_chat_msg_1_signature(display_msg.signature.clone().into());
                ui.set_chat_msg_1_is_self(display_msg.is_self);
                ui.set_chat_msg_1_is_verified(display_msg.is_verified);
            }
            2 => {
                ui.set_chat_msg_2_sender_key(display_msg.sender_key.clone().into());
                ui.set_chat_msg_2_sender_key_short(display_msg.sender_key_short.clone().into());
                ui.set_chat_msg_2_content(display_msg.content.clone().into());
                ui.set_chat_msg_2_timestamp(display_msg.timestamp.clone().into());
                ui.set_chat_msg_2_signature(display_msg.signature.clone().into());
                ui.set_chat_msg_2_is_self(display_msg.is_self);
                ui.set_chat_msg_2_is_verified(display_msg.is_verified);
            }
            3 => {
                ui.set_chat_msg_3_sender_key(display_msg.sender_key.clone().into());
                ui.set_chat_msg_3_sender_key_short(display_msg.sender_key_short.clone().into());
                ui.set_chat_msg_3_content(display_msg.content.clone().into());
                ui.set_chat_msg_3_timestamp(display_msg.timestamp.clone().into());
                ui.set_chat_msg_3_signature(display_msg.signature.clone().into());
                ui.set_chat_msg_3_is_self(display_msg.is_self);
                ui.set_chat_msg_3_is_verified(display_msg.is_verified);
            }
            4 => {
                ui.set_chat_msg_4_sender_key(display_msg.sender_key.clone().into());
                ui.set_chat_msg_4_sender_key_short(display_msg.sender_key_short.clone().into());
                ui.set_chat_msg_4_content(display_msg.content.clone().into());
                ui.set_chat_msg_4_timestamp(display_msg.timestamp.clone().into());
                ui.set_chat_msg_4_signature(display_msg.signature.clone().into());
                ui.set_chat_msg_4_is_self(display_msg.is_self);
                ui.set_chat_msg_4_is_verified(display_msg.is_verified);
            }
            5 => {
                ui.set_chat_msg_5_sender_key(display_msg.sender_key.clone().into());
                ui.set_chat_msg_5_sender_key_short(display_msg.sender_key_short.clone().into());
                ui.set_chat_msg_5_content(display_msg.content.clone().into());
                ui.set_chat_msg_5_timestamp(display_msg.timestamp.clone().into());
                ui.set_chat_msg_5_signature(display_msg.signature.clone().into());
                ui.set_chat_msg_5_is_self(display_msg.is_self);
                ui.set_chat_msg_5_is_verified(display_msg.is_verified);
            }
            6 => {
                ui.set_chat_msg_6_sender_key(display_msg.sender_key.clone().into());
                ui.set_chat_msg_6_sender_key_short(display_msg.sender_key_short.clone().into());
                ui.set_chat_msg_6_content(display_msg.content.clone().into());
                ui.set_chat_msg_6_timestamp(display_msg.timestamp.clone().into());
                ui.set_chat_msg_6_signature(display_msg.signature.clone().into());
                ui.set_chat_msg_6_is_self(display_msg.is_self);
                ui.set_chat_msg_6_is_verified(display_msg.is_verified);
            }
            7 => {
                ui.set_chat_msg_7_sender_key(display_msg.sender_key.clone().into());
                ui.set_chat_msg_7_sender_key_short(display_msg.sender_key_short.clone().into());
                ui.set_chat_msg_7_content(display_msg.content.clone().into());
                ui.set_chat_msg_7_timestamp(display_msg.timestamp.clone().into());
                ui.set_chat_msg_7_signature(display_msg.signature.clone().into());
                ui.set_chat_msg_7_is_self(display_msg.is_self);
                ui.set_chat_msg_7_is_verified(display_msg.is_verified);
            }
            8 => {
                ui.set_chat_msg_8_sender_key(display_msg.sender_key.clone().into());
                ui.set_chat_msg_8_sender_key_short(display_msg.sender_key_short.clone().into());
                ui.set_chat_msg_8_content(display_msg.content.clone().into());
                ui.set_chat_msg_8_timestamp(display_msg.timestamp.clone().into());
                ui.set_chat_msg_8_signature(display_msg.signature.clone().into());
                ui.set_chat_msg_8_is_self(display_msg.is_self);
                ui.set_chat_msg_8_is_verified(display_msg.is_verified);
            }
            9 => {
                ui.set_chat_msg_9_sender_key(display_msg.sender_key.clone().into());
                ui.set_chat_msg_9_sender_key_short(display_msg.sender_key_short.clone().into());
                ui.set_chat_msg_9_content(display_msg.content.clone().into());
                ui.set_chat_msg_9_timestamp(display_msg.timestamp.clone().into());
                ui.set_chat_msg_9_signature(display_msg.signature.clone().into());
                ui.set_chat_msg_9_is_self(display_msg.is_self);
                ui.set_chat_msg_9_is_verified(display_msg.is_verified);
            }
            10 => {
                ui.set_chat_msg_10_sender_key(display_msg.sender_key.clone().into());
                ui.set_chat_msg_10_sender_key_short(display_msg.sender_key_short.clone().into());
                ui.set_chat_msg_10_content(display_msg.content.clone().into());
                ui.set_chat_msg_10_timestamp(display_msg.timestamp.clone().into());
                ui.set_chat_msg_10_signature(display_msg.signature.clone().into());
                ui.set_chat_msg_10_is_self(display_msg.is_self);
                ui.set_chat_msg_10_is_verified(display_msg.is_verified);
            }
            _ => break,
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    // Key state initialization (existing code)
    let key_state = state::create_shared_key_state();
    let key_state_generate = key_state.clone();
    let key_state_import = key_state.clone();

    // Lobby state initialization (Story 2.2)
    let lobby_state = state::create_shared_lobby_state();
    let lobby_state_select = lobby_state.clone();
    let lobby_state_nav_up = lobby_state.clone();
    let lobby_state_nav_down = lobby_state.clone();
    let lobby_state_activate = lobby_state.clone();

    // Message history initialization (Story 4.2)
    let message_history = state::create_shared_message_history();
    let _message_history_update = message_history.clone();
    let message_history_select = message_history.clone();

    // Message event handler for real-time message updates (Story 3.1)
    // NOTE: Handler is created but stored value is intentionally unused for now
    // The callbacks are defined but the handler pattern may be refactored in future stories
    let _message_event_handler =
        profile_client::connection::client::MessageEventHandler::with_callbacks(
            {
                let message_history = message_history.clone();
                let ui_weak = ui.as_weak();
                let key_state = key_state.clone();

                move |_message: profile_client::state::messages::ChatMessage| {
                    let ui_weak = ui_weak.clone();
                    let message_history = message_history.clone();
                    let key_state = key_state.clone();

                    let _ = slint::spawn_local(async move {
                        let Some(ui) = ui_weak.upgrade() else {
                            return;
                        };

                        // Get user's public key for self-detection
                        let my_key = {
                            let state = key_state.lock().await;
                            state.public_key().map(hex::encode).unwrap_or_default()
                        };

                        update_chat_messages_ui(&ui, &message_history, &my_key).await;
                    });
                }
            },
            |_notification: String| {
                // Handle invalid signature notification (can be logged or shown to user)
            },
            |_error: String| {
                // Handle error (can be logged or shown to user)
            },
            |_notification: String| {
                // Handle general notification (e.g., offline status)
            },
        );

    // Initial lobby UI update (empty state)
    let ui_weak_lobby_update = ui.as_weak();
    let lobby_state_init = lobby_state.clone();
    let _ = slint::spawn_local(async move {
        if let Some(ui) = ui_weak_lobby_update.upgrade() {
            update_lobby_ui(&ui, &lobby_state_init).await;
        }
    });

    // Initial chat messages UI update (empty state)
    let ui_weak_messages_update = ui.as_weak();
    let message_history_init = message_history.clone();
    let _ = slint::spawn_local(async move {
        if let Some(ui) = ui_weak_messages_update.upgrade() {
            update_chat_messages_ui(&ui, &message_history_init, "").await;
        }
    });

    let ui_weak_generate = ui.as_weak();
    let ui_weak_show_import = ui.as_weak();
    let ui_weak_import_attempt = ui.as_weak();
    let ui_weak_cancel_import = ui.as_weak();
    let ui_weak_copy = ui.as_weak();
    let ui_weak_lobby_select = ui.as_weak();
    let ui_weak_lobby_nav_up = ui.as_weak();
    let ui_weak_lobby_nav_down = ui.as_weak();
    let ui_weak_lobby_activate = ui.as_weak();
    let key_state_lobby_select = key_state.clone();

    // Re-entry guards to prevent race conditions from multiple button clicks
    let generating = Arc::new(AtomicBool::new(false));
    let importing = Arc::new(AtomicBool::new(false));

    ui.on_generate_key_pressed(move || {
        // Check if already generating - prevent re-entry
        if generating.swap(true, Ordering::SeqCst) {
            return; // Already generating, ignore this click
        }
        
        let key_state = key_state_generate.clone();
        let ui_weak = ui_weak_generate.clone();
        let generating = generating.clone();

        let _ = slint::spawn_local(async move {
            let Some(ui) = ui_weak.upgrade() else {
                generating.store(false, Ordering::SeqCst);
                return;
            };

            ui.set_status_message("Generating key…".into());

            // Add timeout to prevent indefinite hang if OsRng blocks
            // Normal key generation completes in <1ms (see test_key_generation_completes_quickly)
            // 5-second timeout is very conservative and should never fire in normal operation
            // If timeout fires: UI remains responsive, error message shown, re-entry guard clears
            let result = async {
                match tokio::time::timeout(
                    Duration::from_secs(5),
                    handlers::handle_generate_new_key(&key_state)
                ).await {
                    Ok(Ok(public_key_hex)) => Ok(public_key_hex),
                    Ok(Err(err)) => Err(err),
                    Err(_) => Err("Key generation took too long (>5s). This may indicate a system problem. Try closing other applications or restarting Profile.".to_string()),
                }
            }.await;

            match result {
                    Ok(public_key_hex) => {
                    ui.set_public_key_display(public_key_hex.into());
                    ui.set_current_view("key-display".into());
                    ui.set_status_is_error(false);
                    ui.set_status_message("Your key has been generated. This is your identity. Keep your private key secure.".into());
                }
                Err(err) => {
                    ui.set_status_is_error(true);
                    ui.set_status_message(err.into());
                }
            }
            
            // Reset guard to allow future generations
            generating.store(false, Ordering::SeqCst);
        });
    });

    // Show import screen
    ui.on_show_import_screen(move || {
        let Some(ui) = ui_weak_show_import.upgrade() else {
            return;
        };
        ui.set_current_view("import".into());
        ui.set_import_key_input("".into());
        ui.set_show_import_error(false);
        ui.set_import_error_message("".into());
    });

    // Handle import key attempt
    ui.on_import_key_attempt(move |key_input| {
        // Check if already importing - prevent re-entry
        if importing.swap(true, Ordering::SeqCst) {
            return; // Already importing, ignore this click
        }
        
        let key_state = key_state_import.clone();
        let ui_weak = ui_weak_import_attempt.clone();
        let importing = importing.clone();

        let _ = slint::spawn_local(async move {
            let Some(ui) = ui_weak.upgrade() else {
                importing.store(false, Ordering::SeqCst);
                return;
            };

            // Clear previous errors
            ui.set_show_import_error(false);
            ui.set_import_error_message("".into());

            // Add timeout to prevent indefinite hang
            // Import validation should complete in <1ms (no blocking operations)
            // 5-second timeout is very conservative and should never fire in normal operation
            let result = async {
                match tokio::time::timeout(
                    Duration::from_secs(5),
                    handlers::handle_import_key(&key_state, key_input.to_string())
                ).await {
                    Ok(Ok(public_key_hex)) => Ok(public_key_hex),
                    Ok(Err(err)) => Err(err),
                    Err(_) => Err("Key import took too long (>5s). This may indicate a system problem. Try closing other applications or restarting Profile.".to_string()),
                }
            }.await;

            match result {
                Ok(public_key_hex) => {
                    // Success - show key display
                    ui.set_public_key_display(public_key_hex.into());
                    ui.set_current_view("key-display".into());
                    ui.set_status_is_error(false);
                    ui.set_status_message("Your key has been imported successfully.".into());
                }
                Err(err) => {
                    // Show error in import screen
                    ui.set_import_error_message(err.into());
                    ui.set_show_import_error(true);
                }
            }
            
            // Reset guard to allow future imports
            importing.store(false, Ordering::SeqCst);
        });
    });

    // Cancel import and return to welcome screen
    ui.on_cancel_import(move || {
        let Some(ui) = ui_weak_cancel_import.upgrade() else {
            return;
        };
        ui.set_current_view("welcome".into());
        ui.set_import_key_input("".into());
        ui.set_show_import_error(false);
        ui.set_import_error_message("".into());
    });

    ui.on_copy_public_key(move || {
        let Some(ui) = ui_weak_copy.upgrade() else {
            return;
        };

        // Get the current public key from UI
        let public_key = ui.get_public_key_display().to_string();

        // Copy to clipboard
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                match clipboard.set_text(&public_key) {
                    Ok(_) => {
                        ui.set_status_is_error(false);
                        ui.set_status_message("Public key copied to clipboard!".into());
                        ui.set_copy_feedback_visible(true);

                        // Reset feedback after 2 seconds
                        let ui_weak_feedback = ui.as_weak();
                        let _ = slint::spawn_local(async move {
                            slint::Timer::single_shot(Duration::from_secs(2), move || {
                                if let Some(ui) = ui_weak_feedback.upgrade() {
                                    ui.set_copy_feedback_visible(false);
                                }
                            });
                        });
                    }
                    Err(e) => {
                        // Parse common Windows clipboard errors into user-friendly messages
                        let user_message = parse_clipboard_error(&e.to_string());
                        ui.set_status_is_error(true);
                        ui.set_status_message(user_message.into());
                        ui.set_copy_feedback_visible(false);
                    }
                }
            }
            Err(e) => {
                let user_message = parse_clipboard_error(&e.to_string());
                ui.set_status_is_error(true);
                ui.set_status_message(format!("Clipboard unavailable: {}", user_message).into());
                ui.set_copy_feedback_visible(false);
            }
        }
    });

    // Lobby callbacks (Story 2.2)

    // Handle user selection from lobby (click or keyboard)
    ui.on_lobby_user_selected(move |public_key| {
        let Some(_ui) = ui_weak_lobby_select.upgrade() else {
            return;
        };

        let lobby_state = lobby_state_select.clone();
        let message_history = message_history_select.clone();
        let key_state = key_state_lobby_select.clone();
        let ui_weak = ui_weak_lobby_select.clone();

        let _ = slint::spawn_local(async move {
            // Get user's public key for self-detection
            let my_key = {
                let state = key_state.lock().await;
                state.public_key().map(hex::encode).unwrap_or_default()
            };

            // Update lobby state selection
            handlers::handle_lobby_user_select(&lobby_state, public_key.as_str()).await;

            // Update UI to reflect selection
            if let Some(ui) = ui_weak.upgrade() {
                update_lobby_ui(&ui, &lobby_state).await;
                update_chat_messages_ui(&ui, &message_history, &my_key).await;
            }
        });
    });

    // Handle keyboard navigation up (ArrowUp)
    ui.on_lobby_navigate_up(move || {
        let Some(_ui) = ui_weak_lobby_nav_up.upgrade() else {
            return;
        };

        let lobby_state = lobby_state_nav_up.clone();
        let ui_weak = ui_weak_lobby_nav_up.clone();

        let _ = slint::spawn_local(async move {
            if let Some(_new_key) = handlers::handle_lobby_navigate_up(&lobby_state).await {
                // Update UI to reflect new selection
                if let Some(ui) = ui_weak.upgrade() {
                    update_lobby_ui(&ui, &lobby_state).await;
                }
            }
        });
    });

    // Handle keyboard navigation down (ArrowDown)
    ui.on_lobby_navigate_down(move || {
        let Some(_ui) = ui_weak_lobby_nav_down.upgrade() else {
            return;
        };

        let lobby_state = lobby_state_nav_down.clone();
        let ui_weak = ui_weak_lobby_nav_down.clone();

        let _ = slint::spawn_local(async move {
            if let Some(_new_key) = handlers::handle_lobby_navigate_down(&lobby_state).await {
                // Update UI to reflect new selection
                if let Some(ui) = ui_weak.upgrade() {
                    update_lobby_ui(&ui, &lobby_state).await;
                }
            }
        });
    });

    // Handle lobby activation (Enter key)
    ui.on_lobby_activate_selection(move || {
        let Some(_ui) = ui_weak_lobby_activate.upgrade() else {
            return;
        };

        let lobby_state = lobby_state_activate.clone();
        let ui_weak = ui_weak_lobby_activate.clone();

        let _ = slint::spawn_local(async move {
            // Get currently selected user
            let selected_user = {
                let state = lobby_state.lock().await;
                state.selected_user().map(|s| s.to_string())
            };

            if let Some(key) = selected_user {
                // Focus composer field when user activates selection
                // NOTE: Slint 1.5 does not support programmatic focus() API.
                // The composer_focused property is a visual indicator workaround.
                // Real keyboard focus requires user interaction (Tab key or mouse click).
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_composer_focused(true);
                    ui.set_lobby_selected_user(key.clone().into());

                    // Visual feedback timeout - composer_focused is cleared after 100ms
                    // This provides temporary visual indication that focus occurred
                    let ui_weak_delayed = ui_weak.clone();
                    let _ = slint::spawn_local(async move {
                        slint::Timer::single_shot(Duration::from_millis(100), move || {
                            if let Some(ui) = ui_weak_delayed.upgrade() {
                                ui.set_composer_focused(false);
                            }
                        });
                    });
                }
            }
        });
    });

    // Drill-down modal callbacks (Story 4.1 + 4.2)
    let ui_weak_drill_down_clicked = ui.as_weak();
    let ui_weak_drill_down_close = ui.as_weak();
    let ui_weak_drill_down_copy_key = ui.as_weak();
    let ui_weak_drill_down_copy_message = ui.as_weak();
    let ui_weak_drill_down_copy_signature = ui.as_weak();

    // Handle chat message click - opens drill-down modal
    ui.on_chat_message_clicked(move |slot_index| {
        let Some(ui) = ui_weak_drill_down_clicked.upgrade() else {
            return;
        };

        // Get message details from the corresponding slot
        let (sender_key, _sender_key_short, content, timestamp, signature, is_self, is_verified) = match slot_index {
            1 => (
                ui.get_chat_msg_1_sender_key().to_string(),
                ui.get_chat_msg_1_sender_key_short().to_string(),
                ui.get_chat_msg_1_content().to_string(),
                ui.get_chat_msg_1_timestamp().to_string(),
                ui.get_chat_msg_1_signature().to_string(),
                ui.get_chat_msg_1_is_self(),
                ui.get_chat_msg_1_is_verified(),
            ),
            2 => (
                ui.get_chat_msg_2_sender_key().to_string(),
                ui.get_chat_msg_2_sender_key_short().to_string(),
                ui.get_chat_msg_2_content().to_string(),
                ui.get_chat_msg_2_timestamp().to_string(),
                ui.get_chat_msg_2_signature().to_string(),
                ui.get_chat_msg_2_is_self(),
                ui.get_chat_msg_2_is_verified(),
            ),
            3 => (
                ui.get_chat_msg_3_sender_key().to_string(),
                ui.get_chat_msg_3_sender_key_short().to_string(),
                ui.get_chat_msg_3_content().to_string(),
                ui.get_chat_msg_3_timestamp().to_string(),
                ui.get_chat_msg_3_signature().to_string(),
                ui.get_chat_msg_3_is_self(),
                ui.get_chat_msg_3_is_verified(),
            ),
            4 => (
                ui.get_chat_msg_4_sender_key().to_string(),
                ui.get_chat_msg_4_sender_key_short().to_string(),
                ui.get_chat_msg_4_content().to_string(),
                ui.get_chat_msg_4_timestamp().to_string(),
                ui.get_chat_msg_4_signature().to_string(),
                ui.get_chat_msg_4_is_self(),
                ui.get_chat_msg_4_is_verified(),
            ),
            5 => (
                ui.get_chat_msg_5_sender_key().to_string(),
                ui.get_chat_msg_5_sender_key_short().to_string(),
                ui.get_chat_msg_5_content().to_string(),
                ui.get_chat_msg_5_timestamp().to_string(),
                ui.get_chat_msg_5_signature().to_string(),
                ui.get_chat_msg_5_is_self(),
                ui.get_chat_msg_5_is_verified(),
            ),
            6 => (
                ui.get_chat_msg_6_sender_key().to_string(),
                ui.get_chat_msg_6_sender_key_short().to_string(),
                ui.get_chat_msg_6_content().to_string(),
                ui.get_chat_msg_6_timestamp().to_string(),
                ui.get_chat_msg_6_signature().to_string(),
                ui.get_chat_msg_6_is_self(),
                ui.get_chat_msg_6_is_verified(),
            ),
            7 => (
                ui.get_chat_msg_7_sender_key().to_string(),
                ui.get_chat_msg_7_sender_key_short().to_string(),
                ui.get_chat_msg_7_content().to_string(),
                ui.get_chat_msg_7_timestamp().to_string(),
                ui.get_chat_msg_7_signature().to_string(),
                ui.get_chat_msg_7_is_self(),
                ui.get_chat_msg_7_is_verified(),
            ),
            8 => (
                ui.get_chat_msg_8_sender_key().to_string(),
                ui.get_chat_msg_8_sender_key_short().to_string(),
                ui.get_chat_msg_8_content().to_string(),
                ui.get_chat_msg_8_timestamp().to_string(),
                ui.get_chat_msg_8_signature().to_string(),
                ui.get_chat_msg_8_is_self(),
                ui.get_chat_msg_8_is_verified(),
            ),
            9 => (
                ui.get_chat_msg_9_sender_key().to_string(),
                ui.get_chat_msg_9_sender_key_short().to_string(),
                ui.get_chat_msg_9_content().to_string(),
                ui.get_chat_msg_9_timestamp().to_string(),
                ui.get_chat_msg_9_signature().to_string(),
                ui.get_chat_msg_9_is_self(),
                ui.get_chat_msg_9_is_verified(),
            ),
            10 => (
                ui.get_chat_msg_10_sender_key().to_string(),
                ui.get_chat_msg_10_sender_key_short().to_string(),
                ui.get_chat_msg_10_content().to_string(),
                ui.get_chat_msg_10_timestamp().to_string(),
                ui.get_chat_msg_10_signature().to_string(),
                ui.get_chat_msg_10_is_self(),
                ui.get_chat_msg_10_is_verified(),
            ),
            _ => return, // Invalid slot index
        };

        // Set modal properties
        ui.set_drill_down_sender_key(sender_key.clone().into());
        ui.set_drill_down_message_content(content.clone().into());
        ui.set_drill_down_timestamp(timestamp.clone().into());
        ui.set_drill_down_signature(signature.clone().into());
        ui.set_drill_down_is_verified(is_verified);

        // Set verification text and explanation
        if is_verified {
            ui.set_drill_down_verification_text("Verified".into());
            if is_self {
                ui.set_drill_down_verification_explanation(
                    "This message was cryptographically verified. It came from your public key.".into()
                );
            } else {
                // Show abbreviated fingerprint (first 8 chars + "...")
                let fingerprint = if sender_key.len() > 12 {
                    format!("{}...{}", &sender_key[..8], &sender_key[sender_key.len()-4..])
                } else {
                    sender_key.clone()
                };
                ui.set_drill_down_verification_explanation(
                    format!("This message was cryptographically verified. It came from the owner of {}.", fingerprint).into()
                );
            }
        } else {
            ui.set_drill_down_verification_text("Not Verified".into());
            ui.set_drill_down_verification_explanation(
                "This message failed signature verification. It may have been tampered with."
                    .into()
            );
        }

        // Show modal
        ui.set_drill_down_modal_visible(true);
    });

    // Handle drill-down modal close (Escape or X button)
    ui.on_drill_down_modal_close(move || {
        let Some(ui) = ui_weak_drill_down_close.upgrade() else {
            return;
        };

        // Clear modal properties FIRST (prevents visual flicker)
        ui.set_drill_down_sender_key("".into());
        ui.set_drill_down_message_content("".into());
        ui.set_drill_down_timestamp("".into());
        ui.set_drill_down_signature("".into());
        ui.set_drill_down_verification_text("".into());
        ui.set_drill_down_verification_explanation("".into());
        ui.set_drill_down_key_copied(false);
        ui.set_drill_down_message_copied(false);
        ui.set_drill_down_signature_copied(false);
        ui.set_drill_down_key_error(false);
        ui.set_drill_down_message_error(false);
        ui.set_drill_down_signature_error(false);

        // Hide modal AFTER properties are cleared
        ui.set_drill_down_modal_visible(false);
    });

    // Handle copy public key from drill-down modal
    ui.on_drill_down_copy_key(move || {
        let Some(ui) = ui_weak_drill_down_copy_key.upgrade() else {
            return;
        };

        let text_to_copy = ui.get_drill_down_sender_key().to_string();

        // Attempt copy and show appropriate feedback
        match copy_to_clipboard(&text_to_copy) {
            Ok(()) => {
                // Success: show "Copied!" feedback
                ui.set_drill_down_key_error(false);
                ui.set_drill_down_key_copied(true);
                let ui_weak = ui.as_weak();
                let _ = slint::spawn_local(async move {
                    slint::Timer::single_shot(Duration::from_secs(1), move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_drill_down_key_copied(false);
                        }
                    });
                });
            }
            Err(msg) => {
                // Error: show "Error!" feedback for 2 seconds
                ui.set_drill_down_key_copied(false);
                ui.set_drill_down_key_error(true);
                let ui_weak = ui.as_weak();
                let _ = slint::spawn_local(async move {
                    slint::Timer::single_shot(Duration::from_secs(2), move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_drill_down_key_error(false);
                        }
                    });
                });
                eprintln!("Clipboard error: {}", msg);
            }
        }
    });

    // Handle copy message content from drill-down modal
    ui.on_drill_down_copy_message(move || {
        let Some(ui) = ui_weak_drill_down_copy_message.upgrade() else {
            return;
        };

        let text_to_copy = ui.get_drill_down_message_content().to_string();

        // Attempt copy and show appropriate feedback
        match copy_to_clipboard(&text_to_copy) {
            Ok(()) => {
                // Success: show "Copied!" feedback
                ui.set_drill_down_message_error(false);
                ui.set_drill_down_message_copied(true);
                let ui_weak = ui.as_weak();
                let _ = slint::spawn_local(async move {
                    slint::Timer::single_shot(Duration::from_secs(1), move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_drill_down_message_copied(false);
                        }
                    });
                });
            }
            Err(msg) => {
                // Error: show "Error!" feedback for 2 seconds
                ui.set_drill_down_message_copied(false);
                ui.set_drill_down_message_error(true);
                let ui_weak = ui.as_weak();
                let _ = slint::spawn_local(async move {
                    slint::Timer::single_shot(Duration::from_secs(2), move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_drill_down_message_error(false);
                        }
                    });
                });
                eprintln!("Clipboard error: {}", msg);
            }
        }
    });

    // Handle copy signature from drill-down modal
    ui.on_drill_down_copy_signature(move || {
        let Some(ui) = ui_weak_drill_down_copy_signature.upgrade() else {
            return;
        };

        let text_to_copy = ui.get_drill_down_signature().to_string();

        // Attempt copy and show appropriate feedback
        match copy_to_clipboard(&text_to_copy) {
            Ok(()) => {
                // Success: show "Copied!" feedback
                ui.set_drill_down_signature_error(false);
                ui.set_drill_down_signature_copied(true);
                let ui_weak = ui.as_weak();
                let _ = slint::spawn_local(async move {
                    slint::Timer::single_shot(Duration::from_secs(1), move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_drill_down_signature_copied(false);
                        }
                    });
                });
            }
            Err(msg) => {
                // Error: show "Error!" feedback for 2 seconds
                ui.set_drill_down_signature_copied(false);
                ui.set_drill_down_signature_error(true);
                let ui_weak = ui.as_weak();
                let _ = slint::spawn_local(async move {
                    slint::Timer::single_shot(Duration::from_secs(2), move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_drill_down_signature_error(false);
                        }
                    });
                });
                eprintln!("Clipboard error: {}", msg);
            }
        }
    });

    ui.run()
}
