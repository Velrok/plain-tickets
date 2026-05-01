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

## `tickets list` — display contract

- Reads all `.md` files from `all/` only
- Prints one line per ticket: `id  status  type  title` (fixed-width padded columns)
- Silent (no output) when there are no tickets
- Hard error (`error: tickets directory not initialised — run \`tickets init\` first`, exit 1) when `all/` does not exist
