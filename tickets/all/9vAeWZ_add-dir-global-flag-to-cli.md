---
id: 9vAeWZ
title: Add --dir global flag to CLI
type: task
status: done
tags:
- cli
parent: null
blocked_by: []
created_at: 2026-05-01T21:57:05.142012Z
updated_at: 2026-05-01T22:00:31.869957Z
---

## What to build

Add a `--dir` global flag to the `tickets` CLI that overrides the base directory for all subcommands.

End-to-end: flag is declared on `Cli`, passed through `main()`, and replaces the `TICKETS_DIR`-only `resolve_dir()` logic. All subcommands (`init`, `new`, `edit`) respect it.

## Acceptance criteria

- [ ] `tickets --dir <path> new --title "foo"` creates a ticket under `<path>/all/`
- [ ] `tickets --dir <path> init` creates `<path>/all/` and `<path>/archived/`
- [ ] `tickets --dir <path> edit <id> --title "bar"` updates the ticket under `<path>/all/`
- [ ] Precedence: `--dir` > `TICKETS_DIR` > `./tickets/`
- [ ] Tests cover the precedence logic

## Blocked by

None — can start immediately