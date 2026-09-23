---
id: 9b88c0
title: deps-graph renders disjoint chains as separate roots
type: task
status: draft
tags:
- deps-graph
parent: fyihf8
blocked_by:
- ihqh45
created_at: 2026-09-23T10:13:43.601196Z
updated_at: 2026-09-23T10:13:43.601196Z
---

## Behaviour

Unrelated chains render as separate top-level roots, in created_at order,
with no connecting lines between them. A ticket that blocks nothing and is
blocked by nothing renders as a lone root line.

## Test first

- e2e - two independent chains render as two roots
- e2e - an orphan ticket renders as a single root line
