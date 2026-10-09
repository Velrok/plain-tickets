---
id: 3fdgtlvb98ee4
title: Archive all done tickets with a --all-done flag
type: feature
status: todo
parent: null
blocked_by: []
tags:
- archive
created_at: 2026-10-09T01:57:44.250Z
updated_at: 2026-10-09T01:57:44.250Z
---

`tickets archive` only has `--all-rejected`. Archiving every done ticket needs IDs piped through xargs. Add `--all-done` (and maybe `--all-terminal`).

Acceptance: `tickets archive --all-done` archives every active done ticket; conflicts with naming IDs, like `--all-rejected`.