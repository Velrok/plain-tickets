---
id: rm49xa
title: list --unblocked resolves archived and missing blockers
type: task
status: draft
tags:
- deps-graph
parent: fyihf8
blocked_by:
- 3p7tpt
- 7se1mu
- xptucc
- t9p76e
created_at: 2026-09-23T10:14:02.474485Z
updated_at: 2026-09-23T10:14:02.474485Z
---

## Behaviour

`--unblocked` uses the same blocker resolution as deps-graph. Only `done`
satisfies a dependency, whether the blocker is active or archived. Every
other case leaves the ticket blocked so the user has to decide whether the
dep is still needed.

## Test first

- e2e - blocker archived and done, ticket is listed
- e2e - blocker archived and rejected, ticket is not listed
- e2e - blocker id missing entirely, ticket is not listed
- e2e - blocker active and rejected, ticket is not listed
