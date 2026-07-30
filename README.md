# plain-tickets

A plain-text, markdown-based ticket system for solo developers. Tickets are `.md` files with YAML front matter — readable by humans and AI alike.

## Philosophy

Plain text as the data layer. No database, no lock-in. Git for versioning. A TUI for human ergonomics; a CLI for scripting and AI access.

## TUI

Running `tickets` with no subcommand launches an interactive kanban board (via [ratatui](https://github.com/ratatui-org/ratatui)). Columns default to `todo` / `in-progress` / `done`, configurable via `[tui] kanban_columns` in `.tickets.toml`.

## Installation

```bash
cargo build --release
# Binary: target/release/tickets
```

## Quick Start

```bash
# Initialise a tickets directory
tickets init

# Create a ticket
tickets new --title "Fix login bug" --type bug --tag auth

# List all tickets
tickets list

# Show a single ticket
tickets show <id>

# Edit a ticket
tickets edit <id> --status in-progress

# Archive done tickets
tickets archive <id>

# View the dependency graph
tickets graph
```

## Data Format

Each ticket is a `.md` file:

```
tickets/
├── all/        # active tickets
└── archived/   # archived tickets
```

Filename: `<6-char-id>_<slugified-title>.md` — e.g. `a3f9c1_fix-login-bug.md`

### Front Matter

```yaml
---
id: a3f9c1
title: Fix login bug
type: task
status: draft
tags: []
parent: null
blocked_by: []
created_at: 2026-04-30T19:00:00Z
updated_at: 2026-04-30T19:00:00Z
---

Ticket body in markdown.
```

Only `title` is required. All other fields have defaults.

### Types

`epic` / `story` / `task` / `bug`

### Statuses

`draft` / `todo` / `in-progress` / `done` / `rejected`

### Title Validation

Titles must be 120 characters or fewer and may only contain letters, numbers, spaces, `_`, `-`, and `.`.

## CLI Reference

### `tickets init`

```
tickets init [--force]
```

Scaffolds `tickets/all/`, `tickets/archived/`, and `.tickets.toml`.

- `--force` overwrites an existing `.tickets.toml`, backfilling any missing keys (existing values are preserved).

### `tickets new`

```
tickets new --title "..." [--type <type>] [--status <status>] [--tag <tag>]...
            [--parent <id>] [--blocked-by <id>]... [--body -]
```

- `--body -` reads the ticket body from STDIN.
- `--tag` is repeatable.

### `tickets edit <id>`

```
tickets edit <id> [--title "..."] [--type <type>] [--status <status>]
             [--tag <tag>]... [--parent <id>] [--clear-parent]
             [--blocked-by <id>]... [--clear-blocked-by] [--body -]
```

Only fields explicitly passed are updated. `updated_at` is bumped automatically.

### `tickets list`

```
tickets list [--status <status>]... [--type <type>]... [--tag <tag>]...
```

Prints all tickets in `tickets/all/`, sorted by status then creation date.

- `--status` / `--type` are repeatable with OR semantics (match any given value).
- `--tag` is repeatable with AND semantics (ticket must have all given tags).

### `tickets show <id>`

Pretty-prints a single ticket with emojis, human-readable timestamps, and optional body rendering.

- Empty/null fields (`tags`, `parent`, `blocked_by`) are omitted.
- Timestamps shown as `YYYY-MM-DD · N days ago`.
- Body is rendered via `bat --language=md` if available, otherwise printed raw.

### `tickets archive`

```
tickets archive <id>...
tickets archive --all-rejected
```

Moves tickets to `tickets/archived/`. Pass one or more IDs, or `--all-rejected` to bulk-archive all rejected tickets.

### `tickets graph [id]`

Prints the dependency graph derived from `blocked_by` relationships.

- No ID: renders the full forest (all root tickets and their trees).
- With ID: renders the tree rooted at that ticket.
- Warns on stderr if a dependency cycle is detected.

## Global Flags

| Flag | Description |
|------|-------------|
| `--dir <path>` | Override the tickets directory (takes precedence over `TICKETS_DIR`) |
| `--version`, `-V` | Print the CLI version and build SHA |

## Environment

| Variable | Description |
|----------|-------------|
| `TICKETS_DIR` | Path to the tickets directory (default: `./tickets/`) |

## Tech Stack

- [Rust](https://www.rust-lang.org/)
- [clap](https://github.com/clap-rs/clap) — CLI argument parsing
- [serde](https://serde.rs/) + [serde_yaml](https://github.com/dtolnay/serde-yaml) — YAML front matter
- [nanoid](https://github.com/nikolaigirgin/nanoid.rs) — ticket ID generation
- [chrono](https://github.com/chronotope/chrono) — timestamps
- [ratatui](https://github.com/ratatui-org/ratatui) + [crossterm](https://github.com/crossterm-rs/crossterm) — TUI

## Configuration

Create `tickets/.tickets.toml` (via `tickets init`) to configure per-repo behaviour:

```toml
[git]
auto_commit = true   # stage and commit the ticket file after new/edit/archive

[new]
default_type = "task"     # default --type for `tickets new`
default_status = "draft"  # default --status for `tickets new`

[tui]
kanban_columns = ["todo", "in-progress", "done"]  # columns shown in the TUI board
```

## Planned

- `tickets search <query>` — fuzzy search across title, tags, and type
