---
id: 9y20si
title: TUI reads kanban columns from config
type: task
status: in-progress
tags:
- tui
- config
parent: null
blocked_by: []
created_at: 2026-05-03T23:37:49.885880Z
updated_at: 2026-05-03T23:53:51.005016Z
---

The TUI currently hardcodes kanban_columns = ["todo", "in-progress", "done"] in tui::run(). It should read from [tui] kanban_columns in .tickets.toml instead, falling back to the same defaults if not set.

## Acceptance criteria
- [ ] Config struct has a TuiConfig with kanban_columns: Vec<String>
- [ ] Default is ["todo", "in-progress", "done"]
- [ ] tui::run() uses cfg.tui.kanban_columns instead of the hardcoded vec
- [ ] .tickets.toml [tui] kanban_columns is parsed correctly