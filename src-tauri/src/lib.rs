pub mod commands;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = commands::AppState::current().expect("failed to initialize application state");
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::discover,
            commands::catalog_sources,
            commands::catalog_targets,
            commands::preview_names,
            commands::build_plan,
            commands::execute_plan,
            commands::synchronize_run,
            commands::cancel_run,
            commands::record_ams_verification,
            commands::restore_preview,
            commands::restore_owned,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Bambu Filament Migrator");
}
