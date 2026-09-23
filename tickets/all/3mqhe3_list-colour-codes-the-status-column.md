---
id: 3mqhe3
title: list colour codes the status column
type: task
status: todo
tags:
- list
- ux
parent: null
blocked_by:
- chphvf
created_at: 2026-09-23T11:12:47.313368Z
updated_at: 2026-09-23T11:12:47.313368Z
---

## Behaviour

`tickets list` colours the status column so the state of the board reads at a
glance.

| Status | Colour |
|--------|--------|
| `done` | green |
| `rejected` | grey |
| `in-progress` | yellow |
| `review` | purple |
| `draft` | default, uncoloured |
| `todo` | default, uncoloured |

`draft` and `todo` were not specified. Leaving them uncoloured is the
proposal - they are the resting states and colouring everything would defeat
the purpose - but confirm before building.

Only the status cell is coloured. Id, type and title stay plain.

## Never emit colour into a pipe

This is the part that matters most. `tickets show` already renders through
`bat` and emits ANSI escapes **even when piped or under `NO_COLOR`**, which
makes its output unusable for scripting - the `tickets-cli-expert` skill
documents it as a trap, and it has already misled an agent in this repo into
parsing prose instead of front matter.

Do not repeat that mistake in `list`. `list` is the command people pipe into
`rg`, `wc` and `awk`, and it is the dependable one precisely because its
output is plain.

Required:

- Colour only when stdout is a terminal.
- Honour `NO_COLOR` - if it is set to anything, emit no escapes.
- Honour `CLICOLOR_FORCE` for the opposite case.
- Piping to a file or another process produces byte-identical output to
  today.

`anstream` and `anstyle` are already in `Cargo.lock` transitively via clap
and handle all three rules correctly, so adding them to `[dependencies]`
pulls in nothing new. Prefer that over hand-rolled escape strings and a
hand-rolled TTY check.

## The trap - pad before colouring, never after

`cmd_list` pads the status column to a computed width with `{:<status_w$}`
(`src/commands.rs:283-302`). ANSI escapes are bytes with **zero** display
width. Colour the string first and the padding formatter counts the escape
bytes, so every coloured row indents its title differently and the table
falls apart - worse, it looks fine in any test that compares a coloured
string against another coloured string.

Pad to width first, then wrap the padded cell in the style. Assert alignment
by column position in the uncoloured path rather than by whole-line compare.

## Test first

- e2e - piped output, the default in the test harness, contains no ANSI
  escape bytes at all
- e2e - piped output is byte-identical to the current output, proving the
  change is invisible to scripts
- e2e - with `NO_COLOR` set, no escapes are emitted even when forced
- e2e - with colour forced on, `done` carries the green code and `review` the
  purple one
- e2e - with colour forced on, the title column stays aligned across rows of
  mixed status, asserted by column position
- unit - every `TicketStatus` variant maps to a style, including the two that
  map to no style

## Keep it exhaustive-safe

`TicketStatus` gained `review` recently and will gain more. Map status to
style with an explicit arm per variant and **no wildcard `_ =>` arm**, so the
next status addition fails to compile rather than silently rendering
uncoloured. That is the convention the rest of the codebase already follows.
