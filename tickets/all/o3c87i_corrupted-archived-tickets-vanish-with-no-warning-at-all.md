---
id: o3c87i
title: corrupted archived tickets vanish with no warning at all
type: bug
status: done
tags:
- cli
parent: null
blocked_by: []
created_at: 2026-09-25T13:34:49.411076Z
updated_at: 2026-09-25T14:10:00.469897Z
---

## Behaviour

`fd38vu` fixed silent ticket drops in the TUI, `4aawv9` fixed them in `cmd_list`.
A third loader was left untouched in both, and it still drops silently:
`load_dir` in `src/deps_graph.rs`, backing `load_active`/`load_archived`, used
by `deps-graph` and `list --unblocked`.

Reproduced during independent verification of `4aawv9` on 2026-09-25.

Corrupt an **archived** ticket that is named as a blocker, then

```
$ tickets list --unblocked
(no stdout, the dependent is correctly excluded)
(stderr completely empty, no warning at all)
```

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
near-identical ticket-loading paths and `peoza2` exists to collapse them. Either
land that extraction first and build on it, or write this fix so it is trivially
absorbed by it.

Note `load_dir` returns `HashMap<TicketId, Ticket>` rather than a `Vec`, so it
is not a drop-in for the other two - that difference is part of what `peoza2`
has to reconcile.

Also settle what `deps-graph` should render for a blocker whose file exists but
will not parse. It is neither `ArchivedStub` nor `Missing` in the current
`BlockerResolution` - it is a fourth case, "present but unreadable", and
silently treating it as `Missing` would be wrong.

## Related

`fd38vu`, `4aawv9` (the first two loaders), `peoza2` (the extraction),
`bmtthv` (architecture review, where duplicated loading is finding #1).

## Implementation notes

Landed as `32d70b2` (red tests) + `d8bc161` (fix). Touches
`src/deps_graph.rs`, `src/commands.rs`, `tests/cli_deps_graph.rs`,
`tests/cli_list.rs`, `tests/common/mod.rs`.

**Reproduced first, as required.** Added `common::corrupt_archived_file`
(archives a ticket via the real CLI, then overwrites its already-archived
file with unparseable front matter) and three red e2e tests:
`list_unblocked_reports_corrupted_archived_blocker_on_stderr`,
`deps_graph_reports_corrupted_archived_blocker_on_stderr`, and
`deps_graph_unreadable_archived_blocker_renders_distinctly_from_missing`.
Actual red output, exactly matching the ticket's premise:

```
stderr must name the corrupted archived blocker file, got: ""
```

for both `list --unblocked` and `deps-graph`, and for the third test:

```
a present-but-unreadable blocker must not render as missing: [missing: hatpjm]
└── itk0tm  todo  Depends on corrupted archive
```

i.e. the corrupted archived blocker was silently swallowed and, on the
`deps-graph` path, actively mislabelled as a typo'd/missing id.

**Design — extended `load_dir` in place, third small copy accepted.**
`load_dir` (backing `load_active`/`load_archived`) now returns
`(HashMap<TicketId, Ticket>, Vec<LoadFailure>)` instead of silently
dropping failures, mirroring the shape `fd38vu` and `4aawv9` established —
but its own local `LoadFailure`/`format_load_failures`, not a shared type,
since `load_dir`'s `HashMap` return (keyed, not ordered) genuinely isn't a
drop-in for the other two loaders' `Vec`, as the ticket predicted. This is
a deliberate third copy of near-identical load-and-warn logic, documented
here rather than silently added — `peoza2` is the tracked follow-up to
reconcile all three; this fix is written so it's trivially absorbed by
that extraction (same `(items, failures)` shape, same warning-string
format as `commands.rs`'s version).

`cmd_deps_graph` reports `graph.load_failures()` (active + archived
failures combined) on stderr via the new `deps_graph::format_load_failures`,
after the forest and before the cycle warning. `cmd_list --unblocked`
reports `active_failures` and `archived_failures` from the deps-graph
loader separately from the pre-existing `all/` warning from `load_tickets`
— chose NOT to merge/dedup the `active_failures` (which overlaps with
`all/`, already covered by `load_tickets`'s own warning) into a second
warning line, since doing so would print a duplicate line for a corrupted
file in `all/` when `--unblocked` is passed. `archived_failures` is the
only genuinely new information that path adds, so that's the one always
reported.

**The fourth `BlockerResolution` case, settled.** Added
`ArchivedUnreadable(TicketId, String)` — distinct from both `ArchivedStub`
(readable, wrong status) and `Missing` (no file anywhere). The hard part:
a file that fails to *parse* may have no readable `id` field at all, so
the id used to match it against a `blocked_by` reference has to come from
the **filename** (`<id>_<slug>.md`) rather than the file's content — added
`id_from_filename` (split on first `_`) and `failures_by_id` to build an
id → reason lookup from archived load failures, consulted by
`resolve_blocker` only after both `active` and `archived` miss. Renders as
`[unreadable: <id> (<reason>)]`, visually and semantically distinct from
`[missing: <id>]` — the file exists and the data may be recoverable, unlike
a genuine typo. Not satisfied (same as `ArchivedStub`/`Missing`) — a file
that can't be verified `done` cannot satisfy a dependency.

`is_unblocked` and `resolve_blocker` both gained an `archived_failures: &HashMap<TicketId, String>` parameter as a result; all call sites and unit
tests updated (existing tests pass an empty map, two new unit tests cover
`ArchivedUnreadable` resolution and satisfaction).

**Verified against the real binary**, not just the test suite: built a temp
dir, created and archived a blocker, corrupted its archived file directly,
created a dependent `blocked_by` it, and ran both `tickets list --unblocked`
and `tickets deps-graph` with stdout/stderr captured separately — dependent
correctly held back on stdout, corrupted filename named on stderr for both
commands, `deps-graph`'s stdout rendered `[unreadable: <id> (<reason>)]`
rather than `[missing: <id>]`.

**Tests.** 3 new e2e tests (2 in `tests/cli_deps_graph.rs`, 1 in
`tests/cli_list.rs`) + 2 new unit tests in `src/deps_graph.rs`, plus a new
shared `corrupt_archived_file` helper in `tests/common/mod.rs`. Suite:
306 passed across 15 binaries (`cli_deps_graph.rs` 18→20, `cli_list.rs`
33→34, unit tests in `src/main.rs`'s bin target 174→176), 0 failed.
`cargo clippy --all-targets` and `cargo fmt --check` clean.

**Mutation testing**, per the standing rule: three separate mutations,
each restored via `git show HEAD:<path> > <path>` (never `git checkout --`)
and confirmed back to a clean `git diff --stat` before the next one.

- Collapsing `resolve_blocker`'s `archived_failures` lookup back to
  `Missing` reddened both the new unit test
  (`blocker_present_in_archived_failures_resolves_archived_unreadable_not_missing`)
  and the e2e `deps_graph_unreadable_archived_blocker_renders_distinctly_from_missing`.
- Deleting `cmd_deps_graph`'s `format_load_failures`/`eprintln!` call
  reddened `deps_graph_reports_corrupted_archived_blocker_on_stderr`.
- Deleting `cmd_list`'s `archived_failures` `eprintln!` call reddened
  `list_unblocked_reports_corrupted_archived_blocker_on_stderr`.

**Judgement calls:**

- Local third `LoadFailure`/`format_load_failures` copy in `deps_graph.rs`
  rather than reusing `commands.rs`'s or the TUI's — see design note above;
  `peoza2` is the tracked place to unify all three.
- `cmd_list --unblocked` does not re-report `active_failures` when they
  duplicate what `load_tickets`'s `all/` warning already said, to avoid a
  confusing duplicate stderr line — only `archived_failures` (the actually
  new information this path adds) is always reported.
- Derived the failed-file's id from its filename rather than leaving
  present-but-unparseable archived blockers unresolvable — filenames are
  the one thing guaranteed to survive a parse failure, and the repo's own
  filename convention (`<id>_<slug>.md`) already encodes exactly the id
  needed.
