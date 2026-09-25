---
id: y64m0g
title: Rename graph command to graph-blockers
type: task
status: draft
tags:
- ux
parent: null
blocked_by: []
created_at: 2026-07-30T23:10:03.482057Z
updated_at: 2026-09-25T12:15:48.978953Z
---


The `graph` command name doesn't communicate what it traverses (blocker edges). A ticket with no blockers of its own renders as a single node, giving no hint that it may still have children or downstream dependents — confusing in practice.

## Acceptance criteria
- [ ] Rename `graph` subcommand to `graph-blockers` (same behaviour: no id = forest of unblocked roots, id = blocker chain for that ticket)
- [ ] Keep `graph` as a hidden/deprecated alias for one release so existing scripts don't break
- [ ] Update --help text and docs
- [ ] Tree nodes with status `done`/`rejected` are pruned from the tree by default; `--include-done` flag shows them

## Archived 2026-09-25 — superseded

Existed only on `origin/main` as a `draft`; never present in local `main`. Local work
took a different and newer direction: rather than renaming `graph` to `graph-blockers`,
epic `fyihf8` builds `deps-graph` as the blocker-tree renderer and `fe17ed` removes the
`graph` subcommand outright. A rename plus a deprecated alias is moot once the command
is deleted.

The `done`/`rejected` pruning criterion here is separately covered: `7se1mu` (done)
handles archived-and-done blockers in `deps-graph`, and the deliberate asymmetry between
`deps-graph` and `list --unblocked` for active-but-done blockers is settled and
documented in `docs/contributor-orientation.md`.

Archived rather than deleted so the decision is recorded. See also `w477ho`, archived at
the same time for the same reason.
