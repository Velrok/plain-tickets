---
id: 3o1db2mi9al9i
title: Pretty show output
type: task
status: done
parent: 3a38v7oybcjd3
blocked_by:
- 3pb0v9gmgk5o0
tags:
- cli
created_at: 2026-10-09T00:18:31.000Z
updated_at: 2026-10-09T00:34:45.000Z
---

Bold title, meta line, resolved parent and blockers, dates, rule, raw body. See design note on parent.

Architecture: separate presenters per format in src/cli/presenter.rs. The existing TicketCliLinePresenter and TicketCliDetailPresenter are the plain ones (rename to PlainLinePresenter and PlainDetailPresenter, behaviour unchanged). Add PrettyLinePresenter and PrettyDetailPresenter next to them. The handler picks the pair from output::current(format).format, and pretty presenters take Mode.colour. Remove the expect(dead_code) on Mode once it is read.

Done. PrettyDetailPresenter in src/cli/presenter.rs, fed pre-resolved Relation values by the handler (archived tickets resolve; unreadable ones show '(not found)'). Parts a ticket lacks are omitted; the rule appears only with a body. Colours live in the new src/cli/style.rs (Style::status, type, bold) and are shared with the pretty list, so a status looks the same in both.
