---
id: 00w41qy33mefn
title: A stray non-ticket file in tickets/all breaks every list
type: bug
status: todo
parent: null
blocked_by: []
tags:
- store
created_at: 2026-10-09T01:00:09.000Z
updated_at: 2026-10-09T01:10:33.000Z
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
