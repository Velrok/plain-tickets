---
id: t9p76e
title: deps-graph shows missing blocker ids as stub nodes
type: task
status: todo
tags:
- deps-graph
parent: fyihf8
blocked_by:
- ihqh45
created_at: 2026-09-23T10:13:43.844093Z
updated_at: 2026-09-23T10:28:42.403386Z
---

## Behaviour

A blocked_by id found neither in the active set nor in the archive is a data
problem, not an intentional filter. It does not satisfy the dependency, and
renders as a distinct stub so bad references never silently vanish.

    [missing: zz99]
    └── a1  todo  Migrate to new queue

## Test first

- e2e - a missing blocker id renders as a missing stub, dependent nested
- e2e - missing and archived stubs are visually distinct from each other
