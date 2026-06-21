# Event Model

File events are immutable observations of metadata changes.

## Future event types

- `created`: a path appears within an indexed folder.
- `modified`: relevant filesystem metadata changes.
- `deleted`: a previously present path is no longer observed.
- `renamed`: identity is retained while the name changes.
- `moved`: identity is retained while the containing path changes.

Events include detection time, optional filesystem time, old and new paths, confidence, and source. Rename and move confidence must remain explicit because filesystem APIs do not always provide stable identity.

Milestone 0 defines the schema and models only. It does not watch the filesystem, scan folders, infer identity, or insert events.
