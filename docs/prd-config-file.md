# PRD: Config File (`tickets/.tickets.toml`)

## Problem

The git auto-commit feature (committing ticket changes to version control) cannot be controlled per-project. Detecting a `.git` folder and assuming the user wants auto-commits is too presumptuous. We need an explicit opt-in mechanism.

## Solution

Introduce a TOML config file at `tickets/.tickets.toml`. It travels with the tickets data, so it works correctly regardless of `TICKETS_DIR`.

---

## Config File

**Location:** `tickets/.tickets.toml`

**Format:**
```toml
# plain-tickets configuration

[git]
# Automatically commit changes when creating or editing tickets.
# Requires a .git repository in or above the tickets directory.
# auto_commit = false
```

---

## Behaviour Spec

### `tickets init`
- Always creates `tickets/.tickets.toml` with defaults commented out
- If `.git` is detected in or above the tickets directory, print a hint:
  > Tip: set `git.auto_commit = true` in `tickets/.tickets.toml` to automatically commit ticket changes.

### Default value
- If the config file does not exist, or `auto_commit` is not set, default is `false`
- Auto-commit is **opt-in**

### `auto_commit = true` — happy path
- After `tickets new` or `tickets edit`, the affected ticket file is staged and committed
- Commit happens as the **final step** after the file is written to disk
- Only the **single ticket file** is staged (no other changes)
- Commit message format:
  - `tickets: new <id> "<title>"`
  - `tickets: edit <id> "<title>"`
- Uses the existing `git config user.name` / `user.email` identity

### `auto_commit = true` — error cases
Both are hard errors — the user explicitly opted into versioning; silently skipping would be confusing.

- **`git` not on PATH** — `error: auto_commit is enabled but git was not found on PATH`
- **No git repository detected** — `error: auto_commit is enabled but the tickets directory is not inside a git repository`

Detection uses `git -C <tickets_dir> rev-parse --git-dir` (shells out; no `.git` walk in code).

---

## Out of Scope (for now)
- Configurable git author
- Configurable commit message format
- `[git]` settings beyond `auto_commit`
