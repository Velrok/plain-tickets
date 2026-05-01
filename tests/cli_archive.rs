mod common;

use std::fs;

fn setup(name: &str) -> std::path::PathBuf {
    let dir = common::test_dir(name);
    common::tickets(&dir, &["init"]);
    dir
}

// ── K_VfeX: archive by ID list ────────────────────────────────────────────────

#[test]
fn archive_single_id_moves_file() {
    let dir = setup("archive_single_id");
    let (id, filename) = common::create_ticket(&dir, "Move me");
    let out = common::tickets(&dir, &["archive", &id]);
    assert!(out.status.success(), "archive failed: {:?}", out);
    assert!(!dir.join("all").join(&filename).exists(), "file still in all/");
    assert!(dir.join("archived").join(&filename).exists(), "file not in archived/");
}

#[test]
fn archive_multiple_ids_moves_all() {
    let dir = setup("archive_multiple_ids");
    let (id1, f1) = common::create_ticket(&dir, "First");
    let (id2, f2) = common::create_ticket(&dir, "Second");
    let out = common::tickets(&dir, &["archive", &id1, &id2]);
    assert!(out.status.success(), "archive failed: {:?}", out);
    assert!(dir.join("archived").join(&f1).exists());
    assert!(dir.join("archived").join(&f2).exists());
}

#[test]
fn archive_unknown_id_errors_no_files_moved() {
    let dir = setup("archive_unknown_id");
    let (id, filename) = common::create_ticket(&dir, "Keep me");
    let out = common::tickets(&dir, &["archive", &id, "xxxxxx"]);
    assert!(!out.status.success(), "expected failure");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("xxxxxx"), "expected failing id in stderr: {stderr}");
    assert!(stderr.contains("no files moved"), "expected no files moved: {stderr}");
    // original file untouched
    assert!(dir.join("all").join(&filename).exists(), "file was moved despite error");
}

#[test]
fn archive_already_archived_id_errors() {
    let dir = setup("archive_already_archived");
    let (id, filename) = common::create_ticket(&dir, "Archive me twice");
    common::tickets(&dir, &["archive", &id]);
    let out = common::tickets(&dir, &["archive", &id]);
    assert!(!out.status.success(), "expected failure on second archive");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(&id), "expected id in stderr: {stderr}");
    // file stays in archived/
    assert!(dir.join("archived").join(&filename).exists());
}

#[test]
fn archive_success_output_format() {
    let dir = setup("archive_output_format");
    let (id, _) = common::create_ticket(&dir, "Format test");
    let out = common::tickets(&dir, &["archive", &id]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains(&id), "id missing from output: {stdout}");
    assert!(stdout.contains("archived"), "word 'archived' missing: {stdout}");
}

#[test]
fn archive_not_initialised_errors() {
    let dir = common::test_dir("archive_not_init");
    let out = common::tickets(&dir, &["archive", "abc123"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("error:"), "expected error: {stderr}");
}

// ── EoNYIL: --all-rejected ────────────────────────────────────────────────────

#[test]
fn archive_all_rejected_moves_rejected_tickets() {
    let dir = setup("archive_all_rejected");
    let (id1, f1) = common::create_ticket(&dir, "Rejected one");
    let (id2, f2) = common::create_ticket(&dir, "Rejected two");
    let (_id3, f3) = common::create_ticket(&dir, "Keep this");
    common::tickets(&dir, &["edit", &id1, "--status", "rejected"]);
    common::tickets(&dir, &["edit", &id2, "--status", "rejected"]);
    let out = common::tickets(&dir, &["archive", "--all-rejected"]);
    assert!(out.status.success(), "all-rejected failed: {:?}", out);
    assert!(dir.join("archived").join(&f1).exists(), "f1 not archived");
    assert!(dir.join("archived").join(&f2).exists(), "f2 not archived");
    assert!(dir.join("all").join(&f3).exists(), "f3 should stay in all/");
}

#[test]
fn archive_all_rejected_no_matches_exits_zero() {
    let dir = setup("archive_all_rejected_none");
    common::create_ticket(&dir, "Not rejected");
    let out = common::tickets(&dir, &["archive", "--all-rejected"]);
    assert!(out.status.success(), "expected exit 0: {:?}", out);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.is_empty(), "expected a message on stderr: {stderr}");
}

#[test]
fn archive_all_rejected_and_ids_is_hard_error() {
    let dir = setup("archive_all_rejected_and_ids");
    let (id, _) = common::create_ticket(&dir, "Combo");
    let out = common::tickets(&dir, &["archive", "--all-rejected", &id]);
    assert!(!out.status.success(), "expected failure");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("error:"), "expected error: {stderr}");
}

// ── front matter unchanged after move ─────────────────────────────────────────

#[test]
fn archive_does_not_mutate_front_matter() {
    let dir = setup("archive_no_mutation");
    let (id, filename) = common::create_ticket(&dir, "Immutable");
    let before = fs::read_to_string(dir.join("all").join(&filename)).unwrap();
    common::tickets(&dir, &["archive", &id]);
    let after = fs::read_to_string(dir.join("archived").join(&filename)).unwrap();
    assert_eq!(before, after, "file content changed after archive");
}
