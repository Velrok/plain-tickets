---
id: eb7a7ac490fa0db4
title: Check the release tag matches the Cargo version
type: task
status: done
parent: null
blocked_by: []
tags:
- ci
created_at: 1791504687
updated_at: 1791507406
---

release.yml triggers on v*.*.* tags but never compares the tag with version in Cargo.toml, so tickets --version can disagree with the release name.

Acceptance: the release workflow fails early when the tag (minus the leading v) differs from the Cargo.toml version.

Done. scripts/check-release-tag.sh <tag> [Cargo.toml] compares the tag minus its leading v with the first top-level version line. release.yml has a new check-tag job that build needs (release needs build), so nothing builds or publishes on a mismatch. The tag reaches the script via env, not interpolated into the run line. tests/release_tag.rs covers match, mismatch and prerelease (unix only, since Windows CI may lack sh). Not run on real GitHub Actions: no actionlint here, YAML only checked with yq.
