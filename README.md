# plain-tickets

A plain-text, markdown-based ticket system for solo developers. Tickets are `.md` files with YAML front matter — readable by humans and AI alike.

## Philosophy

Plain text as the data layer. No database, no lock-in. Git for versioning. A TUI for human ergonomics; a CLI for scripting and AI access.

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

Scaffolds `tickets/all/` and `tickets/archived/`.

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

Prints all tickets in `tickets/all/`, sorted by status then creation date.

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

## Global Flags

| Flag | Description |
|------|-------------|
| `--dir <path>` | Override the tickets directory (takes precedence over `TICKETS_DIR`) |

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

## Configuration

Create `tickets/.tickets.toml` (via `tickets init`) to configure per-repo behaviour:

```toml
[git]
auto_commit = true   # stage and commit the ticket file after new/edit/archive
```

## Planned

- `tickets search <query>` — fuzzy search across title, tags, and type
- TUI — interactive interface via [ratatui](https://github.com/ratatui-org/ratatui)
