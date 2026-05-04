---
id: zf1ixl
title: TUI kanban cards render as bordered boxes
type: task
status: done
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-05-03T23:50:41.284959Z
updated_at: 2026-05-04T00:07:48.529554Z
---

Replace the per-column List widget with manual Block rendering so each ticket card gets a proper border.

## Layout

- Each card is a Block with Borders::ALL
- Block title (top): type icon + ticket id, e.g. '📋 abc123'
- Block content: ticket title text, wrapped (variable card height)
- Block footer (bottom): coloured #tag spans right-aligned (omitted if no tags)
- Card height: driven by wrapped title, minimum 3 lines
- Overflow: clip at column bottom — cards that don't fit are not rendered

## Selection highlight

- Focused card gets a yellow border (consistent with focused-column style)
- No other highlight

## Acceptance criteria

- [ ] Column rendering replaced: List widget removed, cards rendered manually as Blocks
- [ ] Card top border label = type_span icon + ticket id
- [ ] Card content = title text, wrapped to card width
- [ ] Card footer = #tag spans using tag_color(), right-aligned; absent when no tags
- [ ] Focused card has yellow border; unfocused cards have default border
- [ ] Cards that overflow the column area are clipped silently
- [ ] Existing snapshot tests updated