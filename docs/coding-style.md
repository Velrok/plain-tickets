# Coding Style

## Language & Edition

- Rust, edition 2024
- British English in comments, docs, and user-facing messages

## Types

### Newtype wrappers for domain values

Prefer typed wrappers over bare `String` for domain values. Each wrapper owns
its validation and serialises transparently via `#[serde(transparent)]`.

Current wrappers:

| Type | Inner | Constraints |
|------|-------|-------------|
| `TicketId` | `String` | No validation — any non-empty string |
| `Title` | `String` | Trimmed, non-empty, max 120 chars, only letters/digits/spaces/`_`/`-`/`.` |
| `Tag` | `String` | Non-empty, letters/digits/`_`/`-` only |

**Pattern for a new wrapper:**

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
struct MyType(String);

impl std::str::FromStr for MyType {
    type Err = String;   // human-readable error, used by clap
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // validate, return Err("...") on failure
        Ok(MyType(s.to_string()))
    }
}

impl std::fmt::Display for MyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
```

- `FromStr` doubles as the clap value parser — validation happens at CLI parse
  time, before any command logic runs.
- `Display` is the canonical way to get the string back out; avoid reaching
  into `.0` outside the type's own `impl` blocks. Exception: domain methods on
  the type itself (e.g. `Title::slugify`) may access `.0` internally.

### Enums for closed sets

Use enums (not strings) for fields with a fixed set of values.
Derive `clap::ValueEnum` so clap validates and tab-completes them.
Use `#[serde(rename_all = "kebab-case")]` to match the YAML format.

```rust
#[derive(clap::ValueEnum, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
enum TicketType { Epic, Story, #[default] Task, Bug }
```

## Parse, Don't Validate

Prefer types that are impossible to construct in an invalid state over runtime
existence checks scattered through command logic.

**Example:** `WorkingDir::new(base) -> Result<WorkingDir>` checks that `all/`
and `archived/` exist at construction time. Every function that receives a
`WorkingDir` can assume the directory is initialised — no per-function guard
needed. Functions that intentionally operate before initialisation (e.g.
`cmd_init`) take a plain `PathBuf` instead.

The same principle applies to domain newtypes: `Title`, `Tag`, and `TicketId`
validate in `FromStr`, so call sites never need to re-validate.

## Error Handling

- **Command functions** return `anyhow::Result<()>`; errors propagate via `?`
- **`main()`** calls `run() -> Result<()>` and prints `error: {e}` + exits 1
- **`bail!("...")`** for logic errors (not-found, already-initialised, etc.)
- **`.with_context(|| ...)`** to annotate I/O errors with the file/dir path
- **`.map_err(anyhow::Error::msg)`** to bridge `Result<_, String>` into anyhow
  (used at call sites for `git::git_commit` and `config::load`)
- **Validation errors** → returned from `FromStr` as `String`; clap surfaces
  them with context automatically
- No `unwrap()` or `expect()` on fallible operations in command logic

## CLI Structure

- One `cmd_<name>` function per subcommand
- `cmd_<name>` takes typed arguments — no raw `String` for domain values
- Destructure the command enum in `main` and pass fields positionally to
  `cmd_<name>`

## Filename Format

Ticket files follow `<id>_<slug>.md` — e.g. `a3f9c1_fix-login-bug.md`.

- `_` separates the id from the slug (easy to split on the first `_`)
- `-` separates words within the slug
- Slug is derived from `Title::slugify()`: lowercased, non-alphanumeric chars
  replaced with `-`, consecutive `-` collapsed

## Serialisation

- YAML front matter via `serde_yaml`
- All domain types serialise transparently (strings) or as kebab-case (enums)
- Timestamps use `DateTime<Utc>` — serde handles RFC3339 automatically
- The `r#` raw identifier prefix is used for `type` (a Rust keyword); serde
  strips the prefix so the YAML key is plain `type`

## File Layout

Group by domain concept, not by layer.

Current layout:

| File | Contents |
|------|----------|
| `src/main.rs` | `Cli`, `Commands` (clap structs), `main()` |
| `src/types.rs` | Domain newtypes (`TicketId`, `Title`, `Tag`), enums (`TicketType`, `TicketStatus`), `FrontMatter` |
| `src/commands.rs` | `resolve_dir`, `cmd_init`, `cmd_new`, `init_directories` |

When `commands.rs` grows, split into `src/commands/` with one file per subcommand (e.g. `src/commands/new.rs`).

## Testing

Two tiers — keep them separate:

### Unit tests (`#[cfg(test)]` in `src/`)

- Live in the same file as the code they test, inside a `mod tests` block
- Test pure logic: `FromStr` validation, `Display`, helper methods (e.g. `Title::slugify`)
- No filesystem, no process spawning
- Run with `cargo test --bin tickets`

### E2E / CLI tests (`tests/`)

- One file per subcommand: `tests/cli_init.rs`, `tests/cli_new.rs`, `tests/cli_edit.rs`
- Shared helpers in `tests/common/mod.rs` (`test_dir`, `tickets`, `create_ticket`)
- Each test gets an isolated directory under `.testing/<test-name>/` via the `TICKETS_DIR` env var
- Invoke the real binary via `std::process::Command` — test CLI behaviour end-to-end
- Input validation tests live in `tests/cli_validation.rs`
- Run with `cargo test` (all) or `cargo test --test cli_<name>` (one file)

### Conventions

- Test names describe the behaviour, not the implementation: `title_empty_is_err`, not `test_from_str_1`
- Each test is self-contained — no shared mutable state between tests
- `.testing/` is git-ignored; leave artefacts on disk for post-failure inspection
