mod common;
use common::*;

#[test]
fn list_prints_nothing_when_there_are_no_tickets() {
    let dir = initialised("list-empty");
    let out = tickets(&dir, &["list"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "");
}

#[test]
fn list_prints_one_tab_separated_line_per_ticket() {
    let dir = initialised("list-line");
    let id = new_id(&dir, &["Fix it", "-s", "in progress", "-t", "bug"]);
    let out = tickets(&dir, &["list"]);
    assert_eq!(stdout(&out), format!("{id}\tin progress\tbug\tFix it"));
}

fn titles(dir: &std::path::Path, args: &[&str]) -> Vec<String> {
    let mut full = vec!["list"];
    full.extend(args);
    let out = tickets(dir, &full);
    assert!(out.status.success(), "{}", stderr(&out));
    stdout(&out)
        .lines()
        .map(|l| l.rsplit('\t').next().unwrap().to_string())
        .collect()
}

#[test]
fn list_orders_by_configured_status_then_done_then_rejected() {
    let dir = initialised("list-order");
    new_id(&dir, &["Rejected one", "-s", "rejected"]);
    new_id(&dir, &["Done one", "-s", "done"]);
    new_id(&dir, &["Doing one", "-s", "in progress"]);
    new_id(&dir, &["Todo one", "-s", "todo"]);
    assert_eq!(
        titles(&dir, &[]),
        ["Todo one", "Doing one", "Done one", "Rejected one"]
    );
}

#[test]
fn list_status_filter_is_repeatable_with_or_semantics() {
    let dir = initialised("list-status");
    new_id(&dir, &["Todo one", "-s", "todo"]);
    new_id(&dir, &["Doing one", "-s", "in progress"]);
    new_id(&dir, &["Done one", "-s", "done"]);
    assert_eq!(titles(&dir, &["-s", "Done"]), ["Done one"]);
    assert_eq!(
        titles(&dir, &["-s", "todo", "--status", "done"]),
        ["Todo one", "Done one"]
    );
}

#[test]
fn list_tag_filter_is_repeatable_with_or_semantics() {
    let dir = initialised("list-tag");
    new_id(&dir, &["Alpha", "-g", "a"]);
    new_id(&dir, &["Beta", "-g", "b", "-g", "c"]);
    new_id(&dir, &["Plain"]);
    assert_eq!(titles(&dir, &["-g", "a"]), ["Alpha"]);
    let mut both = titles(&dir, &["-g", "a", "--tag", "c"]);
    both.sort();
    assert_eq!(both, ["Alpha", "Beta"]);
}

#[test]
fn list_type_filter_matches_the_ticket_type() {
    let dir = initialised("list-type");
    new_id(&dir, &["A bug", "-t", "bug"]);
    new_id(&dir, &["A task", "-t", "task"]);
    new_id(&dir, &["Untyped"]);
    assert_eq!(titles(&dir, &["-t", "bug"]), ["A bug"]);
}

#[test]
fn list_parent_filter_matches_direct_children() {
    let dir = initialised("list-parent");
    let parent = new_id(&dir, &["Parent"]);
    new_id(&dir, &["Child", "-p", &parent]);
    new_id(&dir, &["Orphan"]);
    assert_eq!(titles(&dir, &["-p", &parent]), ["Child"]);
}

#[test]
fn list_blocked_shows_tickets_with_unfinished_blockers() {
    let dir = initialised("list-blocked");
    let open = new_id(&dir, &["Open blocker"]);
    let done = new_id(&dir, &["Finished blocker", "-s", "done"]);
    new_id(&dir, &["Waiting", "-b", &open]);
    new_id(&dir, &["Mixed", "-b", &done, "-b", &open]);
    new_id(&dir, &["Unblocked", "-b", &done]);
    new_id(&dir, &["Free"]);
    let mut blocked = titles(&dir, &["--blocked"]);
    blocked.sort();
    assert_eq!(blocked, ["Mixed", "Waiting"]);
}

#[test]
fn list_ready_shows_open_tickets_without_unfinished_blockers() {
    let dir = initialised("list-ready");
    let open = new_id(&dir, &["Open blocker"]);
    let done = new_id(&dir, &["Finished blocker", "-s", "done"]);
    new_id(&dir, &["Waiting", "-b", &open]);
    new_id(&dir, &["Unblocked", "-b", &done]);
    new_id(&dir, &["Rejected", "-s", "rejected"]);
    let mut ready = titles(&dir, &["--ready"]);
    ready.sort();
    assert_eq!(ready, ["Open blocker", "Unblocked"]);
}

#[test]
fn list_names_a_corrupt_ticket_file_in_the_error() {
    let dir = initialised("list-corrupt");
    new_id(&dir, &["Fine"]);
    std::fs::write(dir.join("tickets/all/broken.md"), "not a ticket").unwrap();
    let out = tickets(&dir, &["list"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("broken.md"), "{}", stderr(&out));
}

#[test]
fn list_treats_a_deleted_blocker_as_not_blocking() {
    let dir = initialised("list-ghost");
    let gone = new_id(&dir, &["Gone"]);
    new_id(&dir, &["Orphaned", "-b", &gone]);
    std::fs::remove_file(dir.join(format!("tickets/all/{gone}.md"))).unwrap();
    assert!(titles(&dir, &["--blocked"]).is_empty());
    assert_eq!(titles(&dir, &["--ready"]), ["Orphaned"]);
}

#[test]
fn list_accepts_format_plain() {
    let dir = initialised("list-format-plain");
    let id = new_id(&dir, &["Fix it", "-t", "bug"]);
    let out = tickets(&dir, &["list", "--format", "plain"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), format!("{id}\ttodo\tbug\tFix it"));
}

#[test]
fn pretty_list_prints_a_header_and_aligned_columns() {
    let dir = initialised("list-pretty");
    let a = new_id(&dir, &["Fix it", "-s", "in progress", "-t", "bug"]);
    let b = new_id(&dir, &["Write docs"]);
    let out = tickets_env(&dir, &["list", "--format", "pretty"], &[("NO_COLOR", "1")]);
    assert!(out.status.success(), "{}", stderr(&out));
    let lines: Vec<String> = stdout(&out).lines().map(str::to_string).collect();
    assert_eq!(lines.len(), 3, "{lines:?}");
    assert_eq!(
        lines[0],
        format!(
            "ID{}STATUS{}TYPE{}TITLE",
            " ".repeat(16),
            " ".repeat(10),
            " ".repeat(4)
        )
    );
    assert_eq!(
        lines[1],
        format!("{b}  ⚪ todo{}Write docs", " ".repeat(17))
    );
    assert_eq!(lines[2], format!("{a}  🟡 in progress  🐛 bug  Fix it"));
}

#[test]
fn pretty_list_prints_nothing_when_there_are_no_tickets() {
    let dir = initialised("list-pretty-empty");
    let out = tickets(&dir, &["list", "--format", "pretty"]);
    assert_eq!(stdout(&out), "");
}

#[test]
fn pretty_list_is_coloured_when_piped_unless_no_color_is_set() {
    let dir = initialised("list-pretty-colour");
    new_id(&dir, &["Fix it", "-t", "bug"]);
    let args = ["list", "--format", "pretty"];
    assert!(stdout(&tickets_env(&dir, &args, &[])).contains("\x1b[31mbug\x1b[0m"));
    let off = tickets_env(&dir, &args, &[("NO_COLOR", "1")]);
    assert!(!stdout(&off).contains('\x1b'));
}
