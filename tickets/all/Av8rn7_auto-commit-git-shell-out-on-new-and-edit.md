---
id: Av8rn7
title: auto-commit - git shell-out on new and edit
type: task
status: draft
tags: []
parent: null
blocked_by:
- gs79xq
created_at: 2026-05-01T22:12:09.484758Z
updated_at: 2026-05-01T22:12:09.484758Z
---

When auto_commit = true: guard checks git on PATH and repo detection (git -C <tickets_dir> rev-parse --git-dir) with distinct hard errors. Implement shared git_commit(dir, file, message) helper that shells out git add then git commit. Wire into cmd_new and cmd_edit as final step after file write. Commit messages: tickets: new <id> title / tickets: edit <id> title. Git errors surface as-is. No git2 crate. Ref: docs/prd-config-file.md, docs/adr/0001-git-shell-out-over-git2.md