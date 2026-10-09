---
id: 00w41r5htj9xn
title: Show the build SHA in --version again
type: feature
status: done
parent: null
blocked_by: []
tags:
- build
created_at: 2026-10-09T01:04:15.000Z
updated_at: 2026-10-09T01:40:39.453Z
---

## Problem

`tickets --version` prints clap's plain version. The old implementation in `../main` also printed the build SHA (`build.rs` sets `TICKETS_VERSION_STRING`). This repo has no `build.rs`.

## Acceptance

- `tickets --version` prints `<version> (<short sha>)`.
- Falls back gracefully without git, e.g. a source tarball.
- Integration test; the release-tag CI check keeps working.

Done: build.rs sets TICKETS_VERSION_STRING to '<CARGO_PKG_VERSION> (<git rev-parse --short=7 HEAD>)', 'unknown' when git or the repo is missing (checked by hand on an export with GIT_DIR=/nonexistent; build scripts have no automated test). Release-tag script reads Cargo.toml only, so it is unaffected.
