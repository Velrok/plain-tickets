---
id: 04HRDu
title: auto-commit - hard error if no .git
type: task
status: rejected
tags: []
parent: null
blocked_by:
- w0-eam
created_at: 2026-04-30T23:12:39.597025Z
updated_at: 2026-05-01T22:12:19.593574Z
---

If auto_commit = true, check git availability and repo detection before committing. Uses 'git -C <tickets_dir> rev-parse --git-dir' (shell-out, no manual .git walk). Guard runs per write command (not at startup). Two distinct hard errors: (1) git not on PATH, (2) tickets dir not inside a git repo. Blocked by config parse slice. Ref: docs/prd-config-file.md, docs/adr/0001-git-shell-out-over-git2.md