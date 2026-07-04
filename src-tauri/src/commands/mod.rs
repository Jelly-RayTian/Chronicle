pub mod application;
pub mod database;
pub mod folders;
pub mod timeline;
pub mod version_families;
pub mod watchers;

#[cfg(test)]
mod tests {
    use crate::{database::Database, models::TimelineRequest};

    use super::{
        application::get_application_info_impl, database::get_database_status_impl,
        folders::list_indexed_folders_impl, timeline::query_timeline_page_impl,
        watchers::list_monitoring_statuses_impl,
    };

    #[test]
    fn application_info_reports_the_real_package() {
        let info = get_application_info_impl();
        assert_eq!(info.name, "Chronicle");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert!(!info.platform.is_empty());
    }

    #[test]
    fn database_status_reports_the_applied_schema() {
        let database = Database::open_in_memory()
            .unwrap_or_else(|error| panic!("in-memory database should open: {error}"));
        let status = get_database_status_impl(&database)
            .unwrap_or_else(|error| panic!("database status should succeed: {error:?}"));
        assert_eq!(status.schema_version, 6);
    }

    #[test]
    fn folder_and_timeline_commands_return_real_empty_data() {
        let database = Database::open_in_memory()
            .unwrap_or_else(|error| panic!("in-memory database should open: {error}"));
        let folders = list_indexed_folders_impl(&database)
            .unwrap_or_else(|error| panic!("folder command should succeed: {error:?}"));
        let page = query_timeline_page_impl(&database, TimelineRequest::first_page(50))
            .unwrap_or_else(|error| panic!("timeline command should succeed: {error:?}"));

        assert!(folders.is_empty());
        assert!(page.items.is_empty());
        assert_eq!(page.next_cursor, None);
        assert!(!page.has_more);
        assert!(
            list_monitoring_statuses_impl(&database)
                .unwrap_or_else(|error| panic!("watcher statuses should succeed: {error:?}"))
                .is_empty()
        );
    }
}
