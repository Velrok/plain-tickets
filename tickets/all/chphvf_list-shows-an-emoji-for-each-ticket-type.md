---
id: chphvf
title: list shows an emoji for each ticket type
type: task
status: review
tags:
- list
- ux
parent: null
blocked_by: []
created_at: 2026-09-23T11:12:26.120717Z
updated_at: 2026-09-23T11:26:31.855916Z
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

## Implementation notes

### src/domain_types.rs

- `pub fn display_width(s: &str) -> usize` wrapping
  `unicode_width::UnicodeWidthStr::width`, doc-commented with why byte length
  and char count are both wrong.
- `impl TicketType { pub fn emoji(&self) -> &'static str }` - exhaustive match
  (desirable here, `TicketType` is a closed set and the compiler should shout
  if a type is added). 🎯 epic, 📖 story, 🔧 task, 🐛 bug. Single source of
  truth, reusable by `deps-graph`, `show` and the TUI later.

### src/commands.rs

`cmd_list`'s width block now builds `(id, status, type_col, title)` rows up
front, where `type_col = format!("{} {}", r#type.emoji(), r#type)` - emoji
alongside the word, never replacing it, so `tickets list | rg bug` still
works. All widths come from `display_width`, and a private
`pad_to_display_width(s, width)` pads by hand rather than using Rust's
formatter, which pads by `char` count and cannot safely be handed a
display-width number.

`unicode-width = "0.2"` added to `Cargo.toml`. The `Cargo.lock` diff is a
single line - the crate was already locked transitively via ratatui.

### Finding - the row-alignment test cannot catch this bug

Every `TicketType` contributes exactly one emoji, so the byte-versus-char
discrepancy is **uniform across every row** and cancels out in relative
terms. Reverting to the buggy `.to_string().len()` code and running
`list_title_column_aligns_across_mixed_type_widths` showed it **passing
against the bug**.

What the bug actually does is over-pad every row by a constant amount -
wasted whitespace, not raggedness. Hence
`list_widest_type_column_has_no_wasted_padding`, which asserts the gap after
the widest row's type text is exactly the two-space separator. Confirmed RED
against the byte-length bug (5 spaces instead of 2) and GREEN after the fix.

Both tests were kept; the alignment one is still a legitimate regression
check. The caveat is specific to columns carrying exactly one wide glyph per
row.

### Note for 3mqhe3

The block is now: build `rows` -> compute widths via `display_width` -> print
via `pad_to_display_width`. Colouring the status will need padding computed on
the **plain** text before colouring, or escape codes stripped before
measuring - `unicode-width` does not account for ANSI sequences.

Unlike this ticket, a status column carries a *variable* number of
zero-width escape bytes per row, so row-alignment tests likely **would** catch
a mistake there.

### Tests

Unit: `display_width_ascii_char_is_one`, `display_width_emoji_is_two`,
`ticket_type_emoji_is_distinct_per_type`.
E2e: `list_shows_emoji_for_each_ticket_type`,
`list_type_word_still_present_alongside_emoji`,
`list_title_column_aligns_across_mixed_type_widths`,
`list_widest_type_column_has_no_wasted_padding`.
