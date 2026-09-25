mod common;

use std::process::{Command, Stdio};

#[test]
fn list_filter_status_returns_matching_only() {
    let dir = common::test_dir("list_filter_status_returns_matching_only");
    common::tickets(&dir, &["init"]);
    let (id_todo, _) = common::create_ticket(&dir, "Todo ticket");
    common::tickets(&dir, &["edit", &id_todo, "--status", "todo"]);
    let (_id_draft, _) = common::create_ticket(&dir, "Draft ticket");

    let out = common::tickets(&dir, &["list", "--status", "todo"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected 1 line, got: {:?}", lines);
    assert!(
        lines[0].contains("Todo ticket"),
        "expected todo ticket: {}",
        lines[0]
    );
}

#[test]
fn list_filter_status_review_returns_matching_only() {
    let dir = common::test_dir("list_filter_status_review_returns_matching_only");
    common::tickets(&dir, &["init"]);
    let (id_review, _) = common::create_ticket(&dir, "Review ticket");
    common::tickets(&dir, &["edit", &id_review, "--status", "review"]);
    let (_id_todo, _) = common::create_ticket(&dir, "Todo ticket");

    let out = common::tickets(&dir, &["list", "--status", "review"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected 1 line, got: {:?}", lines);
    assert!(
        lines[0].contains("Review ticket"),
        "expected review ticket: {}",
        lines[0]
    );
}

#[test]
fn list_filter_status_or_semantics() {
    let dir = common::test_dir("list_filter_status_or_semantics");
    common::tickets(&dir, &["init"]);
    let (id_todo, _) = common::create_ticket(&dir, "Todo ticket");
    common::tickets(&dir, &["edit", &id_todo, "--status", "todo"]);
    let (id_done, _) = common::create_ticket(&dir, "Done ticket");
    common::tickets(&dir, &["edit", &id_done, "--status", "done"]);
    let (_id_draft, _) = common::create_ticket(&dir, "Draft ticket");

    let out = common::tickets(&dir, &["list", "--status", "todo", "--status", "done"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "expected 2 lines, got: {:?}", lines);
    assert!(stdout.contains("Todo ticket"));
    assert!(stdout.contains("Done ticket"));
    assert!(!stdout.contains("Draft ticket"));
}

#[test]
fn list_filter_type_returns_matching_only() {
    let dir = common::test_dir("list_filter_type_returns_matching_only");
    common::tickets(&dir, &["init"]);
    common::tickets(&dir, &["new", "--title", "A bug", "--type", "bug"]);
    common::tickets(&dir, &["new", "--title", "A task", "--type", "task"]);

    let out = common::tickets(&dir, &["list", "--type", "bug"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected 1 line, got: {:?}", lines);
    assert!(stdout.contains("A bug"));
    assert!(!stdout.contains("A task"));
}

#[test]
fn list_filter_tag_and_semantics() {
    let dir = common::test_dir("list_filter_tag_and_semantics");
    common::tickets(&dir, &["init"]);
    common::tickets(
        &dir,
        &[
            "new",
            "--title",
            "Auth and API",
            "--tag",
            "auth",
            "--tag",
            "api",
        ],
    );
    common::tickets(&dir, &["new", "--title", "Auth only", "--tag", "auth"]);
    common::tickets(&dir, &["new", "--title", "No tags"]);

    let out = common::tickets(&dir, &["list", "--tag", "auth", "--tag", "api"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "expected 1 line (must have both tags), got: {:?}",
        lines
    );
    assert!(stdout.contains("Auth and API"));
}

#[test]
fn list_no_filters_returns_all() {
    let dir = common::test_dir("list_no_filters_returns_all");
    common::tickets(&dir, &["init"]);
    common::tickets(&dir, &["new", "--title", "First ticket", "--type", "bug"]);
    common::tickets(&dir, &["new", "--title", "Second ticket", "--type", "task"]);

    let out = common::tickets(&dir, &["list"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.contains("First ticket"));
    assert!(stdout.contains("Second ticket"));
}

#[test]
fn list_errors_when_not_initialised() {
    let dir = common::test_dir("list_errors_when_not_initialised");
    let out = common::tickets(&dir, &["list"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("not initialised"),
        "unexpected stderr: {}",
        stderr
    );
}

#[test]
fn list_empty_is_silent() {
    let dir = common::test_dir("list_empty_is_silent");
    common::tickets(&dir, &["init"]);
    let out = common::tickets(&dir, &["list"]);
    assert!(out.status.success(), "list failed: {:?}", out);
    assert_eq!(out.stdout, b"", "expected no output, got: {:?}", out.stdout);
}

#[test]
fn list_shows_one_ticket() {
    let dir = common::test_dir("list_shows_one_ticket");
    common::tickets(&dir, &["init"]);
    let (id, _) = common::create_ticket(&dir, "Fix login bug");

    let out = common::tickets(&dir, &["list"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected 1 line, got: {:?}", lines);
    assert!(
        lines[0].starts_with(&id),
        "line should start with id: {}",
        lines[0]
    );
    assert!(
        lines[0].contains("draft"),
        "line should contain status: {}",
        lines[0]
    );
    assert!(
        lines[0].contains("task"),
        "line should contain type: {}",
        lines[0]
    );
    assert!(
        lines[0].contains("Fix login bug"),
        "line should contain title: {}",
        lines[0]
    );
}

#[test]
fn list_shows_multiple_tickets() {
    let dir = common::test_dir("list_shows_multiple_tickets");
    common::tickets(&dir, &["init"]);
    common::create_ticket(&dir, "Fix login bug");
    common::create_ticket(&dir, "Add OAuth support");
    common::create_ticket(&dir, "Write docs");

    let out = common::tickets(&dir, &["list"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "expected 3 lines, got: {:?}", lines);

    let titles: Vec<&str> = lines.iter().map(|l| l.trim()).collect();
    assert!(titles.iter().any(|l| l.contains("Fix login bug")));
    assert!(titles.iter().any(|l| l.contains("Add OAuth support")));
    assert!(titles.iter().any(|l| l.contains("Write docs")));
}

#[test]
fn list_unblocked_includes_ticket_with_no_blockers() {
    let dir = common::test_dir("list_unblocked_includes_ticket_with_no_blockers");
    common::tickets(&dir, &["init"]);
    common::create_ticket(&dir, "Free-standing ticket");

    let out = common::tickets(&dir, &["list", "--unblocked"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected 1 line, got: {:?}", lines);
    assert!(lines[0].contains("Free-standing ticket"));
}

#[test]
fn list_unblocked_excludes_ticket_blocked_by_todo() {
    let dir = common::test_dir("list_unblocked_excludes_ticket_blocked_by_todo");
    common::tickets(&dir, &["init"]);
    let (blocker_id, _) = common::create_ticket(&dir, "Blocker still todo");
    common::tickets(&dir, &["edit", &blocker_id, "--status", "todo"]);
    common::tickets(
        &dir,
        &[
            "new",
            "--title",
            "Blocked ticket",
            "--blocked-by",
            &blocker_id,
        ],
    );

    let out = common::tickets(&dir, &["list", "--unblocked"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected 1 line, got: {:?}", lines);
    assert!(lines[0].contains("Blocker still todo"));
    assert!(!stdout.contains("Blocked ticket"));
}

#[test]
fn list_unblocked_includes_ticket_whose_only_blocker_is_done() {
    let dir = common::test_dir("list_unblocked_includes_ticket_whose_only_blocker_is_done");
    common::tickets(&dir, &["init"]);
    let (blocker_id, _) = common::create_ticket(&dir, "Blocker now done");
    common::tickets(&dir, &["edit", &blocker_id, "--status", "done"]);
    common::tickets(
        &dir,
        &[
            "new",
            "--title",
            "Unblocked ticket",
            "--blocked-by",
            &blocker_id,
        ],
    );

    let out = common::tickets(&dir, &["list", "--unblocked"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "expected 2 lines, got: {:?}", lines);
    assert!(stdout.contains("Blocker now done"));
    assert!(stdout.contains("Unblocked ticket"));
}

#[test]
fn list_unblocked_and_tag_composes() {
    let dir = common::test_dir("list_unblocked_and_tag_composes");
    common::tickets(&dir, &["init"]);

    // Matches both filters: tagged backend and no unfinished blockers.
    common::tickets(
        &dir,
        &[
            "new",
            "--title",
            "Backend and unblocked",
            "--tag",
            "backend",
        ],
    );

    // Tagged backend but blocked - excluded by --unblocked.
    let (blocker_id, _) = common::create_ticket(&dir, "Blocker still todo");
    common::tickets(&dir, &["edit", &blocker_id, "--status", "todo"]);
    common::tickets(
        &dir,
        &[
            "new",
            "--title",
            "Backend but blocked",
            "--tag",
            "backend",
            "--blocked-by",
            &blocker_id,
        ],
    );

    // Unblocked but not tagged backend - excluded by --tag.
    common::tickets(&dir, &["new", "--title", "Unblocked but untagged"]);

    let out = common::tickets(&dir, &["list", "--unblocked", "--tag", "backend"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected 1 line, got: {:?}", lines);
    assert!(lines[0].contains("Backend and unblocked"));
}

#[test]
fn list_unblocked_and_status_composes() {
    let dir = common::test_dir("list_unblocked_and_status_composes");
    common::tickets(&dir, &["init"]);

    // Matches both filters: status todo and no unfinished blockers.
    common::tickets(
        &dir,
        &["new", "--title", "Todo and unblocked", "--status", "todo"],
    );

    // Status todo but blocked - excluded by --unblocked. The blocker itself
    // is left at a non-todo, non-done status so it doesn't also match
    // --status todo.
    let (blocker_id, _) = common::create_ticket(&dir, "Blocker in progress");
    common::tickets(&dir, &["edit", &blocker_id, "--status", "in-progress"]);
    common::tickets(
        &dir,
        &[
            "new",
            "--title",
            "Todo but blocked",
            "--status",
            "todo",
            "--blocked-by",
            &blocker_id,
        ],
    );

    // Unblocked but not status todo - excluded by --status.
    common::tickets(
        &dir,
        &["new", "--title", "Unblocked but done", "--status", "done"],
    );

    let out = common::tickets(&dir, &["list", "--unblocked", "--status", "todo"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected 1 line, got: {:?}", lines);
    assert!(lines[0].contains("Todo and unblocked"));
}

#[test]
fn list_unblocked_and_repeated_tag_keeps_and_semantics() {
    let dir = common::test_dir("list_unblocked_and_repeated_tag_keeps_and_semantics");
    common::tickets(&dir, &["init"]);

    // Has both tags and no unfinished blockers - matches.
    common::tickets(
        &dir,
        &[
            "new",
            "--title",
            "Auth and API unblocked",
            "--tag",
            "auth",
            "--tag",
            "api",
        ],
    );

    // Only one of the two tags - excluded by AND semantics.
    common::tickets(&dir, &["new", "--title", "Auth only", "--tag", "auth"]);

    // Has both tags but blocked - excluded by --unblocked.
    let (blocker_id, _) = common::create_ticket(&dir, "Blocker still todo");
    common::tickets(&dir, &["edit", &blocker_id, "--status", "todo"]);
    common::tickets(
        &dir,
        &[
            "new",
            "--title",
            "Auth and API blocked",
            "--tag",
            "auth",
            "--tag",
            "api",
            "--blocked-by",
            &blocker_id,
        ],
    );

    let out = common::tickets(
        &dir,
        &["list", "--unblocked", "--tag", "auth", "--tag", "api"],
    );
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected 1 line, got: {:?}", lines);
    assert!(lines[0].contains("Auth and API unblocked"));
}

#[test]
fn list_shows_emoji_for_each_ticket_type() {
    let dir = common::test_dir("list_shows_emoji_for_each_ticket_type");
    common::tickets(&dir, &["init"]);
    common::tickets(&dir, &["new", "--title", "An epic", "--type", "epic"]);
    common::tickets(&dir, &["new", "--title", "A story", "--type", "story"]);
    common::tickets(&dir, &["new", "--title", "A task", "--type", "task"]);
    common::tickets(&dir, &["new", "--title", "A bug", "--type", "bug"]);

    let out = common::tickets(&dir, &["list"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains('🎯'), "missing epic emoji: {}", stdout);
    assert!(stdout.contains('📖'), "missing story emoji: {}", stdout);
    assert!(stdout.contains('🔧'), "missing task emoji: {}", stdout);
    assert!(stdout.contains('🐛'), "missing bug emoji: {}", stdout);
}

#[test]
fn list_type_word_still_present_alongside_emoji() {
    let dir = common::test_dir("list_type_word_still_present_alongside_emoji");
    common::tickets(&dir, &["init"]);
    common::tickets(&dir, &["new", "--title", "A bug", "--type", "bug"]);
    common::tickets(&dir, &["new", "--title", "A task", "--type", "task"]);

    let out = common::tickets(&dir, &["list"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let bug_line = stdout
        .lines()
        .find(|l| l.contains("A bug"))
        .expect("bug line present");
    assert!(
        bug_line.contains("bug"),
        "type word should still be present for filtering: {}",
        bug_line
    );
}

#[test]
fn list_title_column_aligns_across_mixed_type_widths() {
    let dir = common::test_dir("list_title_column_aligns_across_mixed_type_widths");
    common::tickets(&dir, &["init"]);
    // "bug" (short word) vs "story" (long word), each with its own emoji.
    common::tickets(&dir, &["new", "--title", "A bug", "--type", "bug"]);
    common::tickets(&dir, &["new", "--title", "A story", "--type", "story"]);

    let out = common::tickets(&dir, &["list"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "expected 2 lines, got: {:?}", lines);

    let bug_line = lines
        .iter()
        .find(|l| l.contains("A bug"))
        .expect("bug line present");
    let story_line = lines
        .iter()
        .find(|l| l.contains("A story"))
        .expect("story line present");

    // Column position, not string content: the display width (terminal
    // columns) of everything before the title text must match across rows.
    let title_start_col = |line: &str, title: &str| -> usize {
        let byte_idx = line.find(title).expect("title present in line");
        unicode_width::UnicodeWidthStr::width(&line[..byte_idx])
    };
    assert_eq!(
        title_start_col(bug_line, "A bug"),
        title_start_col(story_line, "A story"),
        "title column should start at the same display column regardless of emoji width:\n{}\n{}",
        bug_line,
        story_line
    );
}

#[test]
fn list_widest_type_column_has_no_wasted_padding() {
    // Row-to-row alignment alone can pass even against the byte-length bug
    // for this glyph set: every type column carries exactly one emoji, so a
    // byte/char measure mismatch adds the same *uniform* extra padding to
    // every row and cancels out in relative terms. What it does NOT cancel
    // out is the *absolute* gap after the widest column - a byte-length
    // width computation treats a 4-byte, 2-column-wide emoji as needing 4
    // display columns, so it over-pads the row that defines the column
    // width by the byte/char difference. Assert that gap is tight instead.
    let dir = common::test_dir("list_widest_type_column_has_no_wasted_padding");
    common::tickets(&dir, &["init"]);
    // "🎯 epic" (7 display columns) is wider than "🐛 bug" (6), so epic's
    // row drives the column width. Titles avoid the word "epic"/"bug" so
    // the first occurrence found is unambiguously the type word.
    common::tickets(&dir, &["new", "--title", "Ticket one", "--type", "epic"]);
    common::tickets(&dir, &["new", "--title", "Ticket two", "--type", "bug"]);

    let out = common::tickets(&dir, &["list"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let epic_line = stdout
        .lines()
        .find(|l| l.contains("Ticket one"))
        .expect("epic line present");

    let word_end = epic_line.find("epic").expect("type word present") + "epic".len();
    let title_start = epic_line.find("Ticket one").expect("title present");
    let gap = &epic_line[word_end..title_start];
    assert_eq!(
        gap, "  ",
        "widest type column should be followed by exactly the two-space \
         column separator, not extra padding from a byte-length width \
         calculation: {:?}",
        epic_line
    );
}

#[test]
fn list_sorted_by_status_then_created_at() {
    let dir = common::test_dir("list_sorted_by_status_then_created_at");
    common::tickets(&dir, &["init"]);

    // Create in reverse desired order: draft first, then in-progress
    let (id_draft, _) = common::create_ticket(&dir, "Draft ticket");
    common::tickets(&dir, &["edit", &id_draft, "--status", "draft"]);

    let (id_todo, _) = common::create_ticket(&dir, "Todo ticket");
    common::tickets(&dir, &["edit", &id_todo, "--status", "todo"]);

    let (id_active, _) = common::create_ticket(&dir, "Active ticket");
    common::tickets(&dir, &["edit", &id_active, "--status", "in-progress"]);

    let out = common::tickets(&dir, &["list"]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);

    // in-progress first, then todo, then draft
    assert!(
        lines[0].contains("Active ticket"),
        "first should be in-progress: {}",
        lines[0]
    );
    assert!(
        lines[1].contains("Todo ticket"),
        "second should be todo: {}",
        lines[1]
    );
    assert!(
        lines[2].contains("Draft ticket"),
        "third should be draft: {}",
        lines[2]
    );
}

// ── colour ───────────────────────────────────────────────────────────────

#[test]
fn list_piped_output_has_no_ansi_escapes() {
    // The test harness always pipes stdout (`std::process::Command::output`
    // never allocates a pty), so this exercises the default, script-facing
    // path with no env-var gymnastics required.
    let dir = common::test_dir("list_piped_output_has_no_ansi_escapes");
    common::tickets(&dir, &["init"]);
    common::create_ticket(&dir, "Done ticket");
    let (id, _) = common::create_ticket(&dir, "Done ticket two");
    common::tickets(&dir, &["edit", &id, "--status", "done"]);

    let out = common::tickets(&dir, &["list"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains('\u{1b}'),
        "piped output must contain no ANSI escape bytes: {:?}",
        stdout
    );
}

#[test]
fn list_piped_output_is_byte_identical_to_the_uncoloured_format() {
    // Without any colour-forcing env var, a pipe is not a terminal, so the
    // default path must produce exactly the pre-colour row format — proving
    // the change is invisible to scripts that don't opt in with
    // `CLICOLOR_FORCE`. Assert against a hand-built expected string (rather
    // than comparing two live runs) so a regression that coloured the
    // default pipe path would be caught even though it "looks the same as
    // itself".
    let dir = common::test_dir("list_piped_output_is_byte_identical_to_the_uncoloured_format");
    common::tickets(&dir, &["init"]);
    let (id_done, _) = common::create_ticket(&dir, "Ticket one");
    common::tickets(&dir, &["edit", &id_done, "--status", "done"]);
    let (id_draft, _) = common::create_ticket(&dir, "Ticket two");

    let out = common::tickets(&dir, &["list"]);
    assert!(out.status.success(), "list failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);

    // Widths: id is always 6 chars; status column is `max(len, 6)` (both
    // "draft" and "done" are shorter, so it's 6); type column is
    // `max(display_width("🔧 task"), 4)` = 7 (emoji is 2 display columns).
    let expected = format!(
        "{id_draft}  draft   🔧 task  Ticket two\n{id_done}  done    🔧 task  Ticket one\n",
    );
    assert_eq!(
        stdout, expected,
        "piped output must match the plain, uncoloured row format byte for byte"
    );
}

#[test]
fn list_no_color_suppresses_escapes_even_when_forced() {
    // NO_COLOR must win over CLICOLOR_FORCE when both are set.
    let dir = common::test_dir("list_no_color_suppresses_escapes_even_when_forced");
    common::tickets(&dir, &["init"]);
    let (id, _) = common::create_ticket(&dir, "Done ticket");
    common::tickets(&dir, &["edit", &id, "--status", "done"]);

    let out = common::tickets_with_envs(
        &dir,
        &["list"],
        &[("NO_COLOR", "1"), ("CLICOLOR_FORCE", "1")],
    );
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains('\u{1b}'),
        "NO_COLOR must suppress escapes even with CLICOLOR_FORCE set: {:?}",
        stdout
    );
}

#[test]
fn list_forced_colour_marks_done_green_and_review_purple() {
    let dir = common::test_dir("list_forced_colour_marks_done_green_and_review_purple");
    common::tickets(&dir, &["init"]);
    let (id_done, _) = common::create_ticket(&dir, "Done ticket");
    common::tickets(&dir, &["edit", &id_done, "--status", "done"]);
    let (id_review, _) = common::create_ticket(&dir, "Review ticket");
    common::tickets(&dir, &["edit", &id_review, "--status", "review"]);

    let out = common::tickets_with_envs(&dir, &["list"], &[("CLICOLOR_FORCE", "1")]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let done_line = stdout
        .lines()
        .find(|l| l.contains("Done ticket"))
        .expect("done line present");
    let review_line = stdout
        .lines()
        .find(|l| l.contains("Review ticket"))
        .expect("review line present");

    assert!(
        done_line.contains("\u{1b}[32m"),
        "done row should carry the green code: {:?}",
        done_line
    );
    assert!(
        review_line.contains("\u{1b}[35m"),
        "review row should carry the purple (magenta) code: {:?}",
        review_line
    );
}

#[test]
fn list_forced_colour_leaves_draft_and_todo_plain() {
    let dir = common::test_dir("list_forced_colour_leaves_draft_and_todo_plain");
    common::tickets(&dir, &["init"]);
    let (id_todo, _) = common::create_ticket(&dir, "Todo ticket");
    common::tickets(&dir, &["edit", &id_todo, "--status", "todo"]);
    common::create_ticket(&dir, "Draft ticket"); // draft is the default status

    let out = common::tickets_with_envs(&dir, &["list"], &[("CLICOLOR_FORCE", "1")]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains('\u{1b}'),
        "draft and todo rows should carry no escapes even with colour forced: {:?}",
        stdout
    );
}

#[test]
fn list_forced_colour_keeps_title_column_aligned_by_position() {
    // Column position, not whole-line string compare, is the trustworthy way
    // to assert alignment survives colouring: a padding-after-colouring bug
    // would make every row a different length but the naive "does it look
    // right" check on colourless output would never catch it.
    let dir = common::test_dir("list_forced_colour_keeps_title_column_aligned_by_position");
    common::tickets(&dir, &["init"]);
    let (id_done, _) = common::create_ticket(&dir, "Alpha title");
    common::tickets(&dir, &["edit", &id_done, "--status", "done"]);
    let (id_review, _) = common::create_ticket(&dir, "Beta title");
    common::tickets(&dir, &["edit", &id_review, "--status", "review"]);
    common::create_ticket(&dir, "Gamma title"); // draft, uncoloured

    let out = common::tickets_with_envs(&dir, &["list"], &[("CLICOLOR_FORCE", "1")]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);

    // Compare the *stripped* (colour-code-free) column position of each
    // title, since colour codes shift raw byte offsets but must not shift
    // the rendered column a real terminal would show. Find each row by
    // content rather than assuming an index — `list` sorts by status, so
    // the coloured `done` and `review` rows are not necessarily first.
    let stripped: Vec<String> = lines.iter().map(|l| strip_ansi(l)).collect();
    let find_col = |needle: &str| {
        stripped
            .iter()
            .find_map(|l| l.find(needle))
            .unwrap_or_else(|| panic!("{needle} not found in output: {stripped:?}"))
    };
    let alpha_col = find_col("Alpha title");
    let beta_col = find_col("Beta title");
    let gamma_col = find_col("Gamma title");
    assert_eq!(
        alpha_col, beta_col,
        "coloured (done) and coloured (review) rows must align"
    );
    assert_eq!(
        beta_col, gamma_col,
        "coloured and uncoloured rows must align"
    );
}

/// Strip ANSI CSI escape sequences (`\x1b[...m`) from `s`, for asserting on
/// visible column position independent of colour codes.
fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // Consume `[`, then digits/`;`, then the terminating letter.
            if chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
                continue;
            }
        }
        out.push(c);
    }
    out
}

// ── broken pipe ──────────────────────────────────────────────────────────

/// Pipe `tickets list`'s stdout into `head -n1`, which reads one line then
/// closes its end of the pipe. Returns `list`'s exit status and stderr.
fn list_piped_into_head_one(dir: &std::path::Path) -> (std::process::ExitStatus, String) {
    let mut list = Command::new(common::bin())
        .args(["list"])
        .env("TICKETS_DIR", dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn tickets list");

    let list_stdout = list.stdout.take().expect("list stdout piped");
    let mut head = Command::new("head")
        .args(["-n1"])
        .stdin(Stdio::from(list_stdout))
        .stdout(Stdio::null())
        .spawn()
        .expect("failed to spawn head");

    head.wait().expect("head did not run");
    let output = list.wait_with_output().expect("list did not run");
    (
        output.status,
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn list_piped_into_head_exits_cleanly_with_no_stderr() {
    // `tickets list | head -1` is exactly the pattern the ticket calls out —
    // scripts pipe `list` into `rg`, `wc`, `awk` and friends, all of which
    // may stop reading before `list` finishes writing. `list` must not treat
    // the reader closing early as its own failure.
    let dir = common::test_dir("list_piped_into_head_exits_cleanly_with_no_stderr");
    common::tickets(&dir, &["init"]);
    for i in 0..10 {
        common::create_ticket(&dir, &format!("Ticket {i}"));
    }

    let (status, stderr) = list_piped_into_head_one(&dir);
    assert!(
        status.success(),
        "list should exit 0 when its reader closes early, got {status:?}"
    );
    assert_eq!(
        stderr, "",
        "list should print nothing to stderr on a broken pipe"
    );
}

#[test]
fn list_piped_into_head_exits_cleanly_with_large_output() {
    // The failure is buffer-size dependent, not row-count dependent: a small
    // listing can finish writing before `head` closes its end of the pipe,
    // masking the bug. A large listing guarantees `list` is still writing
    // when the pipe closes.
    let dir = common::test_dir("list_piped_into_head_exits_cleanly_with_large_output");
    common::tickets(&dir, &["init"]);
    for i in 0..500 {
        common::create_ticket(&dir, &format!("Ticket number {i} with a longer title"));
    }

    let (status, stderr) = list_piped_into_head_one(&dir);
    assert!(
        status.success(),
        "list should exit 0 when its reader closes early, got {status:?}"
    );
    assert_eq!(
        stderr, "",
        "list should print nothing to stderr on a broken pipe"
    );
}
