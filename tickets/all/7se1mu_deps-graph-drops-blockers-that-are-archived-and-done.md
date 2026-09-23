---
id: 7se1mu
title: deps-graph drops blockers that are archived and done
type: task
status: todo
tags:
- deps-graph
parent: fyihf8
blocked_by:
- ihqh45
created_at: 2026-09-23T10:13:43.678203Z
updated_at: 2026-09-23T10:28:42.222190Z
---

## Behaviour

Archived tickets are never graph nodes. When a blocked_by id is not in the
active set, do one fallback lookup in the archive. If it is there with status
`done`, the dependency is satisfied - drop the edge silently and render no
node for it. The dependent becomes a root if that was its only blocker.

## Test first

- e2e - a ticket blocked only by an archived done ticket renders as a root
- e2e - no node or label appears anywhere for that archived done blocker
