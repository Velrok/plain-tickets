---
id: xkbw11
title: Add a review status between in-progress and done
type: task
status: done
tags:
- workflow
parent: null
blocked_by: []
created_at: 2026-09-23T10:28:20.779278Z
updated_at: 2026-09-23T11:00:02.358028Z
---

## Why

There is currently no way to say "the work is built but not yet checked".
A ticket goes from `in-progress` straight to `done`, so `done` ends up
meaning two different things - sometimes "finished and verified", sometimes
"the author thinks it is finished". That ambiguity showed up immediately
while working epic fyihf8, where ihqh45 was marked `done` while its
independent verification was still running.

Add a `review` status between `in-progress` and `done`.

## Behaviour

- New `review` variant on `TicketStatus`, ordered between `InProgress` and
  `Done`, serialised kebab-case as `review` like the others.
- Accepted everywhere a status is accepted - `tickets new --status review`,
  `tickets edit <id> --status review`, `tickets list --status review`, and
  as a `[tui] kanban_columns` entry.
- Rendered as `review` by `Display`.
- `review` does NOT satisfy a dependency. Only `done` ever does. This must
  stay consistent with the blocker resolution rule in epic fyihf8, so a
  ticket blocked by a `review` ticket is still blocked and is excluded by
  `list --unblocked`.
- `review` is not a terminal status - `tickets archive --all-rejected` must
  not touch it.

## Config

Add `review` to `kanban_columns` in this repo's own `tickets/.tickets.toml`,
between `in-progress` and `done`, so the board shows the new column.

Leave the compiled-in default `kanban_columns`
(`TuiConfig::default_kanban_columns`) as todo, in-progress, done. Changing
the default would silently add a column for every existing user on upgrade;
opting in via config is the safer move.

## Test first

- unit - `TicketStatus` round-trips `review` through FromStr/ValueEnum and
  Display
- unit - a `kanban_columns` list containing `review` loads without error
- e2e - `tickets new --status review` writes `status: review` to the front
  matter
- e2e - `tickets edit <id> --status review` sets it on an existing ticket
- e2e - `tickets list --status review` selects exactly those tickets
- e2e - `tickets archive --all-rejected` leaves a `review` ticket alone

## Watch out

`TicketStatus` is matched in several places - `src/domain_types.rs`
(Display), `src/config.rs`, `src/tui/`, and the list/graph rendering in
`src/commands.rs`. Adding a variant will surface non-exhaustive match errors
at compile time. Handle each explicitly rather than reaching for a wildcard
arm, so the next status addition stays equally loud.

Check the TUI kanban board still renders sensibly with five columns rather
than four, since column widths are computed from the column count.

## Implementation notes

Added `TicketStatus::Review` between `InProgress` and `Done` in
`src/domain_types.rs`, kebab-case serialised as `review`, with a `Display`
arm alongside the others.

Only two `match` statements over `TicketStatus` existed in the crate before
this change, both went non-exhaustive on `cargo build` and both were given
explicit `Review` arms (no wildcard):

- `src/domain_types.rs` — the `Display` impl.
- `src/commands.rs::cmd_list` — the `status_order` closure used to sort
  `tickets list` output. Placed `Review` right after `InProgress` in that
  ordering (`in-progress, review, todo, draft, done, rejected`) since a
  ticket awaiting review is next-most-active after one actively being
  worked — a judgement call, not specified by the ticket.

Everywhere else `TicketStatus` appears it's compared by `==`/membership or
stored in a `Vec` (filters, kanban columns, front matter), so no other arm
additions were needed. In particular `archive_all_rejected` filters on
`status == TicketStatus::Rejected` directly, so it already leaves `review`
tickets untouched with no code change required.

Checked the "review does NOT satisfy a dependency" requirement against
`src/deps_graph.rs` and `src/graph.rs`: neither currently has any
status-based blocker-resolution logic at all (edges are added purely based
on whether the blocker id is present in `all/`; `deps_graph.rs`'s own
comments confirm this is deliberately deferred to future epic-fyihf8
tickets such as `rm49xa`). So there was nothing to change there — this
ticket does not touch deps-graph work, per its own scope note, and the
requirement holds vacuously today.

Added `review` to `tickets/.tickets.toml`'s `[tui] kanban_columns`, between
`in-progress` and `done` (this repo's config already carries a leading
`draft` column too, so the resulting order is
`draft, todo, in-progress, review, done`). Left
`TuiConfig::default_kanban_columns` unchanged (`todo, in-progress, done`) as
instructed.

Added a `board_renders_five_columns` snapshot test in `src/tui/render.rs`
covering all five statuses at once (draft/todo/in-progress/review/done) at
width 100. Reviewed the generated snapshot before accepting it: column
width is `Constraint::Ratio(1, col_count)`, so it already adapts generically
to five columns (20 chars each at width 100) — each column renders a
correctly bordered card with its full title visible, no overlap or
truncation. No existing snapshot changed, since every other render.rs test
constructs its own fixed, literal column list rather than reading real
config.

Tests added (6 required by the ticket + 1 TUI rendering check):

- `domain_types::tests::status_review_round_trips_through_value_enum_and_display`
- `config::tests::tui_kanban_columns_with_review_loads`
- `new_with_status_review` (`tests/cli_new.rs`)
- `edit_updates_status_to_review` (`tests/cli_edit.rs`)
- `list_filter_status_review_returns_matching_only` (`tests/cli_list.rs`)
- `archive_all_rejected_leaves_review_ticket_alone` (`tests/cli_archive.rs`)
- `tui::render::tests::board_renders_five_columns` (`src/tui/render.rs`)

Final verification: `cargo test` — 193 passed, 0 failed, 0 ignored, across
15 binaries (up from the 186-test green baseline, +7 for the tests above).
`cargo clippy --all-targets -- -D warnings` — clean. `cargo fmt --check` —
clean. `git status --porcelain` — clean (no stray `.snap.new`).
