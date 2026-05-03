---
id: i7a5dv
title: TUI open in editor and new ticket
type: task
status: todo
tags:
- tui
parent: null
blocked_by:
- bgk1hv
created_at: 2026-05-03T23:02:18.624151Z
updated_at: 2026-05-03T23:02:48.986976Z
---

e suspends the TUI, opens the focused ticket in EDITOR, then resumes with a refreshed board. n creates a new draft ticket with status set to the current column's status, then does the same suspend/resume flow.

## Acceptance criteria
- [ ] `e` on the board suspends TUI and opens focused ticket in `$EDITOR`
- [ ] TUI resumes with refreshed board after editor exits
- [ ] `n` creates a draft ticket with status = current column's status
- [ ] `n` then opens the new ticket in `$EDITOR` via same suspend/resume flow
- [ ] New ticket appears in board after editor exits