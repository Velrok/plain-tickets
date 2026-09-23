---
id: 3p7tpt
title: list --unblocked filters to tickets with no unfinished blockers
type: task
status: in-progress
tags:
  - deps-graph
parent: fyihf8
blocked_by: []
created_at: 2026-09-23T10:14:02.399599Z
updated_at: 2026-09-23T10:50:20.597299Z
---

## Behaviour

New boolean `--unblocked` flag on the existing `list` command. Filters to
tickets where every blocker is empty or done. Reuses the existing list table,
id status type title with dynamic column widths, rather than introducing a
new output format. There is deliberately no standalone `unblocked`
subcommand.

## Test first

- e2e - a ticket with no blockers at all is listed
- e2e - a ticket blocked by a todo ticket is not listed
- e2e - a ticket whose only blocker is done is listed

## Implementation notes

- Added `--unblocked: bool` to `ListArgs` (`src/application_types.rs`).
- `cmd_list` (`src/commands.rs`) loads the active-ticket map via
  `deps_graph::load_active` only when `--unblocked` is set, and filters
  with a new `deps_graph::is_unblocked(active, ticket)` predicate,
  composed with the existing status/type/tag filter chain. No new output
  format — same `id status type title` table.
- **Resolution seam** (for the next contributor, e.g. rm49xa/0awkzp):
  lives in `src/deps_graph.rs`, next to the existing `resolve_blocker` /
  `BlockerResolution` seam built for `deps-graph`:
  - `fn blocker_satisfied(active, &BlockerResolution) -> bool` — the
    single place that decides whether a resolved blocker counts as
    satisfied. Compares `TicketStatus == TicketStatus::Done` (not an
    exhaustive match), so it keeps working unchanged once `review` lands
    as a new status — a `review` blocker still blocks.
  - `pub(crate) fn is_unblocked(active, ticket) -> bool` — true when
    every `blocked_by` id resolves (via `resolve_blocker`) and is
    satisfied (via `blocker_satisfied`), vacuously true for an empty
    list. This is what `list --unblocked` calls, and what `deps-graph`
    should switch to using once it needs a boolean rather than an edge.
  - `pub(crate) fn load_active` — was already private to the module;
    widened visibility so `commands::cmd_list` can build the same active
    map `deps_graph` builds internally, rather than re-implementing
    ticket loading.
  - Scope is intentionally the in-active-set case only: `resolve_blocker`
    still only ever returns `Active` for an id found in `all/` — a
    `blocked_by` id that's archived or missing resolves to `None`, so
    `is_unblocked` currently treats it as unsatisfied (blocking). Widening
    `BlockerResolution` to add `Archived`/`Missing` variants (7se1mu,
    xptucc, t9p76e) and updating `blocker_satisfied`'s match arm for them
    is exactly the seam rm49xa extends — no new predicate needed.
- Judgment call: chose not to have `cmd_list` re-parse tickets into the
  active map from the `Vec<Ticket>` it already built for the table (no
  `Clone` on `Ticket`/`FrontMatter`), so `--unblocked` does a second,
  independent directory read via `deps_graph::load_active`. Simple and
  consistent with `deps-graph`'s own loading; revisit only if `list`
  becomes performance-sensitive.
- Verified: `cargo test` → 189 passed, 0 failed, 0 ignored (was 186
  before, +3 new e2e tests: `list_unblocked_includes_ticket_with_no_blockers`,
  `list_unblocked_excludes_ticket_blocked_by_todo`,
  `list_unblocked_includes_ticket_whose_only_blocker_is_done`).
  `cargo clippy --all-targets -- -D warnings` clean. `cargo fmt --check`
  clean.
- Did not touch `--status`/`--type`/`--tag` composition (0awkzp) beyond
  not breaking it — the existing filter chain composes `--unblocked` with
  the others via a plain `.filter().filter()` chain, unchanged in shape.
