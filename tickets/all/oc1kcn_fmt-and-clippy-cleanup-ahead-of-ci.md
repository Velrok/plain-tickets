---
id: oc1kcn
title: fmt and clippy cleanup ahead of CI
type: task
status: done
tags: []
parent: null
blocked_by: []
created_at: 2026-07-30T18:41:50.134443Z
updated_at: 2026-07-30T18:48:06.233832Z
---

## What
Bring the codebase in line with `cargo fmt` and `cargo clippy -- -D warnings` so both can be enforced in CI.

## Tasks
- Run `cargo fmt` across the repo and commit the reformat as its own commit (no logic changes).
- Fix the pre-existing clippy warnings under `-D warnings`:
  - `src/tui/render.rs:266` — redundant closure (`.map(|l| Line::from(l))` -> `.map(Line::from)`)
  - `src/graph.rs:340` — manual_contains (`lines.iter().any(|l| *l == x)` -> `lines.contains(&x)`)
  - `src/tui/render.rs:480` — needless_range_loop (`for row in 0..ids.len()` -> iterator + `.enumerate()`)
- Verify `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` both pass clean.

## Why
Ticket 67nvy3 adds a CI workflow that enforces `cargo fmt --check` and `cargo clippy -- -D warnings`. Both currently fail on existing code, so this needs to land first.
