---
id: Av8rn7
title: auto-commit - git shell-out on new and edit
type: task
status: todo
tags: []
parent: null
blocked_by:
- gs79xq
created_at: 2026-05-01T22:12:09.484758Z
updated_at: 2026-05-01T22:49:17.208120Z
---

## What to build

Add git auto-commit support to `tickets new` and `tickets edit`, gated behind `config.git.auto_commit`. Lives in a new `src/git.rs` module.

## Implementation

**`src/git.rs`** owns two public functions:

- `git_detect(dir: &Path) -> Result<()>` — runs `git -C <dir> rev-parse --git-dir`; returns distinct errors for "not on PATH" vs "not a repo"
- `git_commit(dir: &Path, file: &Path, message: &str) -> Result<()>` — single call: `git -C <dir> commit -m "<message>" -- <file>`; git's stderr is piped straight through; no `--allow-empty`

**Wiring:** after the ticket file is written to disk, if `config.git.auto_commit == true`, call `git_detect` then `git_commit`. Errors are printed to stderr and exit 1.

## Acceptance criteria

- [ ] `src/git.rs` module with `git_detect` and `git_commit`
- [ ] `git_detect` returns a distinct error when `git` is not on PATH vs when the directory is not inside a git repo
- [ ] `git_commit` uses a single `git commit -m "<msg>" -- <file>` subprocess (no separate `git add`)
- [ ] Git's stderr is piped directly to the user's stderr — no wrapping or prefix
- [ ] If `git commit` fails (e.g. hook rejection, nothing to commit), exits 1 with git's own output
- [ ] Commit message format: `tickets: new <id> "<title>"` / `tickets: edit <id> "<title>"`
- [ ] Auto-commit runs as the final step after file write in `cmd_new` and `cmd_edit`
- [ ] Only triggered when `config.git.auto_commit == true`; silent no-op when false
- [ ] No `git2` or `libgit2` dependency

## Blocked by

gs79xq (config file — provides `Config` struct with `git.auto_commit`)
