---
id: dswtzd
title: Scroll offset lags by one card when navigating down
type: bug
status: done
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-05-04T08:49:15.895997Z
updated_at: 2026-05-14T00:15:39.628684Z
---

When navigating down (j) in a column with overflow, the scroll offset updates too late. The first card that goes out of view remains invisible until focus reaches the last card in the list.

## Steps to reproduce
1. Have a column with more cards than fit vertically
2. Navigate down with j
3. The card just above the focused card stays scrolled off one step too long

## Expected
As soon as j moves focus past the visible window, the window scrolls so the focused card is always visible.

## Actual
The focused card disappears from view for one step before the scroll catches up.