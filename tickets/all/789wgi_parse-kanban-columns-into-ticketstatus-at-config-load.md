---
id: 789wgi
title: Parse kanban_columns into TicketStatus at config load
type: bug
status: todo
tags:
- config
- tui
parent: null
blocked_by: []
created_at: 2026-07-30T17:42:48.432735Z
updated_at: 2026-07-30T17:42:48.432735Z
---

## Problem

`TuiConfig::kanban_columns` is `Vec<String>` (`src/config.rs:29`), so invalid column names load without complaint and fail silently later. The two consumers also disagree on matching:

- **Grouping** (`src/tui/app.rs:41`): `status.to_string() == *col_name` — case-sensitive, exact
- **Moving** (`src/tui/app.rs:111`): `TicketStatus::from_str(&name, true)` — clap `ValueEnum`, case-insensitive

Failure modes:

1. A typo (`"in_progress"`) renders a permanently empty column, no error.
2. A wrong-case name (`"In-Progress"`) never groups anything, but a move into it *succeeds* — the ticket status is mutated to `in-progress` and it vanishes from the column it was dropped in. Silent data mutation.

Contrast `NewConfig` (`src/config.rs:21`), which already types its fields as `TicketStatus` / `TicketType` and gets this right.

## Approach — parse, do not validate

Make the type make the illegal state unrepresentable, rather than adding a validation pass that leaves `String` in place.

- Change `kanban_columns` to `Vec<TicketStatus>`; serde rejects unknown names at load with a field-path error.
- Update `TuiConfig::default_kanban_columns` to return `TicketStatus` variants.
- Change `App.columns` to `Vec<TicketStatus>`; `col_indices` becomes an enum comparison.
- Delete the now-unreachable `Err` branch in `move_ticket_to` (`src/tui/app.rs:111`) — no reparse needed.
- Render column headers via `Display`.

Note `TicketStatus` derives `Deserialize` with `#[serde(rename_all = "kebab-case")]`, so TOML keeps accepting `"in-progress"` unchanged. Existing valid configs are unaffected.

## Test changes

`config.rs:120` `tui_kanban_columns_parsed_from_config` currently asserts `["backlog", "active", "closed"]` parses successfully — it codifies the bug. Replace with:

- a valid-statuses case asserting the parsed enum values
- an invalid-name case asserting `load` returns `Err`

`app.rs` `default_columns()` helper and its ~20 call sites need the new type.
