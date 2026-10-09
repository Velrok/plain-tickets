---
id: 1mo5l6mlnpala
title: new -m rejects a message that starts with dashes
type: bug
status: done
parent: null
blocked_by: []
tags:
- cli
created_at: 2026-10-09T00:18:34.000Z
updated_at: 2026-10-09T00:39:33.000Z
---

tickets new -m '--format ...' fails with 'unexpected argument'. Clap treats the value as a flag. Consider allow_hyphen_values on -m, or document the -m=VALUE form.

Chose to document, not to change parsing: allow_hyphen_values would let a forgotten -m value silently swallow the next flag. Workarounds verified: --message=VALUE and -m=VALUE for new, and note <id> -- TEXT. Added after_help to new and note, with tests on the help text.
