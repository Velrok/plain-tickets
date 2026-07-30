---
id: 9293zo
title: Generate the config scaffold from the Config struct on init
type: task
status: todo
tags:
- config
parent: null
blocked_by: []
created_at: 2026-07-30T17:46:27.189071Z
updated_at: 2026-07-30T17:46:27.189071Z
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

## Consequences

- The generated file has no comments and no commented-out lines. Every option appears as a live key at its default. This is a deliberate trade: self-documenting and always complete, but loses the prose hints.
- `auto_commit = false` is now written explicitly rather than commented out. The existing git-detected hint at `src/commands.rs:41` still applies and should be kept.
- `to_string_pretty` expands arrays across multiple lines. Cosmetic, accept it.
- Consider dropping the redundant `impl Default for Config` and `impl Default for GitConfig` (`src/config.rs:51-61`) in favour of `#[derive(Default)]` while in here.

## Tests

- Assert the scaffold `init` writes parses back via `config::load` without error, and round-trips to an equal `Config`.
- Assert the scaffold contains a key for every section, so a newly added option that is skipped during serialisation fails the test.
