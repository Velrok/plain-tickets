---
id: 1w544fdz3x7bq
title: Pretty list output
type: task
status: done
parent: 3a38v7oybcjd3
blocked_by:
- 3pb0v9gmgk5o0
tags:
- cli
created_at: 2026-10-09T00:18:31.000Z
updated_at: 2026-10-09T00:23:52.000Z
---

Header row, full 16-hex IDs, status/type colours, title truncated to terminal width. See design note on parent.

Architecture: separate presenters per format in src/cli/presenter.rs. The existing TicketCliLinePresenter and TicketCliDetailPresenter are the plain ones (rename to PlainLinePresenter and PlainDetailPresenter, behaviour unchanged). Add PrettyLinePresenter and PrettyDetailPresenter next to them. The handler picks the pair from output::current(format).format, and pretty presenters take Mode.colour. Remove the expect(dead_code) on Mode once it is read.

Done. PlainLinePresenter and PlainDetailPresenter renamed; PrettyListPresenter is a whole-table presenter (not per line), since column widths depend on every row. Colour pads outside the escape codes. Width comes from the terminal_size crate; None (piped) means no truncation. Verified on a pty with script(1).
