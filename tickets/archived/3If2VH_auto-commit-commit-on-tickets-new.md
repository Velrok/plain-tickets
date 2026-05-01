---
id: 3If2VH
title: auto-commit - commit on tickets new
type: task
status: rejected
tags: []
parent: null
blocked_by:
- 04HRDu
created_at: 2026-04-30T23:12:39.781074Z
updated_at: 2026-05-01T22:12:19.783860Z
---

After writing the ticket file, call shared git_commit(dir, file, message) helper. Helper shells out: 'git -C <dir> add <file>' then 'git -C <dir> commit -m <msg>'. Commit message: tickets: new <id> "<title>". Git identity from user's git config. Git errors surface as-is (no wrapping). Uses shared helper — see bYaFJd. Blocked by auto-commit guard slice. Ref: docs/prd-config-file.md, docs/adr/0001-git-shell-out-over-git2.md