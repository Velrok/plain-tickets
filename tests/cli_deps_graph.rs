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
