---
id: q5rxnv
title: Build dependency graph data structure
type: task
status: done
tags: []
parent: null
blocked_by: []
created_at: 2026-05-03T22:50:16.647142Z
updated_at: 2026-05-04T08:45:33.380245Z
---

Load all tickets from `all/` and `archived/`; build an in-memory directed graph (TicketId → Vec<TicketId>); detect cycles; flag missing and archived nodes. No rendering — just the graph model and traversal logic.

## Acceptance criteria
- [ ] Loads active tickets from `all/` and archived from `archived/`
- [ ] Builds adjacency map from `blocked_by` fields
- [ ] Detects cycles and marks repeated nodes
- [ ] Flags missing references
- [ ] Flags archived references distinctly
- [ ] Unit tested