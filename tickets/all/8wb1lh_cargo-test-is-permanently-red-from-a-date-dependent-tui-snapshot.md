---
id: 8wb1lh
title: cargo test is permanently red from a date dependent tui snapshot
type: bug
status: todo
tags:
- test-health
parent: null
blocked_by: []
created_at: 2026-09-23T10:30:40.691786Z
updated_at: 2026-09-23T10:30:40.691786Z
---

## Problem

`cargo test` does not pass on a clean checkout of main. The unit test
`tui::render::tests::detail_view_renders_ticket_fields` fails with

    -  |Created: 2026-07-30 |
    -  |Updated: 2026-07-30 |
    +  |Created: 2026-09-23 |
    +  |Updated: 2026-09-23 |

Confirmed on 2026-09-23 by building at the commit before the deps-graph work
landed, so it is not caused by any recent change. It has been failing every
day since 2026-07-30, the day the snapshot was recorded.

## Root cause

The test helpers in `src/tui/render.rs` build their fixture with the wall
clock:

    fn make_ticket(...) -> Ticket {
        let now = Utc::now();
        ...
    }

`make_ticket_with_tags` does the same. The detail view renders `Created:` and
`Updated:`, so its snapshot captures whatever date the test happened to run
on. The snapshot is only ever correct on the day it was recorded.

Re-recording the snapshot is not a fix - it just resets the clock on the same
bug and it breaks again tomorrow. The fixture has to stop depending on the
current date.

## Why this matters now

A permanently red test is not a neutral cost. It trains everyone, humans and
agents alike, to run `cargo test` and mentally filter out a known failure,
which is exactly how a real regression gets waved through. It already caused
concrete friction - contributors working in `src/tui/` have to be told to
ignore a failing tui test, leaving them unable to tell their own breakage
apart from the known one.

Fix this before building anything further on top.

## Behaviour

- `cargo test` passes completely on a clean checkout, on any date.
- The test suite contains no fixture whose rendered output depends on the
  current date.

## Implementation

- Replace `Utc::now()` in `make_ticket` and `make_ticket_with_tags` in
  `src/tui/render.rs` with a single fixed UTC timestamp constant shared by
  both helpers. Pick an explicit, obviously-fictional date and construct it
  deterministically rather than parsing at runtime.
- Re-record `tickets__tui__render__tests__detail_view_renders_ticket_fields.snap`
  against the fixed timestamp, and check the committed snapshot now shows the
  fixed date.
- Audit the other `Utc::now()` call sites for the same class of problem and
  report what you find. `src/graph.rs:233-234` is in test code and is a
  candidate; the production call sites in `src/commands.rs`, `src/tui/mod.rs`
  and `src/tui/app.rs` are legitimate and must NOT be touched. Only change a
  test-code call site if it actually feeds an assertion or a snapshot.

## Stray artefact

Running the suite leaves an untracked
`src/tui/snapshots/*detail_view_renders_ticket_fields.snap.new` behind, which
disappears once the snapshot matches again. Confirm the tree is clean after a
full `cargo test` run when you are done.

## Test first

This is a fix to test infrastructure, so the usual red-green applies to the
suite itself rather than to new product code.

- Confirm RED first - run the failing test and capture the actual diff before
  changing anything.
- After the fix, `cargo test` is fully green with zero failures.
- Demonstrate date independence rather than asserting it. Running the suite
  under a faked future date, or a brief note showing the fixture no longer
  reads the clock, both count - say which you did.
