---
id: cbec235b92fb3c3c
title: Add icons to the pretty show output
type: task
status: done
parent: null
blocked_by: []
tags:
- cli
created_at: 2026-10-09T00:43:18.000Z
updated_at: 2026-10-09T00:44:02.000Z
---

Design A from the chat. Status icons: todo white circle, in progress yellow circle, done tick, rejected cross. Type icons: bug, feature, task. Line icons: folder for Parent, no-entry for Blocked by, calendar for dates. Single-codepoint emoji only (no variation selectors) so widths stay predictable. Icons are independent of NO_COLOR. Shared with Style so any future presenter can reuse them. Plain output unchanged.

Done. Icons live in src/cli/style.rs: Style::status_with_icon and type_with_icon (plus PARENT_ICON, BLOCKED_ICON, DATES_ICON consts), so the pretty list can reuse them next. Icons ignore NO_COLOR. Unknown statuses and types get no icon.
