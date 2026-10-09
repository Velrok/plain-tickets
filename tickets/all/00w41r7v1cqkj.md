---
id: 00w41r7v1cqkj
title: A crash mid-write can truncate a ticket file
type: bug
status: done
parent: null
blocked_by: []
tags:
- store
created_at: 2026-10-09T01:05:34.000Z
updated_at: 2026-10-09T01:31:40.720Z
---

## Problem

`store::replace` uses `fs::write`, which truncates then writes. A crash or full disk part-way leaves a damaged ticket.

## Acceptance

- Write to a temp file in the same directory, then rename over the target.
- Test that a failed write leaves the old file intact.

Fixed in store::replace: writes <id>.tmp beside the ticket, fsyncs, renames over it, removes the temp on error. Temp name does not end in .md so list ignores a leftover. Tests: failed write (read-only dir) keeps old file; success leaves no extra files.
