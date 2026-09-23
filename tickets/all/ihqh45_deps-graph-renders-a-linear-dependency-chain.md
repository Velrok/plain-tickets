---
id: ihqh45
title: deps-graph renders a linear dependency chain
type: task
status: done
tags:
- deps-graph
parent: fyihf8
blocked_by: []
created_at: 2026-09-23T10:13:43.336869Z
updated_at: 2026-09-23T10:22:55.523174Z
---

## Behaviour

Tracer bullet, the first vertical slice. Proves the whole path end to end:
active tickets load, a petgraph DiGraph is built from blocked_by, and the
indentation renderer prints it.

Given a1 blocking b2 blocking c3, `tickets deps-graph` prints

```
a1  todo  Set up DB schema
└── b2  todo  Write migration
    └── c3  todo  Run migration in CI
```

Node label keeps the old graph format, id then status then title.

## Test first

- e2e - a linear chain renders nested, deepest last
- e2e - an empty tickets dir prints nothing and exits 0

Add petgraph to Cargo.toml in this slice. Only enough graph and renderer code
to pass these two tests. No diamond handling, no archived or missing
resolution, no cycle detection yet - those arrive in their own slices.

The old `graph` command stays in place for now and is removed in its own
ticket once deps-graph covers the same ground.

## Implementation notes

Built `src/deps_graph.rs` as a fresh module (not touching `src/graph.rs`,
which stays until `fe17ed`). New `tickets deps-graph` subcommand wired
through `main.rs` → `cmd_deps_graph` in `commands.rs`.

- `DepsGraph::build` loads only `dir.all()` (active tickets) into a
  `petgraph::graph::DiGraph<TicketId, ()>`. Archived tickets are never
  loaded, matching the epic's "archived tickets are never graph nodes".
- Edge direction is **blocker → dependent** (opposite of the old `graph.rs`,
  which nests a ticket under its blockers). `deps-graph` nests a ticket
  under everything *it* blocks, so `a1` blocking `b2` blocking `c3` renders
  `a1` as the root with `b2` nested under it and `c3` nested under `b2`,
  matching the ticket's worked example exactly.
- Roots = nodes with no incoming edge; children of a node = its outgoing
  neighbours. Both are sorted by `created_at` ascending (the `list`
  convention), not by id — set up now so 2k12y3/9b88c0 don't need to touch
  ordering.
- Added the single resolution seam as a private `resolve_blocker` function
  returning `BlockerResolution` (currently just an `Active(TicketId)`
  variant). A `blocked_by` id that isn't found among active tickets is
  silently dropped (no edge, no node) in this slice — archived/missing
  handling is deliberately out of scope per the ticket, but the seam is in
  place for 7se1mu/xptucc/t9p76e to widen without scattering the check.
  `list --unblocked` (rm49xa) should call this same function rather than
  reimplementing the resolution rule.
- Judgment call: `tickets new`'s default status is `draft`, but the ticket's
  worked examples are all written against `todo`. The e2e test creates
  tickets with explicit `--status todo` via a small local test helper
  (`create_todo_ticket`) so the rendered output matches the ticket body
  verbatim, rather than weakening the assertion to ignore status.
- Both named e2e tests (`linear_chain_renders_nested_deepest_last`,
  `empty_tickets_dir_prints_nothing_and_exits_zero`) live in the new
  `tests/cli_deps_graph.rs`, following the `TICKETS_DIR`-isolated,
  real-binary convention in `docs/coding-style.md`. The empty-dir case
  passed without extra code — it falls out of `roots()` returning nothing
  for an empty ticket set.
- Added `petgraph = "0.6"` to `Cargo.toml` (matches the version already
  pulled transitively; no version conflicts).
- `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt` all
  clean. One pre-existing, unrelated failure noted and left alone:
  `tui::render::tests::detail_view_renders_ticket_fields` — a snapshot test
  whose fixture bakes in "Created: 2026-07-30" and drifts as today's date
  moves on; last touched 2026-07-30, not touched by this ticket's diff.
