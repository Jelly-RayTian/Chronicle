# Security Policy

## Reporting a vulnerability

Please use the repository's private security advisory feature. Do not open a public issue containing sensitive paths, database contents, credentials, or exploit details.

## Security model

Chronicle stores its application database in the operating system application-data directory. The database contains only Chronicle data. Milestone 0 does not scan folders or read file contents.

The frontend receives typed, sanitized errors. Raw SQLite errors and absolute database paths stay in Rust. Chronicle does not include analytics, telemetry, cloud storage, or metadata upload.

Dependencies are locked with `package-lock.json` and `Cargo.lock`. Security-sensitive dependency updates should run all tests and rebuild the installer.

## Data safety

Clearing Chronicle data must only remove Chronicle's own database and settings. It must never delete, modify, or move original files.
