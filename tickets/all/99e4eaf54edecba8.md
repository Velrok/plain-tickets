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
updated_at: 1791506812
---

Reuse Style::status_with_icon and type_with_icon from src/cli/style.rs in PrettyListPresenter. Icons go inside the STATUS and TYPE columns. Emoji are two terminal columns wide, so the column width and padding must count display width, not chars: pad by the width of the raw text plus 2 when an icon is present, and keep the truncation budget in step. Header cells stay icon-free. Unknown statuses and types get no icon but still align. Icons stay on under NO_COLOR. Plain output unchanged. Needs tests for alignment with mixed icon and no-icon rows.

Preview (widths in terminal columns: ID 16, STATUS 14, TYPE 7; each emoji counts as 2):

```
ID                STATUS          TYPE     TITLE
4e480da792336de5  ⚪ todo         🐛 bug   List order is arbitrary for tickets created in the…
99e4eaf54edecba8  🟡 in progress  🧩 task  Add icons to the pretty list output
7c7833977c268256  ✅ done                  Pretty list output
```

- Header cells have no icons.
- The no-type row is padded with spaces, not an icon.
- Padding is computed from the raw text width plus 2 when an icon is present.
- A custom status such as "blocked" gets no icon and pads like any other text.

Also: make the header row bold (Style::bold, so it follows colour and NO_COLOR). Pad each header cell by its raw width, then bold only the text, as the other cells do. Add a test that the header is bold with colour on and unchanged with it off.
