---
id: hx1po6
title: deps-graph warns on cycles instead of hanging
type: task
status: todo
tags:
- deps-graph
parent: fyihf8
blocked_by:
- ihqh45
created_at: 2026-09-23T10:13:43.939887Z
updated_at: 2026-09-23T10:28:42.480502Z
---

## Behaviour

blocked_by should be a DAG, but bad data must not hang or crash the command.
Detect cycles with `petgraph::algo::toposort`, and use `tarjan_scc` to name
every id involved. The tree still renders, the repeated node in the loop is
marked `(cycle: see above)` and recursion stops there. The warning goes to
stderr and the exit code stays 0.

## Test first

- e2e - a three-ticket cycle renders without hanging
- e2e - stderr names every id participating in the cycle
- e2e - exit code is 0 despite the warning

## Observed behaviour before this ticket

Independent verification of ihqh45 on 2026-09-23 probed the cycle case. It
does not hang or crash -- it completes instantly. The actual failure mode is
worse than a hang in one respect and better in another.

Two tickets blocking each other print **nothing at all**, exit 0. Every node
in the cycle has an incoming edge, so none of them qualifies as a root, and
the renderer starts from roots only. The tickets silently vanish from the
graph.

So this ticket is not only about not hanging. It is about cycle members being
invisible. The acceptance criteria should include that every id in the cycle
actually appears in the rendered output, not merely that the command
terminates.

Suggested extra test - a graph consisting solely of a two-ticket cycle
renders both tickets rather than printing nothing.
