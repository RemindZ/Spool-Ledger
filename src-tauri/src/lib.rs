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
pub mod targets;
pub mod transaction;
pub mod writer;

pub use error::AppError;
pub use model::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("failed to run Bambu Filament Migrator");
}
