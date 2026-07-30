mod common;

#[test]
fn graph_errors_when_not_initialised() {
    let dir = common::test_dir("graph_errors_when_not_initialised");
    let out = common::tickets(&dir, &["graph"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("not initialised"), "unexpected stderr: {}", stderr);
}

#[test]
fn graph_empty_is_silent() {
    let dir = common::test_dir("graph_empty_is_silent");
    common::tickets(&dir, &["init"]);
    let out = common::tickets(&dir, &["graph"]);
    assert!(out.status.success(), "graph failed: {:?}", out);
    assert_eq!(out.stdout, b"");
}

#[test]
fn graph_shows_single_ticket_as_root() {
    let dir = common::test_dir("graph_shows_single_ticket_as_root");
    common::tickets(&dir, &["init"]);
    let (id, _) = common::create_ticket(&dir, "Fix login bug");

    let out = common::tickets(&dir, &["graph"]);
    assert!(out.status.success(), "graph failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains(&id), "expected id in output: {}", stdout);
    assert!(stdout.contains("Fix login bug"), "expected title: {}", stdout);
}

#[test]
fn graph_with_id_shows_tree_rooted_at_ticket() {
    let dir = common::test_dir("graph_with_id_shows_tree_rooted_at_ticket");
    common::tickets(&dir, &["init"]);
    let (id_a, _) = common::create_ticket(&dir, "Ticket A");
    let (id_b, _) = common::create_ticket(&dir, "Ticket B");
    // Make B blocked by A
    common::tickets(&dir, &["edit", &id_b, "--blocked-by", &id_a]);

    let out = common::tickets(&dir, &["graph", &id_b]);
    assert!(out.status.success(), "graph failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Ticket B"), "expected Ticket B: {}", stdout);
    assert!(stdout.contains("Ticket A"), "expected Ticket A as blocker: {}", stdout);
}

#[test]
fn graph_warns_on_dependency_cycle() {
    let dir = common::test_dir("graph_warns_on_dependency_cycle");
    common::tickets(&dir, &["init"]);
    let (id_a, _) = common::create_ticket(&dir, "Ticket A");
    let (id_b, _) = common::create_ticket(&dir, "Ticket B");
    common::tickets(&dir, &["edit", &id_a, "--blocked-by", &id_b]);
    common::tickets(&dir, &["edit", &id_b, "--blocked-by", &id_a]);

    let out = common::tickets(&dir, &["graph"]);
    assert!(out.status.success(), "graph failed: {:?}", out);

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("dependency cycle"), "expected cycle warning: {}", stderr);
    assert!(stderr.contains(&id_a), "expected id_a in warning: {}", stderr);
    assert!(stderr.contains(&id_b), "expected id_b in warning: {}", stderr);
}

#[test]
fn graph_acyclic_has_no_cycle_warning() {
    let dir = common::test_dir("graph_acyclic_has_no_cycle_warning");
    common::tickets(&dir, &["init"]);
    let (id_a, _) = common::create_ticket(&dir, "Ticket A");
    let (id_b, _) = common::create_ticket(&dir, "Ticket B");
    common::tickets(&dir, &["edit", &id_b, "--blocked-by", &id_a]);

    let out = common::tickets(&dir, &["graph"]);
    assert!(out.status.success(), "graph failed: {:?}", out);

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("dependency cycle"), "unexpected cycle warning: {}", stderr);
}

#[test]
fn graph_blocked_ticket_not_a_forest_root() {
    let dir = common::test_dir("graph_blocked_ticket_not_a_forest_root");
    common::tickets(&dir, &["init"]);
    let (id_a, _) = common::create_ticket(&dir, "Ticket A");
    let (id_b, _) = common::create_ticket(&dir, "Ticket B");
    common::tickets(&dir, &["edit", &id_b, "--blocked-by", &id_a]);

    let out = common::tickets(&dir, &["graph"]);
    assert!(out.status.success(), "graph failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    // Only A should appear as a root-level line (no leading whitespace/connector)
    let root_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| !l.starts_with(' ') && !l.starts_with('│') && !l.starts_with('├') && !l.starts_with('└'))
        .collect();
    assert_eq!(root_lines.len(), 1, "expected 1 root, got: {:?}", root_lines);
    assert!(root_lines[0].contains("Ticket A"), "root should be A: {:?}", root_lines);
}
