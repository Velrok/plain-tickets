mod common;
use common::*;
use std::path::{Path, PathBuf};

fn active(dir: &Path, id: &str) -> PathBuf {
    dir.join(format!("tickets/all/{id}.md"))
}

fn archived(dir: &Path, id: &str) -> PathBuf {
    dir.join(format!("tickets/archived/{id}.md"))
}

#[test]
fn archive_moves_a_done_ticket_out_of_the_active_list() {
    let dir = initialised("archive-done");
    let id = new_id(&dir, &["Finished", "-s", "done"]);
    let out = tickets(&dir, &["archive", &id]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!active(&dir, &id).exists());
    assert!(archived(&dir, &id).is_file());
    assert_eq!(stdout(&tickets(&dir, &["list"])), "");
}

#[test]
fn archive_refuses_a_ticket_that_is_still_open() {
    let dir = initialised("archive-open");
    let id = new_id(&dir, &["Still going", "-s", "in progress"]);
    let out = tickets(&dir, &["archive", &id]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("in progress"), "{}", stderr(&out));
    assert!(active(&dir, &id).is_file());
    assert!(!archived(&dir, &id).exists());
}

#[test]
fn rejected_tickets_can_be_archived_too() {
    let dir = initialised("archive-rejected");
    let id = new_id(&dir, &["Not doing", "-s", "rejected"]);
    assert!(tickets(&dir, &["archive", &id]).status.success());
    assert!(archived(&dir, &id).is_file());
}

#[test]
fn commands_say_a_ticket_is_archived_when_it_is_in_the_archive() {
    let dir = initialised("archive-message");
    let id = new_id(&dir, &["Finished", "-s", "done"]);
    let other = new_id(&dir, &["Other"]);
    assert!(tickets(&dir, &["archive", &id]).status.success());
    let attempts: Vec<Vec<&str>> = vec![
        vec!["archive", &id],
        vec!["set", &id, "--title", "x"],
        vec!["note", &id, "x"],
        vec!["tag", &id, "x"],
        vec!["new", "Child", "-p", &id],
        vec!["block", &other, &id],
    ];
    for args in attempts {
        let out = tickets(&dir, &args);
        assert!(!out.status.success(), "{args:?}");
        assert!(
            stderr(&out).contains("archived"),
            "{args:?}: {}",
            stderr(&out)
        );
        assert!(stderr(&out).contains(&id), "{args:?}: {}", stderr(&out));
    }
}

#[test]
fn unknown_tickets_are_not_called_archived() {
    let dir = initialised("archive-unknown");
    let out = tickets(&dir, &["archive", "00000000000ff"]);
    assert!(!out.status.success());
    assert!(!stderr(&out).contains("archived"), "{}", stderr(&out));
    assert!(stderr(&out).contains("00000000000ff"), "{}", stderr(&out));
}

#[test]
fn unarchive_restores_the_ticket_byte_for_byte() {
    let dir = initialised("unarchive");
    let id = new_id(&dir, &["Finished", "-s", "done", "-m", "Body"]);
    let original = ticket_text(&dir, &id);
    assert!(tickets(&dir, &["archive", &id]).status.success());
    let out = tickets(&dir, &["unarchive", &id]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!archived(&dir, &id).exists());
    assert_eq!(ticket_text(&dir, &id), original);
    assert!(stdout(&tickets(&dir, &["list"])).contains(&id));
}

#[test]
fn unarchive_fails_for_tickets_that_are_not_archived() {
    let dir = initialised("unarchive-missing");
    let active_id = new_id(&dir, &["Active"]);
    for id in [active_id.as_str(), "00000000000ff"] {
        let out = tickets(&dir, &["unarchive", id]);
        assert!(!out.status.success(), "{id}");
        assert!(stderr(&out).contains(id), "{id}: {}", stderr(&out));
        assert!(stderr(&out).contains("archived"), "{id}: {}", stderr(&out));
    }
    assert!(active(&dir, &active_id).is_file());
}

fn listed(dir: &Path, args: &[&str]) -> Vec<String> {
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
fn list_archived_shows_only_archived_tickets_and_honours_filters() {
    let dir = initialised("list-archived");
    let done = new_id(&dir, &["Old done", "-s", "done", "-g", "a"]);
    let rejected = new_id(&dir, &["Old rejected", "-s", "rejected"]);
    new_id(&dir, &["Still active"]);
    for id in [&done, &rejected] {
        assert!(tickets(&dir, &["archive", id]).status.success());
    }
    assert_eq!(listed(&dir, &[]), ["Still active"]);
    assert_eq!(listed(&dir, &["--archived"]), ["Old done", "Old rejected"]);
    assert_eq!(
        listed(&dir, &["--archived", "-s", "rejected"]),
        ["Old rejected"]
    );
    assert_eq!(listed(&dir, &["--archived", "-g", "a"]), ["Old done"]);
}

#[test]
fn archiving_a_finished_blocker_does_not_block_its_dependants() {
    let dir = initialised("archive-blocker");
    let blocker = new_id(&dir, &["Blocker", "-s", "done"]);
    new_id(&dir, &["Dependant", "-b", &blocker]);
    assert!(tickets(&dir, &["archive", &blocker]).status.success());
    assert_eq!(listed(&dir, &["--ready"]), ["Dependant"]);
    assert!(listed(&dir, &["--blocked"]).is_empty());
}
