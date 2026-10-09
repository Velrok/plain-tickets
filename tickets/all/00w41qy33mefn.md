---
id: 00w41qy33mefn
title: A stray non-ticket file in tickets/all breaks every list
type: bug
status: done
parent: null
blocked_by: []
tags:
- store
created_at: 2026-10-09T01:00:09.000Z
updated_at: 2026-10-09T01:29:42.359Z
---

## Problem

One stray file in `tickets/all` makes `list` fail for every ticket.

## Repro

1. `touch tickets/all/.DS_Store`
1. `tickets list` fails with `missing front matter closing delimiter`

## Cause

`store::read_dir` parses every directory entry and aborts on the first error.

## Acceptance

- Entries that are not `.md` files are skipped.
- A corrupt `.md` file prints a warning on stderr and is skipped (suggested default; the old implementation in `../main` also warns, see its `src/deps_graph.rs`).
- Tests for both cases.

Fixed in store::read_dir: non-.md entries skipped, unparseable .md files warn on stderr and are skipped. Replaced the old list_names_a_corrupt_ticket_file_in_the_error test (it asserted the abort).
