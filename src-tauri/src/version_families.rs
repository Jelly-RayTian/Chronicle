use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::{
    database::Database,
    errors::ChronicleError,
    models::{
        AddVersionFamilyMemberRequest, FileVersionCandidate, ListVersionFamiliesRequest,
        MergeVersionFamiliesRequest, RemoveVersionFamilyMemberRequest, RenameVersionFamilyRequest,
        SplitVersionFamilyRequest, SuggestVersionFamiliesResponse, VersionFamily,
        VersionFamilyDecision, VersionFamilyDetail, VersionFamilyRequest, VersionFamilyStatus,
        VersionFamilySummary,
    },
};

const SCORE_THRESHOLD: f64 = 0.55;

const VERSION_TOKENS: &[&str] = &[
    "final",
    "draft",
    "submit",
    "submission",
    "copy",
    "backup",
    "bak",
    "rev",
    "revision",
    "ver",
    "version",
];

#[derive(Debug, Clone)]
struct CandidateInfo {
    file_id: i64,
    indexed_folder_id: i64,
    parent_path: String,
    extension: Option<String>,
    identity_key: Option<String>,
    stem: String,
    tokens: Vec<String>,
    filesystem_modified_at: String,
    first_indexed_at: String,
}

fn normalize_base_name(name: &str) -> String {
    let stem = Path::new(name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(name);
    let lowered = stem.to_lowercase();
    let mut normalized = String::with_capacity(lowered.len());
    for character in lowered.chars() {
        if character == '-' || character == '_' || character == '.' {
            normalized.push(' ');
        } else {
            normalized.push(character);
        }
    }
    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_version_token(token: &str) -> Option<String> {
    if token.chars().all(|character| character.is_ascii_digit()) {
        return Some(token.to_owned());
    }
    if let Some(rest) = token.strip_prefix('v')
        && rest.chars().all(|character| character.is_ascii_digit())
    {
        return Some(token.to_owned());
    }
    for known in VERSION_TOKENS {
        if token == *known {
            return Some((*known).to_owned());
        }
    }
    for known in VERSION_TOKENS {
        if let Some(rest) = token.strip_prefix(known)
            && rest.chars().all(|character| character.is_ascii_digit())
        {
            return Some((*known).to_owned());
        }
    }
    None
}

fn stem_and_tokens(name: &str) -> (String, Vec<String>) {
    let normalized = normalize_base_name(name);
    let mut stem = Vec::new();
    let mut tokens = Vec::new();
    for token in normalized.split_whitespace() {
        if let Some(version_token) = is_version_token(token) {
            tokens.push(version_token);
        } else {
            stem.push(token);
        }
    }
    (stem.join(" "), tokens)
}

fn extension_key(extension: &Option<String>) -> String {
    extension
        .as_deref()
        .map(str::to_lowercase)
        .unwrap_or_default()
}

fn pair_score(
    a: &CandidateInfo,
    b: &CandidateInfo,
    path_history_confidence: Option<f64>,
) -> (f64, bool, bool) {
    let extension_match =
        extension_key(&a.extension) == extension_key(&b.extension) && a.extension.is_some();

    let token_intersection: Vec<&String> = a
        .tokens
        .iter()
        .filter(|token| b.tokens.contains(token))
        .collect();
    let token_union: HashSet<&String> = a.tokens.iter().chain(b.tokens.iter()).collect();
    let version_score = if a.tokens.is_empty() && b.tokens.is_empty() {
        0.0
    } else if a.tokens.is_empty() || b.tokens.is_empty() {
        1.0
    } else if token_intersection.is_empty() {
        0.0
    } else {
        token_intersection.len() as f64 / token_union.len().max(1) as f64
    };

    let name_similarity = if a.stem.is_empty() && b.stem.is_empty() {
        if token_intersection.is_empty() {
            0.0
        } else {
            1.0
        }
    } else if a.stem.is_empty() || b.stem.is_empty() {
        0.0
    } else if a.stem == b.stem {
        1.0
    } else {
        let common_prefix = a
            .stem
            .chars()
            .zip(b.stem.chars())
            .take_while(|(left, right)| left == right)
            .count();
        let max_len = a.stem.len().max(b.stem.len()).max(1) as f64;
        (common_prefix as f64) / max_len
    };

    let folder_proximity = if a.parent_path == b.parent_path {
        1.0
    } else if a.indexed_folder_id == b.indexed_folder_id {
        0.6
    } else {
        0.2
    };

    let mut score = 0.5 * name_similarity
        + 0.2 * version_score
        + 0.1 * f64::from(extension_match)
        + 0.2 * folder_proximity;

    let mut identity_match = a
        .identity_key
        .as_deref()
        .zip(b.identity_key.as_deref())
        .is_some_and(|(left, right)| !left.is_empty() && left == right && left.starts_with("ino:"));
    let mut path_history = path_history_confidence.is_some();

    // Identity and path-history evidence are only meaningful when the names already
    // look related. This prevents coarse Windows fingerprints (size + whole-second
    // timestamps) from grouping unrelated files that happen to share metadata.
    if name_similarity < 0.8 {
        identity_match = false;
        path_history = false;
    }

    if identity_match {
        score = (score + 0.2).min(1.0);
    }

    if let Some(confidence) = path_history_confidence {
        score = (score + 0.15 * confidence).min(1.0);
    }

    // Avoid grouping unrelated files across folders purely by a vague stem match.
    if folder_proximity < 1.0 && version_score == 0.0 && !identity_match && !path_history {
        score = 0.0;
    }

    (score, identity_match, path_history)
}

fn analyze_candidates(candidates: &[FileVersionCandidate]) -> Vec<CandidateInfo> {
    candidates
        .iter()
        .map(|candidate| {
            let (stem, tokens) = stem_and_tokens(&candidate.name);
            CandidateInfo {
                file_id: candidate.id,
                indexed_folder_id: candidate.indexed_folder_id,
                parent_path: candidate.parent_path.clone(),
                extension: candidate.extension.clone(),
                identity_key: candidate.identity_key.clone(),
                stem,
                tokens,
                filesystem_modified_at: candidate.filesystem_modified_at.clone(),
                first_indexed_at: candidate.first_indexed_at.clone(),
            }
        })
        .collect()
}

fn build_path_history_lookup(
    candidates: &[FileVersionCandidate],
    history_entries: &[(i64, String, f64)],
) -> HashMap<(i64, i64), f64> {
    let path_to_id: HashMap<String, i64> = candidates
        .iter()
        .map(|candidate| (candidate.normalized_path.clone(), candidate.id))
        .collect();

    let mut pair_confidence: HashMap<(i64, i64), f64> = HashMap::new();
    for (file_id, old_path, confidence) in history_entries {
        if let Some(&other_id) = path_to_id.get(old_path) {
            if other_id == *file_id {
                continue;
            }
            let key = if *file_id < other_id {
                (*file_id, other_id)
            } else {
                (other_id, *file_id)
            };
            let entry = pair_confidence.entry(key).or_insert(0.0);
            *entry = entry.max(*confidence);
        }
    }
    pair_confidence
}

type ScoredEdge = (usize, usize, f64, bool, bool);

fn cluster_files(
    analyzed: &[CandidateInfo],
    pair_confidence: &HashMap<(i64, i64), f64>,
) -> (Vec<Vec<usize>>, Vec<ScoredEdge>) {
    let count = analyzed.len();
    let mut edges: Vec<ScoredEdge> = Vec::new();
    for i in 0..count {
        for j in (i + 1)..count {
            let key = if analyzed[i].file_id < analyzed[j].file_id {
                (analyzed[i].file_id, analyzed[j].file_id)
            } else {
                (analyzed[j].file_id, analyzed[i].file_id)
            };
            let history = pair_confidence.get(&key).copied();
            let (score, identity_match, path_history) =
                pair_score(&analyzed[i], &analyzed[j], history);
            if score >= SCORE_THRESHOLD {
                edges.push((i, j, score, identity_match, path_history));
            }
        }
    }

    let mut parent: Vec<usize> = (0..count).collect();
    fn find(parent: &mut [usize], index: usize) -> usize {
        if parent[index] != index {
            parent[index] = find(parent, parent[index]);
        }
        parent[index]
    }
    fn union(parent: &mut [usize], a: usize, b: usize) {
        let root_a = find(parent, a);
        let root_b = find(parent, b);
        if root_a != root_b {
            parent[root_b] = root_a;
        }
    }

    for (i, j, _score, _identity, _history) in &edges {
        union(&mut parent, *i, *j);
    }

    let mut components: HashMap<usize, Vec<usize>> = HashMap::new();
    for index in 0..count {
        let root = find(&mut parent, index);
        components.entry(root).or_default().push(index);
    }

    let clusters: Vec<Vec<usize>> = components
        .into_values()
        .filter(|group| group.len() >= 2)
        .collect();

    (clusters, edges)
}

fn chronological_key(candidate: &CandidateInfo) -> impl Ord + Clone {
    (
        candidate.filesystem_modified_at.clone(),
        candidate.first_indexed_at.clone(),
        candidate.file_id,
    )
}

fn family_confidence_and_evidence(
    cluster: &[usize],
    analyzed: &[CandidateInfo],
    edges: &[ScoredEdge],
) -> (f64, String) {
    let mut total = 0.0;
    let mut count = 0_usize;
    let mut any_identity = false;
    let mut any_history = false;
    let mut extension: Option<String> = None;
    let mut folder_summary = "same_parent";
    let mut all_tokens: HashSet<String> = HashSet::new();

    let ids_in_cluster: HashSet<i64> = cluster
        .iter()
        .map(|index| analyzed[*index].file_id)
        .collect();

    for (i, j, score, identity, history) in edges {
        if ids_in_cluster.contains(&analyzed[*i].file_id)
            && ids_in_cluster.contains(&analyzed[*j].file_id)
        {
            total += *score;
            count += 1;
            if *identity {
                any_identity = true;
            }
            if *history {
                any_history = true;
            }
        }
    }

    let mut parent_paths: HashSet<&str> = HashSet::new();
    let mut folders: HashSet<i64> = HashSet::new();
    for index in cluster {
        let candidate = &analyzed[*index];
        parent_paths.insert(candidate.parent_path.as_str());
        folders.insert(candidate.indexed_folder_id);
        if extension.is_none() {
            extension = candidate.extension.as_deref().map(str::to_lowercase);
        }
        for token in &candidate.tokens {
            all_tokens.insert(token.clone());
        }
    }
    if folders.len() > 1 || parent_paths.len() > 1 {
        folder_summary = "cross_folder";
    }

    let confidence = if count == 0 {
        0.5
    } else {
        (total / count as f64).clamp(0.0, 1.0)
    };

    let evidence = format!(
        "members={};extension={:?};folder={};version_tokens={:?};identity_evidence={};path_history_evidence={};avg_pair_confidence={:.2}",
        cluster.len(),
        extension.unwrap_or_default(),
        folder_summary,
        Vec::from_iter(all_tokens),
        any_identity,
        any_history,
        confidence
    );

    (confidence, evidence)
}

fn build_family_data(
    cluster: &[usize],
    analyzed: &[CandidateInfo],
    edges: &[ScoredEdge],
) -> Option<(String, Vec<i64>, f64, String)> {
    if cluster.len() < 2 {
        return None;
    }

    let mut ordered: Vec<usize> = cluster.to_vec();
    ordered.sort_by_key(|index| chronological_key(&analyzed[*index]));

    let file_ids: Vec<i64> = ordered
        .iter()
        .map(|index| analyzed[*index].file_id)
        .collect();

    let display_name = {
        let first = &analyzed[ordered[0]];
        let ext = first.extension.as_deref().unwrap_or("");
        if ext.is_empty() {
            first.stem.clone()
        } else {
            format!("{}.{}", first.stem, ext.to_lowercase())
        }
    };

    let (confidence, evidence) = family_confidence_and_evidence(cluster, analyzed, edges);

    Some((display_name, file_ids, confidence, evidence))
}

pub fn suggest_version_families(
    database: &Database,
    folder_id: Option<i64>,
) -> Result<SuggestVersionFamiliesResponse, ChronicleError> {
    let candidates = database.files_for_version_analysis(folder_id)?;
    if candidates.len() < 2 {
        return Ok(SuggestVersionFamiliesResponse {
            families_created: 0,
        });
    }

    let already_grouped: HashSet<i64> = database.existing_family_file_ids()?;
    let filtered: Vec<FileVersionCandidate> = candidates
        .into_iter()
        .filter(|candidate| !already_grouped.contains(&candidate.id))
        .collect();
    if filtered.len() < 2 {
        return Ok(SuggestVersionFamiliesResponse {
            families_created: 0,
        });
    }

    let file_ids: Vec<i64> = filtered.iter().map(|candidate| candidate.id).collect();
    let history_entries = database.path_history_for_candidates(&file_ids)?;
    let pair_confidence = build_path_history_lookup(&filtered, &history_entries);

    let analyzed = analyze_candidates(&filtered);
    let (clusters, edges) = cluster_files(&analyzed, &pair_confidence);

    if clusters.is_empty() {
        return Ok(SuggestVersionFamiliesResponse {
            families_created: 0,
        });
    }

    let family_data: Vec<(String, Vec<i64>, f64, String)> = clusters
        .iter()
        .filter_map(|cluster| build_family_data(cluster, &analyzed, &edges))
        .collect();

    let created = family_data.len();
    database.persist_version_family_suggestions(&family_data)?;

    Ok(SuggestVersionFamiliesResponse {
        families_created: created,
    })
}

pub fn list_version_families(
    database: &Database,
    request: &ListVersionFamiliesRequest,
) -> Result<Vec<VersionFamilySummary>, ChronicleError> {
    database.list_version_families(request)
}

pub fn get_version_family(
    database: &Database,
    request: &VersionFamilyRequest,
) -> Result<VersionFamilyDetail, ChronicleError> {
    database.get_version_family(request.family_id)
}

pub fn accept_version_family(
    database: &Database,
    request: &VersionFamilyRequest,
) -> Result<VersionFamily, ChronicleError> {
    database.update_version_family_decision(
        request.family_id,
        VersionFamilyStatus::Confirmed,
        Some(VersionFamilyDecision::Accepted),
    )
}

pub fn reject_version_family(
    database: &Database,
    request: &VersionFamilyRequest,
) -> Result<VersionFamily, ChronicleError> {
    database.update_version_family_decision(
        request.family_id,
        VersionFamilyStatus::Rejected,
        Some(VersionFamilyDecision::Rejected),
    )
}

pub fn rename_version_family(
    database: &Database,
    request: &RenameVersionFamilyRequest,
) -> Result<VersionFamily, ChronicleError> {
    database.rename_version_family(request.family_id, &request.display_name)
}

pub fn split_version_family(
    database: &Database,
    request: &SplitVersionFamilyRequest,
) -> Result<VersionFamilyDetail, ChronicleError> {
    database.split_version_family(
        request.family_id,
        &request.file_ids,
        request.display_name.as_deref(),
    )
}

pub fn merge_version_families(
    database: &Database,
    request: &MergeVersionFamiliesRequest,
) -> Result<VersionFamilyDetail, ChronicleError> {
    database.merge_version_families(request.target_family_id, &request.source_family_ids)
}

pub fn add_version_family_member(
    database: &Database,
    request: &AddVersionFamilyMemberRequest,
) -> Result<VersionFamilyDetail, ChronicleError> {
    database.add_version_family_member(request.family_id, request.file_id)
}

pub fn remove_version_family_member(
    database: &Database,
    request: &RemoveVersionFamilyMemberRequest,
) -> Result<VersionFamilyDetail, ChronicleError> {
    database.remove_version_family_member(request.family_id, request.file_id)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, fs, sync::atomic::AtomicBool};

    use tempfile::TempDir;

    use crate::{database::Database, folders::register_folder, scanner};

    use super::suggest_version_families;

    fn setup() -> (TempDir, Database, std::path::PathBuf, i64) {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        let database = Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        let folder = register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("root should register: {error}"))
            .folder;
        (directory, database, root, folder.id)
    }

    fn scan(database: &Database, root: &std::path::Path, folder_id: i64) {
        let run = database
            .create_scan_run(folder_id)
            .unwrap_or_else(|error| panic!("scan should start: {error}"));
        let counts = scanner::traverse(
            database,
            run.scan_run_id,
            folder_id,
            root,
            &AtomicBool::new(false),
        )
        .unwrap_or_else(|error| panic!("scan should traverse: {error}"));
        database
            .complete_scan(
                run.scan_run_id,
                folder_id,
                counts.files_seen,
                counts.warnings,
                counts.errors,
            )
            .unwrap_or_else(|error| panic!("scan should publish: {error}"));
    }

    #[test]
    fn obvious_version_token_sequence_forms_one_family() {
        let (_directory, database, root, folder_id) = setup();
        fs::write(root.join("essay.docx"), b"a").unwrap_or_else(|e| panic!("{e}"));
        fs::write(root.join("essay-final.docx"), b"b").unwrap_or_else(|e| panic!("{e}"));
        fs::write(root.join("essay-final-v2.docx"), b"c").unwrap_or_else(|e| panic!("{e}"));
        scan(&database, &root, folder_id);

        let result = suggest_version_families(&database, None)
            .unwrap_or_else(|error| panic!("suggestion should succeed: {error:?}"));
        assert_eq!(result.families_created, 1);

        let families = database
            .list_version_families(&crate::models::ListVersionFamiliesRequest {
                status: None,
                folder_id: None,
            })
            .unwrap_or_else(|error| panic!("list should succeed: {error:?}"));
        assert_eq!(families.len(), 1);
        assert_eq!(families[0].member_count, 3);

        let detail = database
            .get_version_family(families[0].family.id)
            .unwrap_or_else(|error| panic!("detail should load: {error:?}"));
        let names: HashSet<String> = detail
            .members
            .iter()
            .map(|member| member.file.name.clone())
            .collect();
        assert_eq!(
            names,
            HashSet::from([
                "essay.docx".to_owned(),
                "essay-final.docx".to_owned(),
                "essay-final-v2.docx".to_owned(),
            ])
        );

        let mut previous: Option<&str> = None;
        for member in &detail.members {
            if let Some(previous_modified) = previous {
                assert!(
                    member.file.filesystem_modified_at.as_str() >= previous_modified,
                    "members should be ordered by modified time"
                );
            }
            previous = Some(&member.file.filesystem_modified_at);
        }
    }

    #[test]
    fn unrelated_same_extension_files_stay_ungrouped() {
        let (_directory, database, root, folder_id) = setup();
        fs::write(root.join("report.docx"), b"a").unwrap_or_else(|e| panic!("{e}"));
        fs::write(root.join("summary.docx"), b"b").unwrap_or_else(|e| panic!("{e}"));
        scan(&database, &root, folder_id);

        let result = suggest_version_families(&database, None)
            .unwrap_or_else(|error| panic!("suggestion should succeed: {error:?}"));
        assert_eq!(result.families_created, 0);
    }

    #[test]
    fn cross_folder_version_files_are_suggested_with_lower_confidence() {
        let (_directory, database, root, folder_id) = setup();
        let project_a = root.join("project-a");
        let project_b = root.join("project-b");
        fs::create_dir(&project_a).unwrap_or_else(|e| panic!("{e}"));
        fs::create_dir(&project_b).unwrap_or_else(|e| panic!("{e}"));
        fs::write(project_a.join("essay.docx"), b"a").unwrap_or_else(|e| panic!("{e}"));
        fs::write(project_b.join("essay-final.docx"), b"b").unwrap_or_else(|e| panic!("{e}"));
        scan(&database, &root, folder_id);

        let result = suggest_version_families(&database, None)
            .unwrap_or_else(|error| panic!("suggestion should succeed: {error:?}"));
        assert_eq!(result.families_created, 1);

        let detail = database
            .list_version_families(&crate::models::ListVersionFamiliesRequest {
                status: None,
                folder_id: None,
            })
            .unwrap_or_else(|error| panic!("list should succeed: {error:?}"));
        assert_eq!(detail[0].member_count, 2);
        let suggestion = database
            .version_family_suggestions(detail[0].family.id)
            .unwrap_or_else(|error| panic!("suggestions should load: {error:?}"));
        assert!(suggestion[0].confidence < 1.0);
        assert!(suggestion[0].evidence.contains("cross_folder"));
    }

    #[test]
    fn chronological_order_respects_modified_time() {
        let (_directory, database, root, folder_id) = setup();
        let older = root.join("draft.docx");
        let newer = root.join("draft-final.docx");
        fs::write(&older, b"old").unwrap_or_else(|e| panic!("{e}"));
        std::thread::sleep(std::time::Duration::from_millis(50));
        fs::write(&newer, b"new").unwrap_or_else(|e| panic!("{e}"));
        scan(&database, &root, folder_id);

        suggest_version_families(&database, None)
            .unwrap_or_else(|error| panic!("suggestion should succeed: {error:?}"));
        let families = database
            .list_version_families(&crate::models::ListVersionFamiliesRequest {
                status: None,
                folder_id: None,
            })
            .unwrap_or_else(|error| panic!("list should succeed: {error:?}"));
        assert!(!families.is_empty(), "expected at least one family");
        let detail = database
            .get_version_family(families[0].family.id)
            .unwrap_or_else(|error| panic!("detail should load: {error:?}"));
        assert_eq!(detail.members[0].file.name, "draft.docx");
        assert_eq!(detail.members[1].file.name, "draft-final.docx");
    }

    #[test]
    fn accept_reject_split_merge_and_member_edits_persist() {
        let (_directory, database, root, folder_id) = setup();
        fs::write(root.join("essay.docx"), b"a").unwrap_or_else(|e| panic!("{e}"));
        fs::write(root.join("essay-final.docx"), b"b").unwrap_or_else(|e| panic!("{e}"));
        fs::write(root.join("essay-v2.docx"), b"c").unwrap_or_else(|e| panic!("{e}"));
        scan(&database, &root, folder_id);

        suggest_version_families(&database, None)
            .unwrap_or_else(|error| panic!("suggestion should succeed: {error:?}"));
        let families = database
            .list_version_families(&crate::models::ListVersionFamiliesRequest {
                status: None,
                folder_id: None,
            })
            .unwrap_or_else(|error| panic!("list should succeed: {error:?}"));
        let family_id = families[0].family.id;

        let accepted = database
            .update_version_family_decision(
                family_id,
                crate::models::VersionFamilyStatus::Confirmed,
                Some(crate::models::VersionFamilyDecision::Accepted),
            )
            .unwrap_or_else(|error| panic!("accept should succeed: {error:?}"));
        assert_eq!(
            accepted.status,
            crate::models::VersionFamilyStatus::Confirmed
        );

        let detail = database
            .get_version_family(family_id)
            .unwrap_or_else(|error| panic!("detail should load: {error:?}"));
        let v2_file_id = detail
            .members
            .iter()
            .find(|member| member.file.name == "essay-v2.docx")
            .map(|member| member.file.id)
            .unwrap_or_else(|| panic!("v2 member should exist"));

        let split = database
            .split_version_family(family_id, &[v2_file_id], Some("essay-v2.docx"))
            .unwrap_or_else(|error| panic!("split should succeed: {error:?}"));
        assert_eq!(split.members.len(), 1);
        assert_eq!(
            split.family.status,
            crate::models::VersionFamilyStatus::Confirmed
        );

        let original = database
            .get_version_family(family_id)
            .unwrap_or_else(|error| panic!("original should load: {error:?}"));
        assert_eq!(original.members.len(), 2);
        assert_eq!(
            original.family.status,
            crate::models::VersionFamilyStatus::Superseded
        );

        let merged = database
            .merge_version_families(family_id, &[split.family.id])
            .unwrap_or_else(|error| panic!("merge should succeed: {error:?}"));
        assert_eq!(merged.members.len(), 3);

        let rejected = database
            .update_version_family_decision(
                family_id,
                crate::models::VersionFamilyStatus::Rejected,
                Some(crate::models::VersionFamilyDecision::Rejected),
            )
            .unwrap_or_else(|error| panic!("reject should succeed: {error:?}"));
        assert_eq!(
            rejected.status,
            crate::models::VersionFamilyStatus::Rejected
        );

        let lone = root.join("lone.docx");
        fs::write(&lone, b"x").unwrap_or_else(|e| panic!("{e}"));
        let run = database
            .create_scan_run(folder_id)
            .unwrap_or_else(|error| panic!("scan should start: {error}"));
        let counts = scanner::traverse(
            &database,
            run.scan_run_id,
            folder_id,
            &root,
            &AtomicBool::new(false),
        )
        .unwrap_or_else(|error| panic!("scan should traverse: {error}"));
        database
            .complete_scan(
                run.scan_run_id,
                folder_id,
                counts.files_seen,
                counts.warnings,
                counts.errors,
            )
            .unwrap_or_else(|error| panic!("scan should publish: {error}"));
        let lone_file = database
            .list_file_records(folder_id)
            .unwrap_or_default()
            .into_iter()
            .find(|file| file.name == "lone.docx")
            .unwrap_or_else(|| panic!("lone file should exist"));

        let added = database
            .add_version_family_member(family_id, lone_file.id)
            .unwrap_or_else(|error| panic!("add should succeed: {error:?}"));
        assert_eq!(added.members.len(), 4);

        let removed = database
            .remove_version_family_member(family_id, lone_file.id)
            .unwrap_or_else(|error| panic!("remove should succeed: {error:?}"));
        assert_eq!(removed.members.len(), 3);
    }
}
