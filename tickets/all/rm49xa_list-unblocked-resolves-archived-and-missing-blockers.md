---
id: rm49xa
title: list --unblocked resolves archived and missing blockers
type: task
status: todo
tags:
- deps-graph
parent: fyihf8
blocked_by:
- 3p7tpt
- 7se1mu
- xptucc
- t9p76e
created_at: 2026-09-23T10:14:02.474485Z
updated_at: 2026-09-23T10:28:42.767578Z
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

## Add a review case to the test list

Flagged during independent verification of xkbw11 on 2026-09-23.

The four e2e tests above cover archived-and-done, archived-and-rejected,
missing, and active-and-rejected. None covers the `review` status, which did
not exist when this ticket was written.

Add a fifth - **blocker active and review, ticket is not listed**.

## Why this specific test earns its place

The rule is "only `done` satisfies". Written positively as
`status == TicketStatus::Done`, a new status can never accidentally satisfy a
dependency, and the existing seam `deps_graph::blocker_satisfied` already
does exactly that.

Written as an exclusion instead, say `status != Todo && status != InProgress`,
`review` would silently count as resolved and `list --unblocked` would start
recommending work whose blocker is still under review. That is precisely the
mistake the review status exists to prevent.

The compiler cannot catch it. A `!=` chain is not a `match`, so exhaustiveness
checking never fires and the bug is invisible until someone acts on a bad
recommendation. This test is the only thing standing in the way of it.

Keep the predicate positive, and keep this test.
