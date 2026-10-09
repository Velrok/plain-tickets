mod common;
use common::*;

#[test]
fn version_shows_the_package_version_and_a_short_sha() {
    let dir = scratch("version");
    let out = tickets(&dir, &["--version"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    let rest = text
        .strip_prefix(concat!("tickets ", env!("CARGO_PKG_VERSION"), " ("))
        .and_then(|r| r.strip_suffix(')'))
        .unwrap_or_else(|| panic!("unexpected --version output: {text}"));
    let is_sha = rest.len() == 7 && rest.chars().all(|c| c.is_ascii_hexdigit());
    assert!(is_sha || rest == "unknown", "{text}");
}
