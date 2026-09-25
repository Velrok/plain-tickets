---
id: r0ro1x
title: front matter accepts ticket ids that FromStr would reject
type: bug
status: done
tags:
- validation
parent: null
blocked_by: []
created_at: 2026-09-25T13:27:39.105525Z
updated_at: 2026-09-25T13:56:48.503352Z
---

## Behaviour

`TicketId` has `FromStr` validation and is used as a clap value parser, so ids
arriving via the CLI are checked. Ids arriving via **serde, from a ticket file's
front matter, are not**. Deserialisation goes straight to the newtype and skips
the validation entirely.

Found during independent verification of `t9p76e` on 2026-09-25.

```
blocked_by: [""]
```

parses without complaint, and `deps-graph` then renders

```
[missing: ]
└── a1  todo  Some ticket
```

an empty-labelled stub. `list --unblocked` also keeps the ticket blocked, so
behaviour is at least consistent across both consumers - the seam handles it
fine. The gap is that the value was ever allowed in.

This is not a blocker-resolution bug and is not a regression from `xptucc` or
`t9p76e`. It is missing validation at the parse boundary.

## Why it matters

This is the parse-don't-validate pattern failing at exactly the point it is
supposed to hold. `docs/contributor-orientation.md` says any function taking a
`WorkingDir` may assume an initialised directory because `WorkingDir::new`
validated it. The same reasoning is applied to `TicketId` throughout the
codebase - code assumes an id is well-formed because the type says so - but
that guarantee is only true on the CLI path.

Also worth checking whether `Title` and `Tag` have the same hole, since they
follow the same newtype-with-`FromStr` pattern.

## Reproduce before fixing

This is a `bug` ticket, so the standing rule in `docs/contributor-orientation.md`
applies: **write a failing test first**. The first commit should contain only
the red test.

Write a unit test that deserialises front matter containing an empty-string id
(and a malformed one, e.g. wrong length or illegal characters) and asserts it is
rejected. Today it is accepted. Record the actual output.

## Shape of the fix

Implement `Deserialize` for the newtypes so it routes through the existing
`FromStr`, rather than duplicating the validation rule in a second place - the
same single-seam principle the blocker-resolution code follows.

Decide and record what happens to a ticket file that is already on disk with a
bad id. Rejecting it at parse time turns it into a load failure, which
`fd38vu` and `4aawv9` now surface properly rather than dropping silently - so
the machinery to report it already exists. Confirm that is the behaviour you
want before committing to it.

## Test first

- unit - empty-string id in front matter is rejected
- unit - malformed id (wrong length, illegal characters) is rejected
- unit - a valid id still round-trips
- same coverage for `Title` and `Tag` if they share the hole
- e2e - a ticket file with a bad id is reported as a load failure, not dropped

## Implementation notes

**Confirmed and widened during reproduction.** `Title` and `Tag` do share the
hole - both derive `Deserialize` via `#[serde(transparent)]`, which bypasses
their `FromStr` validation exactly like `TicketId`.

`TicketId` turned out worse than the ticket's premise: its `FromStr` had
`type Err = std::convert::Infallible` and performed **no validation at
all**, not even non-empty. So "front matter accepts ids `FromStr` would
reject" was not quite accurate for `TicketId` as found - `FromStr` itself
would not have rejected `""` either. `docs/coding-style.md`'s wrapper table
documents `TicketId` as "No validation — any non-empty string", so I treated
that as the intended contract and added the missing non-empty check to
`FromStr` alongside the `Deserialize` fix, rather than inventing new
length/character constraints (6-char/alphanumeric is a property of
generation via `nanoid`, not a documented validation rule - not adding it
here).

**Red run (first commit, `ddc6a0d`).** 8 new tests, all failing for the
right reason before any production code changed:

- `ticket_id_empty_via_from_str_is_err` - failed: `"".parse::<TicketId>()`
  was `Ok` (Infallible)
- `ticket_id_empty_via_deserialize_is_err`,
  `ticket_front_matter_blocked_by_empty_id_is_err` - failed: derived
  `Deserialize` accepted `""` with no error
- `title_empty_via_deserialize_is_err`,
  `title_invalid_chars_via_deserialize_is_err` - failed: derived
  `Deserialize` bypassed `Title::from_str`'s checks
- `tag_empty_via_deserialize_is_err`,
  `tag_invalid_chars_via_deserialize_is_err` - same, for `Tag`
- `load_tickets_reports_ticket_with_empty_id_as_failure_not_dropped` (TUI) -
  failed: `left: 2, right: 1` - the bad-id ticket loaded as a second valid
  ticket instead of being recorded as a failure

**Fix.** Removed `Deserialize` from each newtype's derive list and hand-wrote
`impl<'de> Deserialize<'de>` for `TicketId`, `Title` and `Tag`: deserialise to
`String`, then `.parse().map_err(serde::de::Error::custom)` - routes through
the existing `FromStr`, one seam, no duplicated rule. `TicketId::FromStr`
gained the non-empty check (`Err = String` now, was `Infallible`); no other
call site relied on the old `Infallible` error type. `Serialize` stays
derived on all three - only deserialisation needed the seam.

**Load-failure decision.** A ticket file already on disk with a bad id fails
at `Ticket::from_str` exactly like any other invalid front matter field (bad
`status`, missing field, etc.) - it was never a distinct code path, so no new
handling was needed for it specifically. That means its fate today is
whatever the *existing* front-matter-parse-failure fate is per surface:

- **TUI** (`fd38vu`, landed): reported via the flash mechanism, not dropped.
  Verified directly - added
  `load_tickets_reports_ticket_with_empty_id_as_failure_not_dropped`.
- **`cmd_list` / `deps-graph`** (`4aawv9`, still in-progress at the time of
  this work): still silently drops via `filter_map(...).ok()`, same as any
  other malformed front matter. Per contributor-orientation's explicit
  instruction, deliberately left untouched here to avoid colliding with
  `4aawv9`'s in-flight change. Manually verified with a hand-crafted
  `id: ""` ticket file: `tickets list` against it prints nothing and exits
  0 - unchanged, not worsened, by this fix.

**Mutation testing.** Reddened deliberately, confirmed the right tests failed,
restored, confirmed green:

- Removing the `TicketId::FromStr` non-empty check reddened
  `ticket_id_empty_via_from_str_is_err`,
  `ticket_id_empty_via_deserialize_is_err`, and
  `ticket_front_matter_blocked_by_empty_id_is_err`.
- Removing the custom `TicketId` `Deserialize` impl (reverting to derived)
  reddened `ticket_id_empty_via_deserialize_is_err`,
  `ticket_front_matter_blocked_by_empty_id_is_err`, and the TUI
  `load_tickets_reports_ticket_with_empty_id_as_failure_not_dropped`.

**Suite.** 297 passed, 0 failed across 15 test binaries (measured against
`ddc6a0d` + the fix, `cargo test`). `cargo clippy --all-targets` and
`cargo fmt -- --check` both clean.
