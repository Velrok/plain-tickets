---
id: w696dr
title: show graph and deps-graph panic when their output is piped
type: bug
status: in-progress
tags:
- cli
parent: null
blocked_by: []
created_at: 2026-09-25T12:38:10.471312Z
updated_at: 2026-09-25T14:11:29.908572Z
---

## Behaviour

Every command except `list` writes with `println!`/`print!`, which panics when
the reader closes the pipe early. Confirmed pre-existing on `ee88666` during
independent verification of `3mqhe3`, so this is not a regression from that
ticket - it only became visible because `list` was fixed and the others were
not.

    $ tickets show <id> | true
    thread 'main' panicked ... failed printing to stdout: Broken pipe (os error 32)
    exit 101

The user sees a panic message that invites a bug report, for the entirely
ordinary act of piping into `head`, `less` that is quit early, or `grep -q`.

Affected: `show`, `graph`, `deps-graph`, `new`, `edit`, `archive`, `init` -
every command with a `println!`. `show` and `deps-graph` are the ones users will
actually hit, since both can emit long output.

## Reproduce before fixing

This is a `bug` ticket, so the standing rule in `docs/contributor-orientation.md`
applies: **write a test that reproduces the panic and watch it fail first**,
before touching production code. The first commit should contain only the red
test.

Assert exit code 0 and empty stderr for `<cmd> | head -1`. Today `show` exits
101 with a panic message on stderr. Record that actual output in the notes.

## Shape of the fix

`3mqhe3` fixed this for `cmd_list` by matching on the `writeln!` result and
treating `ErrorKind::BrokenPipe` as a clean exit while still propagating every
other error via `.context(...)`. Do NOT copy that match into seven call sites -
that is the duplication this repo keeps having to undo.

Introduce one output seam that every command writes through, and put the
broken-pipe rule in it once. `cmd_list`'s existing handling should then be
expressed in terms of that seam rather than kept as a special case.

Note `tickets show` renders through `bat` and emits ANSI even when piped, so its
path may differ - establish that before designing.

## Test first

- e2e - `show <id> | head -1` exits 0 with empty stderr
- e2e - `deps-graph | head -1` exits 0 with empty stderr
- e2e - a genuine write error (not a broken pipe) still exits non-zero
- e2e - `list` keeps its existing behaviour, no regression

## Related

Relates to the output-seam observation recorded on `bmtthv`, the architecture
review ticket. If that review lands first it may reshape this fix.
