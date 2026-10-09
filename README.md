# plain-tickets

Markdown tickets with YAML front matter, one file per ticket.

- Config: `.tickets/config.toml` (custom statuses; `done` and `rejected` are built in)
- Tickets: `tickets/all/<id>.md`
- IDs: 13 lowercase base36 characters (a random u64), always passed in full
- Timestamps: `created_at` and `updated_at` are RFC 3339 UTC with milliseconds, e.g. `2026-04-30T19:00:00.123Z`

## Commands

- [x] `tickets init` — create `.tickets/config.toml` with the default statuses
- [x] `tickets new <title>` — create a ticket and print its ID
  - `-t, --type <type>` (optional)
  - `-s, --status <status>` (default: first configured status)
  - `-g, --tag <tag>` (repeatable)
  - `-p, --parent <id>` (must exist)
  - `-b, --blocked-by <id>` (repeatable, must exist)
  - `-m, --message <text>` body
- [x] `tickets show <id>` — print front matter and body (active or archived; the stored file text)
- [x] `tickets list` — one line per ticket: `<id>\t<status>\t<type>\t<title>`
  - [x] `--format plain|pretty` (also on `show`): pretty by default on a TTY, plain when piped; `NO_COLOR` drops colour. Pretty `list` is an aligned table; pretty `show` is a header block (title, ids, parent and blockers, dates) then the raw body
  - [x] `-s, --status <status>` (repeatable, matches any)
  - [x] `-g, --tag <tag>` (repeatable, matches any)
  - [x] `-t, --type <type>`
  - [x] `-p, --parent <id>`
  - [x] `--blocked` — has an unfinished blocker
  - [x] `--ready` — open, and every blocker is done or rejected
  - [x] `--archived` — archived tickets only, instead of active ones
- [x] `tickets edit <id>` — open in `$VISUAL` / `$EDITOR`; `id` and `created_at` are immutable
- [x] `tickets set <id>` — `--title`, `--status`, `--type`, `--parent`, `--clear-type`, `--clear-parent`
- [x] `tickets tag <id> <tag>...` — add tags
- [x] `tickets untag <id> <tag>...` — remove tags
- [x] `tickets block <id> <blocker>...` — add blockers
- [x] `tickets unblock <id> <blocker>...` — remove blockers
- [x] `tickets note <id> <text>` — append text to the body
- [x] `tickets archive <id>...` — move done or rejected tickets to `tickets/archived/`; all IDs are checked first
  - [x] `--all-rejected` — archive every rejected active ticket (conflicts with IDs)
- [x] `tickets unarchive <id>` — restore an archived ticket
