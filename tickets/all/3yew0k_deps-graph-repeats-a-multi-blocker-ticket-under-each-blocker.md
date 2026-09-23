---
id: 3yew0k
title: deps-graph repeats a multi-blocker ticket under each blocker
type: task
status: todo
tags:
- deps-graph
parent: fyihf8
blocked_by:
- ihqh45
created_at: 2026-09-23T10:13:43.510347Z
updated_at: 2026-09-23T10:28:42.061745Z
---

## Behaviour

A ticket with more than one blocker is nested under EVERY blocker it has. The
first occurrence expands its subtree in full. Later occurrences print as a
leaf marked `(see above)`, so no dependency is hidden and nothing loops.

Given a1 and c3 both blocking d4, and d4 blocking f6

    a1  todo  Set up DB schema
    └── d4  todo  Run migration in CI
        └── f6  todo  Enable feature flag
    c3  todo  Design API contract
    └── d4  todo  Run migration in CI  (see above)

## Test first

- e2e - a diamond renders the shared ticket under both of its blockers
- e2e - the repeated occurrence is marked and does not re-expand its subtree
- e2e - nested diamonds, two converging pairs feeding one final ticket
