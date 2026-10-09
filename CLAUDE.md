# plain-tickets

Rust CLI (`tickets`) for markdown tickets. See `README.md` for the commands.

## Dog-food rule

**Track all work on this project with this CLI.** Do not use TaskCreate, TODO
files or Jira for this repo's own work.

Run it from the repo root. Prefer the built binary so a broken build does not
block ticket access:

```bash
cargo build -q && export PATH="$PWD/target/debug:$PATH"   # or: cargo run -q -- <args>
```

First time only: `tickets init` creates `.tickets/config.toml`. Statuses are
`todo`, `in progress`, then the built-ins `done` and `rejected`.

## Workflow

1. **Find work:** `tickets list --ready` (open, nothing unfinished blocking it).
1. **Read it:** `tickets show <id>`.
1. **Claim it:** `tickets set <id> --status "in progress"` before starting.
1. **Log decisions and findings:** `tickets note <id> "..."`, not chat.
1. **Finish:** `tickets set <id> --status done`
1. **Not doing it:** `--status rejected` with a `note` saying why.

## Creating tickets

- `tickets new "Short imperative title" -t task -g area -m "Why and acceptance"`
- It prints the 16-hex ID. Capture it: `id=$(tickets new "..." )`.
- Split by dependency: `-p <parent>` for hierarchy, `-b <blocker>` for order.
- Titles are one line. Tags are single words.
- **Found a bug or gap in the CLI while using it?** File a ticket for it
  immediately (`-t bug`), then carry on. Friction here is the product feedback.

## Rules

- IDs are always the full 13 base36 characters (`0-9a-z`).
- **Whenever you mention a ticket ID to the user, include its title** (e.g.
  `16utbc9j3f6jp` List order is arbitrary…). Bare IDs are unreadable.
- Commit `tickets/` together with the code change it tracks.
- Only `done` or `rejected` tickets can be archived.

## Development

- TDD: one failing test, then the minimal code, then repeat. Integration tests
  in `tests/` drive the real binary; shared helpers live in `tests/common`.
- Before a commit: `cargo test` passes, `cargo build` is warning-free,
  `cargo fmt --check` is clean and `cargo clippy --all-targets -- -D warnings`
  passes (CI enforces the last two).
- Keep the `README.md` checklist in step with the commands.
