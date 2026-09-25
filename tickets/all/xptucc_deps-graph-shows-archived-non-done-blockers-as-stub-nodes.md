---
id: xptucc
title: deps-graph shows archived non-done blockers as stub nodes
type: task
status: done
tags:
- deps-graph
parent: fyihf8
blocked_by:
- ihqh45
created_at: 2026-09-23T10:13:43.770803Z
updated_at: 2026-09-25T13:26:34.848885Z
---

## Behaviour

An archived blocker whose status is not `done`, for instance `rejected`, does
NOT satisfy the dependency. Only `done` ever does. The dep stays and renders
as a stub root so the user can decide whether to drop it.

    [archived: zz99 rejected]
    └── a1  todo  Migrate to new queue

The stub carries the id as well as the status - without the id the user
cannot act on it. This is a deliberate refinement of the epic, which wrote
the stub as `[archived: <status>]`.

## Test first

- e2e - archived rejected blocker keeps the dependent blocked, stub rendered
- e2e - the stub names both the archived ticket id and its status

## Implementation notes

Landed as `c2a448a`, merged to main 2026-09-25 alongside `t9p76e`.

Widened the shared seam in `src/deps_graph.rs` rather than forking it.
`BlockerResolution` gained `ArchivedDone(TicketId)` and
`ArchivedStub(TicketId, TicketStatus)`; `resolve_blocker` became total and no
longer returns `Option`, with `Missing` absorbing the old `None`.
`blocker_satisfied` matches all four variants explicitly, no wildcard, and the
`Active` arm stays a positive `== TicketStatus::Done`.

Rendering changes were kept to `from_maps`, `label` and the sort-key lookup.
`render_node`, `roots()`, `children()` and all cycle/diamond code were left
untouched for the concurrent rendering lane.

## Decision - how stub roots are ordered

A stub has no `created_at` of its own, so it sorts by the **earliest
`created_at` among the dependents it blocks**.

Neither this ticket nor `t9p76e` specified it. Recording it here at the
verifier's recommendation, because a future reader changing `from_maps` would
otherwise have no ticket-level spec to check the behaviour against.

Verified deterministic - a stub shared by two dependents amongst three real
roots sorted to the same position on three consecutive runs, byte-identical.
Ordering comes from `sort_by_key`, so `HashMap` iteration order does not leak
into it.

## Independent verification (PASS)

Exact rendered output confirmed for archived `rejected`, `review` and
`in-progress` blockers - each stub names its real status rather than a generic
bucket. Archived-and-done blockers remain silently satisfied with no stub. A
single stub blocking several dependents renders as one node with several
children, not duplicates. A ticket with one satisfied and one unsatisfied
blocker appears only under the unsatisfied one.

`list --unblocked` and `deps-graph` agree, both deriving from the one seam, and
the deliberate disagreement over ACTIVE done blockers is preserved.

Mutations, each reddening named tests: `ArchivedStub => true` (2 unit tests);
dropping the `ArchivedStub` edge in `from_maps` while keeping the node (3 e2e
tests); and flipping `== Done` to `!= Done`, which reddened five
`list --unblocked` e2e tests - exactly the regression `rm49xa`'s notes warn
about.
