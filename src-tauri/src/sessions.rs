#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

use std::collections::{HashMap, HashSet};

use crate::{
    database::Database,
    errors::ChronicleError,
    models::{
        ActivitySession, ActivitySessionDetail, GenerateSessionsRequest, GenerateSessionsResponse,
        ListSessionsRequest, SessionRequest, UpdateSessionRequest,
    },
};

const DEFAULT_GAP_MINUTES: u32 = 30;
const MAX_EVENTS_PER_SESSION: usize = 100;
const MAX_SESSIONS: usize = 1_000;

type SessionTuple = (String, String, Option<i64>, Vec<i64>, Vec<i64>);

pub fn list_sessions(
    database: &Database,
    request: ListSessionsRequest,
) -> Result<Vec<crate::models::ActivitySessionSummary>, ChronicleError> {
    database.list_sessions(&request)
}

pub fn get_session(
    database: &Database,
    request: &SessionRequest,
) -> Result<ActivitySessionDetail, ChronicleError> {
    database.get_session(request.session_id)
}

pub fn generate_sessions(
    database: &Database,
    request: GenerateSessionsRequest,
) -> Result<GenerateSessionsResponse, ChronicleError> {
    let gap_minutes = request.gap_minutes.unwrap_or(DEFAULT_GAP_MINUTES);
    let gap_seconds = i64::from(gap_minutes) * 60;

    let mut events = database.list_recent_file_events(50_000)?;
    events.sort_by(|left, right| {
        left.detected_at
            .cmp(&right.detected_at)
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut sessions: Vec<SessionTuple> = Vec::new();
    let mut current_events: Vec<&crate::models::FileEvent> = Vec::new();

    for event in &events {
        if let Some(last) = current_events.last() {
            let gap = time_gap_seconds(&last.detected_at, &event.detected_at).unwrap_or(0);
            if gap > gap_seconds || current_events.len() >= MAX_EVENTS_PER_SESSION {
                if sessions.len() >= MAX_SESSIONS {
                    break;
                }
                sessions.push(build_session(database, &current_events)?);
                current_events.clear();
            }
        }
        current_events.push(event);
    }
    if !current_events.is_empty() && sessions.len() < MAX_SESSIONS {
        sessions.push(build_session(database, &current_events)?);
    }

    let created = database.replace_activity_sessions(sessions)?;
    Ok(GenerateSessionsResponse {
        sessions_created: created,
    })
}

pub fn update_session(
    database: &Database,
    request: UpdateSessionRequest,
) -> Result<ActivitySession, ChronicleError> {
    let title = request
        .title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    database.update_session(request.session_id, title, request.project_id)
}

pub fn accept_session(
    database: &Database,
    request: SessionRequest,
) -> Result<ActivitySession, ChronicleError> {
    database.accept_session(request.session_id)
}

pub fn reject_session(
    database: &Database,
    request: SessionRequest,
) -> Result<ActivitySession, ChronicleError> {
    database.reject_session(request.session_id)
}

pub fn merge_sessions(
    database: &Database,
    target_id: i64,
    source_id: i64,
) -> Result<ActivitySession, ChronicleError> {
    database.merge_activity_sessions(target_id, source_id)
}

fn build_session(
    database: &Database,
    events: &[&crate::models::FileEvent],
) -> Result<SessionTuple, ChronicleError> {
    let event_ids: Vec<i64> = events.iter().map(|event| event.id).collect();
    let file_ids: Vec<i64> = events
        .iter()
        .filter_map(|event| event.file_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let started_at = events
        .first()
        .map(|event| event.detected_at.clone())
        .unwrap_or_default();
    let project_id = choose_project(database, &file_ids)?;
    let project_name = project_id
        .and_then(|id| database.project_name(id).ok())
        .unwrap_or_default();

    let title = if project_name.is_empty() {
        format!("Activity · {}", format_time(&started_at))
    } else {
        format!("{} · {}", project_name, format_time(&started_at))
    };

    let summary = summarize_events(events);
    Ok((title, summary, project_id, event_ids, file_ids))
}

fn choose_project(database: &Database, file_ids: &[i64]) -> Result<Option<i64>, ChronicleError> {
    if file_ids.is_empty() {
        return Ok(None);
    }
    let mapping = database.map_files_to_project_ids(file_ids)?;
    let mut counts: HashMap<i64, usize> = HashMap::new();
    for project_ids in mapping.values() {
        for project_id in project_ids {
            *counts.entry(*project_id).or_insert(0) += 1;
        }
    }
    Ok(counts
        .into_iter()
        .max_by_key(|(_project_id, count)| *count)
        .map(|(project_id, _count)| project_id))
}

fn summarize_events(events: &[&crate::models::FileEvent]) -> String {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for event in events {
        *counts.entry(event.event_type.as_str()).or_insert(0) += 1;
    }
    let parts: Vec<String> = counts
        .into_iter()
        .map(|(event_type, count)| format!("{} {}", count, event_type))
        .collect();
    parts.join(", ")
}

fn format_time(detected_at: &str) -> String {
    detected_at
        .replace('T', " ")
        .split('.')
        .next()
        .unwrap_or(detected_at)
        .to_owned()
}

fn time_gap_seconds(left: &str, right: &str) -> Option<i64> {
    let left_dt = chrono::DateTime::parse_from_rfc3339(left).ok()?;
    let right_dt = chrono::DateTime::parse_from_rfc3339(right).ok()?;
    Some((right_dt - left_dt).num_seconds())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::{
        database::Database,
        models::{
            GenerateSessionsRequest, ListSessionsRequest, SessionRequest, UpdateSessionRequest,
        },
        scanner::DiscoveredFile,
    };

    use super::*;

    fn temp_file(root: &std::path::Path, name: &str) -> std::path::PathBuf {
        let path = root.join(name);
        std::fs::write(&path, name).expect("test fixture should be written");
        path
    }

    fn discovered(path: &std::path::Path, root: &std::path::Path) -> DiscoveredFile {
        let parent = path.parent().unwrap_or(root).to_string_lossy().into_owned();
        DiscoveredFile {
            normalized_path: path.to_string_lossy().into_owned(),
            name: path.file_name().unwrap().to_string_lossy().into_owned(),
            parent_path: parent,
            extension: path
                .extension()
                .map(|value| value.to_string_lossy().into_owned()),
            size_bytes: path.metadata().map(|metadata| metadata.len()).unwrap_or(0),
            filesystem_created_at: None,
            filesystem_modified_at: "2026-06-23T12:00:00.000Z".to_owned(),
            identity_key: None,
        }
    }

    fn scan_files(database: &Database, folder_id: i64, files: &[DiscoveredFile]) {
        let run = database
            .create_scan_run(folder_id)
            .expect("scan run should start");
        database
            .stage_scan_batch(run.scan_run_id, folder_id, files, files.len() as u64, 0, 0)
            .expect("batch should stage");
        database
            .complete_scan(run.scan_run_id, folder_id, files.len() as u64, 0, 0)
            .expect("scan should complete");
    }

    fn clear_events(database: &Database) {
        let connection = database
            .connection
            .lock()
            .expect("database mutex should not be poisoned");
        connection
            .execute("DELETE FROM file_events", [])
            .expect("events should be cleared");
    }

    fn insert_event(
        database: &Database,
        file_id: i64,
        folder_id: i64,
        event_type: &str,
        detected_at: &str,
    ) {
        let connection = database
            .connection
            .lock()
            .expect("database mutex should not be poisoned");
        connection
            .execute(
                "INSERT INTO file_events (
                    file_id, indexed_folder_id, event_type, detected_at,
                    filesystem_time, old_path, new_path, confidence, event_source
                 ) VALUES (?1, ?2, ?3, ?4, NULL, NULL, NULL, 1.0, 'reconciliation')",
                rusqlite::params![file_id, folder_id, event_type, detected_at],
            )
            .expect("event should be inserted");
    }

    #[test]
    fn session_generation_splits_on_large_time_gaps() {
        let directory = TempDir::new().expect("temp dir should be created");
        let database = Database::open(directory.path().join("chronicle.sqlite3"))
            .expect("database should open");
        let root = directory.path().join("root");
        std::fs::create_dir(&root).expect("root should be created");
        let file_a = temp_file(&root, "morning.txt");

        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .expect("folder should register")
            .folder;
        scan_files(&database, folder.id, &[discovered(&file_a, &root)]);

        let files = database
            .list_present_file_records()
            .expect("files should load");
        let file_id = files[0].id;
        clear_events(&database);

        insert_event(
            &database,
            file_id,
            folder.id,
            "created",
            "2026-06-23T08:00:00.000Z",
        );
        insert_event(
            &database,
            file_id,
            folder.id,
            "modified",
            "2026-06-23T08:05:00.000Z",
        );
        insert_event(
            &database,
            file_id,
            folder.id,
            "modified",
            "2026-06-23T10:00:00.000Z",
        );

        let response = generate_sessions(
            &database,
            GenerateSessionsRequest {
                gap_minutes: Some(30),
            },
        )
        .expect("sessions should be generated");
        assert_eq!(
            response.sessions_created, 2,
            "large gap should split sessions"
        );

        let sessions = list_sessions(&database, ListSessionsRequest { project_id: None })
            .expect("sessions should list");
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn sessions_link_to_related_projects() {
        let directory = TempDir::new().expect("temp dir should be created");
        let database = Database::open(directory.path().join("chronicle.sqlite3"))
            .expect("database should open");
        let root = directory.path().join("root");
        std::fs::create_dir(&root).expect("root should be created");
        let file_a = temp_file(&root, "report.txt");
        let file_b = temp_file(&root, "note.txt");

        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .expect("folder should register")
            .folder;
        scan_files(
            &database,
            folder.id,
            &[discovered(&file_a, &root), discovered(&file_b, &root)],
        );

        let files = database
            .list_present_file_records()
            .expect("files should load");
        let project = database
            .create_project("Reports", "", &[files[0].id])
            .expect("project should be created");
        clear_events(&database);

        insert_event(
            &database,
            files[0].id,
            folder.id,
            "modified",
            "2026-06-23T09:00:00.000Z",
        );
        insert_event(
            &database,
            files[1].id,
            folder.id,
            "modified",
            "2026-06-23T09:02:00.000Z",
        );

        generate_sessions(
            &database,
            GenerateSessionsRequest {
                gap_minutes: Some(30),
            },
        )
        .expect("sessions should be generated");

        let sessions = list_sessions(&database, ListSessionsRequest { project_id: None })
            .expect("sessions should list");
        assert_eq!(sessions.len(), 1);
        let detail = get_session(
            &database,
            &SessionRequest {
                session_id: sessions[0].session.id,
            },
        )
        .expect("session detail should load");
        assert_eq!(detail.session.project_id, Some(project.project.id));
    }

    #[test]
    fn session_title_can_be_edited() {
        let directory = TempDir::new().expect("temp dir should be created");
        let database = Database::open(directory.path().join("chronicle.sqlite3"))
            .expect("database should open");
        let root = directory.path().join("root");
        std::fs::create_dir(&root).expect("root should be created");
        let file = temp_file(&root, "draft.txt");

        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .expect("folder should register")
            .folder;
        scan_files(&database, folder.id, &[discovered(&file, &root)]);

        let files = database
            .list_present_file_records()
            .expect("files should load");
        clear_events(&database);
        insert_event(
            &database,
            files[0].id,
            folder.id,
            "created",
            "2026-06-23T09:00:00.000Z",
        );

        generate_sessions(
            &database,
            GenerateSessionsRequest {
                gap_minutes: Some(30),
            },
        )
        .expect("sessions should be generated");

        let sessions = list_sessions(&database, ListSessionsRequest { project_id: None })
            .expect("sessions should list");
        assert_eq!(sessions.len(), 1);
        let session_id = sessions[0].session.id;

        let updated = update_session(
            &database,
            UpdateSessionRequest {
                session_id,
                title: Some("Deep work".to_owned()),
                project_id: None,
            },
        )
        .expect("session should be updated");
        assert_eq!(updated.title, "Deep work");
        assert_eq!(updated.status, crate::models::ActivitySessionStatus::Edited);
    }

    #[test]
    fn sessions_do_not_include_productivity_scores() {
        let directory = TempDir::new().expect("temp dir should be created");
        let database = Database::open(directory.path().join("chronicle.sqlite3"))
            .expect("database should open");
        let root = directory.path().join("root");
        std::fs::create_dir(&root).expect("root should be created");
        let file = temp_file(&root, "task.txt");

        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .expect("folder should register")
            .folder;
        scan_files(&database, folder.id, &[discovered(&file, &root)]);

        let files = database
            .list_present_file_records()
            .expect("files should load");
        clear_events(&database);
        insert_event(
            &database,
            files[0].id,
            folder.id,
            "created",
            "2026-06-23T09:00:00.000Z",
        );
        insert_event(
            &database,
            files[0].id,
            folder.id,
            "modified",
            "2026-06-23T09:05:00.000Z",
        );

        generate_sessions(
            &database,
            GenerateSessionsRequest {
                gap_minutes: Some(30),
            },
        )
        .expect("sessions should be generated");

        let sessions = list_sessions(&database, ListSessionsRequest { project_id: None })
            .expect("sessions should list");
        let detail = get_session(
            &database,
            &SessionRequest {
                session_id: sessions[0].session.id,
            },
        )
        .expect("session detail should load");

        let summary = detail.session.event_summary.to_lowercase();
        assert!(
            !summary.contains("score")
                && !summary.contains("productivity")
                && !summary.contains("points"),
            "session summary should not contain productivity metrics: {summary}"
        );
    }
}
