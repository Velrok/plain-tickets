---
id: 4wquzb
title: Add graph-subtickets command for parent-child tree
type: task
status: draft
tags:
- ux
parent: null
blocked_by: []
created_at: 2026-07-30T23:10:25.999226Z
updated_at: 2026-07-30T23:10:31.452115Z
---

`graph`/`graph-blockers` only walks the `blocked_by` axis; parent/child hierarchy (the `parent` field) has no equivalent visualisation and currently requires `rg -l '^parent: <id>' tickets/all/` to reconstruct, one level at a time, with no recursion and no rendering.

## Acceptance criteria
- [ ] New `tickets graph-subtickets <id>` subcommand renders the recursive child tree rooted at `<id>` (reuse the existing ASCII tree renderer)
- [ ] No-id form: forest of all tickets with no parent (top-level roots)
- [ ] Hard error if id doesn't resolve
- [ ] Documented in --help and docs
- [ ] Tree nodes with status `done`/`rejected` are pruned from the tree by default; `--include-done` flag shows them
