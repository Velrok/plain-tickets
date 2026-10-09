---
id: 00w41qy33qcxp
title: Front matter without a trailing newline after the closing delimiter is rejected
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

A ticket file with no newline after the closing `---` is rejected.

## Repro

1. Strip the final newline from a ticket file.
1. `tickets show <id>` or `tickets list` fails with `missing front matter closing delimiter`.

## Cause

`Ticket::from_str` only looks for `"\n---\n"`.

## Acceptance

- A closing `---` at the end of the file is accepted, with and without a body.
- Unit test for both.
