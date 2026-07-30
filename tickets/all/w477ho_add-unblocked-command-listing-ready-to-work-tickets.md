---
id: w477ho
title: Add unblocked command listing ready-to-work tickets
type: task
status: draft
tags:
- ux
parent: null
blocked_by: []
created_at: 2026-07-30T23:10:34.834370Z
updated_at: 2026-07-30T23:10:54.014349Z
---

The forest form of `graph`/`graph-blockers` (no id) surfaces 'what's ready to work on' but the name and output don't say so directly — you have to know that roots-with-no-blockers means unblocked. Worth a purpose-named command as the primary entry point for 'what can I pick up next'.

## Acceptance criteria
- [ ] New `tickets unblocked` subcommand: flat list of tickets with no active (non-done) blockers, matching current `graph` (no id) semantics
- [ ] Excludes tickets with status `done` or `rejected` by default
- [ ] `--include-done` flag to include them
- [ ] Documented in --help and docs
