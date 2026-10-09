mod common;
use common::*;

fn raw_stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn show_prints_the_front_matter_and_body() {
    let dir = initialised("show-basic");
    let id = new_id(&dir, &["Fix it", "-t", "bug", "-m", "Some body"]);
    let out = tickets(&dir, &["show", &id]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(raw_stdout(&out), format!("{}\n", ticket_text(&dir, &id)));
}

#[test]
fn show_prints_only_the_front_matter_when_there_is_no_body() {
    let dir = initialised("show-no-body");
    let id = new_id(&dir, &["Fix it"]);
    let out = tickets(&dir, &["show", &id]);
    assert_eq!(raw_stdout(&out), ticket_text(&dir, &id));
    assert!(raw_stdout(&out).ends_with("---\n"));
}

#[test]
fn show_finds_archived_tickets() {
    let dir = initialised("show-archived");
    let id = new_id(&dir, &["Finished", "-s", "done", "-m", "Body"]);
    let before = raw_stdout(&tickets(&dir, &["show", &id]));
    assert!(tickets(&dir, &["archive", &id]).status.success());
    let out = tickets(&dir, &["show", &id]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(raw_stdout(&out), before);
}

#[test]
fn show_of_an_unknown_ticket_fails_naming_the_id_without_output() {
    let dir = initialised("show-unknown");
    let out = tickets(&dir, &["show", "00000000000000ff"]);
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("00000000000000ff"),
        "{}",
        stderr(&out)
    );
    assert_eq!(raw_stdout(&out), "");
}

#[test]
fn show_accepts_format_plain() {
    let dir = initialised("show-format-plain");
    let id = new_id(&dir, &["Fix it"]);
    let plain = tickets(&dir, &["show", &id, "--format", "plain"]);
    assert!(plain.status.success(), "{}", stderr(&plain));
    assert_eq!(stdout(&plain), stdout(&tickets(&dir, &["show", &id])));
}

#[test]
fn an_unknown_format_is_rejected() {
    let dir = initialised("format-unknown");
    let id = new_id(&dir, &["Fix it"]);
    let show = ["show", id.as_str(), "--format", "fancy"];
    for args in [&["list", "--format", "fancy"][..], &show[..]] {
        let out = tickets(&dir, args);
        assert!(!out.status.success());
        assert!(stderr(&out).contains("plain"), "{}", stderr(&out));
    }
}
