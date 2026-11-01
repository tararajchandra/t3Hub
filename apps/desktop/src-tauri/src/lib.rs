pub mod commands;
pub mod database;
pub mod download;
pub mod installer;
pub mod logging;
pub mod manifest;
pub mod repository;
pub mod security;
pub mod uninstall;
pub mod updater;

use std::sync::Arc;
use tauri::Manager;

use crate::commands::AppState;
use crate::database::DatabaseManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            logging::init_logging();

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Establish database directory
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));

            let db_path = app_data_dir.join("t3hub.db");
            log::info!("Connecting to SQLite database at: {:?}", db_path);

            let db_manager = DatabaseManager::new(db_path)
                .expect("Critical: Failed to initialize T3 Hub SQLite database");

            app.manage(AppState {
                db: Arc::new(db_manager),
            });

            log::info!("T3 Hub backend initialization complete.");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_system_info,
            commands::get_database_status,
            commands::get_settings,
            commands::update_setting,
            commands::get_repositories,
            commands::add_repository,
            commands::get_installed_applications,
            commands::get_operations_log,
            commands::search_catalog,
            commands::get_package_details,
            commands::sync_repositories,
            commands::validate_manifest_string,
            commands::install_package,
            commands::uninstall_package,
            commands::get_detected_installed_apps,
            commands::check_updates,
            commands::toggle_repository,
            commands::remove_repository,
            commands::update_repository_trust,
            commands::verify_signature,
        ])
        .run(tauri::generate_context!())
        .expect("error while running T3 Hub application");
}
