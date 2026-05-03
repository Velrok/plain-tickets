---
id: myvd79
title: Add filters to list command
type: task
status: todo
tags:
- cli
parent: null
blocked_by: []
created_at: 2026-05-03T22:43:11.210743Z
updated_at: 2026-05-03T22:43:19.077668Z
---

Add --status, --type, and --tag filter flags to `tickets list`.

## Behaviour
- `--status` repeatable, OR semantics
- `--type` repeatable, OR semantics  
- `--tag` repeatable, AND semantics (ticket must carry every specified tag)
- All filters are optional; no flags = current behaviour unchanged
- Always reads from `all/` only — filters do not change the source directory