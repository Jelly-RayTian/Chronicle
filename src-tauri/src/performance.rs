#[cfg(test)]
mod tests {
    use std::{fs, time::Instant};

    use tempfile::TempDir;

    use crate::{database::Database, folders::register_folder, scanner};

    fn create_tree(root: &std::path::Path, file_count: usize) {
        fs::create_dir(root.join("a")).unwrap_or_else(|error| panic!("dir a: {error}"));
        fs::create_dir(root.join("b")).unwrap_or_else(|error| panic!("dir b: {error}"));
        for index in 0..file_count {
            let dir = if index % 2 == 0 { "a" } else { "b" };
            let path = root.join(dir).join(format!("file_{index}.txt"));
            fs::write(&path, format!("content {index}").as_bytes())
                .unwrap_or_else(|error| panic!("write {path:?}: {error}"));
        }
    }

    fn scan(root: &std::path::Path, database: &Database, folder_id: i64) {
        let run = database
            .create_scan_run(folder_id)
            .unwrap_or_else(|error| panic!("scan start: {error}"));
        let counts = scanner::traverse(
            database,
            run.scan_run_id,
            folder_id,
            root,
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap_or_else(|error| panic!("traverse: {error}"));
        database
            .complete_scan(
                run.scan_run_id,
                folder_id,
                counts.files_seen,
                counts.warnings,
                counts.errors,
            )
            .unwrap_or_else(|error| panic!("complete: {error}"));
    }

    fn run_with(count: usize) -> f64 {
        let directory = TempDir::new().unwrap_or_else(|error| panic!("tempdir: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root: {error}"));
        create_tree(&root, count);
        let database = Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database: {error}"));
        let folder = register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("register: {error}"))
            .folder;
        let start = Instant::now();
        scan(&root, &database, folder.id);
        let elapsed = start.elapsed().as_secs_f64();
        let files = database
            .list_file_records(folder.id)
            .unwrap_or_else(|error| panic!("list files: {error}"));
        assert_eq!(files.len(), count, "all files should be indexed");
        elapsed
    }

    #[test]
    fn small_tree_scans_quickly() {
        let elapsed = run_with(10);
        assert!(elapsed < 5.0, "small tree scan took {elapsed}s");
    }

    #[test]
    fn medium_tree_scans_within_reasonable_time() {
        let elapsed = run_with(200);
        assert!(elapsed < 15.0, "medium tree scan took {elapsed}s");
    }

    #[test]
    fn large_tree_scans_without_crash() {
        let elapsed = run_with(2000);
        assert!(elapsed < 60.0, "large tree scan took {elapsed}s");
    }
}
