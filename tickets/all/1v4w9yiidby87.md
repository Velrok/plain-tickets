---
id: 1v4w9yiidby87
title: Make the test suite compile on Windows (unix-only Permissions::from_mode)
type: bug
status: done
parent: null
blocked_by: []
tags:
- release
created_at: 2026-10-09T02:03:18.485Z
updated_at: 2026-10-09T02:08:36.856Z
---

v0.2.0 release run 37872523514 failed on windows-latest: tests new/show/modify use std::os::unix and Permissions::from_mode. Acceptance: cfg(unix)-gate those tests/helpers; cargo test passes on the Windows release job.

Fixed: cfg(unix) gating for the edit tests, editor helper and a store test; private_temp_dir only sets mode 0700 on unix. Verified with cargo check --all-targets --target x86_64-pc-windows-msvc.
