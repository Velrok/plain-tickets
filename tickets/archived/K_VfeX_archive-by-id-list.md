---
id: K_VfeX
title: archive by ID list
type: task
status: done
tags: []
parent: null
blocked_by: []
created_at: 2026-05-01T22:34:32.079962Z
updated_at: 2026-05-01T23:05:04.974157Z
---

## What to build

Add `tickets archive <id>...` command. Accepts one or more ticket IDs, validates all upfront (fail-fast), then moves matching files from `all/` to `archived/`. No status field mutation.

## Acceptance criteria

- [ ] `tickets archive <id>...` moves each ticket file from `all/` to `archived/`
- [ ] Ticket front matter (including status) is unchanged after move
- [ ] All IDs validated before any file is moved
- [ ] If any ID is missing or already in `archived/`, prints every failing ID and an explicit message that no files were moved, then exits 1
- [ ] On success, prints one line per ticket to stdout: `<id>  archived → <path>`
- [ ] Works with a single ID or multiple IDs
- [ ] Hard error if tickets directory is not initialised

## Blocked by

None — can start immediately