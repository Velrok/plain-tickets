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
| `Title` | `String` | Trimmed, non-empty, max 120 chars |
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
  into `.0` outside the type's own `impl` blocks.

### Enums for closed sets

Use enums (not strings) for fields with a fixed set of values.
Derive `clap::ValueEnum` so clap validates and tab-completes them.
Use `#[serde(rename_all = "kebab-case")]` to match the YAML format.

```rust
#[derive(clap::ValueEnum, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
enum TicketType { Epic, Story, #[default] Task, Bug }
```

## Error Handling

- **User-facing errors** → `eprintln!("error: ...")` + `process::exit(1)`
- **Validation errors** → returned from `FromStr` as `String`; clap surfaces
  them with context automatically
- No `unwrap()` or `expect()` on fallible operations in command logic

## CLI Structure

- One `cmd_<name>` function per subcommand
- `cmd_<name>` takes typed arguments — no raw `String` for domain values
- Destructure the command enum in `main` and pass fields positionally to
  `cmd_<name>`

## Serialisation

- YAML front matter via `serde_yaml`
- All domain types serialise transparently (strings) or as kebab-case (enums)
- Timestamps use `DateTime<Utc>` — serde handles RFC3339 automatically
- The `r#` raw identifier prefix is used for `type` (a Rust keyword); serde
  strips the prefix so the YAML key is plain `type`

## File Layout

Everything lives in `src/main.rs` until the file warrants splitting. When
splitting, prefer grouping by domain concept (e.g. `types.rs`, `commands/`)
over grouping by layer.
