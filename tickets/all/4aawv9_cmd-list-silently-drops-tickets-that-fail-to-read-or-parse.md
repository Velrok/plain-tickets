---
id: 4aawv9
title: cmd_list silently drops tickets that fail to read or parse
type: bug
status: done
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-09-25T12:28:32.761590Z
updated_at: 2026-09-25T14:01:00.272085Z
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

## Reproduce before fixing

This is a `bug` ticket, so the standing rule in `docs/contributor-orientation.md`
applies: **write a test that reproduces the silent drop and watch it fail first**,
before touching `src/commands.rs`.

Concretely, the first commit on this ticket should contain only a red test:

- build a fixture dir with two valid tickets and one unreadable file
  (`chmod 000`), plus a second fixture with one unparseable file
- run `tickets list` against it
- assert the valid tickets are listed AND that the failure is reported

Today that test must fail on the second assertion specifically - `list` prints
the valid rows and says nothing at all about the bad file, exit 0. Record that
actual output in the implementation notes. If the test goes green as written,
stop and report it - the premise is wrong.

Only once it is red do you change `cmd_list`.

`fd38vu` is the worked example for the TUI half of this same bug, including the
`(Vec<Ticket>, Vec<LoadFailure>)` shape. Read its implementation notes before
starting.

## Implementation notes

Landed as `909b9c1` (red test) + `179333b` (fix). Touches `src/commands.rs`
and `tests/cli_list.rs` only — `src/deps_graph.rs`'s `load_dir` deliberately
left alone, per scope note (that's `o3c87i`, a sibling ticket).

**Reproduced first, as required.** Before touching `cmd_list`, added four
e2e tests to `tests/cli_list.rs` (unreadable file, unparseable file, stdout
stays clean, exit code unchanged) and ran them red. Actual failure output on
the two load-failure assertions:

```
thread 'list_reports_unparseable_file_without_dropping_valid_ones' panicked:
stderr must name the unparseable file, got:
thread 'list_reports_unreadable_file_without_dropping_valid_ones' panicked:
stderr must name the unreadable file, got:
```

i.e. exactly the predicted symptom — valid tickets listed, stderr empty,
exit 0, no mention of the bad file anywhere. The other two tests (stdout
stays clean, exit code unchanged) passed even before the fix, since there
was nothing on stdout/stderr to begin with — kept as regression coverage
for the *fixed* state rather than evidence of the bug itself.

**Design.** Mirrored `fd38vu`'s shape exactly: a private `LoadFailure { path, reason }` struct and `load_tickets(all_dir: &Path) -> Result<(Vec<Ticket>, Vec<LoadFailure>)>` in `src/commands.rs`, replacing the
two `filter_map(...).ok()` calls. `cmd_list` calls it, and if
`format_load_failures` returns `Some(warning)`, prints it via `eprintln!`
before the table — stdout untouched. Did not reuse the TUI's `LoadFailure`/
`load_tickets`/`format_load_failures` (they live in `src/tui/mod.rs`,
private to that module, and the CLI's surface — a stderr line, not a flash
— is different enough that sharing would mean threading an output-target
parameter through for one call site each). Kept the duplication local
rather than introducing a shared module for two three-function pairs;
worth revisiting if a third caller needs the same shape.

**Stderr format:** `warning: N ticket(s) could not be loaded: <file> (<reason>)[, <file> (<reason>)...]` — same "name the files, not just a
count" approach as the TUI flash text, prefixed with `warning:` since
there's no footer chrome on the CLI to signal severity the way the TUI's
flash bar does.

**Verified against the real binary**, not just the test suite: built a temp
dir with one valid ticket and one file with `status: nonsense`, ran
`tickets list` with stdout and stderr captured separately. Stdout contained
only the valid row; stderr contained the warning naming the file; exit code
0 in both cases.

**Kept `tickets list | head` pipeable**, per `3mqhe3`'s broken-pipe fix on
this exact loop — the existing `list_piped_into_head_exits_cleanly_with_*`
tests (no bad files in their fixtures) still pass unchanged, confirming the
new warning path doesn't touch stdout or the broken-pipe handling.

**Tests:** 4 new e2e tests in `tests/cli_list.rs` (`cli_list.rs` total
29 → 33). Full suite: `cargo test` all green, `cargo clippy --all-targets`
and `cargo fmt --check` clean.

**Judgement calls:**

- Chose `eprintln!` directly over threading the warning through
  `anyhow::Result` or a return value — `cmd_list`'s exit code must stay 0
  per the ticket, so a load failure is not an error condition for this
  command, just a thing to report.
- Did not attempt to unify with the TUI's `LoadFailure`/`load_tickets` pair
  into a shared helper — see design note above.
