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

# Edit a ticket
tickets edit <id> --status in-progress
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

## Planned

- `tickets list [--status] [--type] [--tag]` — filtered ticket listing
- `tickets show <id>` — print a single ticket
- `tickets search <query>` — fuzzy search across title, tags, and type
- `tickets archive <id>` — move a ticket to `archived/`
- `--dir <path>` global flag — override tickets directory at runtime
- `tickets/.tickets.toml` config file — per-repo configuration
- Git auto-commit — stage and commit the ticket file after `new` / `edit`
- TUI — interactive interface via [ratatui](https://github.com/ratatui-org/ratatui)
