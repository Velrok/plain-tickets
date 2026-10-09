---
id: 99e4eaf54edecba8
title: Add icons to the pretty list output
type: task
status: todo
parent: null
blocked_by: []
tags:
- cli
created_at: 1791506747
updated_at: 1791506747
---

Reuse Style::status_with_icon and type_with_icon from src/cli/style.rs in PrettyListPresenter. Icons go inside the STATUS and TYPE columns. Emoji are two terminal columns wide, so the column width and padding must count display width, not chars: pad by the width of the raw text plus 2 when an icon is present, and keep the truncation budget in step. Header cells stay icon-free. Unknown statuses and types get no icon but still align. Icons stay on under NO_COLOR. Plain output unchanged. Needs tests for alignment with mixed icon and no-icon rows.