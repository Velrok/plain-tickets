use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
    println!("cargo:rerun-if-env-changed=GITHUB_ACTIONS");

    let version_string = if std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true") {
        let ref_name = std::env::var("GITHUB_REF_NAME").unwrap_or_default();
        let version = ref_name.strip_prefix('v').unwrap_or(&ref_name);
        let sha = std::env::var("GITHUB_SHA").unwrap_or_default();
        let sha = short_sha(&sha);
        format!("{version} ({sha})")
    } else {
        let sha = git_short_sha().unwrap_or_else(|| "unknown".to_string());
        format!("dev-build ({sha})")
    };

    println!("cargo:rustc-env=TICKETS_VERSION_STRING={version_string}");
}

fn short_sha(sha: &str) -> String {
    if sha.len() >= 7 {
        sha[..7].to_string()
    } else {
        "unknown".to_string()
    }
}

fn git_short_sha() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let sha = String::from_utf8(output.stdout).ok()?;
    let sha = sha.trim();
    if sha.is_empty() {
        None
    } else {
        Some(sha.to_string())
    }
}
