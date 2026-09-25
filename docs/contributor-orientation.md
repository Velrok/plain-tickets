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

### Bug tickets: reproduce before you fix

**Standing rule, no exceptions.** A ticket of type `bug` starts with a test that
**reproduces the reported issue and fails for that reason**. You run it, you
watch it go red, and you record the actual failure output. Only then do you
touch production code.

Not "a test that covers the area". Not "a test written alongside the fix". The
red run has to happen first and it has to fail *because of the bug*, not because
of a typo, a missing fixture or a compile error.

Why it is worth the discipline every time:

- It proves the bug is real and that you have understood it. More than one
  "bug" here has turned out to already work — `2k12y3` and `9b88c0` were filed
  as gaps and were largely implemented. A red test is the cheapest way to find
  that out before writing code nobody needed.
- It proves the test is discriminating, for free. A test written after the fix
  has never been observed failing, so nothing rules out its passing vacuously.
  This is the same property the mutation-testing rule below buys, except here
  you get it without having to break anything.
- It pins the *reported* symptom rather than your theory of the cause. `hx1po6`
  is the cautionary case: its title says the command "hangs", and it does not —
  it exits 0 and silently omits the tickets. A test written from the title would
  have asserted termination and passed against the broken code.

Report the red output in your implementation notes, then the green. If the test
will not go red, stop and say so — the ticket's premise is wrong and that is a
finding, not an obstacle to work around.

### Counting the suite

`cargo test` builds ~15 separate test binaries and prints a separate
`test result: ok. N passed` line for **each**. There is no grand total line.
Reading one line and calling it the total has already produced a bogus "tests
have gone missing" scare. Sum them:

```sh
cargo test 2>&1 | grep -oE '^test result: ok\. [0-9]+' \
  | grep -oE '[0-9]+' | awk '{s+=$1} END {print s}'
```

Also count the `test result:` lines — a binary that fails to build or is
silently skipped is a real defect, and a raw pass count will not show it.
Always state which commit you measured against; the baseline moves.

### Prove your tests are discriminating

Do this yourself; don't leave it to a reviewer. Once a test is green,
deliberately break the production code it covers and confirm **that** test goes
red for the right reason, then restore and confirm green again. Report which
mutation reddened which test.

**Restore from your last commit, not with `git checkout --`, and never via a
temp-file copy:**

```sh
git show HEAD:src/thing.rs > src/thing.rs
```

Then confirm with `git status --short` and `git diff --stat` — both read-only
and prompt-free. A clean diff is the proof the tree is back at HEAD.

Why not the two obvious alternatives:

- **`git checkout -- <file>`** discards working-tree changes, so it is a
  destructive operation that prompts the user for permission on every call.
  Several mutations across several concurrent agents turns that into a flood of
  prompts for what is only putting a file back.
- **`cp` to a temp path** looks harmless and is not. `/tmp/claude/` is a
  **shared** working directory, not private per agent. Two agents backing up
  `src/commands.rs` under the same filename clobbered each other, and the
  restore returned a *different ticket's* code — losing uncommitted work. If
  you back up by copying, the filename must be unique per agent, and even then
  `git show` is strictly better because nothing is written outside the worktree
  at all.

The same reasoning rules out `git stash`, on top of the shared-stack hazard
described below.

**Commit each green step before you start mutating.** Mutation testing is the
one part of the workflow that deliberately corrupts your working tree, so it is
the worst possible moment to be carrying uncommitted work. The incident above
only cost an hour because the fix had never been committed.

This is not ceremony. It is how the real defects in this repo have been caught:
injecting `return Vec::new()` into `col_indices` reddened three tests;
reintroducing the byte-length padding bug proved a regression test was
discriminating. A test that still passes when you break the thing it claims to
cover is worthless, and a ticket will be failed for shipping one.

It matters most when a ticket turns out to be **test-only** — a test that passes
against unchanged code proves nothing until you have shown it fails when the
behaviour is broken.

`cargo insta --check` passing only proves snapshots match *current* output, not
that the blessed content is *correct*. Eyeball blessed text against the ticket.

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

**Don't fix the silent-drop pattern in `cmd_list` in passing.** `filter_map(...) .ok()` at `src/commands.rs:260-261` discards read and parse errors, and a third
similar loader sits on the deps-graph path. The TUI half of this was fixed in
`fd38vu`; the CLI half is ticketed as `4aawv9` with a specified stderr-based
design. It is tempting to clean up while you are in the area — don't, you will
collide with that ticket.

**Working in a worktree? Your `tickets/` is a stale snapshot.** The tracker
moves on `main` while you work. Read ticket bodies for context, but treat
statuses and blocker edges as possibly out of date, and never write to
`tickets/` from a worktree — the supervisor owns tracker writes, and
`git.auto_commit = true` means a CLI write there creates a commit on your
branch.

**Never `git pull` / `git merge` / `git rebase` from a contributor worktree.**
Branches are cut from **local** `main` deliberately. `origin/main` has diverged
and carries tickets that never existed locally; pulling it in corrupted `main`
once. Note also that `git revert` on a bad merge does not remove those commits
from history — it only negates their content, and the stale lineage then makes a
later `git pull --rebase` replay onto the wrong base.

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

| Command            | Question                            | Active done blocker      |
| ------------------ | ----------------------------------- | ------------------------ |
| `deps-graph`       | what is the shape of my active work | stays a node, edge drawn |
| `list --unblocked` | what can I pick up right now        | satisfied, ticket listed |

The epic is consistent with this. It builds the graph from "active tickets
`blocked_by` edges" with every active ticket a node, and scopes the
resolution rules to ids **not** in the active set. A `done` ticket still
sitting in `all/` is part of the active structure and should render; it just
does not stop you starting the work it was blocking.

Keep both behaviours. Only the out-of-active-set resolution is shared between
the two commands, via `resolve_blocker`.
