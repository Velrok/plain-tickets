---
id: bmtthv
title: Review the architecture and propose refactoring improvements
type: task
status: todo
tags:
- architecture
- tech-debt
parent: null
blocked_by:
- fe17ed
created_at: 2026-09-25T12:34:48.190945Z
updated_at: 2026-09-25T12:34:48.190945Z
---

## Goal

An investigation ticket, not a refactor. Produce a written review of the
current module structure and a prioritised list of proposed refactorings, each
with a rough size and a justification. Landing any of them is a separate
ticket. The deliverable is the recommendation, and explicitly includes
"leave as is" for anything that does not earn its keep.

## Why now

The codebase has grown organically through several epics and nobody has stood
back and looked at the whole shape. A handful of concrete smells have surfaced
as side findings during other work - they are recorded below so the review
starts from evidence rather than taste. Most were noticed while fixing
something else and were correctly left out of scope at the time.

## Blocked by fe17ed

`fe17ed` deletes `src/graph.rs` (475 lines, the second largest non-TUI file)
and `tests/cli_graph.rs`, and corrects the stale file layout table in
`docs/coding-style.md`. Reviewing the architecture before that lands means
reviewing 475 lines that are about to disappear, against a file map that is
known wrong. Wait for it.

## Starting evidence

These are observations, not conclusions. The review should confirm each one
against the tree before acting on it, and is free to disagree.

### 1. Ticket loading is duplicated across three paths

The strongest candidate. There are at least three places that load tickets off
disk, and they have drifted:

- `load_tickets` in `src/tui/mod.rs` - returns `(Vec<Ticket>, Vec<LoadFailure>)`
  since `fd38vu`
- the loader in `cmd_list` (`src/commands.rs`) - still `filter_map(...).ok()`,
  being fixed under `4aawv9`
- `load_active` / `load_archived` in `src/deps_graph.rs`

The same bug (silently dropping unreadable or unparseable files) had to be
found and fixed independently in the first two. The third has not been audited.
Question for the review: is there one loading seam here, and what is the right
shape for it given the callers genuinely differ in how they report failure
(TUI flash vs stderr warning)?

### 2. There is no output seam, and it has already caused a regression

Only `cmd_list` writes through a fallible `writeln!`. Every other command uses
`println!` / `print!`, which **panics on a broken pipe** - exit 101 with a
`thread 'main' panicked` message. Flagged as pre-existing during `3mqhe3`,
which had to add broken-pipe handling to `cmd_list` specifically.

So `tickets list | head -1` is now correct while `tickets show <id> | head -1`
is not. Any command that grows output inherits the bug by default. Question:
does a small output abstraction fix this uniformly, and is that worth it
against just fixing each command?

### 3. `src/commands.rs` is the obvious hotspot

732 lines holding every `cmd_<name>` plus their private helpers. It is the file
most likely to be touched by any ticket, which makes it the file most likely to
produce merge conflicts between concurrent contributors - that has happened
repeatedly. Question: split per-command, split by concern, or leave it and
accept the contention?

### 4. `TicketStatus` exhaustiveness rests on convention, not the type system

The rule "never a wildcard `_ =>` on `TicketStatus`" is enforced by a line in
`docs/contributor-orientation.md` and by reviewers noticing. Likewise "only
`done` satisfies a dependency, written positively as `== Done`". Both are real
correctness properties with no compile-time enforcement.

Note the interaction: epic `o2b2dg` (make statuses configurable) would remove
three compile-time safety nets outright - `clap::ValueEnum`, the no-wildcard
`match` in `style()`, and the
`every_configured_status_is_reachable_in_exactly_one_column` test. The review
should say what, if anything, should replace them, because `o2b2dg` needs that
answer before it starts.

### 5. Auto-commit stages more than it commits

`git_commit` in `src/git.rs` runs `git add -- <file>` then `git commit -m` with
**no pathspec**, so it sweeps in whatever else happens to be staged. Contributors
are told to work around it by passing an explicit pathspec on their own commits.
Small, contained, and a latent correctness bug rather than a style issue.

### 6. `src/tui/render.rs` mixes rendering with its own snapshot tests

643 lines. Worth a look, but the least pressing item here - list it and move on
unless the review finds something sharper.

## Deliverable

A document under `docs/` (or a note linked from the ticket) containing:

- a current module map with line counts, replacing guesswork with measurement
- each proposed refactoring: what, why, rough size, and what it unblocks
- an explicit "not worth doing" list, with reasons - this is as valuable as the
  proposals and stops the same ideas being re-litigated later
- a recommended order, accounting for `o2b2dg` if that epic is still live

Then file the accepted items as their own tickets. Do not refactor under this
ticket.
