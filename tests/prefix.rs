mod common;
use common::*;
use std::path::Path;

/// Writes an active ticket with a chosen ID, so prefixes can be made to collide.
fn write_ticket(dir: &Path, id: &str, title: &str) {
    let text = format!(
        "---\nid: {id}\ntitle: {title}\nstatus: todo\nparent: null\nblocked_by: []\ntags: []\ncreated_at: 2026-01-01T00:00:00.000Z\nupdated_at: 2026-01-01T00:00:00.000Z\n---\n"
    );
    std::fs::create_dir_all(dir.join("tickets/all")).unwrap();
    std::fs::write(dir.join(format!("tickets/all/{id}.md")), text).unwrap();
}

#[test]
fn a_unique_prefix_resolves_to_its_ticket() {
    let dir = initialised("prefix-unique");
    write_ticket(&dir, "0aaaaaaaaaaaa", "Alpha");
    write_ticket(&dir, "0bbbbbbbbbbbb", "Beta");
    let out = tickets(&dir, &["show", "0aa", "--format", "plain"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("title: Alpha"), "{}", stdout(&out));
}

#[test]
fn an_ambiguous_prefix_fails_listing_the_matching_ids_and_titles() {
    let dir = initialised("prefix-ambiguous");
    write_ticket(&dir, "0abc000000001", "First");
    write_ticket(&dir, "0abc000000002", "Second");
    write_ticket(&dir, "0xyz000000003", "Other");
    let out = tickets(&dir, &["note", "0ab", "hi"]);
    assert!(!out.status.success());
    let err = stderr(&out);
    for expected in ["0abc000000001", "First", "0abc000000002", "Second"] {
        assert!(err.contains(expected), "{expected}: {err}");
    }
    assert!(!err.contains("Other"), "{err}");
    assert!(!ticket_text(&dir, "0abc000000001").contains("hi"));
}

#[test]
fn a_prefix_matching_nothing_says_so() {
    let dir = initialised("prefix-missing");
    write_ticket(&dir, "0abc000000001", "First");
    let out = tickets(&dir, &["show", "zzz"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("no ticket"), "{}", stderr(&out));
    assert!(stderr(&out).contains("zzz"), "{}", stderr(&out));
}

#[test]
fn prefixes_also_find_archived_tickets() {
    let dir = initialised("prefix-archived");
    write_ticket(&dir, "0abc000000001", "Old");
    assert!(
        tickets(&dir, &["set", "0abc000000001", "--status", "done"])
            .status
            .success()
    );
    assert!(tickets(&dir, &["archive", "0abc"]).status.success());
    let out = tickets(&dir, &["show", "0abc0", "--format", "plain"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("title: Old"), "{}", stdout(&out));
}

#[test]
fn prefixes_work_for_parents_and_blockers_too() {
    let dir = initialised("prefix-links");
    write_ticket(&dir, "0abc000000001", "Parent");
    write_ticket(&dir, "0def000000002", "Blocker");
    let id = new_id(&dir, &["Child", "-p", "0abc", "-b", "0def"]);
    let text = ticket_text(&dir, &id);
    assert!(text.contains("parent: 0abc000000001"), "{text}");
    assert!(text.contains("0def000000002"), "{text}");
}

const GREY: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";

/// Runs with colour forced on, so the greyed part of an ID is visible in the output.
fn pretty(dir: &Path, args: &[&str]) -> String {
    let mut full = args.to_vec();
    full.extend(["--format", "pretty"]);
    stdout(&tickets_env(dir, &full, &[("NO_COLOR", "")]))
}

#[test]
fn pretty_list_shows_full_ids_with_the_part_beyond_the_unique_prefix_in_grey() {
    let dir = initialised("prefix-pretty-list");
    write_ticket(&dir, "0abc000000001", "First");
    write_ticket(&dir, "0xyz000000003", "Other");
    let out = pretty(&dir, &["list"]);
    let row = out.lines().find(|l| l.contains("Other")).unwrap();
    assert!(
        row.starts_with(&format!("0xy{GREY}z000000003{RESET} ")),
        "{out}"
    );
    let plain = stdout(&tickets(&dir, &["list", "--format", "plain"]));
    assert!(plain.contains("0xyz000000003\t"), "{plain}");
    assert!(!plain.contains('\x1b'), "{plain}");
}

#[test]
fn prefixes_are_unique_among_archived_tickets_too() {
    let dir = initialised("prefix-pretty-archived");
    write_ticket(&dir, "0xyz000000003", "Other");
    write_ticket(&dir, "0xya000000004", "Gone");
    assert!(
        tickets(&dir, &["set", "0xya000000004", "--status", "done"])
            .status
            .success()
    );
    assert!(
        tickets(&dir, &["archive", "0xya000000004"])
            .status
            .success()
    );
    let out = pretty(&dir, &["list"]);
    let row = out.lines().find(|l| l.contains("Other")).unwrap();
    assert!(
        row.starts_with(&format!("0xyz{GREY}000000003{RESET} ")),
        "{out}"
    );
}

#[test]
fn pretty_show_greys_the_same_way_for_the_ticket_and_its_relations() {
    let dir = initialised("prefix-pretty-show");
    write_ticket(&dir, "0abc000000001", "Parent");
    let id = new_id(&dir, &["Child", "-p", "0abc"]);
    let out = pretty(&dir, &["show", &id]);
    assert!(
        out.contains(&format!("Parent: 0ab{GREY}c000000001{RESET} Parent")),
        "{out}"
    );
}

#[test]
fn without_colour_pretty_output_still_shows_every_character() {
    let dir = initialised("prefix-pretty-nocolour");
    write_ticket(&dir, "0xyz000000003", "Other");
    let out = stdout(&tickets_env(
        &dir,
        &["list", "--format", "pretty"],
        &[("NO_COLOR", "1")],
    ));
    assert!(out.contains("0xyz000000003"), "{out}");
    assert!(!out.contains('\x1b'), "{out}");
}
