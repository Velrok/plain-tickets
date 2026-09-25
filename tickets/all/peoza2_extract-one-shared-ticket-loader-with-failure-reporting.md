---
id: peoza2
title: extract one shared ticket loader with failure reporting
type: task
status: todo
tags:
- refactor
parent: null
blocked_by: []
created_at: 2026-09-25T13:35:10.773777Z
updated_at: 2026-09-25T13:35:10.773777Z
---

## Behaviour

There are three near-identical ticket-loading implementations. Confirmed by
independent verification on 2026-09-25.

| Location                            | Returns                        | Tracks failures |
| ----------------------------------- | ------------------------------ | --------------- |
| `src/tui/mod.rs::load_tickets`      | `(Vec<Ticket>, Vec<LoadFailure>)` | yes          |
| `src/commands.rs::load_tickets`     | `(Vec<Ticket>, Vec<LoadFailure>)` | yes          |
| `src/deps_graph.rs::load_dir`       | `HashMap<TicketId, Ticket>`       | **no**       |

The two `LoadFailure` structs are **field-identical** (`path: PathBuf`,
`reason: String`) and the two `load_tickets` bodies are **verbatim-identical
logic** - `read_dir`, filter `.md`, `read_to_string` or record failure, parse or
record failure. This is copy-paste, not convergent design.

## Why this is worth doing

The same silent-drop bug had to be found and fixed **independently twice**
(`fd38vu` for the TUI, `4aawv9` for `cmd_list`) and is **still live in the
third** (`o3c87i`). A third bug or a format tweak now costs three edits, and the
evidence so far is that whoever fixes one does not know to fix the others.

Both copies were individually reasonable decisions - each was correctly scoped
out of a bugfix that should not have grown a cross-module refactor. The debt is
the accumulated result, not any one call.

## Shape

One loader module owning: directory scan, `.md` filtering, read, parse, and
failure collection. Reporting stays with the caller - the TUI seeds a footer
flash, the CLI writes a stderr warning - so the seam is loading, not reporting.

`load_dir` returning a `HashMap` keyed by id rather than a `Vec` is the real
work here. Reconcile it rather than special-casing it, or the extraction leaves
the one loader that still drops silently outside the seam, which defeats the
purpose.

## Sequencing

- Do this **after** the concurrent TUI tickets settle. Touching
  `src/tui/mod.rs` mid-flight is what both earlier tickets rightly avoided.
- `o3c87i` (corrupted archived tickets vanish) should either land on top of
  this, or be written so this absorbs it. Do not let it add a fourth copy.
- `bmtthv` (architecture review) records this as its strongest finding. If that
  review lands first it may reshape the seam - check it before starting.

## Test first

- unit - the shared loader reports an unreadable file rather than dropping it
- unit - the same for an unparseable file
- unit - valid tickets load unaffected alongside failures
- the existing e2e coverage in `fd38vu` and `4aawv9` must stay green unchanged,
  which is what proves the extraction preserved behaviour
