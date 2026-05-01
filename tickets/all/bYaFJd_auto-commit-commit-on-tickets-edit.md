---
id: bYaFJd
title: auto-commit - commit on tickets edit
type: task
status: rejected
tags: []
parent: null
blocked_by:
- 3If2VH
created_at: 2026-04-30T23:12:39.967440Z
updated_at: 2026-05-01T22:12:19.973661Z
---

After writing the ticket file, call shared git_commit(dir, file, message) helper. Helper shells out: 'git -C <dir> add <file>' then 'git -C <dir> commit -m <msg>'. Commit message: tickets: edit <id> "<title>". Git identity from user's git config. Git errors surface as-is (no wrapping). Shares helper with 3If2VH. Blocked by auto-commit on new slice. Ref: docs/prd-config-file.md, docs/adr/0001-git-shell-out-over-git2.md