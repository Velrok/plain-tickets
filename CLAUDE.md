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

- Commands accept any unique prefix of an ID, but scripts and notes should use the full 13
  base36 characters (`0-9a-z`): a prefix can become ambiguous later.
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

## Releasing

Pushing a `vX.Y.Z` tag runs `.github/workflows/release.yml`. CI (`ci.yml`) runs
on pushes to `main` only (it ignores `tickets/**`, `docs/**`, `*.md`).

1. Write the release notes in `docs/releases/vX.Y.Z.md` (the workflow uses the
   file as the GitHub release body and fails early if it is missing).
1. Bump `version` in `Cargo.toml` (and let `Cargo.lock` update), then commit
   notes and bump (`chore: bump version to X.Y.Z`).
1. Make sure `main` is green: `cargo test`, `cargo fmt --check`,
   `cargo clippy --all-targets -- -D warnings`.
1. Push `main` first, then tag that commit (tags here must be annotated):
   `git push origin main && git tag -m vX.Y.Z vX.Y.Z && git push origin vX.Y.Z`.
1. Watch it: `gh run watch` (the release appears under GitHub Releases).

What the workflow does:

| Job | Does |
| --- | --- |
| `check-tag` | Fails if the tag differs from the `Cargo.toml` version (`scripts/check-release-tag.sh`) or `docs/releases/<tag>.md` is missing |
| `build` | Matrix of 4 targets; runs `cargo test --release` (except x86_64 macOS, which is cross-built), builds, packages |
| `release` | Downloads all archives, publishes a GitHub Release with `docs/releases/<tag>.md` as the body |

Assets: `tickets-linux-x86_64.tar.gz`, `tickets-macos-arm64.tar.gz`,
`tickets-macos-x86_64.tar.gz`, `tickets-windows-x86_64.zip`.

Gotchas:

- A tag/version mismatch fails fast. Fix the version, delete the tag
  (`git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z`), then re-tag.
- Push the bump commit **before** the tag, and tag that commit (or a later
  one). Tagging an earlier commit fails `check-tag` (v0.2.0 did).
- The release does not wait for `ci.yml`; check CI yourself before tagging.
- `tickets --version` shows `X.Y.Z (<short sha>)`, taken from git at build time.
