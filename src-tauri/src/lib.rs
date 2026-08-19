pub mod error;
pub mod model;

pub use error::AppError;
pub use model::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("failed to run Bambu Filament Migrator");
}
