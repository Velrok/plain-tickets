---
id: 00w41rdqtrtc4
title: Generate ticket IDs as random u64s written in base36
type: task
status: done
parent: null
blocked_by:
- 00w41r24o1ivp
tags:
- id
created_at: 2026-10-09T01:08:49.000Z
updated_at: 2026-10-09T01:26:05.368Z
---

## Problem

`ID::generate` puts the time in the top 48 bits, so IDs made around the same time share a long prefix. That makes them hard to tell apart. Hex is also longer than it needs to be.

## Acceptance

- `ID::generate` uses 64 random bits (`getrandom`) and no time component. `ID` stays a `u64`.
- IDs are written as base36 (`0-9a-z`), zero-padded to 13 characters (36^13 > 2^64).
  - Parsing accepts exactly 13 lowercase base36 characters and rejects values above `u64::MAX`.
  - Filenames, front matter and CLI output all use this form.
- `create` retries with a new ID if the file already exists, a few times, then fails.
- Replace the test `later_ids_sort_after_earlier_ones` with one checking that IDs differ.
- `list` still orders same-second tickets by creation, using the millisecond `created_at` from the Timestamp ticket. Keep the same-second test in `tests/list.rs`.
- Update the doc comment on `ID::generate`, plus `CLAUDE.md`, `README.md` and the `tickets-cli-expert` skill, which say "16 hex digits".
- Migrate the existing tickets in `tickets/`: rename each file and rewrite every `id`, `parent` and `blocked_by` reference.

Blocked by the Timestamp ticket; otherwise `ce0e286` regresses.

Follow-up: shorter IDs in the pretty output (see the prefix ticket).

Done. ID is a random u64 shown as 13 base36 chars; ID::generate has no time component; new() draws up to 5 IDs and skips any used by an active or archived ticket (store::exists). Same-second ordering now rests on the millisecond created_at (list test passes 25 runs in a row). All 40 existing tickets (24 active, 16 archived) were backfilled: files renamed and id/parent/blocked_by (and any ID in bodies) rewritten by converting the old hex value to base36, so they keep their old time prefixes. Docs updated (README, CLAUDE.md); the tickets-cli-expert skill needed no change.
