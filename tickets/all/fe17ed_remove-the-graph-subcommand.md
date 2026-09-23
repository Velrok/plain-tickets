---
id: fe17ed
title: Remove the graph subcommand
type: task
status: todo
tags:
- deps-graph
parent: fyihf8
blocked_by:
- 2k12y3
- 3yew0k
- 9b88c0
- 7se1mu
- xptucc
- t9p76e
- hx1po6
created_at: 2026-09-23T10:14:02.306072Z
updated_at: 2026-09-23T10:28:42.546516Z
---

## Behaviour

Cleanup slice, no new behaviour. Once deps-graph covers the same ground,
remove the old command rather than keeping two graph views around.

- Delete `Commands::Graph` from src/main.rs and `cmd_graph` from
  src/commands.rs
- Delete tests/cli_graph.rs, and any hand-rolled adjacency or DFS cycle code
  left unused after the petgraph migration
- Correct the stale file layout table in docs/coding-style.md while in there,
  it still describes the old src/types.rs and src/commands.rs monolith

## Note

The `tickets-cli-expert` skill in the user dotfiles documents `tickets graph`
and its blocked-to-blocker edge semantics. It goes stale when this lands and
needs updating separately, outside this repo.
