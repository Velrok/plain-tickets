---
id: 0mvwz9
title: Package the tickets-cli-expert skill in the binary and print it with tickets ai-skill
type: story
status: draft
tags:
- ai-skill
- docs
parent: null
blocked_by:
- fe17ed
created_at: 2026-09-23T10:23:03.398998Z
updated_at: 2026-09-23T10:23:03.398998Z
---

## Goal

The `tickets-cli-expert` skill currently lives in the user's dotfiles, outside
this repo. It documents CLI surface and gotchas that only this codebase knows
about, so it drifts silently every time the CLI changes -- the `graph` removal
in `fe17ed` is the first concrete example, and there will be more.

Bring the skill in house. Keep the canonical text in this repo, compile it into
the binary, and expose it with a new `tickets ai-skill` subcommand that prints
it to stdout.

Then a released binary always carries the skill text for exactly its own
version, and installing or refreshing the skill is a redirect:

    tickets ai-skill > ~/.claude/skills/tickets-cli-expert/SKILL.md

## Behaviour

- New subcommand `tickets ai-skill`, no arguments. Prints the skill markdown to
  stdout and exits 0.
- Output is the complete skill file including its YAML front matter (`name`,
  `description`), so the redirect above produces a valid, immediately usable
  `SKILL.md` with no hand editing.
- Plain stdout, no `bat` rendering and no ANSI escapes -- unlike `tickets show`
  this output is meant to be piped to a file. It must be byte-identical to the
  in-repo source.
- Works with no tickets directory present. It is a static text dump, so it must
  not call `resolve_dir` or require `tickets init` to have been run.

## Implementation

- Canonical text lives at `skills/tickets-cli-expert/SKILL.md` in this repo.
- Embed at compile time with `include_str!` rather than reading from disk at
  runtime. A downloaded release binary has no repo next to it, so a runtime
  file lookup would fail for the main distribution case.
- `cmd_ai_skill` follows the existing one-`cmd_<name>`-per-subcommand pattern.
  Like `cmd_init`, it does not take a `WorkingDir`.

## Content migration

Port the existing dotfiles skill as the starting text, then correct it against
the CLI as it stands once `fe17ed` has landed. Known corrections needed:

- Drop the `graph shows blockers, not children` section entirely.
- Document `deps-graph` and `list --unblocked` in its place, including the
  blocker resolution rule -- only `done` satisfies a dependency, archived
  non-done renders as a stub, a missing id renders as a distinct stub.
- Document `tickets ai-skill` itself.

## Test first

- e2e - `tickets ai-skill` exits 0 and its stdout starts with the front matter
  delimiter
- e2e - stdout is byte-identical to `skills/tickets-cli-expert/SKILL.md`
- e2e - succeeds in a directory with no tickets dir and no config

## Keeping it honest

An in-repo copy that nobody updates is no better than a dotfiles copy that
nobody updates. Add a line to `docs/coding-style.md` stating that any change to
a subcommand, flag or validation rule updates
`skills/tickets-cli-expert/SKILL.md` in the same commit.

## Why blocked by fe17ed

`fe17ed` deletes the `graph` subcommand, which the current skill text documents
at length. Landing this first would mean importing text that is stale on
arrival, then rewriting it. Waiting lets the skill be ported once, against the
final CLI surface.

## Follow-up outside this repo

Once this ships, the dotfiles copy at
`~/.claude/skills/tickets-cli-expert/SKILL.md` becomes a generated artefact.
Regenerate it from the binary and stop hand editing it.
