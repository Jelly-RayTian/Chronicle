pub mod commands;
pub mod database;
pub mod errors;
pub mod events;
pub mod folders;
pub mod identity;
pub mod models;
pub mod platform;
pub mod scanner;
pub mod tasks;
pub mod watcher;

use tauri::Manager;

use commands::{
    application::get_application_info,
    database::get_database_status,
    folders::{
        cancel_folder_scan, get_scan_task, list_indexed_folders, register_indexed_folder,
        remove_indexed_folder, start_folder_scan,
    },
    timeline::{
        confirm_event, get_file_event_history, get_file_path_history, get_scan_history,
        open_timeline_file, query_timeline_page, reject_event, reveal_timeline_file,
    },
    watchers::{
        disable_folder_monitoring, enable_folder_monitoring, get_monitoring_status,
        list_monitoring_statuses, pause_folder_monitoring, resume_folder_monitoring,
    },
};
use database::Database;
use tasks::ScanTaskManager;
use watcher::WatcherManager;

pub fn run() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            let database = Database::open(data_directory.join("chronicle.sqlite3"))
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
            let tasks = ScanTaskManager::default();
            let watchers = WatcherManager::default();
            let _ = watchers.start_enabled_folders(database.clone(), tasks.clone());
            app.manage(database);
            app.manage(tasks);
            app.manage(watchers);
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
            list_monitoring_statuses,
            get_monitoring_status,
            enable_folder_monitoring,
            disable_folder_monitoring,
            pause_folder_monitoring,
            resume_folder_monitoring,
            query_timeline_page,
            get_file_event_history,
            get_scan_history,
            get_file_path_history,
            confirm_event,
            reject_event,
            open_timeline_file,
            reveal_timeline_file
        ])
        .run(tauri::generate_context!());

    if let Err(error) = result {
        eprintln!("failed to run Chronicle: {error}");
    }
}
