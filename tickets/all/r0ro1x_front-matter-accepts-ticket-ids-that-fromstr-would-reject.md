---
id: r0ro1x
title: front matter accepts ticket ids that FromStr would reject
type: bug
status: todo
tags:
- validation
parent: null
blocked_by: []
created_at: 2026-09-25T13:27:39.105525Z
updated_at: 2026-09-25T13:27:39.105525Z
---

## Behaviour

`TicketId` has `FromStr` validation and is used as a clap value parser, so ids
arriving via the CLI are checked. Ids arriving via **serde, from a ticket file's
front matter, are not**. Deserialisation goes straight to the newtype and skips
the validation entirely.

Found during independent verification of `t9p76e` on 2026-09-25.

    blocked_by: [""]

parses without complaint, and `deps-graph` then renders

    [missing: ]
    └── a1  todo  Some ticket

an empty-labelled stub. `list --unblocked` also keeps the ticket blocked, so
behaviour is at least consistent across both consumers - the seam handles it
fine. The gap is that the value was ever allowed in.

This is not a blocker-resolution bug and is not a regression from `xptucc` or
`t9p76e`. It is missing validation at the parse boundary.

## Why it matters

This is the parse-don't-validate pattern failing at exactly the point it is
supposed to hold. `docs/contributor-orientation.md` says any function taking a
`WorkingDir` may assume an initialised directory because `WorkingDir::new`
validated it. The same reasoning is applied to `TicketId` throughout the
codebase - code assumes an id is well-formed because the type says so - but
that guarantee is only true on the CLI path.

Also worth checking whether `Title` and `Tag` have the same hole, since they
follow the same newtype-with-`FromStr` pattern.

## Reproduce before fixing

This is a `bug` ticket, so the standing rule in `docs/contributor-orientation.md`
applies: **write a failing test first**. The first commit should contain only
the red test.

Write a unit test that deserialises front matter containing an empty-string id
(and a malformed one, e.g. wrong length or illegal characters) and asserts it is
rejected. Today it is accepted. Record the actual output.

## Shape of the fix

Implement `Deserialize` for the newtypes so it routes through the existing
`FromStr`, rather than duplicating the validation rule in a second place - the
same single-seam principle the blocker-resolution code follows.

Decide and record what happens to a ticket file that is already on disk with a
bad id. Rejecting it at parse time turns it into a load failure, which
`fd38vu` and `4aawv9` now surface properly rather than dropping silently - so
the machinery to report it already exists. Confirm that is the behaviour you
want before committing to it.

## Test first

- unit - empty-string id in front matter is rejected
- unit - malformed id (wrong length, illegal characters) is rejected
- unit - a valid id still round-trips
- same coverage for `Title` and `Tag` if they share the hole
- e2e - a ticket file with a bad id is reported as a load failure, not dropped
