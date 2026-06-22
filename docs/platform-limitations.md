# Platform Limitations

Windows is the first supported platform. The Rust core avoids unnecessary Windows-only assumptions, but installers and visual verification currently target Windows 11 and WebView2.

Path normalization, case sensitivity, junctions, symbolic links, network shares, removable drives, permissions, and timestamp availability differ by filesystem and platform. These rules belong behind the Rust platform boundary.

Milestone 1 canonicalizes existing roots and uses case-folded comparison keys on Windows. It reports available, missing-or-moved, permission-denied, and inaccessible states. Metadata alone cannot reliably distinguish a moved root from a missing one, so Chronicle deliberately uses the combined label rather than claiming move detection.

Symbolic links and canonical directory entries that escape the authorized root are skipped. Windows reparse-point behavior can vary by filesystem; junctions that the standard library identifies as links are skipped. Network shares, removable media, OneDrive placeholders, ACL changes, path-length limits, and files disappearing during enumeration can produce warnings or a failed scan. A failed scan keeps the prior snapshot. Creation time is nullable because not every filesystem exposes it, and timestamp precision varies.
