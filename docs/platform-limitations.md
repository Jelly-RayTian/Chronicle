# Platform Limitations

Windows is the first supported platform. The Rust core avoids unnecessary Windows-only assumptions, but installers and visual verification currently target Windows 11 and WebView2.

Path normalization, case sensitivity, junctions, symbolic links, network shares, removable drives, permissions, and timestamp availability differ by filesystem and platform. These rules belong behind the Rust platform boundary.

Milestone 0 does not access user-selected paths, so it does not yet resolve these edge cases. Availability states and nullable filesystem creation timestamps are present in the schema for later milestones.
