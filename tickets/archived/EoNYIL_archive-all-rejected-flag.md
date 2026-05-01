---
id: EoNYIL
title: archive --all-rejected flag
type: task
status: done
tags: []
parent: null
blocked_by: []
created_at: 2026-05-01T22:34:42.003110Z
updated_at: 2026-05-01T23:05:05.186638Z
---

## What to build

Add `--all-rejected` flag to `tickets archive`. Scans `all/` for tickets with status `rejected` and moves them to `archived/`. Mutually exclusive with the ID list argument (hard error if both supplied). Prints to stderr when zero matching tickets are found.

## Acceptance criteria

- [ ] `tickets archive --all-rejected` moves all rejected tickets from `all/` to `archived/`
- [ ] On success, prints one line per ticket to stdout: `<id>  archived → <path>`
- [ ] When no rejected tickets are found, prints a message to stderr (e.g. `nothing to archive`) and exits 0
- [ ] Scans `all/` only — tickets already in `archived/` are not touched
- [ ] Passing both `--all-rejected` and one or more IDs is a hard error (exit 1)
- [ ] Reuses the same move infrastructure as the ID-list slice

## Blocked by

K_VfeX (archive by ID list)