---
id: 7zl07z
title: TUI review column is always empty although tickets have review status
type: bug
status: done
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-09-23T11:25:38.715322Z
updated_at: 2026-09-25T11:45:12.033053Z
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

## Implementation notes

**Both named leads were eliminated with evidence, not just read-throughs.**

- `App::col_indices` (`src/tui/app.rs`) - built a `Review`-status ticket and
  a `[Todo, InProgress, Review, Done]` column set directly in a unit test
  (`col_indices_returns_review_ticket_when_review_is_a_configured_column`).
  It found the ticket correctly. Eliminated.
- `load_tickets`'s `.filter_map(|raw| raw.parse::<Ticket>().ok())`
  (`src/tui/mod.rs`) - parsed the real on-disk `review`-status ticket file
  `pgej72_*.md` directly through `Ticket::from_str` in a throwaway
  diagnostic test. It parsed successfully with `status == Review`.
  Eliminated.
- Went further and drove the **real compiled binary** interactively via
  `tmux` against this repo's own `tickets/` directory
  (`./target/debug/tickets`, i.e. the `cargo build` output, not the stale
  `tickets` on `PATH`). The `review` column rendered correctly with exactly
  the 6 tickets `tickets list --status review` reports
  (`3p7tpt 0awkzp 7se1mu pgej72 chphvf zgfki6`). **The bug does not
  reproduce on current `main` at all** - `col_indices`, `load_tickets` and
  the config-driven `kanban_columns` are all correct as committed.

**Actual root cause: the bug report was filed against the stale globally
installed `tickets` binary on `PATH`, not `cargo run --`.** Proved this
directly: ran the exact same `tickets/` directory through the stale
`~/.cargo/bin/tickets` (a Mach-O binary built 7 May, long before the
`review` status existed, per its file mtime) via the same `tmux` harness.
Result: the `review` column header rendered correctly between
`in-progress` and `done` (its config-driven label is just a string), but
the column was completely empty - the exact reported symptom - while
several `todo` tickets were also silently missing from their column. The
stale binary's compiled `TicketStatus` enum predates `Review`, so
`Ticket::from_str` fails to deserialize `status: review` (and evidently
some other now-valid front matter) into it, and `load_tickets`'s
`.filter_map(...).ok())` silently drops every such ticket. This is exactly
the environment trap already documented in `docs/contributor-orientation.md`
and the ticket-writing instructions ("`tickets` on `PATH` is STALE").

**No code defect exists in the current codebase for this ticket to fix.**
The two required tests were added anyway, as permanent regression coverage
(not diagnostics - the throwaway parse-fixture and tmux-driven checks above
were removed after use):

- `col_indices_returns_review_ticket_when_review_is_a_configured_column`
  (`src/tui/app.rs`) - the specific case from the ticket.
- `every_configured_status_is_reachable_in_exactly_one_column`
  (`src/tui/app.rs`) - builds a board whose columns are **every**
  `TicketStatus` variant via `clap::ValueEnum::value_variants()` (not a
  hand-maintained list), puts one ticket per status, and asserts each
  ticket appears in exactly one column, never zero, never more than one.
  This is the discriminating test: verified it by temporarily short-
  circuiting `col_indices` to return `vec![]` for `Review` and confirming
  it fails with a precise "missing from its configured column" message,
  then reverted. Because it enumerates variants structurally rather than
  naming them, it will automatically pick up the next status added and
  needs no maintenance when that happens.

**Silent-drop follow-up: deliberately not fixed here, flagged as a
follow-up ticket.** `load_tickets`'s `.filter_map(|raw| raw.parse::<Ticket>().ok())`
genuinely can make tickets vanish from the board with zero user-visible
indication - the stale-binary investigation above demonstrates the exact
mechanism (any front matter the running binary's `TicketStatus`/`FrontMatter`
can't deserialize is dropped silently). It is not the cause of *this*
ticket's reported bug on current `main` (nothing on `main` fails to parse),
so fixing it here would be scope creep on a ticket about a specific
symptom. But the hazard is real and worth its own ticket: surface a flash
message or footer warning (`App.flash` already exists as the mechanism) when
`load_tickets` (or its initial-load / file-watch-reload call sites in
`src/tui/mod.rs`) drops a file that failed to parse, so a user is not left
wondering why a ticket silently disappeared from the board.
