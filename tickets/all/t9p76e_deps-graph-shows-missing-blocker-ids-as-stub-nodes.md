---
id: t9p76e
title: deps-graph shows missing blocker ids as stub nodes
type: task
status: done
tags:
- deps-graph
parent: fyihf8
blocked_by:
- ihqh45
created_at: 2026-09-23T10:13:43.844093Z
updated_at: 2026-09-25T13:26:37.347884Z
---

## Behaviour

A blocked_by id found neither in the active set nor in the archive is a data
problem, not an intentional filter. It does not satisfy the dependency, and
renders as a distinct stub so bad references never silently vanish.

    [missing: zz99]
    └── a1  todo  Migrate to new queue

## Test first

- e2e - a missing blocker id renders as a missing stub, dependent nested
- e2e - missing and archived stubs are visually distinct from each other

## Implementation notes

Landed as `733f00d`, merged to main 2026-09-25 alongside `xptucc`. The two
tickets widened the same seam and share the same design; see `xptucc`'s notes
for the full picture, including the stub-ordering decision.

`BlockerResolution::Missing(TicketId)` absorbs what used to be
`resolve_blocker` returning `None`, which is what let the function become total.

## Independent verification (PASS)

Stub renders as `[missing: <id>]`, visually distinct from the archived form, as
a root with its dependents nested underneath.

Probes beyond the acceptance criteria:

- Two tickets blocked by the same missing id produce **one** stub node with two
  children, not two nodes.
- A ticket blocked by both a missing id and an archived stub nests under both,
  confirming "nest under every blocker" applies to stub roots too.
- A blocker id that is a strict prefix of a real active id resolves as
  `Missing`, not as an accidental partial match - resolution is exact HashMap
  key lookup.
- An id present in both the active and archived sets resolves as `Active`
  unconditionally, archived never consulted. Correct, since archived is a
  fallback everywhere - but note it is silent, with no warning on a genuine
  collision.

Mutations `Missing => true` and dropping the `Missing` edge each reddened named
tests.

## Out of scope, found during verification

An empty-string `blocked_by` entry parses as a valid `TicketId` at the
deserialize layer and renders as `[missing: ]` with an empty label. The seam
handles it consistently across both consumers. The real gap is missing
front-matter validation, not blocker resolution. Filed separately.
