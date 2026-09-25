use std::path::Path;

mod common;

/// Create a ticket with an explicit `todo` status (the epic's example
/// scenarios are all written against `todo`, but `new`'s default status is
/// `draft`).
fn create_todo_ticket(dir: &Path, title: &str) -> String {
    let out = common::tickets(dir, &["new", "--title", title, "--status", "todo"]);
    assert!(out.status.success(), "new failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout.trim().split(' ').next().unwrap().to_string()
}

#[test]
fn linear_chain_renders_nested_deepest_last() {
    let dir = common::test_dir("deps_graph_linear_chain_renders_nested_deepest_last");
    common::tickets(&dir, &["init"]);
    let id_a = create_todo_ticket(&dir, "Set up DB schema");
    let id_b = create_todo_ticket(&dir, "Write migration");
    let id_c = create_todo_ticket(&dir, "Run migration in CI");
    // a1 blocks b2, b2 blocks c3
    common::tickets(&dir, &["edit", &id_b, "--blocked-by", &id_a]);
    common::tickets(&dir, &["edit", &id_c, "--blocked-by", &id_b]);

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let expected = format!(
        "{id_a}  todo  Set up DB schema\n└── {id_b}  todo  Write migration\n    └── {id_c}  todo  Run migration in CI\n"
    );
    assert_eq!(stdout, expected);
}

#[test]
fn empty_tickets_dir_prints_nothing_and_exits_zero() {
    let dir = common::test_dir("deps_graph_empty_tickets_dir_prints_nothing_and_exits_zero");
    common::tickets(&dir, &["init"]);

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);
    assert_eq!(out.stdout, b"");
}

#[test]
fn blocker_archived_and_done_renders_dependent_as_root() {
    let dir = common::test_dir("deps_graph_blocker_archived_and_done_renders_dependent_as_root");
    common::tickets(&dir, &["init"]);
    let blocker = create_todo_ticket(&dir, "Old finished setup");
    common::tickets(&dir, &["edit", &blocker, "--status", "done"]);
    let out = common::tickets(&dir, &["archive", &blocker]);
    assert!(out.status.success(), "archive failed: {:?}", out);
    let dependent = create_todo_ticket(&dir, "Build on top");
    common::tickets(&dir, &["edit", &dependent, "--blocked-by", &blocker]);

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let expected = format!("{dependent}  todo  Build on top\n");
    assert_eq!(stdout, expected);
}

#[test]
fn blocker_archived_and_done_never_appears_as_a_node_or_label() {
    let dir =
        common::test_dir("deps_graph_blocker_archived_and_done_never_appears_as_node_or_label");
    common::tickets(&dir, &["init"]);
    let blocker = create_todo_ticket(&dir, "Old finished setup");
    common::tickets(&dir, &["edit", &blocker, "--status", "done"]);
    let out = common::tickets(&dir, &["archive", &blocker]);
    assert!(out.status.success(), "archive failed: {:?}", out);
    let dependent = create_todo_ticket(&dir, "Build on top");
    common::tickets(&dir, &["edit", &dependent, "--blocked-by", &blocker]);

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains(blocker.as_str()),
        "archived done blocker id must not appear anywhere: {stdout}"
    );
    assert!(
        !stdout.contains("Old finished setup"),
        "archived done blocker title must not appear anywhere: {stdout}"
    );
}

// --- xptucc: archived non-done blocker stub ---

#[test]
fn archived_non_done_blocker_keeps_dependent_blocked_and_renders_stub_root() {
    let dir = common::test_dir(
        "deps_graph_archived_non_done_blocker_keeps_dependent_blocked_and_renders_stub_root",
    );
    common::tickets(&dir, &["init"]);
    let blocker = create_todo_ticket(&dir, "Old rejected setup");
    common::tickets(&dir, &["edit", &blocker, "--status", "rejected"]);
    let out = common::tickets(&dir, &["archive", &blocker]);
    assert!(out.status.success(), "archive failed: {:?}", out);
    let dependent = create_todo_ticket(&dir, "Migrate to new queue");
    common::tickets(&dir, &["edit", &dependent, "--blocked-by", &blocker]);

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let expected =
        format!("[archived: {blocker} rejected]\n└── {dependent}  todo  Migrate to new queue\n");
    assert_eq!(stdout, expected);
}

#[test]
fn archived_stub_names_both_the_archived_ticket_id_and_its_status() {
    let dir = common::test_dir(
        "deps_graph_archived_stub_names_both_the_archived_ticket_id_and_its_status",
    );
    common::tickets(&dir, &["init"]);
    let blocker = create_todo_ticket(&dir, "Abandoned approach");
    common::tickets(&dir, &["edit", &blocker, "--status", "rejected"]);
    common::tickets(&dir, &["archive", &blocker]);
    let dependent = create_todo_ticket(&dir, "Depends on abandoned work");
    common::tickets(&dir, &["edit", &dependent, "--blocked-by", &blocker]);

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&format!("[archived: {blocker} rejected]")),
        "stub must name both the archived id and its status: {stdout}"
    );
}
