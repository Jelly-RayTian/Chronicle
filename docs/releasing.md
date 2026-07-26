# Releasing Chronicle

## Release contract

Chronicle publishes Windows 11 x64 NSIS installers from a manually created
matching tag. The workflow never creates a tag. A public release is created only
after version validation, all frontend and Rust gates, installer build, artifact
validation, and SHA-256 generation succeed.

## Prepare

1. Keep the worktree clean and ensure `main` is at the reviewed release commit.
2. Align `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`,
   `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`.
3. Add `docs/release-notes-vX.Y.Z.md` and update README, CHANGELOG, roadmap,
   known limitations, privacy/security, testing, and screenshots.
4. Run `npm run release:validate -- --tag vX.Y.Z`.
5. Run every command in the required-checks section of `AGENTS.md`.
6. Run `npm run tauri build` and `./scripts/validate-windows-artifacts.ps1`.
7. Complete [the smoke test](smoke-test-checklist.md) with an isolated profile.

## Tag and publish

After review, a human creates and pushes the exact stable-semver tag:

```powershell
git tag -a v2.0.0 -m "Chronicle v2.0.0"
git push origin v2.0.0
```

The tag workflow repeats all checks, builds the x64 installer, validates it, and
uploads the installer plus `SHA256SUMS.txt` using the matching release-notes
file. Do not retag a different commit. Use a new patch version if a published
artifact must change.

## Artifact verification

```powershell
Get-FileHash .\Chronicle_2.0.0_x64-setup.exe -Algorithm SHA256
Get-AuthenticodeSignature .\Chronicle_2.0.0_x64-setup.exe
```

The digest must match `SHA256SUMS.txt`. v2.0.0 is unsigned, so `NotSigned` is
expected and disclosed. `HashMismatch`, `NotTrusted`, `UnknownError`, or a
missing/malformed artifact blocks release.
