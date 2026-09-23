---
id: 00ruhc
title: TUI move ticket between columns
type: task
status: done
tags:
- tui
parent: null
blocked_by:
- bgk1hv
created_at: 2026-05-03T23:02:08.072301Z
updated_at: 2026-05-03T23:35:13.191230Z
---

H and L move the focused ticket left and right between Kanban columns, updating the status field in the ticket file. Triggers auto-commit if configured. Board re-renders immediately.

## Acceptance criteria
- [ ] `H` moves focused ticket to the left column (decrements status)
- [ ] `L` moves focused ticket to the right column (increments status)
- [ ] Ticket file `status` field is updated on disk
- [ ] Auto-commit fires if `git.auto_commit = true`
- [ ] Board re-renders immediately after move
- [ ] No-op if already at leftmost/rightmost column