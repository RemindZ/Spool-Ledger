pub mod commands;
#[cfg(debug_assertions)]
pub mod demo;
pub mod discovery;
pub mod error;
pub mod field_policy;
pub mod model;
pub mod naming;
pub mod planner;
pub mod platform;
pub mod profiles;
pub mod receipt;
pub mod resolver;
pub mod sync;
pub mod targets;
pub mod transaction;
pub mod writer;

pub use error::AppError;
pub use model::*;

#[cfg(debug_assertions)]
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = commands::AppState::current().expect("failed to initialize application state");
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::discover,
            commands::choose_manual_source_folder,
            commands::remove_manual_source_folder,
            commands::catalog_sources,
            commands::catalog_targets,
            commands::printer_artwork,
            commands::preview_names,
            commands::build_plan,
            commands::resolve_plan_dependencies,
            commands::execute_plan,
            commands::synchronize_run,
            commands::cancel_run,
            commands::record_ams_verification,
            commands::restore_preview,
            commands::restore_owned,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build Bambu Filament Migrator");
    app.run(|_app_handle, _event| {
        #[cfg(debug_assertions)]
        if matches!(_event, tauri::RunEvent::Exit) {
            let state = _app_handle.state::<commands::AppState>();
            let _ = state.cleanup_demo_workspace();
        }
    });
}
