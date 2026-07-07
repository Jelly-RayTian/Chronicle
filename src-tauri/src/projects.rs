#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::{
    database::Database,
    errors::ChronicleError,
    models::{
        AcceptProjectRequest, AddProjectMemberRequest, CreateProjectRequest, ListProjectsRequest,
        Project, ProjectDetail, ProjectRequest, ProjectSummary, RejectProjectRequest,
        RemoveProjectMemberRequest, SuggestProjectsRequest, SuggestProjectsResponse,
        UpdateProjectRequest,
    },
};

const MAX_GROUP_SIZE: usize = 50;
const MIN_GROUP_SIZE: usize = 2;
const EVENT_LOOKBACK_LIMIT: u32 = 10_000;

type ProjectSuggestionMembers = Vec<(i64, f64, String, String)>;
type ReconciledProjectGroups = Vec<(String, ProjectSuggestionMembers)>;

#[derive(Debug, Clone)]
struct CandidateGroup {
    name: String,
    source: &'static str,
    evidence: String,
    base_confidence: f64,
    file_ids: Vec<i64>,
}

pub fn list_projects(
    database: &Database,
    request: ListProjectsRequest,
) -> Result<Vec<ProjectSummary>, ChronicleError> {
    database.list_projects(&request)
}

pub fn get_project(
    database: &Database,
    request: &ProjectRequest,
) -> Result<ProjectDetail, ChronicleError> {
    database.get_project(request.project_id)
}

pub fn create_project(
    database: &Database,
    request: CreateProjectRequest,
) -> Result<ProjectDetail, ChronicleError> {
    let name = request.name.trim();
    if name.is_empty() {
        return Err(ChronicleError::PathEncoding);
    }
    let description = request.description.unwrap_or_default();
    database.create_project(name, &description, &request.file_ids)
}

pub fn update_project(
    database: &Database,
    request: UpdateProjectRequest,
) -> Result<Project, ChronicleError> {
    let name = request
        .name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let description = request
        .description
        .as_deref()
        .map(str::trim)
        .map(str::to_owned);
    database.update_project(request.project_id, name, description.as_deref())
}

pub fn accept_project(
    database: &Database,
    request: AcceptProjectRequest,
) -> Result<Project, ChronicleError> {
    database.accept_project(request.project_id)
}

pub fn reject_project(
    database: &Database,
    request: RejectProjectRequest,
) -> Result<Project, ChronicleError> {
    database.reject_project(request.project_id)
}

pub fn add_project_member(
    database: &Database,
    request: AddProjectMemberRequest,
) -> Result<ProjectDetail, ChronicleError> {
    database.add_project_member(request.project_id, request.file_id)
}

pub fn remove_project_member(
    database: &Database,
    request: RemoveProjectMemberRequest,
) -> Result<ProjectDetail, ChronicleError> {
    database.remove_project_member(request.project_id, request.file_id)
}

pub fn suggest_projects(
    database: &Database,
    _request: SuggestProjectsRequest,
) -> Result<SuggestProjectsResponse, ChronicleError> {
    let files = database.list_present_file_records()?;
    let version_families = database.list_confirmed_version_families()?;
    let events = database.list_recent_file_events(EVENT_LOOKBACK_LIMIT)?;
    let folders = database.list_indexed_folders()?;

    let groups = build_candidate_groups(&files, &version_families, &events, &folders);
    let reconciled = reconcile_groups(groups);

    let created = database.replace_suggested_projects(reconciled)?;
    Ok(SuggestProjectsResponse {
        projects_created: created,
    })
}

fn build_candidate_groups(
    files: &[crate::models::FileRecord],
    version_families: &[(i64, String, Vec<i64>)],
    events: &[crate::models::FileEvent],
    folders: &[crate::models::IndexedFolder],
) -> Vec<CandidateGroup> {
    let mut groups: Vec<CandidateGroup> = Vec::new();

    folder_groups(files, &mut groups);
    keyword_groups(files, &mut groups);
    git_repo_groups(files, folders, &mut groups);
    version_family_groups(version_families, &mut groups);
    temporal_groups(events, &mut groups);

    groups
}

fn folder_groups(files: &[crate::models::FileRecord], groups: &mut Vec<CandidateGroup>) {
    let mut by_folder: HashMap<&str, Vec<i64>> = HashMap::new();
    for file in files {
        by_folder
            .entry(file.parent_path.as_str())
            .or_default()
            .push(file.id);
    }
    for (parent_path, file_ids) in by_folder {
        if file_ids.len() < MIN_GROUP_SIZE || file_ids.len() > MAX_GROUP_SIZE {
            continue;
        }
        let name = folder_display_name(parent_path);
        groups.push(CandidateGroup {
            name: format!("{name} files"),
            source: "folder_proximity",
            evidence: format!("folder={}", parent_path),
            base_confidence: 0.7,
            file_ids,
        });
    }
}

fn folder_display_name(parent_path: &str) -> String {
    Path::new(parent_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(parent_path)
        .to_owned()
}

fn keyword_groups(files: &[crate::models::FileRecord], groups: &mut Vec<CandidateGroup>) {
    let mut by_keyword: HashMap<String, Vec<i64>> = HashMap::new();
    for file in files {
        for keyword in extract_keywords(&file.name) {
            by_keyword.entry(keyword).or_default().push(file.id);
        }
    }
    for (keyword, file_ids) in by_keyword {
        let unique: Vec<i64> = file_ids
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        if unique.len() < MIN_GROUP_SIZE || unique.len() > 20 {
            continue;
        }
        groups.push(CandidateGroup {
            name: format!("{keyword} files"),
            source: "filename_keyword",
            evidence: format!("keyword={}", keyword),
            base_confidence: 0.6,
            file_ids: unique,
        });
    }
}

fn extract_keywords(name: &str) -> Vec<String> {
    let stem = Path::new(name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(name);
    let lowered = stem.to_lowercase();
    let mut keywords = Vec::new();
    for token in lowered.split(|character: char| !character.is_alphanumeric()) {
        if token.len() < 3 || token.chars().all(|character| character.is_ascii_digit()) {
            continue;
        }
        if STOPWORDS.contains(&token) {
            continue;
        }
        keywords.push(token.to_owned());
    }
    keywords
}

fn git_repo_groups(
    files: &[crate::models::FileRecord],
    folders: &[crate::models::IndexedFolder],
    groups: &mut Vec<CandidateGroup>,
) {
    let folder_paths: HashMap<i64, &str> = folders
        .iter()
        .map(|folder| (folder.id, folder.normalized_path.as_str()))
        .collect();
    let mut git_cache: HashMap<PathBuf, Option<PathBuf>> = HashMap::new();
    let mut by_repo: HashMap<PathBuf, Vec<i64>> = HashMap::new();

    for file in files {
        let Some(folder_path) = folder_paths.get(&file.indexed_folder_id) else {
            continue;
        };
        let Some(repo_root) = find_git_root(&file.parent_path, folder_path, &mut git_cache) else {
            continue;
        };
        by_repo.entry(repo_root).or_default().push(file.id);
    }

    for (repo_root, file_ids) in by_repo {
        if file_ids.len() < MIN_GROUP_SIZE || file_ids.len() > MAX_GROUP_SIZE {
            continue;
        }
        let name = folder_display_name(repo_root.to_str().unwrap_or(""));
        groups.push(CandidateGroup {
            name: format!("{name} repo"),
            source: "git_repo",
            evidence: format!("git_repo={}", repo_root.display()),
            base_confidence: 0.9,
            file_ids,
        });
    }
}

fn find_git_root(
    parent_path: &str,
    folder_path: &str,
    cache: &mut HashMap<PathBuf, Option<PathBuf>>,
) -> Option<PathBuf> {
    let mut current = Path::new(parent_path).to_path_buf();
    let root = Path::new(folder_path);
    loop {
        if let Some(cached) = cache.get(&current) {
            return cached.clone();
        }
        let git_path = current.join(".git");
        let found = std::fs::symlink_metadata(&git_path)
            .ok()
            .map(|metadata| metadata.is_dir())
            .unwrap_or(false);
        let result = if found {
            Some(current.clone())
        } else if current == root || !current.starts_with(root) {
            None
        } else {
            current.pop();
            continue;
        };
        cache.insert(current.clone(), result.clone());
        return result;
    }
}

fn version_family_groups(
    version_families: &[(i64, String, Vec<i64>)],
    groups: &mut Vec<CandidateGroup>,
) {
    for (family_id, display_name, file_ids) in version_families {
        if file_ids.len() < MIN_GROUP_SIZE {
            continue;
        }
        groups.push(CandidateGroup {
            name: format!("{display_name} versions"),
            source: "version_family",
            evidence: format!("version_family_id={}", family_id),
            base_confidence: 0.85,
            file_ids: file_ids.clone(),
        });
    }
}

fn temporal_groups(events: &[crate::models::FileEvent], groups: &mut Vec<CandidateGroup>) {
    let mut by_hour: HashMap<String, Vec<i64>> = HashMap::new();
    for event in events {
        let Some(file_id) = event.file_id else {
            continue;
        };
        let hour_bucket = event
            .detected_at
            .split(':')
            .next()
            .map(|value| value.to_owned());
        if let Some(bucket) = hour_bucket {
            by_hour.entry(bucket).or_default().push(file_id);
        }
    }
    for (bucket, file_ids) in by_hour {
        let unique: Vec<i64> = file_ids
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        if unique.len() < MIN_GROUP_SIZE || unique.len() > 15 {
            continue;
        }
        groups.push(CandidateGroup {
            name: format!("Activity {}", bucket.replace('T', " ")),
            source: "temporal_cooccurrence",
            evidence: format!("hour={}", bucket),
            base_confidence: 0.5,
            file_ids: unique,
        });
    }
}

fn reconcile_groups(groups: Vec<CandidateGroup>) -> ReconciledProjectGroups {
    let mut sorted = groups;
    sorted.sort_by(|left, right| {
        right
            .base_confidence
            .partial_cmp(&left.base_confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut assigned: HashSet<i64> = HashSet::new();
    let mut result: ReconciledProjectGroups = Vec::new();

    for group in sorted {
        let retained: Vec<i64> = group
            .file_ids
            .into_iter()
            .filter(|file_id| !assigned.contains(file_id))
            .collect();
        if retained.len() < MIN_GROUP_SIZE {
            continue;
        }
        let confidence = size_adjusted_confidence(group.base_confidence, retained.len());
        let members: ProjectSuggestionMembers = retained
            .iter()
            .map(|file_id| {
                (
                    *file_id,
                    confidence,
                    group.evidence.clone(),
                    group.source.to_owned(),
                )
            })
            .collect();
        for file_id in &retained {
            assigned.insert(*file_id);
        }
        result.push((group.name, members));
    }

    result
}

fn size_adjusted_confidence(base: f64, size: usize) -> f64 {
    let penalty = if size > 20 {
        0.15
    } else if size > 10 {
        0.08
    } else {
        0.0
    };
    (base - penalty).clamp(0.0, 1.0)
}

const STOPWORDS: &[&str] = &[
    "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
    "from", "as", "is", "was", "are", "were", "be", "been", "have", "has", "had", "do", "does",
    "did", "will", "would", "could", "should", "may", "might", "must", "can", "this", "that",
    "these", "those", "not", "no", "yes", "copy", "final", "draft", "backup", "bak",
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use tempfile::TempDir;

    use crate::{
        database::Database,
        models::{
            AddProjectMemberRequest, CreateProjectRequest, ListProjectsRequest, ProjectRequest,
            ProjectStatus, RemoveProjectMemberRequest, SuggestProjectsRequest,
            UpdateProjectRequest,
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

    #[test]
    fn manual_project_creation_and_membership_edits_persist() {
        let directory = TempDir::new().expect("temp dir should be created");
        let database = Database::open(directory.path().join("chronicle.sqlite3"))
            .expect("database should open");
        let root = directory.path().join("root");
        std::fs::create_dir(&root).expect("root should be created");
        let file_a = temp_file(&root, "report_june.txt");
        let file_b = temp_file(&root, "summary_june.txt");

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
        assert_eq!(files.len(), 2);

        let detail = create_project(
            &database,
            CreateProjectRequest {
                name: "June docs".to_owned(),
                description: None,
                file_ids: vec![files[0].id],
            },
        )
        .expect("project should be created");
        assert_eq!(detail.project.name, "June docs");
        assert_eq!(detail.members.len(), 1);

        let with_member = add_project_member(
            &database,
            AddProjectMemberRequest {
                project_id: detail.project.id,
                file_id: files[1].id,
            },
        )
        .expect("member should be added");
        assert_eq!(with_member.members.len(), 2);

        let without_member = remove_project_member(
            &database,
            RemoveProjectMemberRequest {
                project_id: detail.project.id,
                file_id: files[0].id,
            },
        )
        .expect("member should be removed");
        assert_eq!(without_member.members.len(), 1);

        let renamed = update_project(
            &database,
            UpdateProjectRequest {
                project_id: detail.project.id,
                name: Some("June summary".to_owned()),
                description: None,
            },
        )
        .expect("project should be renamed");
        assert_eq!(renamed.name, "June summary");
    }

    #[test]
    fn suggestion_grouping_creates_reviewable_projects() {
        let directory = TempDir::new().expect("temp dir should be created");
        let database = Database::open(directory.path().join("chronicle.sqlite3"))
            .expect("database should open");
        let root = directory.path().join("root");
        std::fs::create_dir(&root).expect("root should be created");
        let file_a = temp_file(&root, "essay_draft.txt");
        let file_b = temp_file(&root, "essay_final.txt");
        let file_c = temp_file(&root, "notes.txt");

        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .expect("folder should register")
            .folder;
        scan_files(
            &database,
            folder.id,
            &[
                discovered(&file_a, &root),
                discovered(&file_b, &root),
                discovered(&file_c, &root),
            ],
        );

        let response = suggest_projects(&database, SuggestProjectsRequest { folder_id: None })
            .expect("suggestions should run");
        assert!(
            response.projects_created > 0,
            "should create at least one project"
        );

        let all = list_projects(&database, ListProjectsRequest { status: None })
            .expect("projects should list");
        let suggested: Vec<_> = all
            .into_iter()
            .filter(|summary| summary.project.status == ProjectStatus::Suggested)
            .collect();
        assert!(!suggested.is_empty());

        let target = &suggested[0];
        let ids: HashSet<i64> = target.file_ids.iter().copied().collect();
        let file_ids: HashSet<i64> = database
            .list_present_file_records()
            .expect("files should load")
            .into_iter()
            .map(|file| file.id)
            .collect();
        assert!(ids.is_subset(&file_ids));

        let accepted = accept_project(
            &database,
            AcceptProjectRequest {
                project_id: target.project.id,
            },
        )
        .expect("project should be accepted");
        assert_eq!(accepted.status, ProjectStatus::Active);

        let rejected = reject_project(
            &database,
            RejectProjectRequest {
                project_id: accepted.id,
            },
        )
        .expect("project should be rejected");
        assert_eq!(rejected.status, ProjectStatus::Rejected);
    }

    #[test]
    fn duplicate_project_member_is_rejected() {
        let directory = TempDir::new().expect("temp dir should be created");
        let database = Database::open(directory.path().join("chronicle.sqlite3"))
            .expect("database should open");
        let root = directory.path().join("root");
        std::fs::create_dir(&root).expect("root should be created");
        let file = temp_file(&root, "single.txt");

        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .expect("folder should register")
            .folder;
        scan_files(&database, folder.id, &[discovered(&file, &root)]);

        let files = database
            .list_present_file_records()
            .expect("files should load");
        let detail = create_project(
            &database,
            CreateProjectRequest {
                name: "Single".to_owned(),
                description: None,
                file_ids: vec![files[0].id],
            },
        )
        .expect("project should be created");

        let result = add_project_member(
            &database,
            AddProjectMemberRequest {
                project_id: detail.project.id,
                file_id: files[0].id,
            },
        );
        assert!(matches!(
            result,
            Err(ChronicleError::DuplicateProjectMember)
        ));
    }

    #[test]
    fn project_detail_and_timeline_are_loaded() {
        let directory = TempDir::new().expect("temp dir should be created");
        let database = Database::open(directory.path().join("chronicle.sqlite3"))
            .expect("database should open");
        let root = directory.path().join("root");
        std::fs::create_dir(&root).expect("root should be created");
        let file = temp_file(&root, "tracked.txt");

        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .expect("folder should register")
            .folder;
        scan_files(&database, folder.id, &[discovered(&file, &root)]);

        let files = database
            .list_present_file_records()
            .expect("files should load");
        let detail = create_project(
            &database,
            CreateProjectRequest {
                name: "Tracked".to_owned(),
                description: None,
                file_ids: vec![files[0].id],
            },
        )
        .expect("project should be created");

        let loaded = get_project(
            &database,
            &ProjectRequest {
                project_id: detail.project.id,
            },
        )
        .expect("detail should load");
        assert_eq!(loaded.members.len(), 1);

        let timeline = database
            .list_project_timeline_events(detail.project.id, 10)
            .expect("timeline should load");
        assert_eq!(timeline.len(), 1);
    }
}
