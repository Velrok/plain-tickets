---
id: zgfki6
title: TUI help overlay clips its own quit and dismiss keybindings
type: bug
status: in-progress
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-09-23T11:12:44.798707Z
updated_at: 2026-09-23T11:13:20.127102Z
---

## Problem

The TUI help overlay renders into a fixed-proportion box from
`centered_rect(46, 60, f.area())` in `draw_help` (`src/tui/render.rs`). The
box does not grow with its content, so the keybinding list is clipped from
the bottom.

At the 80x24 size used by the snapshot tests, the clip currently falls before
these lines:

    ? / F1     show this help
    q          quit

    [any key]  dismiss

So the help screen does not show how to quit, and does not show how to
dismiss the help screen itself. That is the worst possible thing for a help
screen to omit - a user who opens it to find out how to get out is told
nothing, and has to guess.

Found while implementing pgej72 on 2026-09-23. It is **pre-existing**, not
caused by that ticket. pgej72 added a `/` line to the list, which lands after
the clip boundary too, so the new keybinding is in the source but invisible
in the rendered overlay.

## Behaviour

- The help overlay shows **every** keybinding it defines, with no silent
  clipping.
- It renders correctly at the 80x24 size the snapshot tests use, and does not
  break at larger sizes.

## Implementation notes

`draw_help` builds a fixed `Vec<Line>` and hands it to a `Paragraph` inside a
bordered `Block`. The height needed is therefore known before rendering -
line count plus two for the borders. Prefer sizing the box from that rather
than from a percentage of the terminal.

Decide deliberately what happens when the terminal is genuinely too short to
fit the list, and say which you chose. Clamping to the available height and
reintroducing a silent clip is not acceptable; scrolling, or a shorter
two-column layout, both are.

Check the same class of bug in `draw_detail`, which also uses
`centered_rect` (80, 80) with content of unbounded length - a ticket body can
be arbitrarily long. Report what you find; fixing it may belong in its own
ticket.

## Test first

- unit or snapshot - the rendered help overlay contains the `q` quit line
- unit or snapshot - the rendered help overlay contains the dismiss line
- snapshot - the overlay renders correctly at 80x24

The existing `help_overlay_renders_keybindings` snapshot currently encodes
the clipped output, so it will need re-recording. Confirm the new rendering
is actually correct before accepting it - do not promote a snapshot merely to
make a test pass.

## Implementation notes

### Sizing

`draw_help` now builds its `Vec<Line>` first, via new `help_lines_single()` /
`help_lines_compact()` helpers, so the required height is known exactly -
`lines.len() + 2` for the borders. Width is derived the same way, as the max
`Line::width()` plus four for borders and one padding column each side.

A new `centered_rect_fixed(width, height, r)` centres an exact-size box within
`r`. The percentage-based `centered_rect` is untouched and still used by
`draw_detail`.

### Too-short terminal - compact two-column layout, not scrolling

Scrolling would need new interaction state - a scroll offset on `App`, key
handling to move it, and event-loop changes in `mod.rs`. That is materially
more surface than a render-only bug fix should touch.

The compact layout is pure rendering. Entries split roughly in half via
`div_ceil(2)` into two columns, each row carrying `key/desc key/desc`, cutting
content height from 17 lines (19 rows with borders) to 11 lines (13 rows). It
engages automatically whenever the single-column layout would not fit. Never
clips, never needs new state.

At 80x24 the single-column layout fits, so that is what the main snapshot
exercises; the compact layout has its own snapshot at 80x15.

### Deliberately not handled

A terminal under roughly 13 rows, where even the compact layout will not fit.
`centered_rect_fixed` clamps to the available area, which for so short a
terminal reproduces a clip. Judged a degenerate size and out of scope for a
bug fix; nothing panics. A third fallback tier would be scope creep.

### Tests

- `help_overlay_shows_quit_keybinding_at_80x24` - confirmed RED against the
  pre-fix code before the fix, proving the bug.
- `help_overlay_shows_dismiss_line_at_80x24` - same RED to GREEN confirmation.
- `help_overlay_switches_to_compact_layout_on_short_terminal` - every
  keybinding key and the dismiss line visible at 80x15.
- `help_overlay_compact_layout_renders_correctly` - new snapshot.
- `help_overlay_renders_keybindings` - existing test, snapshot re-recorded to
  show the full 17-line list including the previously clipped `/ filter`,
  `? / F1`, `q quit` and `[any key] dismiss` lines.

`cargo-insta` is not installed in this environment, so both snapshots were
promoted by hand - failing test run, full `.snap.new` read line by line, box
maths checked against the rendered ASCII, `assertion_line:` stripped, then
byte-diffed against the stripped `.snap.new` to confirm nothing else changed.
