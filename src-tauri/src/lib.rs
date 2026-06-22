pub mod commands;
pub mod database;
pub mod errors;
pub mod events;
pub mod folders;
pub mod models;
pub mod platform;
pub mod scanner;
pub mod tasks;

use tauri::Manager;

use commands::{
    application::get_application_info,
    database::get_database_status,
    folders::{
        cancel_folder_scan, get_scan_task, list_indexed_folders, register_indexed_folder,
        remove_indexed_folder, start_folder_scan,
    },
    timeline::query_timeline_page,
};
use database::Database;
use tasks::ScanTaskManager;

pub fn run() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            let database = Database::open(data_directory.join("chronicle.sqlite3"))
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
            app.manage(database);
            app.manage(ScanTaskManager::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_application_info,
            get_database_status,
            list_indexed_folders,
            register_indexed_folder,
            remove_indexed_folder,
            start_folder_scan,
            get_scan_task,
            cancel_folder_scan,
            query_timeline_page
        ])
        .run(tauri::generate_context!());

    if let Err(error) = result {
        eprintln!("failed to run Chronicle: {error}");
    }
}
