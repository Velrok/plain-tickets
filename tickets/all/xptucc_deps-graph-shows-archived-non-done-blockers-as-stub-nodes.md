---
id: xptucc
title: deps-graph shows archived non-done blockers as stub nodes
type: task
status: draft
tags:
- deps-graph
parent: fyihf8
blocked_by:
- ihqh45
created_at: 2026-09-23T10:13:43.770803Z
updated_at: 2026-09-23T10:13:43.770803Z
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
