---
id: gs79xq
title: config file - scaffold parse and load .tickets.toml
type: task
status: done
tags: []
parent: null
blocked_by: []
created_at: 2026-05-01T22:12:03.712872Z
updated_at: 2026-05-01T22:55:53.639600Z
---

## What to build

Introduce `tickets/.tickets.toml` config file support. Define the `Config` struct, load it at startup, and pass it to all commands.

## Implementation

**`src/config.rs`** — new module:
- `Config { git: GitConfig }` and `GitConfig { auto_commit: bool }` structs
- Default: `auto_commit = false`
- `load(dir: &Path) -> Result<Config>` — reads `<dir>/.tickets.toml`; missing file returns defaults; invalid TOML is a hard error; unknown fields are rejected (`deny_unknown_fields`)

**`main.rs`** — after `resolve_dir`, call `config::load(&dir)`. Pass `config` as second argument to every command function.

**`cmd_init`** — always creates `<dir>/.tickets.toml` with defaults commented out. After creation, calls `git_detect(&dir)`: on `Ok`, prints the hint; on `Err`, silently skips. No hard error.

**Cargo.toml** — add `toml` and `serde` dependencies (with `derive` feature).

## Acceptance criteria

- [ ] `src/config.rs` with `Config`, `GitConfig` structs and `load(dir) -> Result<Config>`
- [ ] Missing `.tickets.toml` returns default config (`auto_commit = false`) — no error
- [ ] Invalid TOML is a hard error (printed to stderr, exit 1)
- [ ] Unknown fields are rejected (`deny_unknown_fields`) — hard error
- [ ] `tickets init` creates `.tickets.toml` with all defaults commented out
- [ ] `tickets init` prints git hint when `git_detect` succeeds; silently skips when it fails
- [ ] Config loaded once in `main` after `resolve_dir`, passed to all commands
- [ ] No special-casing for `tickets init` — missing file naturally yields defaults

## References

- `docs/prd-config-file.md`
- `docs/adr/0001-git-shell-out-over-git2.md`
