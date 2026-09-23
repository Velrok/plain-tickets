---
id: 4x7e81
title: TUI detail view silently truncates long ticket bodies
type: bug
status: todo
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-09-23T11:21:55.199665Z
updated_at: 2026-09-23T11:21:55.199665Z
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
