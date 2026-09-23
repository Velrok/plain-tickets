---
id: xkbw11
title: Add a review status between in-progress and done
type: task
status: in-progress
tags:
- workflow
parent: null
blocked_by: []
created_at: 2026-09-23T10:28:20.779278Z
updated_at: 2026-09-23T10:46:24.867078Z
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
