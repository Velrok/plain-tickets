---
id: pgej72
title: TUI filter tickets by id or title with slash
type: story
status: todo
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-09-23T10:51:52.149259Z
updated_at: 2026-09-23T10:51:52.149259Z
---

## Behaviour

In the TUI board, `/` opens a filter prompt. Typing narrows the board to
tickets whose **id or title** matches the query. One query, matched against
both fields - not two separate modes.

- `/` from the board enters filter input. The footer becomes a prompt showing
  `/` followed by the query so far, in place of the usual key hint.
- Matching is **case-insensitive substring**, against the id and the title.
  A ticket matches if either field contains the query. Ids are lowercase
  alphanumeric, titles are mixed case, so a case-sensitive match would be a
  trap for the user.
- Filtering is **live** - the board narrows as each character is typed, not
  only on Enter.
- `Enter` commits the filter and returns to normal board navigation with the
  filter still applied.
- `Esc` from the prompt cancels and restores the unfiltered board, including
  when a filter was already active before `/` was pressed.
- With a filter applied, `Esc` from the board clears it.
- `Backspace` deletes a character. Deleting to an empty query shows the full
  board again but stays in input mode.

## The filter must be visible

A board silently hiding tickets is a bug magnet - the user presses `/`,
wanders off, comes back and thinks tickets have vanished. While a filter is
active and the prompt is not open, the footer must show the active query and
how to clear it.

## Key capture is the main trap

`key_to_message(KeyCode, &Screen)` in `src/tui/mod.rs` maps keys to messages
per screen. While the filter prompt is open, **every printable character is
literal text, not a command**. `q` must not quit, `j`/`k` must not move,
`n` must not create a ticket, `e` must not launch the editor. Only `Enter`,
`Esc` and `Backspace` are control keys in that mode.

The existing `Screen` enum (`Board`, `Detail`, `Help`) is the natural place
for this - the mapping function already dispatches on it, so a new input mode
fits the architecture rather than fighting it. Prefer that over a boolean
flag on `App` that the key mapper has to consult separately.

## Filter once, in one place

Apply the filter inside `App::col_indices` (`src/tui/app.rs:42`), which is
already the single source of truth for which tickets appear in a column.
Everything downstream - rendering, scroll, focus movement, `H`/`L` ticket
moves, `y` copy id, `Enter` detail - then respects the filter for free.

Do not filter at the render layer. That would leave navigation moving through
invisible tickets, which is worse than not having the feature.

## Focus must survive filtering

`App::col` and `App::row` are positional indices. Narrowing a column can put
`row` out of range, and the focused ticket may be filtered out entirely.

- Clamp `row` to the filtered column length whenever the query changes.
- If the whole board is empty under the filter, `focused_ticket()` returns
  `None` and the board renders empty columns. Nothing may panic, and `Enter`,
  `y`, `e`, `H`, `L` must all be safe no-ops.

## Scope

- Session only. The query is not persisted to `.tickets.toml` and does not
  survive a restart.
- Board screen only. `/` does nothing in the detail and help screens.
- Plain substring. No regex, no fuzzy matching, no field-qualified syntax
  like `id:abc`. If those are wanted they are their own ticket.
- The filter narrows what is shown. It does not change any ticket, and it
  does not interact with the `list` command's `--status`/`--type`/`--tag`
  flags, which are a separate CLI concern.

## Test first

`key_to_message` is a pure function and the render path is snapshot-tested,
so this is well covered by unit tests - no manual driving needed.

- unit - `/` on the board enters filter mode
- unit - in filter mode, `q` produces a text-input message, not `Quit`
- unit - in filter mode, `Esc` cancels and `Enter` commits
- unit - `/` in the detail and help screens produces no message
- unit - `col_indices` returns only matching tickets for a query
- unit - a query matching an id but not a title still matches, and vice versa
- unit - matching is case-insensitive in both directions
- unit - `row` is clamped when the filter shrinks the focused column
- unit - `focused_ticket()` is `None` and no key panics when nothing matches
- snapshot - the board renders the filter prompt while typing
- snapshot - the board renders the active-filter indicator after `Enter`

## Housekeeping

- Add `/` to the help overlay keybinding list in `draw_help`
  (`src/tui/render.rs`), and to the footer hint string in `draw_board`.
- Both are snapshot-tested, so `help_overlay_renders_keybindings` and the
  board snapshots will change. Re-record them only after confirming the new
  rendering is actually correct - do not accept a snapshot merely to make a
  test pass.
- The footer hint is already long. Check it still fits at 80 columns before
  appending to it, and shorten the existing hint if not.

## Note for whoever picks this up

This touches `src/tui/mod.rs`, `src/tui/app.rs` and `src/tui/render.rs`.
Ticket `xkbw11` is changing the same files to add the `review` status, so
take this one after that has landed rather than alongside it.

Test fixtures in the tui test modules use a local `fixed_timestamp()` helper
rather than `Utc::now()`, deliberately - the suite must stay date
independent. Follow that convention.
