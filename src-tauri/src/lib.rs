pub mod commands;
pub mod database;
pub mod errors;
pub mod events;
pub mod models;
pub mod platform;
pub mod scanner;
pub mod tasks;

use tauri::Manager;

use commands::{
    application::get_application_info, database::get_database_status,
    folders::list_indexed_folders, timeline::query_timeline_page,
};
use database::Database;

pub fn run() {
    let result = tauri::Builder::default()
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            let database = Database::open(data_directory.join("chronicle.sqlite3"))
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_application_info,
            get_database_status,
            list_indexed_folders,
            query_timeline_page
        ])
        .run(tauri::generate_context!());

    if let Err(error) = result {
        eprintln!("failed to run Chronicle: {error}");
    }
}
