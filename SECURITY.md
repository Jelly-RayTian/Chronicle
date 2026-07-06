# Security Policy

## Reporting a vulnerability

Please use the repository's private security advisory feature. Do not open a public issue containing sensitive paths, database contents, credentials, or exploit details.

## Security model

Chronicle stores its application database in the operating system application-data directory. Scanning is limited to explicitly selected canonical roots; metadata scans read directory entries and filesystem metadata without opening file contents, and skip symbolic links by default.

Content indexing is opt-in per folder and restricted by extension, file size, and exclusion patterns. Indexed text contents are stored only in the local SQLite database and are never uploaded or analyzed by AI.

The frontend receives typed, sanitized errors. Raw SQLite errors and absolute database paths stay in Rust. Chronicle does not include analytics, telemetry, cloud storage, or metadata upload. The diagnostics export contains counts and status only; no file paths, names, or contents are included.

Scan commands accept a registered folder id rather than an arbitrary path. Incomplete discovery is isolated in a staging table and cannot replace the last successful snapshot. Removing an indexed folder clears Chronicle metadata for that root but leaves original files untouched.

Dependencies are locked with `package-lock.json` and `Cargo.lock`. Security-sensitive dependency updates should run all tests, the migration upgrade test, and rebuild the installer.

## Data safety

Clearing Chronicle data must only remove Chronicle's own database and settings. It must never delete, modify, or move original files.
