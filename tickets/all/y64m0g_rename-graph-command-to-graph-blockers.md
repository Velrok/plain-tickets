---
id: y64m0g
title: Rename graph command to graph-blockers
type: task
status: draft
tags:
- ux
parent: null
blocked_by: []
created_at: 2026-07-30T23:10:03.482057Z
updated_at: 2026-07-30T23:10:24.123964Z
---

The `graph` command name doesn't communicate what it traverses (blocker edges). A ticket with no blockers of its own renders as a single node, giving no hint that it may still have children or downstream dependents — confusing in practice.

## Acceptance criteria
- [ ] Rename `graph` subcommand to `graph-blockers` (same behaviour: no id = forest of unblocked roots, id = blocker chain for that ticket)
- [ ] Keep `graph` as a hidden/deprecated alias for one release so existing scripts don't break
- [ ] Update --help text and docs
