//! Profile client application (Slint UI + core crypto functionality).

use profile_client::{state, handlers};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
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

    // Initial lobby UI update (empty state)
    let ui_weak_lobby_update = ui.as_weak();
    let lobby_state_init = lobby_state.clone();
    let _ = slint::spawn_local(async move {
        if let Some(ui) = ui_weak_lobby_update.upgrade() {
            update_lobby_ui(&ui, &lobby_state_init).await;
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
        let ui_weak = ui_weak_lobby_select.clone();

        let _ = slint::spawn_local(async move {
            // Update lobby state selection
            handlers::handle_lobby_user_select(&lobby_state, &public_key.to_string()).await;

            // Update UI to reflect selection
            if let Some(ui) = ui_weak.upgrade() {
                update_lobby_ui(&ui, &lobby_state).await;
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
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_composer_focused(true);
                    ui.set_lobby_selected_user(key.clone().into());

                    // Clear focus after 100ms (simulated focus action)
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

    ui.run()
}
