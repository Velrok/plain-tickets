---
id: fwjmyh
title: TUI screen corrupted after move ticket
type: bug
status: draft
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-05-03T23:28:53.774131Z
updated_at: 2026-05-03T23:28:53.774131Z
---

After pressing H or L to move a ticket between columns, the screen is partially corrupted. Git auto-commit output bleeds into the TUI (visible in the done column area and footer). The terminal is not fully restored to raw/alternate-screen mode after the git subprocess runs.

## Steps to reproduce
1. Run `tickets` with auto_commit = true
2. Focus a ticket and press L to move it right
3. Observe: git commit output appears in the TUI frame

## Root cause (suspected)
`save_focused` calls `git::git_commit` which spawns a subprocess that writes to stdout/stderr. The TUI alternate screen is still active so the output bleeds into the ratatui buffer.

## Acceptance criteria
- [ ] Moving a ticket with H/L produces no visible stdout/stderr in the TUI
- [ ] Board re-renders cleanly after move