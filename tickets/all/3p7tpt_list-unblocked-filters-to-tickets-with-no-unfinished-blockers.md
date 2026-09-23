---
id: 3p7tpt
title: list --unblocked filters to tickets with no unfinished blockers
type: task
status: draft
tags:
- deps-graph
parent: fyihf8
blocked_by: []
created_at: 2026-09-23T10:14:02.399599Z
updated_at: 2026-09-23T10:14:02.399599Z
---

## Behaviour

New boolean `--unblocked` flag on the existing `list` command. Filters to
tickets where every blocker is empty or done. Reuses the existing list table,
id status type title with dynamic column widths, rather than introducing a
new output format. There is deliberately no standalone `unblocked`
subcommand.

## Test first

- e2e - a ticket with no blockers at all is listed
- e2e - a ticket blocked by a todo ticket is not listed
- e2e - a ticket whose only blocker is done is listed
