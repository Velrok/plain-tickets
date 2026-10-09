---
id: 00w41rwtv8j6o
title: Show shortest unique ID prefixes in pretty output and accept prefixes as input
type: feature
status: todo
parent: null
blocked_by:
- 00w41rdqtrtc4
tags:
- id
created_at: 2026-10-09T01:19:23.000Z
updated_at: 2026-10-09T01:19:23.000Z
---

## Problem

Base36 u64 IDs are 13 characters long. jj shows the shortest unique prefix and accepts any unique prefix, so nobody types a full ID.

## Acceptance

- Pretty `list` and `show` print the shortest prefix that is unique among all tickets (active and archived), minimum 3 characters. Plain output keeps full IDs, so scripts stay stable.
- Commands accept any prefix of an ID, not only the full 13 characters.
  - A prefix matching one ticket resolves to it.
  - An ambiguous prefix is an error that lists the matching IDs and titles.
  - A prefix matching nothing says so.
- The prefix lookup covers active and archived tickets, like `store::find`.
- Tests for unique, ambiguous and missing prefixes, and for pretty versus plain output.
- Update `CLAUDE.md`, `README.md` and the `tickets-cli-expert` skill, which say IDs are always passed in full.

A prefix printed earlier can become ambiguous later. Scripts should capture the full ID from `tickets new`.