//! Profile client application (Slint UI + core crypto functionality).

mod state;
mod handlers;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let key_state = state::create_shared_key_state();
    let ui_weak_generate = ui.as_weak();
    let ui_weak_import = ui.as_weak();
    let ui_weak_copy = ui.as_weak();

    ui.on_generate_key_pressed(move || {
        let key_state = key_state.clone();
        let ui_weak = ui_weak_generate.clone();

        let _ = slint::spawn_local(async move {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };

            ui.set_status_message("Generating key…".into());

            match handlers::handle_generate_new_key(&key_state).await {
                Ok(public_key_hex) => {
                    ui.set_public_key_display(public_key_hex.into());
                    ui.set_key_generated(true);
                    ui.set_status_message("Key generated successfully.".into());
                }
                Err(err) => {
                    ui.set_status_message(err.into());
                }
            }
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
