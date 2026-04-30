---
id: 04HRDu
title: auto-commit - hard error if no .git
type: task
status: draft
tags: []
parent: null
blocked_by:
- w0-eam
created_at: 2026-04-30T23:12:39.597025Z
updated_at: 2026-04-30T23:15:33.593622Z
---

If auto_commit = true and no .git found, fail with a clear error message. No actual committing yet — just the guard. Blocked by config parse slice. Ref: docs/prd-config-file.md