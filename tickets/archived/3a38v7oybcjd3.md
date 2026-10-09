---
id: 3a38v7oybcjd3
title: Design a human-friendly TTY output for list and show
type: feature
status: done
parent: null
blocked_by: []
tags:
- cli
created_at: 2026-10-09T00:12:38.000Z
updated_at: 2026-10-09T00:18:34.000Z
---

Today list prints tab-separated lines and show prints the raw file. Design a readable view for when stdout is a TTY, keeping the current plain output when piped.

Design questions:
- Columns, alignment and truncation for list; colour for status and type
- Layout of show: header block, tags, parent and blockers, rendered body
- TTY detection, NO_COLOR, and a flag to force either mode

Acceptance: a written design (as a note on this ticket) agreed before any implementation; implementation split into child tickets.

Design agreed.
- Mode: --format plain|pretty. Default: pretty when stdout is a TTY, plain otherwise. NO_COLOR keeps pretty layout but drops colour. No separate --color flag.
- list (pretty): header row ID STATUS TYPE TITLE; FULL 16-hex ID (copy-pasteable); status colour todo=grey, in progress=yellow, done=green, rejected=red; type colour bug=red, feature=cyan, task=default; title truncated to terminal width with an ellipsis.
- show (pretty): bold title; meta line 'id · type · status · #tags'; parent and blockers resolved to 'id title [status]'; created/updated dates; rule; body printed raw (no markdown rendering for now).
- plain output stays exactly as today (tab-separated list, raw file for show).
