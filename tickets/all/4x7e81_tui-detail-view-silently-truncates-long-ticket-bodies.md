---
id: 4x7e81
title: TUI detail view silently truncates long ticket bodies
type: bug
status: done
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-09-23T11:21:55.199665Z
updated_at: 2026-09-25T12:05:08.588836Z
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

## Rework (2026-09-25), commit `5ee8c05` — independently verified

The 2026-09-24 fix **failed independent verification**. It budgeted on
unwrapped `Line` count, but the detail box is ~62 columns inside its border
at 80 cols while this repo's ticket bodies wrap at ~75 chars. Most body lines
therefore cost two rows: kept lines overran the box and pushed the marker
itself off the bottom — a silent cut on this very ticket at the default
80x24 — and bodies of a few very long lines never triggered truncation at
all. The original notes called this "a very long single line could
under-truncate"; that understated it badly.

**Fix:** budget in *rendered rows* via `Paragraph::line_count`, which needs
ratatui's `unstable-rendered-line-info` feature (enabled in `Cargo.toml`).
The marker's own row cost is measured rather than assumed to be one, so it
can no longer be pushed off-screen. Five tests added, four RED-first, all
asserting the marker's presence in the rendered `TestBackend` buffer.

**Why not hand-roll the measurement:** `Wrap { trim: false }` breaks on word
boundaries, so `ceil(display_width / inner_width)` is a *lower* bound.
Verification demonstrated this empirically rather than by argument: a line of
25x `日本語テスト` has display width 303, which naive division puts at 5 rows
at width 62, but it actually renders as 6 — the CJK run is one unsplittable
word. Hand-rolling would have under-budgeted by a row and walked straight
back into the original bug.

**Unstable-feature risk, assessed:** removal would be a hard compile error
(verified experimentally — reverting the feature while keeping the call
yields `error[E0624]: method 'line_count' is private`). A future ratatui
could in principle change `line_count`'s semantics while keeping its
signature, which would be silent; the mitigation is that all five new tests
assert against the rendered buffer, so such a change surfaces on upgrade.
`ratatui = "0.29"` pins to 0.29.x, so no drive-by upgrade.

The singular `"1 more line"` branch was dead under the old line-count budget
(minimum hidden was always 2). Under row budgeting it is genuinely reachable
— a body ending in a line that wraps onto two rows — and is pinned by a test.

**Verification evidence** (real binary under tmux, counts hand-checked):
Repro A, this ticket's own body at 80x24, now shows
`… 123 more lines — press e to open` on the last row inside the border
(129 body lines − 6 shown = 123; header 7 + body 10 + marker 1 = 18 =
inner height). Repro B fixed. Also probed and correct: CJK and emoji bodies
(borders intact), mixed wrapped/unwrapped, a single line wrapping 10 rows,
long titles whose headers wrap and correctly shrink the body budget, exact
boundary and boundary+1, and narrow terminals down to 28 cols where the
marker wraps to two rows and stays inside the border. No panic at any size
down to 2x2. Prior good behaviour unregressed (11 → no marker, 12 → 2 more,
40 → 30 more; `e` still opens the editor). `cargo test` 250 passed / 0
failed at `5ee8c05`.

## Known limitation — deliberate, not an oversight

**Terminals of roughly 10 rows or fewer at 80 columns cut the body with no
indication.** When the header alone fills the box, no row remains for the
marker and it is clipped. Boundary measured: 80x11 shows
`… 41 more lines — press e to open`; 80x10 shows the six header lines and
drops a 40-line body entirely, silently.

This violates the ticket's own acceptance criterion, and is accepted anyway
— the same call the help overlay (`zgfki6`) made for its sub-13-row tier.
A 10-row terminal is reachable (a horizontal tmux split), so this is a real
if narrow gap, not an impossible one. Decision taken deliberately by the
user on 2026-09-25 rather than spending another rework round, and recorded
in a comment at the truncation site in `src/tui/render.rs` so a future
reader does not mistake it for a bug. If it is ever revisited, the fix is
the same shape as the one above: when `header_rows + marker_rows >
inner_height`, let the marker win — truncate the header, or fall back to a
shorter marker.
