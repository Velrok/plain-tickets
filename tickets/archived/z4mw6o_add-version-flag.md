---
id: z4mw6o
title: add --version flag
type: task
status: done
tags:
- version
- cli
parent: null
blocked_by: []
created_at: 2026-07-30T19:50:18.742809Z
updated_at: 2026-07-30T19:54:37.252939Z
---

## Goal

`tickets --version` / `-V` prints the crate version plus the build's git sha,
e.g. `tickets 0.1.0 (a1b2c3d)`.

## Design

- **Flag wiring**: use clap's built-in `#[command(version = ...)]` on `Cli`
  (main.rs) — gives `-V`/`--version` for free, no manual handling.
- **Format**: `tickets <version> (<short-sha>)`. Bare version (no leading
  `v`), 7-char short sha. Local dev builds: `tickets dev-build (<short-sha>)`.
- **New `build.rs`** (no new dependencies) composes one env var,
  `TICKETS_VERSION_STRING`, consumed via
  `#[command(version = env!("TICKETS_VERSION_STRING"))]`.
  - If `GITHUB_ACTIONS=true` (any Actions run, not just tag releases):
    version = `$GITHUB_REF_NAME` with a leading `v` stripped, sha = first 7
    chars of `$GITHUB_SHA`.
  - Otherwise (local build): version = literal `dev-build`, sha = output of
    `git rev-parse --short HEAD`, or `unknown` if that command fails (no
    git / no `.git`, e.g. a source tarball).
  - Emit `cargo:rerun-if-changed=.git/HEAD` (and refs) so the sha updates
    across commits/branch switches without a `cargo clean`.

## Context

- `scripts/release.sh` bumps `Cargo.toml` version, commits, tags `vX.Y.Z`,
  and only pushes from a clean, up-to-date `main` — so at release time the
  tag commit's sha is unambiguous and `Cargo.toml` version already matches
  the tag (minus `v`).
- `.github/workflows/release.yml`'s `build` job does a normal
  `actions/checkout` (fetch-depth 1) — fine for `GITHUB_SHA`/`GITHUB_REF_NAME`,
  no checkout changes needed. (Ruled out `git describe --tags` in build.rs —
  would've required `fetch-tags: true`.)
- Matches this repo's existing style of shelling out to `git` via
  `std::process::Command` (see `src/git.rs`) rather than adding a git crate
  (`vergen`, `built`, etc. all considered and rejected — no new deps needed).

## Out of scope

- No `tickets version` subcommand — flag only.
- No `-dirty` suffix on local shas.
