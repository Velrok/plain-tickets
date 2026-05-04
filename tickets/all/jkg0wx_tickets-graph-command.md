---
id: jkg0wx
title: tickets graph command
type: task
status: done
tags: []
parent: null
blocked_by:
- q5rxnv
- 63gjvr
created_at: 2026-05-03T22:50:35.713486Z
updated_at: 2026-05-04T08:45:33.989306Z
---

Wire up the `tickets graph [id]` clap subcommand. No ID → forest mode (roots = tickets with no active blockers). With ID → single tree rooted at that ticket. Prints to stdout.

## Acceptance criteria
- [ ] `tickets graph` prints full forest (all active tickets with no active blockers as roots)
- [ ] `tickets graph <id>` prints tree rooted at given ticket
- [ ] Hard error if tickets directory not initialised
- [ ] Documented in `tickets graph --help`