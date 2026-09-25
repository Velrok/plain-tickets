---
id: 4wquzb
title: Add graph-subtickets command for parent-child tree
type: task
status: draft
tags:
- ux
parent: null
blocked_by: []
created_at: 2026-07-30T23:10:25.999226Z
updated_at: 2026-09-25T12:16:19.032744Z
---


`graph`/`graph-blockers` only walks the `blocked_by` axis; parent/child hierarchy (the `parent` field) has no equivalent visualisation and currently requires `rg -l '^parent: <id>' tickets/all/` to reconstruct, one level at a time, with no recursion and no rendering.

## Acceptance criteria
- [ ] New `tickets graph-subtickets <id>` subcommand renders the recursive child tree rooted at `<id>` (reuse the existing ASCII tree renderer)
- [ ] No-id form: forest of all tickets with no parent (top-level roots)
- [ ] Hard error if id doesn't resolve
- [ ] Documented in --help and docs
- [ ] Tree nodes with status `done`/`rejected` are pruned from the tree by default; `--include-done` flag shows them

## Archived 2026-09-25 — not pursued

Existed only on `origin/main` as a `draft`; never present in local `main`.

Unlike `y64m0g` and `w477ho`, archived alongside it, this one was **not** superseded.
It walks a different axis entirely: `parent`/child hierarchy, where `deps-graph` and the
whole of epic `fyihf8` walk `blocked_by`. Nothing in local `main` provides it, and
reconstructing a subticket tree still means `rg -l '^parent: <id>' tickets/all/` one
level at a time, exactly as this ticket describes.

Archived by explicit user decision on 2026-09-25 while clearing the divergent
`origin/main` drafts, with that distinction understood — a deliberate drop, not a
duplicate.

If it is ever revived, the ASCII tree renderer built for `deps-graph` in `src/deps_graph.rs`
is the natural thing to reuse, and the parent-axis walk is independent of the blocker
resolution seam (`resolve_blocker`, `BlockerResolution`), so it would not disturb it.
