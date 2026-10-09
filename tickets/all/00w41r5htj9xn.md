---
id: 00w41r5htj9xn
title: Show the build SHA in --version again
type: feature
status: todo
parent: null
blocked_by: []
tags:
- build
created_at: 2026-10-09T01:04:15.000Z
updated_at: 2026-10-09T01:10:33.000Z
---

## Problem

`tickets --version` prints clap's plain version. The old implementation in `../main` also printed the build SHA (`build.rs` sets `TICKETS_VERSION_STRING`). This repo has no `build.rs`.

## Acceptance

- `tickets --version` prints `<version> (<short sha>)`.
- Falls back gracefully without git, e.g. a source tarball.
- Integration test; the release-tag CI check keeps working.
