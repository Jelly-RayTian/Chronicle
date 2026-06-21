use crate::models::ApplicationInfo;

#[must_use]
pub fn get_application_info_impl() -> ApplicationInfo {
    ApplicationInfo {
        name: "Chronicle".to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        platform: std::env::consts::OS.to_owned(),
    }
}

#[tauri::command]
pub fn get_application_info() -> ApplicationInfo {
    get_application_info_impl()
}
