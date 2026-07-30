mod common;

use common::bin;
use std::process::Command;

#[test]
fn version_flag_prints_version_and_sha() {
    let out = Command::new(bin())
        .arg("--version")
        .output()
        .expect("failed to run tickets binary");

    assert!(out.status.success(), "--version failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stdout = stdout.trim();

    let rest = stdout
        .strip_prefix("tickets ")
        .unwrap_or_else(|| panic!("expected output to start with 'tickets ', got: {stdout:?}"));
    let (version, sha) = rest
        .rsplit_once(" (")
        .unwrap_or_else(|| panic!("expected '<version> (<sha>)', got: {stdout:?}"));
    assert!(!version.is_empty(), "version token was empty");

    let sha = sha
        .strip_suffix(')')
        .unwrap_or_else(|| panic!("expected sha segment to end with ')', got: {stdout:?}"));
    assert!(
        sha == "unknown" || (sha.len() == 7 && sha.chars().all(|c| c.is_ascii_hexdigit())),
        "sha segment {sha:?} was neither 'unknown' nor a 7-char hex string"
    );
}

#[test]
fn short_version_flag_matches_long_flag() {
    let long = Command::new(bin())
        .arg("--version")
        .output()
        .expect("failed to run tickets binary");
    let short = Command::new(bin())
        .arg("-V")
        .output()
        .expect("failed to run tickets binary");

    assert!(short.status.success(), "-V failed: {:?}", short);
    assert_eq!(long.stdout, short.stdout);
}
