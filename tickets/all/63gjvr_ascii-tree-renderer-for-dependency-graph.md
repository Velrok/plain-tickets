---
id: 63gjvr
title: ASCII tree renderer for dependency graph
type: task
status: todo
tags: []
parent: null
blocked_by:
- q5rxnv
created_at: 2026-05-03T22:50:24.333551Z
updated_at: 2026-05-03T22:50:24.333551Z
---

Given a root TicketId and the graph, produce an ASCII tree string. Handles [cycle: id], [missing: id], [archived] labels, and `id  status  title` node format.

## Acceptance criteria
- [ ] Renders a single tree from a root node
- [ ] Node format: `id  status  title`
- [ ] Cycle nodes render as `[cycle: <id>]`
- [ ] Missing refs render as `[missing: <id>]`
- [ ] Archived nodes render with `[archived]` label
- [ ] Unit tested