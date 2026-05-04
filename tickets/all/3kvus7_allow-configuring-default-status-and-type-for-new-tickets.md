---
id: 3kvus7
title: Allow configuring default status and type for new tickets
type: task
status: todo
tags: []
parent: null
blocked_by: []
created_at: 2026-05-04T00:09:08.789572Z
updated_at: 2026-05-04T00:09:17.274799Z
---

Allow users to configure default values for `--status` and `--type` when creating new tickets via `tickets new`.

## Acceptance Criteria

- Config file (e.g. `.tickets.toml` or `tickets.toml`) supports `[new]` section with `default_status` and `default_type` fields
- `tickets new` reads these defaults if no explicit flag is passed
- Explicit CLI flags still override config defaults
- `tickets init` scaffolds the config with commented-out defaults