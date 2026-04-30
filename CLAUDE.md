# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build                  # build
cargo run -- <subcommand>    # run (e.g. cargo run -- init)
cargo test                   # run tests
cargo clippy                 # lint
cargo fmt                    # format
```

Single test: `cargo test <test_name>`

## Project Overview

`plain-tickets` is a plain-text, markdown-based ticket system for solo developers. Each ticket is a `.md` file with YAML front matter. The binary is `tickets`.

**Data directory:** resolved from `TICKETS_DIR` env var, fallback `./tickets/` relative to CWD.

**File structure:**
```
tickets/
├── all/        # active tickets
└── archived/   # archived tickets
```

**Filename format:** `<6-char-nanoid>_<slugified-title>.md` — e.g. `a3f9c1_fix-login-bug.md`

## Architecture

**Modules:**
- `src/main.rs` — `Cli`, `Commands` (clap structs), `main()`
- `src/types.rs` — domain newtypes (`TicketId`, `Title`, `Tag`), enums (`TicketType`, `TicketStatus`), `FrontMatter`, `Ticket`
- `src/commands.rs` — `resolve_dir`, `cmd_init`, `cmd_new`, `cmd_edit`

**Key types:**
- `Ticket` — owns `FrontMatter` + `body: String`; implements `FromStr` (parse file content) and `Display` (serialise to file format)
- `FrontMatter` — serde struct for YAML front matter (id, title, type, status, tags, parent, blocked_by, created_at, updated_at)
- `TicketId`, `Title`, `Tag` — newtype wrappers with `FromStr` validation (used as clap value parsers)
- `TicketType`, `TicketStatus` — enums with `clap::ValueEnum` + `#[serde(rename_all = "kebab-case")]`

**Command pattern:** one `cmd_<name>` function per subcommand, called from `main` with typed args.

## Dogfooding

This project tracks its own work using itself. The `tickets/` directory contains the MVP slice tasks. Use `cargo run --` as the `tickets` binary when working on features.

## Coding Style

Read `docs/coding-style.md` before writing any code.
