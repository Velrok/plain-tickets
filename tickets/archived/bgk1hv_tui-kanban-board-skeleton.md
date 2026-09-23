---
id: bgk1hv
title: TUI kanban board skeleton
type: task
status: done
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-05-03T23:01:59.982982Z
updated_at: 2026-05-03T23:35:12.891909Z
---

Launch ratatui app when tickets is run with no subcommand. Reads active tickets from all/. Renders a Kanban board with configurable columns from .tickets.toml [tui] kanban_columns (default: todo, in-progress, done). hjkl and arrow keys navigate between columns and tickets. q quits.

## Acceptance criteria
- [ ] `tickets` with no subcommand launches the TUI
- [ ] Kanban columns read from `[tui] kanban_columns` in `.tickets.toml`, defaulting to `todo`, `in-progress`, `done`
- [ ] Tickets from `all/` rendered as cards under their status column
- [ ] `h`/`←` and `l`/`→` move focus between columns
- [ ] `j`/`↓` and `k`/`↑` move focus between tickets in a column
- [ ] `q` quits