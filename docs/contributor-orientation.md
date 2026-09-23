# Contributor orientation

Read this before touching code. It exists so you don't spend your first
twenty tool calls rediscovering what every previous contributor already
found. It complements `docs/coding-style.md` (conventions) — this file is
*where things are* and *what will bite you*.

Verified against the tree on 2026-09-23.

## Module map

| File                       | Lines | What it owns                                                                       |
| -------------------------- | ----- | ---------------------------------------------------------------------------------- |
| `src/main.rs`              | 91    | `Cli`, `Commands` enum, dispatch. Every subcommand is wired here                   |
| `src/application_types.rs` | 111   | clap arg structs (`ListArgs`, `NewArgs`, `EditArgs`, `ArchiveArgs`) + `WorkingDir` |
| `src/domain_types.rs`      | 388   | `TicketId`, `Title`, `Tag`, `TicketType`, `TicketStatus`, `FrontMatter`, `Ticket`  |
| `src/commands.rs`          | 732   | One `cmd_<name>` per subcommand + their private helpers                            |
| `src/deps_graph.rs`        | 173   | petgraph `deps-graph`, **and the shared blocker-resolution seam**                  |
| `src/graph.rs`             | 475   | Legacy `graph` command. **Deleted by ticket `fe17ed`** — don't invest here         |
| `src/config.rs`            | 168   | `.tickets.toml` loading, `TuiConfig::kanban_columns`                               |
| `src/git.rs`               | 165   | Auto-commit plumbing                                                               |
| `src/tui/mod.rs`           | 395   | Event loop, `key_to_message`, editor launching                                     |
| `src/tui/app.rs`           | 563   | `App`, `Screen`, `Message`, `Cmd`, `update`, `col_indices`                         |
| `src/tui/render.rs`        | 643   | `draw_board`, `draw_detail`, `draw_help` + snapshot tests                          |

`WorkingDir::new` validates that `all/` and `archived/` exist, so any
function taking a `WorkingDir` may assume an initialised directory. That is
the parse-don't-validate pattern the style guide describes — don't re-check.

## The shared blocker-resolution seam

**This is the single most important thing to know.** Several tickets
converge on one rule, and it must stay in one place.

Location: `src/deps_graph.rs`, top of file.

```rust
enum BlockerResolution { Active(TicketId) }          // widened by 7se1mu / xptucc / t9p76e

fn resolve_blocker(active, id) -> Option<BlockerResolution>
fn blocker_satisfied(active, resolution) -> bool     // Active(id) => status == Done
pub(crate) fn is_unblocked(active, ticket) -> bool   // every blocker satisfied, or none
pub(crate) fn load_active(dir) -> Result<HashMap<TicketId, Ticket>>
```

Rules, non-negotiable:

- **Only `done` ever satisfies a dependency.** Write it as equality
  (`status == TicketStatus::Done`), never as a negative (`!= Todo`) and
  never as an exhaustive `match` on `TicketStatus`.
- **Extend `resolve_blocker` / `blocker_satisfied`. Do not write a second
  copy.** `list --unblocked` and `deps-graph` both defer here by design.
- Tickets that widen this seam: `7se1mu` (archived+done → satisfied),
  `xptucc` (archived non-done → `[archived: <id> <status>]` stub),
  `t9p76e` (unknown id → `[missing: <id>]` stub), `rm49xa` (the same
  resolution, reached via `list --unblocked`).

## Which file your ticket lives in

| Ticket                              | Primary file(s)                                                 |
| ----------------------------------- | --------------------------------------------------------------- |
| `2k12y3` `3yew0k` `9b88c0` `hx1po6` | `src/deps_graph.rs` + `tests/cli_deps_graph.rs`                 |
| `7se1mu` `xptucc` `t9p76e`          | the seam above + `tests/cli_deps_graph.rs`                      |
| `0awkzp` `rm49xa`                   | `cmd_list` in `src/commands.rs` + `tests/cli_list.rs`           |
| `fe17ed`                            | deletes `src/graph.rs`, `tests/cli_graph.rs`, `Commands::Graph` |
| `pgej72`                            | `src/tui/{mod,app,render}.rs`                                   |
| `0mvwz9`                            | new `skills/` dir + a new `cmd_ai_skill`                        |

## Testing

Two tiers, kept separate (see `docs/coding-style.md`).

- Unit tests: `#[cfg(test)] mod tests` in the same file. Pure logic only.
- E2E: `tests/cli_<subcommand>.rs`, driving the real binary.

Helpers in `tests/common/mod.rs` — use these, don't hand-roll:

```rust
bin() -> PathBuf                                  // path to the built binary
test_dir(name) -> PathBuf                         // isolated dir under .testing/
tickets(dir, &["list", "--status", "todo"])       // run the real binary
create_ticket(dir, title) -> (id, filename)
```

`create_ticket` uses the config default status (`draft`). If your assertion
depends on a specific status, pass `--status` explicitly rather than
asserting against the default.

## Traps that have already caught someone

**Date-dependent fixtures.** The suite was permanently red for ~8 weeks
because a snapshot fixture called `Utc::now()`. Fixed in `8wb1lh`. The tui
test modules now each define a local `fixed_timestamp()`. **Never write a
fixture whose assertion or snapshot depends on the current date or year** —
that includes `stdout.contains("2026-")`-style assertions. Match the date
*shape*, not a literal year.

**Exhaustive `match` on `TicketStatus`.** The enum gained `Review` in
`xkbw11` and will gain more. Prefer `==` or membership. If you genuinely
need a `match`, add explicit arms — **never a wildcard `_ =>`**, so the next
addition stays loud at compile time.

**`git stash`.** Never use bare `git stash` / `git stash pop`. The stash
stack is shared across all worktrees, the main checkout and other concurrent
sessions — a bare pop can restore someone else's work. Use a temporary WIP
commit, or `git show HEAD:<path>` to inspect an earlier state.

**`tickets show` emits ANSI even when piped.** It renders through `bat`.
Never parse its output. Use `tickets list`, or read the `.md` file.

**`tickets edit` replaces, never merges.** `--tag` and `--blocked-by`
overwrite the whole list — re-pass every value you want to keep. `--body`
replaces the entire body; to append, edit the `.md` file directly.

**mdformat-on-save hook.** Editing a ticket `.md` with an editor tool can
reformat the YAML front matter (`parent: null` → `parent:`). Harmless but
non-canonical. Prefer the `tickets` CLI for status changes.

**Auto-commit has no pathspec.** `git_commit` in `src/git.rs` runs
`git add -- <file>` then `git commit -m` with no pathspec, so it commits
whatever else is already staged. When you commit your own work, pass an
explicit pathspec **on the `git commit` command itself**, not just on
`git add`.

**Ticket ids are exact, 6 chars.** Lookup matches the `<id>_` filename
prefix; a partial id never resolves.

## Observed behaviour worth knowing

From independent verification of `ihqh45`, on the code as it stands:

- Fan-out already renders `├──`/`└──` correctly, and disjoint chains
  already render as separate roots. `2k12y3` and `9b88c0` may be largely
  test-only — confirm before writing production code.
- Diamonds already repeat under each blocker, but with **no `(see above)`
  marker** — that half is `3yew0k`.
- **A cycle currently prints nothing at all and exits 0.** Every node in the
  cycle has an incoming edge, so none qualifies as a root. The failure mode
  is silently invisible tickets, not a hang — `hx1po6` must assert cycle
  members actually appear, not merely that the command terminates.

## Settled - deps-graph and list --unblocked treat active done blockers differently

They disagree about an **active blocker whose status is `done`**:

- `is_unblocked` treats it as **satisfied**, so the ticket is listed.
- `DepsGraph::from_active` still **draws the edge** - `resolve_blocker`
  returns `Active` for any present id, with no status check.

This is deliberate, not a bug. Do not "fix" it.

The two answer different questions:

| Command | Question | Active done blocker |
|---|---|---|
| `deps-graph` | what is the shape of my active work | stays a node, edge drawn |
| `list --unblocked` | what can I pick up right now | satisfied, ticket listed |

The epic is consistent with this. It builds the graph from "active tickets
`blocked_by` edges" with every active ticket a node, and scopes the
resolution rules to ids **not** in the active set. A `done` ticket still
sitting in `all/` is part of the active structure and should render; it just
does not stop you starting the work it was blocking.

Keep both behaviours. Only the out-of-active-set resolution is shared between
the two commands, via `resolve_blocker`.
