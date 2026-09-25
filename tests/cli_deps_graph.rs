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

// --- xptucc / t9p76e: out-of-active-set stub nodes ---

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

#[test]
fn missing_blocker_id_renders_as_missing_stub_with_dependent_nested() {
    let dir = common::test_dir(
        "deps_graph_missing_blocker_id_renders_as_missing_stub_with_dependent_nested",
    );
    common::tickets(&dir, &["init"]);
    let dependent = create_todo_ticket(&dir, "Migrate to new queue");
    common::tickets(&dir, &["edit", &dependent, "--blocked-by", "zz9999"]);

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let expected = format!("[missing: zz9999]\n└── {dependent}  todo  Migrate to new queue\n");
    assert_eq!(stdout, expected);
}

#[test]
fn missing_and_archived_stubs_are_visually_distinct_from_each_other() {
    let dir = common::test_dir(
        "deps_graph_missing_and_archived_stubs_are_visually_distinct_from_each_other",
    );
    common::tickets(&dir, &["init"]);
    let archived_blocker = create_todo_ticket(&dir, "Rejected old approach");
    common::tickets(&dir, &["edit", &archived_blocker, "--status", "rejected"]);
    common::tickets(&dir, &["archive", &archived_blocker]);
    let dependent_a = create_todo_ticket(&dir, "Depends on rejected work");
    common::tickets(
        &dir,
        &["edit", &dependent_a, "--blocked-by", &archived_blocker],
    );
    let dependent_b = create_todo_ticket(&dir, "Depends on unknown id");
    common::tickets(&dir, &["edit", &dependent_b, "--blocked-by", "zz9999"]);

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let archived_stub = format!("[archived: {archived_blocker} rejected]");
    let missing_stub = "[missing: zz9999]";
    assert!(
        stdout.contains(&archived_stub),
        "expected archived stub in output: {stdout}"
    );
    assert!(
        stdout.contains(missing_stub),
        "expected missing stub in output: {stdout}"
    );
    assert_ne!(
        archived_stub, missing_stub,
        "the two stub forms must render differently from each other"
    );
}

// --- 2k12y3 / 9b88c0 / 3yew0k / hx1po6: fan-out ordering, disjoint chains,
// repeated multi-blocker tickets, and cycle handling. ---

/// Find the ticket file for `id` under `dir/all` and overwrite its
/// `created_at` front-matter value. Used to force a created_at ordering
/// that disagrees with id ordering — there is no CLI subcommand for
/// backdating a ticket, so this is a direct file edit, test-only.
fn set_created_at(dir: &Path, id: &str, rfc3339: &str) {
    let entry = std::fs::read_dir(dir.join("all"))
        .unwrap()
        .flatten()
        .find(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(&format!("{id}_"))
        })
        .unwrap_or_else(|| panic!("no ticket file found for {id}"));
    let path = entry.path();
    let contents = std::fs::read_to_string(&path).unwrap();
    let updated: String = contents
        .lines()
        .map(|line| {
            if line.starts_with("created_at:") {
                format!("created_at: {rfc3339}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(&path, updated).unwrap();
}

#[test]
fn fan_out_renders_siblings_ordered_by_created_at_not_id() {
    let dir = common::test_dir("deps_graph_fan_out_ordered_by_created_at_not_id");
    common::tickets(&dir, &["init"]);
    let root = create_todo_ticket(&dir, "Ship shared auth lib");
    let first_created = create_todo_ticket(&dir, "Add login screen");
    let second_created = create_todo_ticket(&dir, "Add SSO integration");
    common::tickets(&dir, &["edit", &first_created, "--blocked-by", &root]);
    common::tickets(&dir, &["edit", &second_created, "--blocked-by", &root]);

    // Force id ordering to disagree with created_at ordering: whichever of
    // the two ids sorts first alphabetically gets the *later* created_at,
    // so a render that (wrongly) sorted by id would put the children in
    // the opposite order to a render that correctly sorts by created_at.
    let (alphabetically_first, alphabetically_second) = if first_created < second_created {
        (first_created.clone(), second_created.clone())
    } else {
        (second_created.clone(), first_created.clone())
    };
    let earlier_id = alphabetically_second;
    let later_id = alphabetically_first;
    set_created_at(&dir, &earlier_id, "2026-01-01T00:00:00Z");
    set_created_at(&dir, &later_id, "2026-06-01T00:00:00Z");

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);

    let earlier_pos = stdout.find(&earlier_id).expect("earlier id in output");
    let later_pos = stdout.find(&later_id).expect("later id in output");
    assert!(
        earlier_pos < later_pos,
        "expected id with earlier created_at ({earlier_id}) to render before \
         the id with later created_at ({later_id}), which is alphabetically \
         earlier: {stdout}"
    );
    assert!(
        stdout.contains(&format!("├── {earlier_id}")),
        "expected earlier sibling to use non-last connector: {stdout}"
    );
    assert!(
        stdout.contains(&format!("└── {later_id}")),
        "expected later sibling to use last connector: {stdout}"
    );
}

#[test]
fn disjoint_chains_render_as_two_roots_with_no_connecting_lines() {
    let dir = common::test_dir("deps_graph_disjoint_chains_render_as_two_roots");
    common::tickets(&dir, &["init"]);
    let root_a = create_todo_ticket(&dir, "Chain1 Root");
    let leaf_a = create_todo_ticket(&dir, "Chain1 Leaf");
    let root_b = create_todo_ticket(&dir, "Chain2 Root");
    let leaf_b = create_todo_ticket(&dir, "Chain2 Leaf");
    common::tickets(&dir, &["edit", &leaf_a, "--blocked-by", &root_a]);
    common::tickets(&dir, &["edit", &leaf_b, "--blocked-by", &root_b]);

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);

    let expected = format!(
        "{root_a}  todo  Chain1 Root\n└── {leaf_a}  todo  Chain1 Leaf\n\
         {root_b}  todo  Chain2 Root\n└── {leaf_b}  todo  Chain2 Leaf\n"
    );
    assert_eq!(stdout, expected);
}

#[test]
fn orphan_ticket_renders_as_a_lone_root_line() {
    let dir = common::test_dir("deps_graph_orphan_renders_as_lone_root_line");
    common::tickets(&dir, &["init"]);
    let orphan = create_todo_ticket(&dir, "Lone");

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert_eq!(stdout, format!("{orphan}  todo  Lone\n"));
}

#[test]
fn root_selection_excludes_tickets_with_an_incoming_blocker_edge() {
    let dir = common::test_dir("deps_graph_root_selection_excludes_blocked_tickets");
    common::tickets(&dir, &["init"]);
    let root = create_todo_ticket(&dir, "Root");
    let leaf = create_todo_ticket(&dir, "Leaf");
    common::tickets(&dir, &["edit", &leaf, "--blocked-by", &root]);

    // Force the leaf's created_at earlier than its own blocker's. A roots()
    // that (wrongly) treats every node as a root -- rather than only
    // zero-incoming-edge nodes -- would sort the leaf ahead of its root and
    // hand it out as a top-level entry before the real root ever gets a
    // chance to nest it underneath. This is deliberately independent of the
    // rendered-set dedup added later (3yew0k/hx1po6): correct root
    // selection must never place a blocked ticket at column 0, regardless
    // of created_at, purely because it has an incoming blocker edge.
    set_created_at(&dir, &leaf, "2020-01-01T00:00:00Z");
    set_created_at(&dir, &root, "2020-06-01T00:00:00Z");

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);

    // Top-level (column 0) lines are the ones with no leading indentation
    // or tree-drawing connector — i.e. actual roots.
    let top_level_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.starts_with(' ') && !line.starts_with('├') && !line.starts_with('└'))
        .collect();

    let expected_root_line = format!("{root}  todo  Root");
    assert_eq!(
        top_level_lines,
        vec![expected_root_line.as_str()],
        "the blocked leaf must never appear as a top-level entry, regardless \
         of created_at: {stdout}"
    );
}

#[test]
fn diamond_renders_shared_ticket_under_both_blockers() {
    let dir = common::test_dir("deps_graph_diamond_renders_shared_ticket_under_both_blockers");
    common::tickets(&dir, &["init"]);
    let a = create_todo_ticket(&dir, "Set up DB schema");
    let c = create_todo_ticket(&dir, "Design API contract");
    let d = create_todo_ticket(&dir, "Run migration in CI");
    let f = create_todo_ticket(&dir, "Enable feature flag");
    common::tickets(&dir, &["edit", &d, "--blocked-by", &a, "--blocked-by", &c]);
    common::tickets(&dir, &["edit", &f, "--blocked-by", &d]);

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);

    // Full expansion under the first blocker (a, created first).
    assert!(
        stdout.contains(&format!(
            "{a}  todo  Set up DB schema\n└── {d}  todo  Run migration in CI\n    └── {f}  todo  Enable feature flag\n"
        )),
        "expected full subtree under first blocker: {stdout}"
    );
    // Second occurrence, under c, is a marked leaf and does not re-expand.
    assert!(
        stdout.contains(&format!(
            "{c}  todo  Design API contract\n└── {d}  todo  Run migration in CI  (see above)\n"
        )),
        "expected marked, non-expanding repeat under second blocker: {stdout}"
    );
    assert_eq!(
        stdout.matches(&f).count(),
        1,
        "the final ticket must only be printed once, under the first occurrence: {stdout}"
    );
}

#[test]
fn nested_diamonds_two_converging_pairs_feed_one_final_ticket() {
    let dir = common::test_dir("deps_graph_nested_diamonds_feed_one_final_ticket");
    common::tickets(&dir, &["init"]);
    // a/b -> m ; c/d -> n ; m/n -> z
    let a = create_todo_ticket(&dir, "A");
    let b = create_todo_ticket(&dir, "B");
    let c = create_todo_ticket(&dir, "C");
    let d = create_todo_ticket(&dir, "D");
    let m = create_todo_ticket(&dir, "M");
    let n = create_todo_ticket(&dir, "N");
    let z = create_todo_ticket(&dir, "Z");
    common::tickets(&dir, &["edit", &m, "--blocked-by", &a, "--blocked-by", &b]);
    common::tickets(&dir, &["edit", &n, "--blocked-by", &c, "--blocked-by", &d]);
    common::tickets(&dir, &["edit", &z, "--blocked-by", &m, "--blocked-by", &n]);

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);

    // z's full subtree should be expanded exactly once (under m, its first
    // blocker by created_at), and m/n each individually should also only
    // fully expand once, each under their own first blocker.
    assert_eq!(
        stdout.matches(&z).count(),
        2,
        "z is a leaf of both m and n, so it should appear as a marked repeat \
         once and be expanded once: {stdout}"
    );
    assert!(
        stdout.contains("  (see above)\n"),
        "expected at least one (see above) marker: {stdout}"
    );
    // m and n each appear exactly twice (once fully expanded, once as a
    // marked repeat) since each has two blockers.
    assert_eq!(
        stdout.matches(&m).count(),
        2,
        "m should appear twice: {stdout}"
    );
    assert_eq!(
        stdout.matches(&n).count(),
        2,
        "n should appear twice: {stdout}"
    );
}

#[test]
fn three_ticket_cycle_renders_every_id_without_hanging() {
    let dir = common::test_dir("deps_graph_three_ticket_cycle_renders_without_hanging");
    common::tickets(&dir, &["init"]);
    let a = create_todo_ticket(&dir, "A");
    let b = create_todo_ticket(&dir, "B");
    let c = create_todo_ticket(&dir, "C");
    common::tickets(&dir, &["edit", &a, "--blocked-by", &c]);
    common::tickets(&dir, &["edit", &b, "--blocked-by", &a]);
    common::tickets(&dir, &["edit", &c, "--blocked-by", &b]);

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);
    assert_eq!(out.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&out.stdout);
    for id in [&a, &b, &c] {
        assert!(
            stdout.contains(id.as_str()),
            "expected cycle member {id} to appear in output: {stdout}"
        );
    }
    assert!(
        stdout.contains("(cycle: see above)"),
        "expected a cycle marker: {stdout}"
    );
}

#[test]
fn cycle_warning_names_every_id_on_stderr_with_exit_zero() {
    let dir = common::test_dir("deps_graph_cycle_warning_names_every_id_on_stderr");
    common::tickets(&dir, &["init"]);
    let a = create_todo_ticket(&dir, "A");
    let b = create_todo_ticket(&dir, "B");
    common::tickets(&dir, &["edit", &a, "--blocked-by", &b]);
    common::tickets(&dir, &["edit", &b, "--blocked-by", &a]);

    let out = common::tickets(&dir, &["deps-graph"]);
    assert_eq!(out.status.code(), Some(0));

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains(a.as_str()) && stderr.contains(b.as_str()),
        "expected both cycle ids named on stderr: {stderr}"
    );
}

#[test]
fn pure_two_ticket_cycle_with_no_root_still_renders_both() {
    let dir = common::test_dir("deps_graph_pure_two_ticket_cycle_renders_both");
    common::tickets(&dir, &["init"]);
    let a = create_todo_ticket(&dir, "A");
    let b = create_todo_ticket(&dir, "B");
    common::tickets(&dir, &["edit", &a, "--blocked-by", &b]);
    common::tickets(&dir, &["edit", &b, "--blocked-by", &a]);

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(
        !stdout.is_empty(),
        "cycle members must not be silently omitted"
    );
    assert!(
        stdout.contains(a.as_str()),
        "expected {a} in output: {stdout}"
    );
    assert!(
        stdout.contains(b.as_str()),
        "expected {b} in output: {stdout}"
    );
}

#[test]
fn cycle_reachable_from_an_external_root_does_not_crash() {
    let dir = common::test_dir("deps_graph_cycle_reachable_from_external_root_does_not_crash");
    common::tickets(&dir, &["init"]);
    let root = create_todo_ticket(&dir, "Root");
    let x = create_todo_ticket(&dir, "X");
    let y = create_todo_ticket(&dir, "Y");
    common::tickets(
        &dir,
        &["edit", &x, "--blocked-by", &root, "--blocked-by", &y],
    );
    common::tickets(&dir, &["edit", &y, "--blocked-by", &x]);

    let out = common::tickets(&dir, &["deps-graph"]);
    assert!(out.status.success(), "deps-graph failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(stdout.contains(root.as_str()));
    assert!(stdout.contains(x.as_str()));
    assert!(stdout.contains(y.as_str()));
    assert!(stdout.contains("(cycle: see above)"));
}
