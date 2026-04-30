Plain markdown files + YAML front matter as the data layer for a Trello-style ticket system.

## Design

- Each ticket = `.md` file with YAML front matter
- Front matter fields: `id`, `parent_id`, `type` (epic/story/task/bug), `tags`, `created_at`, `updated_at`
- Ticket body = markdown content

## Interfaces

- **TUI** — human ergonomics (Trello-like board view)
- **CRUD CLI** — scripting & AI accessibility

## Tech

- Written in **Rust**
- If `.git` detected in data folder → use git for versioning automatically

## Philosophy

Shared system for humans and AI. Plain text = AI accessible. TUI = human ergonomic.

## Rust Libraries

- **ratatui** — TUI framework
- **crossterm** — terminal backend for ratatui (macOS)
- **serde** + **serde_yaml** — YAML front matter parse/serialise
- **pulldown-cmark** — markdown parsing (render ticket body in TUI)
- **clap** — CRUD CLI arg parsing
- **uuid** — ticket ID generation
- **chrono** — `created_at` / `updated_at` timestamps
- **git2** — git detection + auto-commit if `.git` found
- **notify** — watch data dir for file changes (live TUI refresh)
- **fuzzy-matcher** — ticket search in TUI

## PRD Notes

### Target User

Solo developer. No team/multi-user support.

### File Structure

```
./tickets/
├── all/
│   └── <id>-<slugified-title>.md
└── archived/
    └── <id>-<slugified-title>.md
```

- `tickets init` creates both dirs
- `tickets init` prompts to initialise `.git` if none detected
- No config file — configuration via global flags or `TICKETS_DIR` env var
- Default data dir: `./tickets/` relative to CWD

### Filename Format

`<6-char-nanoid>_<slugified-title>.md` — e.g. `a3f9c1_fix-login-bug.md`

**Future:** show minimal unique id prefix (à la Jujutsu) for brevity in CLI output.

### Front Matter Schema

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
```

**Only required field: `title`.** All others have defaults:

- `type` → `task`
- `status` → `draft`
- `tags` → `[]`
- `parent` → `null`
- `blocked_by` → `[]`
- `id`, `created_at`, `updated_at` → auto-generated

### Validation

Warn on:

- Missing `title`
- Unparseable YAML front matter

Never modify the file on validation failure — surface errors only.

- **TUI:** render offending card in a visible error state
- **CLI:** print error to stderr with filename and field

### Ticket Statuses

`draft` / `todo` / `in-progress` / `done` / `rejected`

### Ticket Types

`epic` / `story` / `task` / `bug`

### TUI Actions

**Navigation**

- Move focus between columns (←/→)
- Move focus between cards within a column (↑/↓)
- Jump to top/bottom of column

**Card Actions**

- Open card in `$EDITOR`
- Create new card (in focused column)
- Move card left/right between columns
- Archive card (moves file from `all/` to `archived/`)

**View**

- Fuzzy search across all cards (title + tags + type)

**App**

- Quit
- Reload from disk (manual, alongside `notify` auto-refresh)

### CLI Surface

**Create**

```
tickets new --title "..." --type <epic|story|task|bug> --tag <tag> --parent <id> --body
```

**Read**

```
tickets list [--status <status>] [--type <type>] [--tag <tag>]
tickets show <id>
tickets search <query>
```

**Update**

```
tickets edit <id> --title "..." --status <status> --type <type> --tag <tag> --parent <id> --blocked-by <id> --body
```

**Archive**

```
tickets archive <id>
```

**Utility**

```
tickets init
tickets tui   # default if no subcommand given
```

**Notes:**

- `--tag` is repeatable: `--tag foo --tag bar`
- `--blocked-by` is repeatable
- `tickets edit` only updates fields explicitly passed
- `--body` flag (no value) reads body from STDIN
- Global flag: `--dir <path>` (overrides `TICKETS_DIR`)

### `updated_at` Behaviour

- CLI: auto-updated on every `tickets edit`
- TUI: updated when file is saved from `$EDITOR`
- Raw file edits: user's responsibility — not rewritten automatically

### Git Integration

- If `.git` detected in data dir: auto-commit on every mutation (new, edit, archive)
- Commit message format: `tickets: <action> <id> <slugified-title>` e.g. `tickets: add a3f9c1 fix-login-bug`
- No branching, no auto-push — local commits only

## MVP Slices (CLI CRU)

1. **`tickets init`** — scaffold `./tickets/all/` + `./tickets/archived/`, detect/prompt git
2. **`tickets new`** — create ticket file with front matter defaults, title required, no `--body` yet
3. **`tickets list`** — read all tickets from `all/`, print to stdout
4. **`tickets show <id>`** — print a single ticket (front matter + body)
5. **`tickets edit <id>`** — update front matter fields in place, bump `updated_at`
6. **`tickets new --body`** — add STDIN body support to new
7. **`tickets edit --body`** — add STDIN body support to edit
8. **`tickets search <query>`** — fuzzy search across title + tags + type
9. **git auto-commit** — layer git2 auto-commit across new/edit
10. **validation + error reporting** — malformed front matter warnings to stderr
