---
id: o3c87i
title: corrupted archived tickets vanish with no warning at all
type: bug
status: todo
tags:
- cli
parent: null
blocked_by: []
created_at: 2026-09-25T13:34:49.411076Z
updated_at: 2026-09-25T13:34:49.411076Z
---

## Behaviour

`fd38vu` fixed silent ticket drops in the TUI, `4aawv9` fixed them in `cmd_list`.
A third loader was left untouched in both, and it still drops silently:
`load_dir` in `src/deps_graph.rs`, backing `load_active`/`load_archived`, used
by `deps-graph` and `list --unblocked`.

Reproduced during independent verification of `4aawv9` on 2026-09-25.

Corrupt an **archived** ticket that is named as a blocker, then

    $ tickets list --unblocked
    (no stdout, the dependent is correctly excluded)
    (stderr completely empty, no warning at all)

The dependent is correctly held back - its blocker cannot be verified as
satisfied - but the user is told nothing whatsoever about why, and the archived
ticket has effectively disappeared.

## Why `4aawv9` did not cover it

`cmd_list`'s new loader scans only `all/`. `load_dir` scans `all/` **and**
`archived/` with no failure tracking.

So a corrupted file in `all/` is coincidentally reported, because both loaders
read that directory and one of them now warns. A corrupted file in `archived/`
is read only by the untouched loader, and nothing reports it. The overlap is
luck, not design.

This is the same bug the whole chain exists to kill, just relocated to the one
place nobody has looked.

## Reproduce before fixing

This is a `bug` ticket, so the standing rule in `docs/contributor-orientation.md`
applies: **write the failing test first** and commit it on its own.

- e2e - a corrupted file in `archived/` named as a blocker: `list --unblocked`
  reports it on stderr. Today stderr is empty.
- e2e - the same for `deps-graph`.

## Shape of the fix

Do NOT add a fourth copy of the load-and-warn logic. There are already three
near-identical ticket-loading paths and `4z8qct` exists to collapse them. Either
land that extraction first and build on it, or write this fix so it is trivially
absorbed by it.

Note `load_dir` returns `HashMap<TicketId, Ticket>` rather than a `Vec`, so it
is not a drop-in for the other two - that difference is part of what `4z8qct`
has to reconcile.

Also settle what `deps-graph` should render for a blocker whose file exists but
will not parse. It is neither `ArchivedStub` nor `Missing` in the current
`BlockerResolution` - it is a fourth case, "present but unreadable", and
silently treating it as `Missing` would be wrong.

## Related

`fd38vu`, `4aawv9` (the first two loaders), `4z8qct` (the extraction),
`bmtthv` (architecture review, where duplicated loading is finding #1).
