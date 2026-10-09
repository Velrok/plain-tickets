mod common;
use common::*;

#[test]
fn new_prints_the_id_and_writes_the_ticket() {
    let dir = initialised("basic");
    let id = new_id(&dir, &["Fix it", "-m", "Some body", "-g", "a", "-g", "a"]);
    assert_eq!(id.len(), 16, "{id}");
    let text = std::fs::read_to_string(dir.join(format!("tickets/all/{id}.md"))).unwrap();
    assert!(text.contains("title: Fix it"), "{text}");
    assert!(text.contains("status: todo"), "{text}");
    assert!(text.ends_with("Some body"), "{text}");
}

#[test]
fn new_accepts_existing_parent_and_blockers() {
    let dir = initialised("links");
    let parent = new_id(&dir, &["Parent"]);
    let blocker = new_id(&dir, &["Blocker"]);
    let id = new_id(&dir, &["Child", "-p", &parent, "-b", &blocker]);
    let text = std::fs::read_to_string(dir.join(format!("tickets/all/{id}.md"))).unwrap();
    assert!(text.contains(&format!("parent: {parent}")), "{text}");
    assert!(text.contains(&blocker), "{text}");
}

#[test]
fn new_rejects_unknown_parent_and_blockers_without_writing() {
    let dir = initialised("unknown");
    let ghost = "00000000000000ff";
    for flag in ["-p", "-b"] {
        let out = tickets(&dir, &["new", "Child", flag, ghost]);
        assert!(!out.status.success());
        assert!(stderr(&out).contains(ghost), "{}", stderr(&out));
    }
    assert!(!dir.join("tickets").exists());
}

#[test]
fn new_rejects_a_blank_title() {
    let dir = initialised("blank");
    let out = tickets(&dir, &["new", "  "]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("title"), "{}", stderr(&out));
}

#[test]
fn new_works_from_a_nested_directory() {
    let dir = initialised("nested");
    let nested = dir.join("a/b");
    std::fs::create_dir_all(&nested).unwrap();
    let id = new_id(&nested, &["Deep"]);
    assert!(dir.join(format!("tickets/all/{id}.md")).is_file());
}

#[test]
fn new_sets_a_requested_status() {
    let dir = initialised("status");
    let id = new_id(&dir, &["Fix it", "-s", "In Progress"]);
    let text = std::fs::read_to_string(dir.join(format!("tickets/all/{id}.md"))).unwrap();
    assert!(text.contains("status: in progress"), "{text}");
}

#[test]
fn new_rejects_an_unconfigured_status_without_writing() {
    let dir = initialised("bad-status");
    let out = tickets(&dir, &["new", "Fix it", "--status", "blocked"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("blocked"), "{}", stderr(&out));
    assert!(!dir.join("tickets").exists());
}

#[test]
fn new_help_explains_how_to_pass_a_message_starting_with_dashes() {
    let dir = initialised("new-help-dashes");
    let help = stdout(&tickets(&dir, &["new", "--help"]));
    assert!(help.contains(r#"--message="--text""#), "{help}");
}

#[test]
fn note_help_explains_how_to_pass_text_starting_with_dashes() {
    let dir = initialised("note-help-dashes");
    let help = stdout(&tickets(&dir, &["note", "--help"]));
    assert!(help.contains("tickets note <ID> -- "), "{help}");
}
