---
id: 3mqhe3
title: list colour codes the status column
type: task
status: done
tags:
- list
- ux
parent: null
blocked_by:
- chphvf
created_at: 2026-09-23T11:12:47.313368Z
updated_at: 2026-09-25T12:37:53.034402Z
---

## Behaviour

`tickets list` colours the status column so the state of the board reads at a
glance.

| Status        | Colour              |
| ------------- | ------------------- |
| `done`        | green               |
| `rejected`    | grey                |
| `in-progress` | yellow              |
| `review`      | purple              |
| `draft`       | default, uncoloured |
| `todo`        | default, uncoloured |

`draft` and `todo` stay uncoloured. Confirmed on 2026-09-23. They are the
resting states, and colouring every row would defeat the purpose - the point
is that the four coloured statuses stand out against a plain background.

This is settled, not a proposal. Do not add colours for them.

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

## Implementation notes (2026-09-25), commit `f1c7407`

Landed on worktree branch `ticket-3mqhe3`, merged to `main`. Awaiting
independent verification — not yet `done`.

**Dependencies:** added `anstream` and `anstyle` as *direct* dependencies.
Confirmed via `cargo tree -i` that both were already present transitively
via clap, so `Cargo.lock` gained only the two top-level entries and pulled
no new versions.

**Colour mapping:** `TicketStatus::style() -> Option<anstyle::Style>` in
`src/domain_types.rs`, one explicit arm per variant with no wildcard, so
adding a status fails to compile rather than silently rendering plain.
`done` → green, `rejected` → bright black (grey), `in-progress` → yellow,
`review` → magenta (purple), `draft`/`todo` → `None`.

**The padding trap, handled:** the status cell is padded with the existing
`pad_to_display_width` *first*, then wrapped in the style. ANSI escapes are
zero-display-width, so colouring before padding would have corrupted the
column alignment `chphvf` established.

**Pipe safety:** rows go through `anstream::stdout()`, whose `AutoStream`
inspects `NO_COLOR` / `CLICOLOR_FORCE` / TTY and strips escapes when not
appropriate. The required precedence (`NO_COLOR` beats `CLICOLOR_FORCE`) was
confirmed by reading anstream's own `choice()` rather than assumed.

**Verified against the real binary:** `list | cat -v` → zero `^[` escapes,
columns aligned; `CLICOLOR_FORCE=1 ... | cat -v` → `^[[33m`/`^[[35m`/
`^[[32m`/`^[[90m` on in-progress/review/done/rejected, `draft`/`todo`
untouched; `NO_COLOR=1 CLICOLOR_FORCE=1 ...` → no escapes.

**Tests:** unit `ticket_status_style_maps_every_variant` (exhaustive over all
six variants), plus six e2e in `tests/cli_list.rs` covering no-ANSI-when-
piped, byte-identical default output, `NO_COLOR` suppression, forced-colour
mapping, `draft`/`todo` staying plain, and title-column alignment by
position across coloured and uncoloured rows. New `common::tickets_with_envs`
helper added without changing the existing helper's signature.
`chphvf`'s `list_widest_type_column_has_no_wasted_padding` untouched and
still passing. `cargo test` 252 passed / 0 failed on the branch; 257 after
merge to `main`.

**Judgement call:** the ticket's "piped output is byte-identical to the
current output" criterion cannot literally diff against a pre-change binary
in an e2e test, so it was implemented as an exact hand-built expected-string
comparison of the default path — stronger than a self-comparison.

**Out of scope, unchanged:** `tickets show` still emits ANSI into a pipe via
`bat --color=auto`. That is a separate pre-existing wart.

## Implementation notes

Merged to main 2026-09-25. Colour feature `f1c7407`, broken-pipe fix `0e7fb57`.

**Colour mapping** lives in `TicketStatus::style()` in `src/domain_types.rs`, with
an explicit arm per variant and no wildcard, so a new status stays loud at
compile time. `in-progress` yellow, `review` magenta, `done` green, `rejected`
bright-black; `draft` and `todo` uncoloured.

**Broken-pipe regression, found after the feature landed.** `cmd_list` is the
only command that writes fallibly, so `list | head -1` exited 1 with a
`failed to write to stdout` error. The write loop now matches on the `writeln!`
result: `ErrorKind::BrokenPipe` returns `Ok(())` for a clean exit, every other
error still propagates via `.context(...)`. Deliberately not a blanket
`let _ =` - ENOSPC/EIO must still surface.

**Independent verification (PASS).**

- Colour under a real pty via `script -q`, with no forcing env vars. Piped and
  redirected output checked with `xxd` for zero ESC bytes. `NO_COLOR` correctly
  wins over `CLICOLOR_FORCE`; `TERM=dumb` also suppresses.
- Padding proven correct non-visually: ANSI stripped from forced-colour output
  and diffed byte-for-byte against uncoloured output of the same fixture
  (mixed-width titles, CJK, emoji) - identical. This repo has shipped a
  byte-length padding bug before.
- Broken pipe verified from real shell pipelines on 6- and 600-ticket fixtures
  (crossing the pipe buffer boundary), and `| true`. A genuine write error,
  produced with a filled 8 MB disk image, still exits 1.
- Mutation testing confirmed both fixes discriminating. Note one disclosed
  limitation: the small-fixture broken-pipe test stays green when the fix is
  reverted, because a short listing can finish writing before the reader
  closes. The large-fixture test is the one doing the real work, and the
  contributor documented this in a code comment rather than hiding it.

**Two findings routed out of scope.**

- Every other command uses `println!`/`print!` and panics on a broken pipe
  (exit 101). Confirmed pre-existing on `ee88666`, not introduced here. Filed
  separately.
- The colour feature was independently implemented twice, on two lineages
  (`7dc275b` and `f1c7407`), and both reached main. Checked: the colour files
  are byte-identical between them, main has exactly one `style()` and one
  `anstyle` dependency, and no duplicated tests. No dead code.
