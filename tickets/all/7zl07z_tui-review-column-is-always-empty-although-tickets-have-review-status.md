---
id: 7zl07z
title: TUI review column is always empty although tickets have review status
type: bug
status: todo
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-09-23T11:25:38.715322Z
updated_at: 2026-09-23T11:25:38.715322Z
---

## Symptom

The TUI kanban board renders the `review` column, but the column is always
empty. Tickets whose status is `review` appear in no column at all - they are
not misfiled into a neighbouring column, they are simply absent from the
board.

Observed on 2026-09-23 with five tickets in `review` at the time: `3p7tpt`,
`0awkzp`, `pgej72`, `7se1mu`, `zgfki6`.

The column header itself renders correctly, between `in-progress` and `done`,
so the configured `kanban_columns` list is being read.

## Reproduce

1. Set at least one ticket to `review` - `cargo run -- edit <id> --status review`.
2. Confirm the CLI agrees: `cargo run -- list` shows it as `review`.
3. Launch the TUI - `cargo run`.
4. The `review` column is empty. The ticket is in no column.

## Evidence

`tickets list` reports the same tickets as `review` correctly, so the ticket
files on disk are right and the CLI read path works. The defect is specific
to the TUI's view of them.

The `todo` column showed a `down-arrow` scroll indicator at the time, so
columns with more content than fits do signal it. The `review` column showed
no such indicator - consistent with it genuinely holding zero tickets rather
than holding them off-screen.

## Root cause not yet determined

Deliberately left for whoever picks this up. Two places were glanced at while
writing this ticket. Treat both as unverified leads, not findings, and
confirm or eliminate them yourself rather than trusting this note:

- `App::col_indices` in `src/tui/app.rs` filters by `status == *status` and
  then by the filter query. Read once and looked correct, but was not tested
  against a `review` fixture.
- `load_tickets` in `src/tui/mod.rs` builds its list with
  `.filter_map(|raw| raw.parse::<Ticket>().ok())`, which silently discards any
  ticket that fails to parse rather than reporting it.

Whatever the cause turns out to be, note that a silent drop is its own
problem. A ticket that cannot be parsed should not vanish from the board with
no indication - that hides data loss from the user. If the fix does not
already address that, say so and it can be a follow-up ticket.

## Context that may be relevant

The `review` status is recent - added by ticket `xkbw11` on 2026-09-23 - and
this repo's own `tickets/.tickets.toml` opts into the column while the
compiled-in `TuiConfig::default_kanban_columns` deliberately does not. Ticket
`pgej72` changed `col_indices` shortly afterwards to also apply a filter
query. Either is a plausible starting point, but neither has been confirmed.

## Test first

- unit - `col_indices` returns a ticket whose status is `review` when
  `review` is among the configured columns
- unit or e2e - a ticket in every configured status appears in exactly one
  column, with none missing
- the "every status is reachable" test is the one worth keeping - it would
  have caught this, and will catch the next status added
