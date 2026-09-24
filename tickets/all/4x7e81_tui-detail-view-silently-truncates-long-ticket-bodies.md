---
id: 4x7e81
title: TUI detail view silently truncates long ticket bodies
type: bug
status: review
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-09-23T11:21:55.199665Z
updated_at: 2026-09-24T16:25:36.190905Z
---

## Problem

`draw_detail` in `src/tui/render.rs` renders into a fixed-proportion box from
`centered_rect(80, 80, f.area())`. The box does not grow with its content and
the content is a ticket body, which is user-authored and unbounded.

Confirmed empirically on 2026-09-23 while fixing `zgfki6`, with a throwaway
probe test rendering a 40-line ticket body at 80x24. Only the first 11 or so
body lines rendered. The remainder was silently dropped - **no scroll
indicator, no truncation marker, nothing telling the user more content
exists**. The probe was removed rather than committed, since it was
diagnostic rather than a behaviour anyone wants pinned.

This is the same class of bug as `zgfki6`, but it does not have the same fix.

## Why zgfki6's approach does not transfer

`zgfki6` solved the help overlay by sizing the box to its content, because the
help text is a fixed list the code owns. A ticket body is arbitrary length -
this repo already has tickets whose bodies run past a hundred lines. Sizing to
fit is not available. This needs either real scrolling or explicit
truncation, and either way the user must be able to tell there is more.

## Behaviour

- A ticket body longer than the detail view can show is never silently cut.
- The user can either reach the rest of the content, or is clearly told it
  exists.

## Design decision to make deliberately

Pick one and say which, in the implementation notes:

- **Scrolling.** Needs a scroll offset on `App`, keybindings to move it
  (`j`/`k` are already taken by board navigation - decide what detail-view
  scrolling binds to), and a position indicator so the user knows where they
  are. `zgfki6` explicitly declined this as too much surface for a render-only
  fix; here it may be the right answer.
- **Truncate with an explicit marker**, e.g. a final line naming how many more
  lines exist and suggesting `e` to open the ticket in an editor. Cheaper, no
  new state, and arguably right given `e` already opens the full file.

Truncation is the smaller change and may be sufficient. Scrolling is the more
complete answer. Weigh them rather than defaulting.

## Reference

`zgfki6` introduced `centered_rect_fixed(width, height, r)` alongside the
percentage-based `centered_rect`, and split the help content into
`help_lines_single()` / `help_lines_compact()`. Read that code first - the
helpers may be reusable, and its notes record why scrolling was declined
there.

## Test first

- unit or snapshot - a body longer than the visible area is not silently cut
- unit or snapshot - the user-visible signal that more content exists is
  present
- whichever design is chosen, a test covering the boundary where content
  exactly fills the available height

## Implementation notes (2026-09-24)

Supervising session pre-decided the design: **truncate with an explicit
marker**, over real scrolling, specifically because it's render-only and adds
no new `App` state — keeping it disjoint from `7zl07z`'s concurrent work on
`app.rs`/`mod.rs`. Contributor implemented against that brief; commit
`0693726` (`src/tui/render.rs` only, no ticket files touched).

**Reproduction before the fix:** a throwaway `probe_area` test called
`centered_rect(80, 80, Rect{width:80,height:24})` directly, confirming the
detail box is 20 rows tall / 18 inner rows once the border is subtracted. For
a minimal fixture (no tags/parent/blocked_by) the header is 6 metadata lines

- 1 blank separator = 7, leaving exactly 11 rows for the body — matching the
  ticket's "~11 of 40 lines" report exactly. A RED test with a 40-line body
  confirmed the silent drop past line 11 on the unfixed code.

**Fix:** in `draw_detail`, the body is built into a separate `body_lines`
`Vec<Line>` with the header line count (`header_len`) captured first. When
`header_len + body_lines.len() > inner_height`, the body is truncated to
`inner_height - header_len - 1` lines (reserving one row for the marker) and
a `Line::styled` marker in `Color::DarkGray` is appended, matching the
existing footer-hint styling convention in the file.

**Marker format:** `"… {hidden} more line{s} — press e to open"`, e.g.
`… 29 more lines — press e to open`, singular for exactly one hidden line.

**Boundary:** truncation condition is strict `>` — a body landing exactly on
`inner_height` renders in full with no marker; one line over triggers it.
Verified with the 80x24/no-tags fixture: 11 body lines → no marker, 12 body
lines → marker present.

**`e` keybinding confirmed unaffected:** `src/tui/mod.rs` maps
`KeyCode::Char('e')` to `Message::OpenEditor` for both Board and Detail
screens, handled via `Cmd::OpenEditor` — untouched by this ticket, verified
by reading before writing the marker text.

**Tests added** (`src/tui/render.rs`, all test-first RED→GREEN except the
boundary-under case which passed both before and after by construction):

- `detail_view_long_body_is_not_silently_truncated`
- `detail_view_truncation_marker_states_accurate_hidden_count`
- `detail_view_body_exactly_filling_height_shows_no_marker` (boundary)
- `detail_view_body_one_line_over_height_shows_marker` (boundary+1)

**Judgement calls:**

- Truncation is computed against unwrapped `Line` count, not wrapped-row
  count (the `Paragraph` still applies `Wrap { trim: false }` for width
  overflow, unchanged). Mirrors the existing `help_lines_*` approach
  (also Line-count-based). For a very long single line this could
  under-truncate relative to actually-rendered rows — noted as an edge case,
  not fixed here; stays in scope for the vertical-overflow bug this ticket
  targets.
- Marker uses `Color::DarkGray`, matching the existing footer-hint styling
  in `draw_board` rather than inventing a new colour.

**Post-implementation note:** the fix commit briefly went in and back out of
`main` due to an unrelated worktree base-ref problem (the worktree branched
from a stale `origin/main`, dragging in unconnected ticket-file history on
first merge). That was caught, reverted, and the fix commit
(`0693726`) was cherry-picked cleanly onto current `main` with no changes to
its diff — `render.rs` only, verified via `git show --stat` before the
cherry-pick. Unrelated to the design/implementation above.

**Verification (post cherry-pick, on current `main`):** `cargo build`,
`cargo test` (245 passed / 0 failed), `cargo clippy --all-targets` clean,
`cargo fmt` clean.
