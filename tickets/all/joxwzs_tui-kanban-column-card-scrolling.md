---
id: joxwzs
title: TUI kanban column card scrolling
type: task
status: done
tags:
- tui
parent: zf1ixl
blocked_by: []
created_at: 2026-05-03T23:50:49.167689Z
updated_at: 2026-05-04T08:45:34.597185Z
---

When a column contains more cards than fit vertically, the user cannot see or navigate to cards below the fold. Add scroll support to kanban columns.

## Context

Card rendering was switched to manual Block-per-card layout (ticket zf1ixl). Overflow is currently clipped silently. This ticket adds a scroll offset so all cards are reachable.

## Acceptance criteria

- [ ] Each column tracks a scroll offset (first visible card index)
- [ ] j/k navigation scrolls the offset when the focused card would go out of view
- [ ] Cards above the scroll offset are not rendered
- [ ] A scroll indicator is shown when content is clipped (e.g. border title suffix '↓' or a scrollbar)