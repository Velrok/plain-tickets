mod common;
use common::*;

#[test]
fn note_sets_the_body_of_a_ticket_without_one() {
    let dir = initialised("note-empty");
    let id = new_id(&dir, &["Fix it"]);
    let out = tickets(&dir, &["note", &id, "First thought"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(ticket_text(&dir, &id).ends_with("---\n\nFirst thought\n"));
}

#[test]
fn note_appends_after_a_blank_line_to_an_existing_body() {
    let dir = initialised("note-append");
    let id = new_id(&dir, &["Fix it", "-m", "Body"]);
    assert!(tickets(&dir, &["note", &id, "One"]).status.success());
    assert!(tickets(&dir, &["note", &id, "Two"]).status.success());
    assert!(ticket_text(&dir, &id).ends_with("---\n\nBody\n\nOne\n\nTwo\n"));
}

#[test]
fn modifying_an_unknown_ticket_fails_naming_the_id() {
    let dir = initialised("modify-unknown");
    let ghost = "00000000000ff";
    for args in [["note", ghost, "x"], ["tag", ghost, "x"]] {
        let out = tickets(&dir, &args);
        assert!(!out.status.success(), "{args:?}");
        assert!(stderr(&out).contains(ghost), "{args:?}: {}", stderr(&out));
    }
}

fn tags(dir: &std::path::Path, id: &str) -> Vec<String> {
    list_field(dir, id, "tags")
}

fn blockers(dir: &std::path::Path, id: &str) -> Vec<String> {
    list_field(dir, id, "blocked_by")
}

fn list_field(dir: &std::path::Path, id: &str, name: &str) -> Vec<String> {
    let text = ticket_text(dir, id);
    let mut found = Vec::new();
    let mut in_tags = false;
    for line in text.lines() {
        if in_tags {
            match line.strip_prefix("- ") {
                Some(tag) => found.push(tag.to_string()),
                None => break,
            }
        } else if line == format!("{name}:") {
            in_tags = true;
        }
    }
    found
}

#[test]
fn tag_adds_tags_without_duplicates_keeping_order() {
    let dir = initialised("tag-add");
    let id = new_id(&dir, &["Fix it", "-g", "b"]);
    let out = tickets(&dir, &["tag", &id, "a", "b", "c", "a"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(tags(&dir, &id), ["b", "a", "c"]);
}

#[test]
fn tag_with_nothing_new_leaves_the_file_untouched() {
    let dir = initialised("tag-noop");
    let id = new_id(&dir, &["Fix it", "-g", "a"]);
    let aged = age_ticket(&dir, &id);
    assert!(tickets(&dir, &["tag", &id, "a"]).status.success());
    assert_eq!(ticket_text(&dir, &id), aged);
}

#[test]
fn tag_and_new_reject_blank_tags() {
    let dir = initialised("tag-blank");
    let id = new_id(&dir, &["Fix it"]);
    let aged = age_ticket(&dir, &id);
    let out = tickets(&dir, &["tag", &id, "  "]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("tag"), "{}", stderr(&out));
    assert_eq!(ticket_text(&dir, &id), aged);
    let out = tickets(&dir, &["new", "Another", "-g", ""]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("tag"), "{}", stderr(&out));
}

#[test]
fn untag_removes_tags_and_ignores_ones_that_are_absent() {
    let dir = initialised("untag");
    let id = new_id(&dir, &["Fix it", "-g", "a", "-g", "b", "-g", "c"]);
    let out = tickets(&dir, &["untag", &id, "a", "c", "zzz"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(tags(&dir, &id), ["b"]);
    let aged = age_ticket(&dir, &id);
    assert!(tickets(&dir, &["untag", &id, "zzz"]).status.success());
    assert_eq!(ticket_text(&dir, &id), aged);
}

#[test]
fn block_adds_existing_blockers_without_duplicates() {
    let dir = initialised("block-add");
    let a = new_id(&dir, &["A"]);
    let b = new_id(&dir, &["B"]);
    let id = new_id(&dir, &["Fix it", "-b", &b]);
    let out = tickets(&dir, &["block", &id, &a, &b, &a]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(blockers(&dir, &id), [b, a]);
}

#[test]
fn block_rejects_unknown_blockers_and_self_blocking() {
    let dir = initialised("block-reject");
    let id = new_id(&dir, &["Fix it"]);
    let aged = age_ticket(&dir, &id);
    let ghost = "00000000000ff";
    let out = tickets(&dir, &["block", &id, ghost]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains(ghost), "{}", stderr(&out));
    let out = tickets(&dir, &["block", &id, &id]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("itself"), "{}", stderr(&out));
    assert_eq!(ticket_text(&dir, &id), aged);
}

#[test]
fn unblock_removes_blockers_and_ignores_ones_that_are_absent() {
    let dir = initialised("unblock");
    let a = new_id(&dir, &["A"]);
    let b = new_id(&dir, &["B"]);
    let other = new_id(&dir, &["Other"]);
    let id = new_id(&dir, &["Fix it", "-b", &a, "-b", &b]);
    let out = tickets(&dir, &["unblock", &id, &a, &other]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(blockers(&dir, &id), [b]);
    let aged = age_ticket(&dir, &id);
    assert!(tickets(&dir, &["unblock", &id, &other]).status.success());
    assert_eq!(ticket_text(&dir, &id), aged);
}

fn has_line(dir: &std::path::Path, id: &str, line: &str) -> bool {
    ticket_text(dir, id).lines().any(|l| l == line)
}

#[test]
fn set_changes_only_the_given_fields() {
    let dir = initialised("set-fields");
    let parent = new_id(&dir, &["Parent"]);
    let id = new_id(
        &dir,
        &["Old title", "-t", "bug", "-g", "keep", "-m", "Body"],
    );
    let out = tickets(
        &dir,
        &[
            "set",
            &id,
            "--title",
            " New title ",
            "--status",
            "In Progress",
            "--type",
            "task",
            "--parent",
            &parent,
        ],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(has_line(&dir, &id, "title: New title"));
    assert!(has_line(&dir, &id, "status: in progress"));
    assert!(has_line(&dir, &id, "type: task"));
    assert!(has_line(&dir, &id, &format!("parent: {parent}")));
    assert_eq!(tags(&dir, &id), ["keep"]);
    assert!(ticket_text(&dir, &id).ends_with("---\n\nBody"));
}

#[test]
fn set_without_any_field_is_an_error() {
    let dir = initialised("set-nothing");
    let id = new_id(&dir, &["Fix it"]);
    let out = tickets(&dir, &["set", &id]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("nothing to set"), "{}", stderr(&out));
}

#[test]
fn set_can_clear_type_and_parent() {
    let dir = initialised("set-clear");
    let parent = new_id(&dir, &["Parent"]);
    let id = new_id(&dir, &["Fix it", "-t", "bug", "-p", &parent]);
    let out = tickets(&dir, &["set", &id, "--clear-type", "--clear-parent"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = ticket_text(&dir, &id);
    assert!(!text.contains("type:"), "{text}");
    assert!(text.contains("parent: null"), "{text}");
}

#[test]
fn set_refuses_a_value_together_with_its_clear_flag() {
    let dir = initialised("set-conflict");
    let id = new_id(&dir, &["Fix it"]);
    assert!(
        !tickets(&dir, &["set", &id, "--type", "bug", "--clear-type"])
            .status
            .success()
    );
}

#[test]
fn set_rejects_invalid_values_and_leaves_the_ticket_untouched() {
    let dir = initialised("set-invalid");
    let id = new_id(&dir, &["Fix it"]);
    let aged = age_ticket(&dir, &id);
    let cases: [(&[&str], &str); 4] = [
        (&["--title", "  "], "title"),
        (&["--status", "blocked"], "blocked"),
        (&["--parent", "00000000000ff"], "00000000000ff"),
        (&["--parent", &id], "own parent"),
    ];
    for (flags, needle) in cases {
        let mut args = vec!["set", id.as_str()];
        args.extend(flags);
        let out = tickets(&dir, &args);
        assert!(!out.status.success(), "{flags:?}");
        assert!(stderr(&out).contains(needle), "{flags:?}: {}", stderr(&out));
        assert_eq!(ticket_text(&dir, &id), aged, "{flags:?}");
    }
}
