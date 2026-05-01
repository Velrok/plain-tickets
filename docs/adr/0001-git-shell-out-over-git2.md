# ADR 0001: Shell out to `git` CLI instead of using `git2` crate

**Status:** Accepted
**Date:** 2026-05-01

## Context

The auto-commit feature needs to detect whether the tickets directory is inside a git repository, and then stage and commit a single file after `tickets new` or `tickets edit`.

Two options were considered:

1. **`git2` crate** — pure-Rust libgit2 bindings
2. **Shell out to `git` CLI** — invoke `git -C <dir>` subprocesses

## Decision

Shell out to `git` CLI for both detection and committing.

- Detection: `git -C <tickets_dir> rev-parse --git-dir`
- Staging: `git -C <tickets_dir> add <file>`
- Committing: `git -C <tickets_dir> commit -m "<message>"`

## Rationale

- Drops the `git2` native library dependency, simplifying the build
- Automatically respects the user's full git environment: SSH agent, GPG signing, global/local config, hooks
- The target audience (solo devs using a plain-text ticket system) reliably has `git` on `PATH`
- Detection and committing use the same mechanism — no split between two approaches

## Consequences

- `git` must be present on `PATH`; if it is not, commands will fail with a clear error
- No `git2` or `libgit2` dependency in `Cargo.toml`
