---
id: 4aawv9
title: cmd_list silently drops tickets that fail to read or parse
type: bug
status: todo
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-09-25T12:28:32.761590Z
updated_at: 2026-09-25T12:28:32.761590Z
---

## Behaviour

`fd38vu` fixed the silent-drop bug in the TUI. The identical
`filter_map(...).ok()` pattern survives on the CLI read path and was
deliberately left out of scope there.

Sites:

- `src/commands.rs:260-261` - `cmd_list`, two call sites
- a third similar loader in the deps-graph path

Same failure mode: a ticket that cannot be read or whose front matter will not
parse vanishes from `tickets list` with no output and exit 0. The user has no
way to tell an empty result from a broken file. This is exactly how the stale
binary on `PATH` hides tickets today.

## Shape of the fix

Mirror `fd38vu`. Return `(Vec<Ticket>, Vec<LoadFailure>)` rather than
discarding errors, so the type forces each caller to decide.

There is no TUI flash to reuse here, so the CLI needs its own surface. A warning
line on **stderr** naming the failed files, with stdout left clean and the exit
code unchanged, keeps `tickets list` pipeable - which matters, `3mqhe3` has
already had to fix a broken-pipe regression on this exact loop.

## Test first

- e2e - an unreadable file alongside valid tickets: valid ones still listed,
  stderr names the bad file
- e2e - an unparseable file, same
- e2e - stdout stays clean and parseable; the warning is on stderr only
- e2e - exit code is unchanged
