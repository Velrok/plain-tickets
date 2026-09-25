---
id: fd38vu
title: TUI silently drops tickets that fail to read or parse
type: bug
status: done
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-09-25T12:08:10.138081Z
updated_at: 2026-09-25T12:28:15.222024Z
---

## Symptom

A ticket file that cannot be read or cannot be parsed **disappears from the
TUI board with no indication at all**. No warning, no count, no footer
message. The board simply renders fewer tickets than exist on disk, and
nothing tells the user that anything is missing.

This hides data loss. A ticket with a typo in its front matter is
indistinguishable from a ticket that was never created.

## Root cause

`load_tickets` in `src/tui/mod.rs` discards failures twice, both silently:

```rust
.filter_map(|e| std::fs::read_to_string(e.path()).ok())   // unreadable file
.filter_map(|raw| raw.parse::<Ticket>().ok())             // unparseable front matter
```

`.ok()` throws away the error in each case, and `filter_map` then drops the
entry. Note there are **two** drop sites, not one - a file that exists but
cannot be read is lost just as quietly as one that fails to deserialise.

## How this was discovered

Found while investigating `7zl07z` (TUI review column always empty). That
ticket turned out to have no code defect - the reported symptom came from
running the stale `tickets` binary on PATH, built before the `review` status
existed. Its compiled `TicketStatus` could not deserialise `status: review`,
so `Ticket::from_str` failed and these `filter_map`s silently dropped every
affected ticket.

**This silent drop is the mechanism that made that bug invisible and hard to
diagnose.** Had the TUI said "3 tickets could not be read", the root cause
would have been obvious immediately instead of needing a full investigation.

`7zl07z`'s own text called this out ahead of time - "a silent drop is its own
problem... that hides data loss from the user" - and its contributor
deliberately declined to fix it there as out of scope, flagging it for a
follow-up. Independent verification agreed it was correctly deferred and
that the hazard is real.

## Reproduce

1. Copy any ticket from `tickets/all/` into a throwaway dir via `--dir`.
2. Corrupt its front matter - e.g. change `status: todo` to `status: nonsense`.
3. Launch the TUI against that dir.
4. The ticket is absent from every column. Nothing reports it.

`cargo run -- list` behaves the same way and should be checked too - see
scope note below.

## Behaviour wanted

- A ticket that cannot be read or parsed is **never dropped without the user
  being told**.
- The signal names how many files were affected, and ideally which.
- Read failures and parse failures are both covered.

## Design note - the startup call has no App yet

`load_tickets` has four callers in `src/tui/mod.rs`. Three of them (lines
~70, ~173, ~190) run with an `App` in hand and can set `app.flash`, the
existing transient status-bar mechanism used by e.g. `copy_id_to_clipboard`.

The fourth (line ~29) runs at startup, **before** `App::new` is called, so
there is no `app.flash` to write to yet. Whoever implements this needs a
plan for that path - options include returning the failures alongside the
tickets and seeding the flash when the `App` is constructed, or widening the
return type so callers decide. Do not silently leave startup uncovered; the
stale-binary case that motivated this ticket fails precisely at startup.

## Scope question to settle

This ticket covers the **TUI**. Check whether `cmd_list` and the other CLI
read paths have the same pattern - if they do, decide deliberately whether
they are in scope here or a sibling ticket, and say which in the notes.
Do not fix them silently as scope creep.

## Test first

- unit - a directory containing one valid and one unparseable ticket yields
  the valid ticket **and** a recorded failure, rather than just the valid one
- unit - the same for a file that exists but cannot be read
- unit or e2e - the user-visible signal is actually present in the rendered
  output, asserted against the rendered buffer rather than a constructed
  string
- the "nothing is dropped without a signal" test is the one worth keeping -
  it is the invariant, and it will catch the next drop site someone adds

## Implementation notes

Landed as `8e46c2c`, merged to main 2026-09-25. Touches `src/tui/mod.rs` and
`src/tui/render.rs` only.

**Design.** `load_tickets` now returns `(Vec<Ticket>, Vec<LoadFailure>)` instead
of `filter_map(...).ok()`. The type itself forces every caller to decide what to
do with failures rather than letting `.ok()` discard them.

The startup path was the hard case - it runs before `App::new`, so it had no
flash to write to. A new `build_app(working_dir, columns) -> Result<App>` calls
`load_tickets`, constructs the `App`, and seeds `app.flash` before the terminal
is touched. The three post-`App` call sites (fs-watch reload and the two
post-`$EDITOR` reloads) go through `reload_tickets(app, working_dir)`. Both
funnel through one `seed_load_flash` helper - a single seam, not two paths.

Reuses the existing footer-flash mechanism rather than inventing a parallel
stderr codepath, so all four call sites report through one already-tested UI
surface.

**Scope call - `cmd_list` left alone.** `src/commands.rs:260-261` has the
byte-identical `filter_map(...).ok()` pattern. Deliberately not fixed: the
ticket scopes this to the TUI. Follow-up ticket filed.

**Trade-off accepted.** The flash is transient (2s, existing `FLASH_DURATION`),
matching existing UI conventions. Independent verification judged this a genuine
residual weakness for the startup case specifically - a user who looks away
misses it, and there is no way to recall the message. Follow-up ticket filed.

**Tests.** 8 new unit tests in `src/tui/mod.rs` and `src/tui/render.rs`.
Suite 257 -> 265 passed, 0 failed. clippy and fmt clean.

**Independent verification (PASS).** Drove the real TUI for all four call sites
including both `$EDITOR` paths with a fake editor that corrupts on save.
Mutation testing confirmed the tests are discriminating: removing the
`seed_load_flash` call reddened the startup regression test; making
`load_tickets` discard its failure vector reddened three tests. Probing beyond
the author's cases - multiple simultaneous failures (pluralisation holds), all
tickets failing, empty `all/`, zero-byte file, front matter with no body, a
directory named `*.md`, a ~200-char filename - found no defects and no panics.
