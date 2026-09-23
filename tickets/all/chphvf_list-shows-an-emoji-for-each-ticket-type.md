---
id: chphvf
title: list shows an emoji for each ticket type
type: task
status: in-progress
tags:
- list
- ux
parent: null
blocked_by: []
created_at: 2026-09-23T11:12:26.120717Z
updated_at: 2026-09-23T11:13:18.906496Z
---

## Behaviour

`tickets list` shows an emoji alongside the type word in the type column, so
the type is scannable at a glance without reading it.

    a3f9c1  todo  🔧 task   Fix login bug
    b7d2e0  done  🐛 bug    Crash on empty tags

The emoji is added **alongside** the word, not instead of it. Keeping the
word means `tickets list | rg bug` still works, and terminals with poor emoji
support degrade to a stray glyph rather than losing the information entirely.

## Proposed glyphs

| Type | Emoji |
|------|-------|
| `epic` | 🎯 |
| `story` | 📖 |
| `task` | 🔧 |
| `bug` | 🐛 |

These are a starting point, not settled - swap any that read badly in the
terminal. Whatever is chosen, define them in one place keyed off
`TicketType` so `deps-graph`, `show` and the TUI can reuse them later rather
than each inventing their own mapping.

## The trap - column width is currently computed in bytes

`cmd_list` sizes its columns with `.to_string().len()`
(`src/commands.rs:277-295`). On a `String` that is the **byte** length, which
happens to equal the display width today only because every type and status
name is pure ASCII.

Emoji break that twice over. `🐛` is four bytes, one `char`, and **two
terminal columns wide**. Using `.len()` would pad by four, and even `.chars()
.count()` would pad by one - both wrong, and the title column would ragged
out.

Switch the width calculation to a display-width measure.
`unicode-width` is already in `Cargo.lock` transitively via ratatui, so
adding it to `[dependencies]` pulls in nothing new.

Check the result against a real terminal, not only against the test
assertions - a test comparing two equally-wrong strings will happily pass.

## Related known issue

The TUI has the same class of bug already: the `📋` in card top borders
occupies one buffer cell but renders double-width, so top borders sit one
cell short. Out of scope here, but it is the same mistake, and whatever
helper this ticket introduces is what would fix it.

## Test first

- e2e - output contains the expected emoji for each of the four types
- e2e - the type word is still present alongside the emoji, so filtering the
  output by type name still matches
- e2e - the title column stays aligned across rows of mixed types, asserted
  by column position rather than by a whole-line string compare
- unit - the display width helper returns 2 for an emoji and 1 for an ASCII
  character

## Out of scope

`deps-graph`, `show` and the TUI keep their current rendering. This ticket
only changes `list`, but the type-to-emoji mapping it introduces should be
placed where those can adopt it later.
