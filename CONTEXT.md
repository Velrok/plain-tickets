# Context: plain-tickets

A plain-text, markdown-based ticket system for solo developers.

## Glossary

### Ticket
A unit of work tracked as a `.md` file with YAML front matter. Lives in `<tickets-dir>/all/` while active, `<tickets-dir>/archived/` when archived.

### Tickets directory
The root directory for all ticket data. Resolved from `--dir` flag → `TICKETS_DIR` env var → `./tickets/`. Contains `all/` and `archived/` subdirectories.

### Active tickets
Tickets in `<tickets-dir>/all/`. These are the working set — what `tickets list` shows.

### Archived tickets
Tickets in `<tickets-dir>/archived/`. Out of the active flow; not shown by default.

### Initialised
A tickets directory is initialised when both `all/` and `archived/` subdirectories exist. Commands that operate on tickets hard-error if the directory is not initialised.

## TUI

The interactive terminal UI, launched by running `tickets` with no subcommand.

### Kanban board (default screen)

- Columns represent ticket statuses; default columns: `todo`, `in-progress`, `done`
- Configurable via `[tui] kanban_columns` in `.tickets.toml`
- One card per ticket showing `id  title`

### Keybindings

| Key | Action |
|---|---|
| `h`/`←` | Focus left column |
| `l`/`→` | Focus right column |
| `j`/`↓` | Focus ticket below |
| `k`/`↑` | Focus ticket above |
| `Enter`/`Space` | Open detail view |
| `e` | Open focused ticket in `$EDITOR` |
| `n` | New ticket (status = current column); open in `$EDITOR` |
| `H` | Move focused ticket to left column |
| `L` | Move focused ticket to right column |
| `F1`/`?` | Keybinding help overlay |
| `q` | Quit |

### Detail view

Full-screen read-only view of a single ticket (front matter fields + body). `e` opens in `$EDITOR`. `q`/`Escape` returns to board.

### Implementation

Built with `ratatui` + `crossterm` backend.

## `tickets archive` — behaviour contract

- Accepts either a list of one or more ticket IDs **or** `--all-rejected` (mutually exclusive; combining both is a hard error)
- Moves matching ticket files from `all/` to `archived/` (filesystem move; no status field mutation)
- `--all-rejected` scans `all/` only — tickets already in `archived/` are not touched
- **Validation before any move:** all IDs are checked upfront; if any ID is missing or already archived, the command prints every failing ID and exits 1 with an explicit message that no files were moved
- Single-ID failure: hard error regardless of whether the ticket is missing entirely or already in `archived/`

## `tickets graph` — behaviour contract

- Usage: `tickets graph [id]`
  - No ID → full dependency forest of all active tickets
  - With ID → tree rooted at that ticket
- **Direction:** blocker-down — root nodes are tickets that unblock others; children are tickets they unblock
- **Roots (forest mode):** tickets with no active blockers — i.e. `blocked_by` is empty, or all referenced blockers are archived
- **Node format:** `id  status  title` (matches `list` columns minus type)
- **Cycle detection:** render up to the repeated node, then print `[cycle: <id>]` — no hard error
- **Missing references:** render as `[missing: <id>]` — no hard error
- **Archived references:** resolved from `archived/` and rendered with an `[archived]` label; distinct from active nodes
- **Source:** always reads `all/`; additionally reads `archived/` only to resolve `blocked_by` references

## `tickets list` — display contract

- Reads all `.md` files from `all/` only (never `archived/`)
- Prints one line per ticket: `id  status  type  title` (fixed-width padded columns)
- Silent (no output) when there are no tickets (after filtering)
- Hard error (`error: tickets directory not initialised — run \`tickets init\` first`, exit 1) when `all/` does not exist
- **Filters** (all optional, additive — no flags = show all):
  - `--status <STATUS>` repeatable — OR semantics (ticket matches if its status equals any given value)
  - `--type <TYPE>` repeatable — OR semantics
  - `--tag <TAG>` repeatable — AND semantics (ticket must carry every specified tag)
