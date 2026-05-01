---
id: gs79xq
title: config file - scaffold parse and load .tickets.toml
type: task
status: draft
tags: []
parent: null
blocked_by: []
created_at: 2026-05-01T22:12:03.712872Z
updated_at: 2026-05-01T22:12:03.712872Z
---

tickets init creates tickets/.tickets.toml with commented-out defaults. If git is detected (git -C <tickets_dir> rev-parse --git-dir), print hint to enable auto_commit. Add toml dependency. Define Config/GitConfig structs. Load tickets/.tickets.toml at startup (missing file = defaults). Expose config to commands. Ref: docs/prd-config-file.md