---
id: 0awkzp
title: list --unblocked composes with status type and tag filters
type: task
status: in-progress
tags:
- deps-graph
parent: fyihf8
blocked_by:
- 3p7tpt
created_at: 2026-09-23T10:14:02.562292Z
updated_at: 2026-09-23T11:06:24.869610Z
---

## Behaviour

`--unblocked` composes with the existing `--status`, `--type` and `--tag`
filters rather than replacing them.

## Test first

- e2e - `--unblocked --tag backend` applies both filters
- e2e - `--unblocked --status todo` applies both filters
- e2e - repeated `--tag` keeps AND semantics alongside `--unblocked`

## Implementation notes

Implemented via tests only. No production code change was needed.

### Finding

`--unblocked` was already wired into `cmd_list` as a plain `.filter()` step
chained after `matches_filters(t, &args.status, &args.r#type, &args.tag)`,
which already implements OR semantics for repeated `--status`/`--type` and
AND semantics for repeated `--tag`. Composition already worked correctly, so
this ticket was about proving it rather than changing it. `src/commands.rs`,
`src/deps_graph.rs` and the resolution predicate were left untouched; the
predicate remains positively `status == TicketStatus::Done` with no
exhaustive match.

### Proof the tests genuinely exercise the path

Before finishing, the filter line in `cmd_list` was temporarily changed to

    args.unblocked || matches_filters(t, &args.status, &args.r#type, &args.tag)

simulating the exact bug this ticket guards against, where `--unblocked`
bypasses the other filters. All three new tests failed as expected, with
unrelated tickets leaking through. The change was reverted immediately and
`git diff src/commands.rs` against base is empty.

### Tests added in tests/cli_list.rs

- `list_unblocked_and_tag_composes` - `--unblocked --tag backend` excludes
  both a blocked-but-tagged ticket and an unblocked-but-untagged ticket.
- `list_unblocked_and_status_composes` - `--unblocked --status todo` excludes
  a blocked todo ticket and an unblocked done ticket. The blocker fixture
  itself uses `in-progress` rather than `todo`, so it cannot accidentally
  satisfy the `--status todo` filter and inflate the count.
- `list_unblocked_and_repeated_tag_keeps_and_semantics` -
  `--unblocked --tag auth --tag api` excludes a single-tagged ticket, keeping
  AND semantics, and a fully-tagged-but-blocked ticket.

### Incidental fixture trap worth recording

`Title` validation rejects commas. A first draft of the test titles contained
them, and because the `tickets()` test helper does not assert success the way
`create_ticket()` does, `new` failed silently and produced a misleading
"0 lines" assertion failure rather than a validation error. Removing the
commas fixed it.
