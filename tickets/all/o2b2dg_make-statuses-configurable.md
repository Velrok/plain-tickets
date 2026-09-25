---
id: o2b2dg
title: Make statuses configurable
type: epic
status: todo
tags:
- config
- status
parent: null
blocked_by: []
created_at: 2026-09-25T12:07:48.796956Z
updated_at: 2026-09-25T12:07:48.796956Z
---

## Goal

Stop hard-coding the workflow. Today `TicketStatus` is a closed enum
(`draft`, `todo`, `in-progress`, `review`, `done`, `rejected`) and adding a
status means a code change. Projects want their own workflow, so the open
statuses should come from `.tickets.toml`.

## Design

**Only two statuses stay dictated by the binary: `done` and `rejected`.**
They are terminal states and dependency resolution is defined in terms of
them, so they cannot be config-driven. They are implied at the end of the
list and must not be repeated in config.

Everything else is an ordered array:

```toml
statuses = ["scoping", "todo", "in-progress", "review"]
# done and rejected are implied at the end
```

The array is load-bearing in three ways:

1. **Order** — the order statuses render in, both as TUI kanban columns and
   wherever `list` groups or sorts by status.
2. **Default** — the first entry is the default status for `tickets new`.
3. **Membership** — anything not in the array (plus the two implied) is not
   a valid status.

### Icons

Optional map, prefilled on `init`. A status with no entry simply renders no
icon — absence is not an error.

```toml
[status_icons]
scoping     = "🔍"
todo        = "📋"
in-progress = "🚧"
review      = "👀"
done        = "✅"
rejected    = "🚫"
```

### Colours

Optional map, prefilled on `init`. A status with no entry renders plain.

```toml
[status_colours]
in-progress = "yellow"
review      = "magenta"
done        = "green"
rejected    = "bright-black"
```

## Why this is bigger than it looks

`TicketStatus` stops being a closed enum, and three separate safety
mechanisms currently lean on it being closed:

- **`clap::ValueEnum` is derived on it** (`src/domain_types.rs:50`). CLI
  validation of `--status` happens at parse time today. With a runtime set,
  validation has to move to after config load, and `--help` can no longer
  print a static `[possible values: ...]` list.
- **Exhaustive `match` with no wildcard** is deliberately used as a
  compile-time tripwire — see `TicketStatus::style()` (`src/domain_types.rs:69`),
  added by `3mqhe3` specifically so a new status *fails to compile* rather
  than silently rendering plain. That tripwire disappears. Something has to
  replace it, because "silently wrong on an unknown status" is the exact
  failure mode this project keeps hitting.
- **A test enumerates the variants structurally** —
  `every_configured_status_is_reachable_in_exactly_one_column`
  (`src/tui/app.rs:426`) iterates `TicketStatus::value_variants()` so it
  cannot rot. It was proven discriminating during `7zl07z` verification.
  It needs an equivalent that enumerates the *configured* set instead.

Blast radius by file (occurrences of `TicketStatus`): `src/tui/render.rs` 90,
`src/tui/app.rs` 65, `src/domain_types.rs` 25, `src/config.rs` 14,
`src/deps_graph.rs` 12, `src/commands.rs` 11, `src/application_types.rs` 4.

## Decisions to make before starting

These are genuine forks, not rhetorical. Settle them first.

1. **Does `kanban_columns` survive?** `TuiConfig::kanban_columns`
   (`src/config.rs:31`) already orders TUI columns, which is exactly what
   `statuses` now does. Options: delete it and derive columns from
   `statuses`; or keep it as an optional *subset filter* so the TUI can hide
   statuses the board still tracks. Note this repo's own `.tickets.toml`
   currently opts into a `review` column that the compiled-in default omits —
   whichever way this goes, that file needs migrating.

2. **Does `[new] default_status` survive?** It becomes redundant once the
   first entry of `statuses` is the default. Probably delete, but it is a
   breaking config change.

3. **What happens to a ticket whose status is not in the configured set?**
   This is the important one. `load_tickets` currently does
   `.filter_map(|raw| raw.parse::<Ticket>().ok())`, so a ticket it cannot
   parse **vanishes from the board with no indication** — the exact mechanism
   that produced bug `7zl07z`. Making statuses configurable turns a rare
   parse failure into an ordinary event: edit the config, and every ticket in
   a removed status disappears. This must not silently drop tickets. Decide
   the behaviour (refuse to load? surface a warning? render in a catch-all
   column?) before writing code.

4. **Does a `rejected` blocker satisfy a dependency?** Today the predicate is
   positive and `done`-only (`src/deps_graph.rs:52,66`), so a rejected
   blocker blocks forever. Now that `rejected` is being promoted to a
   dictated status, decide deliberately whether it should count as resolved.
   Keep the predicate positive either way — a negative or exhaustive form
   silently lets the wrong statuses through and the compiler cannot catch it.

## Constraints

- **Keep the satisfied predicate positive.** Never a negative chain, never an
  exhaustive match over statuses.
- **Icons are display-width hazards.** Emoji are double-width. `chphvf`
  established `pad_to_display_width` for exactly this and left a proven
  regression test (`list_widest_type_column_has_no_wasted_padding`). Status
  icons must pad in display columns, not bytes or chars.
- **Colour must never reach a pipe.** `3mqhe3` just landed this for the
  status column via `anstream` with `NO_COLOR`/`CLICOLOR_FORCE` handling.
  Config-driven colours must go through the same path, and pad *before*
  colouring — ANSI escapes are zero-display-width and will corrupt alignment
  otherwise.
- `init` must write the full default config out, statuses plus both maps, so
  the shape is discoverable without reading docs.

## Suggested slices

Not yet created as tickets — split once the decisions above are settled.

1. Config schema: parse `statuses`, validate (non-empty, no duplicates, must
   not contain `done`/`rejected`), append the two implied.
2. Replace the `TicketStatus` enum with a validated runtime type; move
   `--status` validation to post-config-load; decide the replacement for the
   lost compile-time tripwire.
3. Unknown-status handling (decision 3) — with a test proving nothing is
   silently dropped.
4. Ordering: TUI columns and `list` both driven by the configured order.
5. Default status for `new` = first entry; retire `[new] default_status`.
6. Optional icon map, display-width safe.
7. Optional colour map, routed through the existing `anstream` path.
8. `init` writes the prefilled defaults.

## Acceptance

- A project can define its own statuses in `.tickets.toml` and the CLI and
  TUI both respect them, in the configured order.
- `done` and `rejected` always exist, always last, and cannot be redefined.
- `tickets new` with no `--status` uses the first configured status.
- Icons and colours are both fully optional; a status missing from either map
  renders without one, and that is not an error.
- A fresh `tickets init` writes statuses and both maps out in full.
- **No ticket ever disappears from the board because of a status change in
  config.**
