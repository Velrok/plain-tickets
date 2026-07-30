---
id: iu5vy7
title: show should not force bat color always
type: bug
status: todo
tags:
- cli
parent: null
blocked_by: []
created_at: 2026-07-30T17:48:32.422700Z
updated_at: 2026-07-30T17:48:32.422700Z
---

## Problem

`print_body` (`src/commands.rs:189-208`) shells out to `bat` with `--color=always`:

```rust
.args(["--language=md", "--style=plain", "--color=always", "-"])
```

This forces ANSI escape codes into the output unconditionally - even when stdout is piped or `NO_COLOR` is set. `tickets show <id> | rg foo` or `tickets show <id> > file.md` gets escape codes mixed into the text, which breaks parsing/grepping and pollutes saved files.

## Fix

Use `--color=auto` (bat's own tty-detection default) instead of `always`. Let `bat` decide based on whether stdout is a terminal, and respect `NO_COLOR`/`--color` env conventions as it already does internally.

## Scope

Just the one flag in `print_body`. Not bundled with the `edit --title` rename ticket - different concern, different call path (`cmd_show` vs `cmd_edit`).

## Test

Hard to unit test subprocess color output directly; at minimum confirm existing tests still pass and manually verify `tickets show <id> | cat` no longer contains ESC sequences.
