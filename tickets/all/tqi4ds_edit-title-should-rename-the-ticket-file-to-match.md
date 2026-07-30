---
id: tqi4ds
title: edit title should rename the ticket file to match
type: task
status: todo
tags:
- cli
parent: null
blocked_by: []
created_at: 2026-07-30T17:49:31.938490Z
updated_at: 2026-07-30T17:49:31.938490Z
---

## Problem

`edit --title` rewrites `FrontMatter.title` but never touches the filename (`src/commands.rs` `cmd_edit`, around line 292). The slug embedded in `<id>_<slug>.md` is generated once at `tickets new` time via `Title::slugify` (`commands.rs:100`) and never revisited.

After a rename, the filename lies: `ls tickets/all/` and `rg` both show stale text. The skill docs already have to warn "always glob by id, never by slug" to work around this - a symptom that the slug has drifted from being trustworthy.

## Fix

When `--title` changes the title, recompute the slug and rename the file as part of the same edit:

1. Write the updated front matter + body to the **old** path first (existing `cmd_edit` write).
2. If the new slug differs from the current filename's slug, call `git::git_mv(repo_root, old_rel, new_rel, message)` (already exists, used by `cmd_archive` - `src/git.rs:50`) so the content-write and the rename land in one commit.
3. If `auto_commit` is off, do a plain `std::fs::rename` instead (mirror the non-git path already used elsewhere for archiving, see `commands.rs:400`/`439`).

No referential integrity work needed: `parent` and `blocked_by` store `TicketId`, and `find_ticket` (`commands.rs:338`) matches the `<id>_` filename prefix - nothing points at the slug itself.

## Things to check while in here

- The TUI's `notify` watcher (`src/tui/mod.rs`, `jhzbj0 tui-live-reload`) watches `tickets/all/` non-recursively and reacts to file events. A rename is delete+create at the OS level - confirm the watcher does not drop the current selection or corrupt the kanban board when its focused ticket's file disappears and reappears under a new name mid-session.
- If the same title is set twice (no-op edit), skip the rename - only act when the computed slug actually differs from the current filename.

## Non-goals

- Not changing `Title` validation (see `789wgi`-adjacent title discussion) - independent of what characters are allowed, the existing `slugify` already maps any non-alphanumeric char to `-`, so it handles today's allowed title charset unchanged.
- Not adding a `--rename` opt-out flag. Filenames are considered a derived, cosmetic index of the id; keeping them accurate is not something a user should need to ask for.

## Tests

- `edit --title` changes the on-disk filename to match the new slug.
- `edit --title` with `auto_commit = true` produces exactly one commit covering both the content change and the rename.
- `edit` on a field other than `--title` (e.g. `--status`) does not touch the filename.
- Setting `--title` to its current value is a no-op on the filename.
