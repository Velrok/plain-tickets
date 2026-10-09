---
id: eb7a7ac490fa0db4
title: Check the release tag matches the Cargo version
type: task
status: todo
parent: null
blocked_by: []
tags:
- ci
created_at: 1791504687
updated_at: 1791504687
---

release.yml triggers on v*.*.* tags but never compares the tag with version in Cargo.toml, so tickets --version can disagree with the release name.

Acceptance: the release workflow fails early when the tag (minus the leading v) differs from the Cargo.toml version.