//! Profile client application (Slint UI + core crypto functionality).

mod state;
mod handlers;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let key_state = state::create_shared_key_state();
    let key_state_generate = key_state.clone();
    let key_state_import = key_state.clone();
    
    let ui_weak_generate = ui.as_weak();
    let ui_weak_show_import = ui.as_weak();
    let ui_weak_import_attempt = ui.as_weak();
    let ui_weak_cancel_import = ui.as_weak();
    let ui_weak_copy = ui.as_weak();
    
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
                    ui.set_status_message("Your key has been generated. This is your identity. Keep your private key secure.".into());
                }
                Err(err) => {
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
        ui.set_status_message("Copy not implemented yet (Story 1.3).".into());
    });

    ui.run()
}
