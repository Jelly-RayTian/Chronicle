# v2.0.0 smoke-test results

Date: 2026-07-27

Platform: Windows 11 x64, WebView2

Profile: isolated under `outputs/release-profile`

Fixture: three real files under `outputs/release-fixture`

## Verified locally

| Check                                               | Result | Evidence                                                                           |
| --------------------------------------------------- | ------ | ---------------------------------------------------------------------------------- |
| Release metadata and exact `v2.0.0` tag validation  | Pass   | `npm run release:validate -- --tag v2.0.0`                                         |
| NSIS installer build                                | Pass   | `Chronicle_2.0.0_x64-setup.exe`                                                    |
| PE, minimum size, and Authenticode-state validation | Pass   | Validator reported `NotSigned`, as documented                                      |
| Final SHA-256                                       | Pass   | `40243bb3dd7155b82ab074e5a54cb3386a3831ca8d6acc00d9d84c23c70fcd02`                 |
| Silent isolated install and generated uninstaller   | Pass   | Installed under `outputs/installed-chronicle`; uninstaller exited 0 and removed it |
| First launch with empty database                    | Pass   | Real Tauri window reported native/database ready and schema V10                    |
| Language switch                                     | Pass   | Simplified Chinese to English verified in the real app                             |
| Add explicitly selected folder                      | Pass   | Only the temporary fixture root was authorized                                     |
| Metadata scan                                       | Pass   | Three files, zero warnings, zero errors                                            |
| Timeline                                            | Pass   | Three real created events rendered                                                 |
| Filename search                                     | Pass   | `project` returned `project-brief.md`                                              |
| Real screenshots                                    | Pass   | Timeline, folders, search, and settings captured at 1182×791                       |
| Uninstall/original-file safety                      | Pass   | Recursive SHA-256 comparison before/after install and uninstall was unchanged      |

## Automated safety coverage

Rust temporary-directory tests cover explicit watcher enable/disable,
coalescing, event-storm stop, content-indexing opt-in, content search,
disable/clear behavior, and original-file preservation. Frontend tests cover
the corresponding controls, accessible labels, state rendering, and dialog
keyboard focus.

## Privacy-setting verification

Monitoring and content-indexing privacy settings were not changed through
desktop automation. Their end-to-end command/service/database flows are covered
by Rust tests using temporary directories and databases: opt-in state,
authorized-root watcher processing, coalescing, content extraction/search,
disable/clear, and original-file preservation all passed. Frontend tests verify
that the controls call only the typed native boundary and render the resulting
states. This limitation of the automation environment does not change the
product default: both features require explicit per-folder opt-in.

## Release conclusion

All automated quality gates, installer validation, isolated install/uninstall,
real scan/search presentation checks, and original-file hash checks passed.
The unsigned-installer warning is a documented distribution limitation, not an
undisclosed failure. This commit is safe to tag `v2.0.0`; the tag is intentionally
not created or pushed by this preparation task.
