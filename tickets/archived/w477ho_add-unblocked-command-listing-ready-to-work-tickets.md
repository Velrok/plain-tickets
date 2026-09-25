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
updated_at: 2026-09-25T12:16:02.971705Z
---


The forest form of `graph`/`graph-blockers` (no id) surfaces 'what's ready to work on' but the name and output don't say so directly — you have to know that roots-with-no-blockers means unblocked. Worth a purpose-named command as the primary entry point for 'what can I pick up next'.

## Acceptance criteria
- [ ] New `tickets unblocked` subcommand: flat list of tickets with no active (non-done) blockers, matching current `graph` (no id) semantics
- [ ] Excludes tickets with status `done` or `rejected` by default
- [ ] `--include-done` flag to include them
- [ ] Documented in --help and docs

## Archived 2026-09-25 — superseded, and already delivered

Existed only on `origin/main` as a `draft`; never present in local `main`. The capability
asked for here has **shipped**, via a different surface than proposed: instead of a
standalone `tickets unblocked` subcommand, it landed as a `--unblocked` filter on `list`.

- `3p7tpt` (done) — `list --unblocked` filters to tickets with no unfinished blockers
- `0awkzp` (done) — composes with `--status`, `--type` and `--tag`
- `rm49xa` (todo) — extends it to archived and missing blockers

Both delivered tickets were independently verified, including the case a negative
predicate would get wrong: a blocker in `review` status correctly still blocks.

The `--include-done` criterion is covered by `--status` composition, which is strictly
more flexible than a single boolean flag.

Archived rather than deleted so the decision is recorded. See also `y64m0g`, archived at
the same time for the same reason.
