//! The release workflow runs `scripts/check-release-tag.sh` before building.
#![cfg(unix)]
mod common;
use common::scratch;
use std::process::{Command, Output};

fn check(name: &str, cargo_version: &str, tag: &str) -> Output {
    let dir = scratch(name);
    let manifest = dir.join("Cargo.toml");
    std::fs::write(
        &manifest,
        format!(
            "[package]\nname = \"x\"\nversion = \"{cargo_version}\"\n\n[dependencies]\nclap = {{ version = \"4\" }}\n"
        ),
    )
    .unwrap();
    Command::new("sh")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scripts/check-release-tag.sh"
        ))
        .args([tag, manifest.to_str().unwrap()])
        .output()
        .unwrap()
}

#[test]
fn a_tag_matching_the_cargo_version_passes() {
    let out = check("tag-match", "1.2.3", "v1.2.3");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn a_tag_that_differs_from_the_cargo_version_fails_and_names_both() {
    let out = check("tag-mismatch", "1.2.3", "v1.2.4");
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("v1.2.4") && err.contains("1.2.3"), "{err}");
}

#[test]
fn prerelease_versions_must_match_in_full() {
    assert!(
        check("tag-pre-ok", "1.2.3-rc.1", "v1.2.3-rc.1")
            .status
            .success()
    );
    assert!(
        !check("tag-pre-bad", "1.2.3-rc.1", "v1.2.3")
            .status
            .success()
    );
}
