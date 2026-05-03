---
id: t0f6nq
title: TUI copy ticket id to clipboard with y
type: task
status: todo
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-05-03T23:52:08.002574Z
updated_at: 2026-05-03T23:52:08.002574Z
---

Pressing 'y' on the board view copies the focused ticket's id to the system clipboard.

## Acceptance criteria

- [ ] y keybinding added to board event loop
- [ ] Focused ticket id is written to system clipboard (arboard or similar crate)
- [ ] Brief confirmation shown (e.g. status bar flash "Copied abc123")
- [ ] No-op if no ticket is focused
- [ ] Keybinding documented in help overlay and footer hint