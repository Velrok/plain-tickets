---
id: 0awkzp
title: list --unblocked composes with status type and tag filters
type: task
status: in-progress
tags:
- deps-graph
parent: fyihf8
blocked_by:
- 3p7tpt
created_at: 2026-09-23T10:14:02.562292Z
updated_at: 2026-09-23T11:03:44.065170Z
---

## Behaviour

`--unblocked` composes with the existing `--status`, `--type` and `--tag`
filters rather than replacing them.

## Test first

- e2e - `--unblocked --tag backend` applies both filters
- e2e - `--unblocked --status todo` applies both filters
- e2e - repeated `--tag` keeps AND semantics alongside `--unblocked`
