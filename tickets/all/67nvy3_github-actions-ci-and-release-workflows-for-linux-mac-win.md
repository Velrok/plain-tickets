---
id: 67nvy3
title: GitHub Actions CI and release workflows for linux mac win
type: task
status: todo
tags: []
parent: null
blocked_by:
- oc1kcn
created_at: 2026-07-30T18:41:50.473860Z
updated_at: 2026-07-30T18:51:20.210294Z
---

## What
Two GitHub Actions workflows: CI verification on push to `main`, and a release workflow on version tags.

## Blocked by
oc1kcn (fmt/clippy must pass clean before this can enforce them)

## Workflow 1: CI (`.github/workflows/ci.yml`)

**Trigger:** push to `main` only (no PR trigger). `paths-ignore` for `tickets/**` and `docs/**`/`*.md` so doc/ticket-only pushes are skipped.

**Matrix:** `ubuntu-latest`, `macos-latest`, `windows-latest` — build + `cargo test` on each. `fail-fast: true` (default) — first failing OS cancels the rest.

**Toolchain:** `stable` (rolling, not pinned).

**Caching:** `Swatinem/rust-cache`.

**Lint (Linux job only, not tripled across the matrix):**
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`

## Workflow 2: Release (`.github/workflows/release.yml`)

**Trigger:** push of tag matching `v*.*.*` (e.g. `v0.1.0`).

**Matrix / targets:**
- `x86_64-unknown-linux-gnu` (ubuntu-latest)
- `aarch64-apple-darwin` (macos-latest, native — build + test)
- `x86_64-apple-darwin` (macos-latest, cross-compiled — build only, no test/Rosetta)
- `x86_64-pc-windows-msvc` (windows-latest)

**Per target:** run `cargo test` (except the cross-compiled x86_64 mac target — build only), then `cargo build --release`.

**Packaging:** archive each binary — `.tar.gz` for linux/mac, `.zip` for windows. Name: `tickets-<os>-<arch>.<ext>`, e.g. `tickets-linux-x86_64.tar.gz`, `tickets-macos-arm64.tar.gz`, `tickets-macos-x86_64.tar.gz`, `tickets-windows-x86_64.zip`.

**Publish:** create a GitHub Release for the tag (e.g. via `softprops/action-gh-release`), attach all 4 archives, auto-generate release notes from commits since the previous tag.

## Explicitly out of scope
- PR-triggered CI (deferred — push-to-main only for now)
- Linux/Windows arm64 builds
- Rolling/"latest" release on every push (rejected in favour of tag-triggered releases)
