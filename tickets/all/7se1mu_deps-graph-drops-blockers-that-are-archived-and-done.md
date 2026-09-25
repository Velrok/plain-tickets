---
id: 7se1mu
title: deps-graph drops blockers that are archived and done
type: task
status: done
tags:
- deps-graph
parent: fyihf8
blocked_by:
- ihqh45
created_at: 2026-09-23T10:13:43.678203Z
updated_at: 2026-09-25T11:38:26.361181Z
---

## Behaviour

Archived tickets are never graph nodes. When a blocked_by id is not in the
active set, do one fallback lookup in the archive. If it is there with status
`done`, the dependency is satisfied - drop the edge silently and render no
node for it. The dependent becomes a root if that was its only blocker.

## Test first

- e2e - a ticket blocked only by an archived done ticket renders as a root
- e2e - no node or label appears anywhere for that archived done blocker

## Implementation notes

### Seam shape after this change, in src/deps_graph.rs

```rust
enum BlockerResolution {
    Active(TicketId),
    ArchivedDone(TicketId),      // new
}

fn resolve_blocker(
    active: &HashMap<TicketId, Ticket>,
    archived: &HashMap<TicketId, Ticket>,   // new parameter
    id: &TicketId,
) -> Option<BlockerResolution>

fn blocker_satisfied(active: &HashMap<TicketId, Ticket>, resolution: &BlockerResolution) -> bool {
    match resolution {
        BlockerResolution::Active(id) => active[id].front_matter.status == TicketStatus::Done,
        BlockerResolution::ArchivedDone(_) => true,
    }
}

pub(crate) fn is_unblocked(active, archived, ticket) -> bool
pub(crate) fn load_active(dir) -> Result<HashMap<TicketId, Ticket>>
pub(crate) fn load_archived(dir) -> Result<HashMap<TicketId, Ticket>>   // new, symmetric
fn load_dir(path) -> Result<HashMap<TicketId, Ticket>>                  // private, shared by both
```

`DepsGraph::build` / `from_maps` (renamed from `from_active`) thread both maps
and match only `Active(_)` when adding edges and nodes, so `ArchivedDone` and
`None` both render nothing.

### The one line that makes the rule shared

`blocker_satisfied`'s `ArchivedDone(_) => true` arm. Because `resolve_blocker`
only ever constructs `ArchivedDone` when the archived ticket's status is
already `Done`, no second status check is needed there. The `Active` arm holds
the only live comparison, and it stays positive - `== TicketStatus::Done`,
never negated, never an exhaustive match against `TicketStatus`.

### cmd_list

Widened to load both maps and pass them through, so `list --unblocked` gets
archived-and-done resolution for free rather than leaving `archived` an
unreachable parameter. The archived-non-done and missing stub behaviour is
deliberately not built here - that stays `rm49xa`'s scope, and
`resolve_blocker` still returns `None` for both cases.

### Asymmetry preserved deliberately

`DepsGraph::from_maps` still matches only `Active(_)` with no status check, so
an active-and-done blocker still draws an edge and a node. `is_unblocked`
treats that same blocker as satisfied. Different questions, consistent
answers, per `docs/contributor-orientation.md`. Only the out-of-active-set
resolution is shared between the two commands.

### Tests

Unit, in `resolve_blocker_tests`:
`active_blocker_resolves_active_regardless_of_status`,
`archived_done_blocker_resolves_archived_done`,
`archived_non_done_blocker_is_not_resolved` (guards xptucc's future scope),
`blocker_missing_everywhere_is_not_resolved` (guards t9p76e's future scope),
`blocker_satisfied_treats_archived_done_as_satisfied`,
`is_unblocked_true_when_only_blocker_is_archived_done`.

E2e, in `tests/cli_deps_graph.rs`:
`blocker_archived_and_done_renders_dependent_as_root`,
`blocker_archived_and_done_never_appears_as_a_node_or_label`.

Note both e2e tests were already accidentally green under the old code, since
any non-active blocker was unconditionally dropped regardless of archive
status. They do not discriminate the fix on their own - the unit tests on
`resolve_blocker` are what drove a genuine red to green cycle.

### Rework note

This ticket was first implemented against a stale worktree that predated both
the `review` status and the `3p7tpt` merge, then merged with main and
reconciled. The reconciliation is where `blocker_satisfied` gained its
`ArchivedDone` arm and `cmd_list` was widened.
