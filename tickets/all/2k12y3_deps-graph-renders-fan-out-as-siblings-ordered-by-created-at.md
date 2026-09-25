---
id: 2k12y3
title: deps-graph renders fan-out as siblings ordered by created_at
type: task
status: in-progress
tags:
- deps-graph
parent: fyihf8
blocked_by:
- ihqh45
created_at: 2026-09-23T10:13:43.435948Z
updated_at: 2026-09-25T12:21:27.816479Z
---

## Behaviour

One root unblocking several tickets renders them as siblings, ordered by
created_at ascending to match the `list` convention rather than by id.

    a1  todo  Ship shared auth lib
    ├── b2  todo  Add login screen
    └── c3  todo  Add SSO integration

Connector is `├──` for every sibling but the last.

## Test first

- e2e - two children of one root render as siblings with correct connectors
- e2e - siblings created out of id order still sort by created_at
