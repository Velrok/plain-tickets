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
