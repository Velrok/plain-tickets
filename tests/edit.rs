mod common;
use common::*;

#[test]
fn edit_saves_what_the_editor_changed() {
    let dir = initialised("edit-saves");
    let id = new_id(&dir, &["Old title", "-m", "Body"]);
    let out = tickets_editing(&dir, &["edit", &id], &set_title("New title"));
    assert!(out.status.success(), "{}", stderr(&out));
    let text = ticket_text(&dir, &id);
    assert!(text.contains("title: New title"), "{text}");
    assert!(text.ends_with("Body"), "{text}");
}

fn field(text: &str, name: &str) -> String {
    text.lines()
        .find_map(|l| l.strip_prefix(&format!("{name}: ")))
        .unwrap()
        .to_string()
}

#[test]
fn edit_stamps_updated_at_itself_and_keeps_created_at() {
    let dir = initialised("edit-stamps");
    let id = new_id(&dir, &["Old title"]);
    let before = ticket_text(&dir, &id);
    let editor = format!(
        "{}\nsed 's/^updated_at: .*/updated_at: 0/' \"$1\" > \"$1.new\" && mv \"$1.new\" \"$1\"",
        set_title("New title")
    );
    let out = tickets_editing(&dir, &["edit", &id], &editor);
    assert!(out.status.success(), "{}", stderr(&out));
    let after = ticket_text(&dir, &id);
    assert_eq!(field(&after, "created_at"), field(&before, "created_at"));
    assert_ne!(field(&after, "updated_at"), "0", "{after}");
}

#[test]
fn edit_without_changes_leaves_the_file_untouched() {
    let dir = initialised("edit-noop");
    let id = new_id(&dir, &["Same title", "-m", "Body"]);
    let path = dir.join(format!("tickets/all/{id}.md"));
    let aged = ticket_text(&dir, &id).replace(
        &format!(
            "updated_at: {}",
            field(&ticket_text(&dir, &id), "updated_at")
        ),
        "updated_at: 5",
    );
    std::fs::write(&path, &aged).unwrap();
    let out = tickets_editing(&dir, &["edit", &id], "true");
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(ticket_text(&dir, &id), aged);
}

#[test]
fn edit_of_an_unknown_ticket_fails_without_starting_the_editor() {
    let dir = initialised("edit-unknown");
    let out = tickets_editing(&dir, &["edit", "00000000000000ff"], "touch editor-ran");
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("00000000000000ff"),
        "{}",
        stderr(&out)
    );
    assert!(!dir.join("editor-ran").exists());
}

#[test]
fn edit_rejects_changes_to_id_and_created_at_and_keeps_the_original() {
    let dir = initialised("edit-immutable");
    let id = new_id(&dir, &["Keep me"]);
    let original = ticket_text(&dir, &id);
    for (field, value) in [("id", "1234567890abcdef"), ("created_at", "7")] {
        let editor = format!(
            "sed 's/^{field}: .*/{field}: {value}/' \"$1\" > \"$1.new\" && mv \"$1.new\" \"$1\""
        );
        let out = tickets_editing(&dir, &["edit", &id], &editor);
        assert!(!out.status.success(), "{field}");
        assert!(stderr(&out).contains(field), "{}", stderr(&out));
        assert_eq!(ticket_text(&dir, &id), original, "{field}");
    }
}

/// Runs the editor body and asserts the edit fails with `needle` in stderr
/// and the ticket file unchanged.
fn assert_rejected(name: &str, editor: &str, needle: &str) {
    let dir = initialised(name);
    let id = new_id(&dir, &["Keep me", "-m", "Body"]);
    let original = ticket_text(&dir, &id);
    let out = tickets_editing(&dir, &["edit", &id], editor);
    assert!(!out.status.success(), "{name}: edit succeeded");
    assert!(stderr(&out).contains(needle), "{name}: {}", stderr(&out));
    assert_eq!(ticket_text(&dir, &id), original, "{name}");
}

#[test]
fn edit_rejects_a_blank_title() {
    assert_rejected("edit-blank", &set_title("   "), "title");
}

#[test]
fn edit_rejects_a_status_the_config_does_not_list() {
    let editor = "sed 's/^status: .*/status: blocked/' \"$1\" > \"$1.new\" && mv \"$1.new\" \"$1\"";
    assert_rejected("edit-status", editor, "blocked");
}

#[test]
fn edit_rejects_broken_front_matter() {
    assert_rejected(
        "edit-broken",
        "echo 'no front matter' > \"$1\"",
        "front matter",
    );
}

#[test]
fn edit_rejects_a_parent_or_blocker_that_does_not_exist() {
    let ghost = "00000000000000ff";
    let parent =
        format!("sed 's/^parent: .*/parent: {ghost}/' \"$1\" > \"$1.new\" && mv \"$1.new\" \"$1\"");
    assert_rejected("edit-parent", &parent, ghost);
    let blocker = format!(
        "sed 's/^blocked_by: .*/blocked_by: [{ghost}]/' \"$1\" > \"$1.new\" && mv \"$1.new\" \"$1\""
    );
    assert_rejected("edit-blocker", &blocker, ghost);
}

#[test]
fn edit_aborts_without_saving_when_the_editor_fails() {
    let editor = format!("{}\nexit 1", set_title("Changed"));
    assert_rejected("edit-abort", &editor, "editor");
}

fn run_edit_with_env(
    name: &str,
    visual: Option<&str>,
    editor: Option<&str>,
) -> (String, String, std::path::PathBuf) {
    let dir = initialised(name);
    let id = new_id(&dir, &["Old title"]);
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_tickets"));
    cmd.args(["edit", &id]).current_dir(&dir);
    cmd.env_remove("VISUAL").env_remove("EDITOR");
    if let Some(v) = visual {
        cmd.env("VISUAL", v);
    }
    if let Some(e) = editor {
        cmd.env("EDITOR", e);
    }
    let out = cmd.output().unwrap();
    (ticket_text(&dir, &id), stderr(&out), dir)
}

#[test]
fn edit_prefers_visual_over_editor() {
    // The trailing `#` comments out the file argument the command is given.
    let (_, _, dir) = run_edit_with_env("edit-visual", Some("touch visual-ran #"), Some("false"));
    assert!(dir.join("visual-ran").exists());
}

#[test]
fn edit_explains_when_no_editor_is_configured() {
    let (_, err, _) = run_edit_with_env("edit-no-editor", None, None);
    assert!(err.contains("VISUAL") && err.contains("EDITOR"), "{err}");
}
