mod common;

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
