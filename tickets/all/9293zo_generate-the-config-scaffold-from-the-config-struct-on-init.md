---
id: 9293zo
title: Generate the config scaffold from the Config struct on init
type: task
status: done
tags:
- config
parent: null
blocked_by: []
created_at: 2026-07-30T17:46:27.189071Z
updated_at: 2026-07-30T18:16:19.644664Z
---

## Problem

`cmd_init` writes a hand-maintained string literal (`src/commands.rs:24-34`). It has already drifted from the code:

- Header reads `Uncomment and set values to override defaults`, but the checked-in `tickets/.tickets.toml` has nothing commented out.
- It advertises `[git]` and `[new]` but never mentions `[tui] kanban_columns`, so that option is undiscoverable.

Any option added to `Config` has to be remembered in a second place. It will drift again.

## Approach

Make the `Config` struct the single source of truth. Derive `Serialize` alongside the existing `Deserialize`, and have `init` write `toml::to_string_pretty(&Config::default())`. The scaffold then lists every option the binary actually supports, at its real default, for free.

### Prerequisite - Option fields must gain concrete defaults

Verified against toml 0.8: `to_string_pretty` does not error on `Option::None`, it silently omits the key. Serialising the current `Config::default()` yields a bare `[new]` header with both fields missing - the opposite of the goal.

So `NewConfig` (`src/config.rs:21`) must change:

```rust
#[serde(default)]
pub default_status: TicketStatus,   // was Option<TicketStatus>, defaults to Draft
#[serde(default)]
pub default_type: TicketType,       // was Option<TicketType>, defaults to Task
```

`TicketStatus` already derives `Default` with `#[default] Draft`. Check `TicketType` does the same. Then update the `unwrap_or` call sites in `cmd_new`.

## Steer - do not rewrite the file on every load

The round-trip should be scoped to `init`, not to `config::load`. Rewriting a parsed config back to disk on every command would:

- destroy any comments the user added, since the toml serialiser emits none
- write to disk on every invocation, including read-only ones like `list` and `show`
- with `auto_commit = true`, produce a commit for each of those writes

If backfilling an existing config is wanted later, it should be an explicit opt-in command, not a side effect of loading. Out of scope here.

**Update (2026-07-30):** the explicit opt-in is now in scope for this ticket — see `--force` below.

## Approach - `--force` backfill flag on init

`cmd_init` currently bails with `already initialised` if `.tickets.toml` exists. Add a `force: bool` field (inline on the `Commands::Init` variant, not a new `InitArgs` struct — matches the `Graph { id: Option<TicketId> }` single-field precedent):

```rust
Init { #[arg(long)] force: bool },
```

`cmd_init(base: PathBuf, force: bool)`:

- No existing `.tickets.toml`: behave exactly as today (force is a no-op), print `  created {path}`.
- Existing `.tickets.toml` and `force == false`: bail as today.
- Existing `.tickets.toml` and `force == true`: `config::load(&base)` the existing file (preserving the user's actual values — this is a normalise/backfill round-trip, not a reset to `Config::default()`), then `toml::to_string_pretty(&loaded_cfg)` and overwrite. Print `  rewrote {path}` instead of `created` so the overwrite is visible.
- If `config::load` fails (e.g. `deny_unknown_fields` rejects a stale/typo'd key), propagate the error and abort — do not write, do not fall back to defaults. The original file is left untouched since the write only happens after a successful load.

`init` still never calls `git::git_commit`, even under `--force` with `auto_commit = true` in the loaded config — consistent with `init` never having participated in the auto-commit flow for any other reason.

## Consequences

- The generated file has no comments and no commented-out lines. Every option appears as a live key at its default. This is a deliberate trade: self-documenting and always complete, but loses the prose hints.
- `auto_commit = false` is now written explicitly rather than commented out. The existing git-detected hint at `src/commands.rs:41` still applies and should be kept.
- `to_string_pretty` expands arrays across multiple lines. Cosmetic, accept it.
- Consider dropping the redundant `impl Default for Config` and `impl Default for GitConfig` (`src/config.rs:51-61`) in favour of `#[derive(Default)]` while in here.

## Tests

- Assert the scaffold `init` writes parses back via `config::load` without error, and round-trips to an equal `Config`.
- Assert the scaffold contains a key for every section, so a newly added option that is skipped during serialisation fails the test.

## Decisions (grill-me, 2026-07-30)

- `cmd_new`'s `.unwrap_or(TicketStatus::Draft)` / `.unwrap_or(TicketType::Task)` fallbacks become dead once `NewConfig` fields are non-`Option` — drop them, call sites become `args.status.unwrap_or(cfg.new.default_status)` etc.
- Drop `impl Default for Config` and `impl Default for GitConfig`, replace with `#[derive(Default)]`. `TuiConfig` keeps its manual impl (needs `default_kanban_columns()`).
- No header comment (`# plain-tickets configuration`) above the generated TOML — write only `toml::to_string_pretty(&Config::default())`'s output.
- Add `#[derive(PartialEq, Debug)]` to `Config`, `GitConfig`, `TuiConfig`, `NewConfig` so the round-trip test can `assert_eq!(loaded, Config::default())`.
- New tests live in a `mod tests` in `commands.rs` (none exists yet — follow the `tmp_dir` pattern from `config.rs`/`git.rs`), calling the real `cmd_init` against a temp dir and reading `.tickets.toml` back. Only assert file content — not stdout or directory creation.

## Decisions (grill-me, `--force`, 2026-07-30)

- `--force` semantics: load-then-reserialise the *existing* config (preserves user values like `auto_commit = true`), not a reset to `Config::default()`. This is the backfill case the original Steer section deferred.
- Unparseable existing config under `--force`: bail with `config::load`'s error, never silently fall back to defaults.
- Flag: `--force` only, no `-f` short alias — matches every other flag in this CLI (`--all-rejected`, `--clear-parent`, etc.), all of which are long-only.
- Shape: inline `Init { force: bool }` on the `Commands::Init` variant, not a new `InitArgs` struct — matches the `Graph { id: Option<TicketId> }` precedent for single-field commands.
- `init` still never calls `git::git_commit`, regardless of `--force` or `auto_commit`.
- `--force` with no existing file: behaves exactly like plain `init`, prints `created`. `--force` overwriting an existing file: prints `rewrote` instead, so the overwrite is visible in output.
- Test coverage for `--force` lives at the CLI level in `tests/cli_init.rs` (spawns the real binary), not just `commands.rs` unit tests — matches that file's existing style and behaviors are user-facing.
