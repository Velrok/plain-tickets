---
id: ihqh45
title: deps-graph renders a linear dependency chain
type: task
status: draft
tags:
- deps-graph
parent: fyihf8
blocked_by: []
created_at: 2026-09-23T10:13:43.336869Z
updated_at: 2026-09-23T10:13:43.336869Z
---

## Behaviour

Tracer bullet, the first vertical slice. Proves the whole path end to end:
active tickets load, a petgraph DiGraph is built from blocked_by, and the
indentation renderer prints it.

Given a1 blocking b2 blocking c3, `tickets deps-graph` prints

    a1  todo  Set up DB schema
    └── b2  todo  Write migration
        └── c3  todo  Run migration in CI

Node label keeps the old graph format, id then status then title.

## Test first

- e2e - a linear chain renders nested, deepest last
- e2e - an empty tickets dir prints nothing and exits 0

Add petgraph to Cargo.toml in this slice. Only enough graph and renderer code
to pass these two tests. No diamond handling, no archived or missing
resolution, no cycle detection yet - those arrive in their own slices.

The old `graph` command stays in place for now and is removed in its own
ticket once deps-graph covers the same ground.
