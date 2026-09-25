---
id: hcug6h
title: TUI load failures are recallable rather than a 2s flash
type: story
status: todo
tags:
- tui
parent: null
blocked_by: []
created_at: 2026-09-25T12:28:45.278444Z
updated_at: 2026-09-25T12:28:45.278444Z
---

## Behaviour

`fd38vu` surfaced ticket load failures through the existing footer flash. That
was the right call for scope - it reuses one already-tested UI surface for all
four call sites - but independent verification flagged a genuine residual gap at
startup.

The flash lasts 2s (`FLASH_DURATION` in `src/tui/render.rs`) and then reverts to
the default hint with no way to recall it. For an in-flight action (fs-watch
reload, post-`$EDITOR` save) the user just acted and is watching the screen, so
2s is plausible. At **startup** the user has just launched the tool, may be
looking away or resizing, and - in the motivating case of a stale binary - has
no reason yet to suspect anything is wrong. Verified empirically: after ~2.2s
the message is gone for good.

## Shape of the fix

Make load failures recallable rather than transient. Options, in rough order of
preference:

- keep the failure list on `App` and surface it in the help overlay (`?`)
- a dedicated key that re-shows the last load report
- a persistent indicator in the footer while any failure is outstanding, with
  the detail available on demand

Not a persistent modal banner - that is new UI territory and was correctly
judged outside `fd38vu`'s scope.

## Test first

- unit - the failure list survives past `FLASH_DURATION` on the `App`
- unit - the recall surface renders every failed filename and its reason
- e2e/unit - the indicator clears once the offending file is fixed and reloaded
