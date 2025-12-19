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
    let ui_weak_generate = ui.as_weak();
    let ui_weak_import = ui.as_weak();
    let ui_weak_copy = ui.as_weak();
    
    // Re-entry guard to prevent race condition from multiple button clicks
    let generating = Arc::new(AtomicBool::new(false));

    ui.on_generate_key_pressed(move || {
        // Check if already generating - prevent re-entry
        if generating.swap(true, Ordering::SeqCst) {
            return; // Already generating, ignore this click
        }
        
        let key_state = key_state.clone();
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
                    ui.set_key_generated(true);
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

    ui.on_import_key_pressed(move || {
        let Some(ui) = ui_weak_import.upgrade() else {
            return;
        };
        ui.set_status_message("Import not implemented yet (Story 1.2).".into());
    });

    ui.on_copy_public_key(move || {
        let Some(ui) = ui_weak_copy.upgrade() else {
            return;
        };
        ui.set_status_message("Copy not implemented yet (Story 1.3).".into());
    });

    ui.run()
}
