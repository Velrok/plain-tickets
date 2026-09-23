---
id: fyihf8
title: Revised deps-graph rendering with petgraph and unblocked list filter
type: epic
status: in-progress
tags:
- deps-graph
parent: null
blocked_by: []
created_at: 2026-09-23T10:09:58.851993Z
updated_at: 2026-09-23T10:28:42.904755Z
---

## Goal

Replace the hand-rolled dependency graph with a petgraph-backed implementation,
change its rendering to an indentation tree that repeats multi-parent tickets
with a marker, and fold "what can I work on now" into `list` instead of a
separate `unblocked` subcommand.

## Remove

- `graph` subcommand entirely: `src/graph.rs` module, CLI wiring in
  `src/main.rs`, `tests/cli_graph.rs`.

## Add: `tickets deps-graph`

- No arguments -- always renders the full forest (no single-ticket rooted
  view, unlike the old `graph <id>`).
- Graph built with `petgraph::graph::DiGraph` from active tickets
  `blocked_by` edges. Archived tickets are never loaded as nodes.
- Cycle detection via `petgraph::algo::toposort`, plus `tarjan_scc` to list
  every id participating in a cycle for the warning message.
- Rendering: indentation tree (`├──`/`└──`). A ticket is nested under EVERY
  blocker it has (diamonds render under each parent). Second and later
  occurrences print as a leaf marked `(see above)` instead of re-expanding.
- Root/sibling ordering: by `created_at` ascending (same convention as
  `list`), not alphabetical by id.
- Cycles: still renders (does not error/hang), stderr warning naming every
  id in the cycle, and the repeated node in the loop is marked
  `(cycle: see above)`.

### Out-of-active-set `blocked_by` resolution

Applies both to `deps-graph` rendering and the `list --unblocked` filter
below. For a `blocked_by` id not found among active tickets:

- Found in archive with status `done` -> satisfied. Edge is silently
  dropped; no node is rendered for it.
- Found in archive with any other status (e.g. `rejected`) -> NOT
  satisfied. Dependency stays and is rendered as a stub:
  `[archived: <status>]`.
- Not found anywhere (missing/typo would-be id) -> NOT satisfied. Rendered
  as a distinct stub: `[missing: <id>]`.

Only `done` ever satisfies a dependency. Other terminal-ish statuses (e.g.
`rejected`) still block; the user must explicitly remove the dependency or
change the blocking ticket status.

## Change: `list --unblocked`

- New boolean flag on the existing `list` command, composes with the
  existing `--status`/`--type`/`--tag` filters.
- Filters to tickets where every `blocked_by` entry is empty or resolves to
  satisfied per the resolution rule above.
- Reuses `list`'s existing table output (`id status type title`,
  dynamic column widths) - no separate output format.
- No standalone `unblocked` subcommand.

## Reference

Full design discussion and rendering-style exploration (git-log-inspired
lanes vs indentation tree) happened in a grill-me session on 2026-09-23. The
scenario set used to compare rendering styles covered: linear chain, single
and nested diamonds, fan-out, fan-in, disjoint chains, mixed-status
unblocking, missing blocker, a cycle, an orphan ticket, and wide fan-in.
Indentation tree with repeat-and-mark won over two git-log-lane styles.
