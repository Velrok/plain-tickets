---
id: jhzbj0
title: TUI live reload on file changes
type: task
status: done
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-05-03T23:32:10.375521Z
updated_at: 2026-05-03T23:35:14.375865Z
---

Watch the tickets/all/ directory for file system events and automatically reload app state when ticket files change on disk. This enables external edits (e.g. via CLI or editor) to appear in the board without quitting.

## Acceptance criteria
- [ ] File watcher monitors `tickets/all/` for create, modify, and delete events
- [ ] App state reloads automatically when a change is detected
- [ ] Board re-renders within ~500ms of the file system event
- [ ] No flicker or cursor jump on reload when nothing has moved
- [ ] Watcher does not interfere with the TUI event loop (non-blocking)