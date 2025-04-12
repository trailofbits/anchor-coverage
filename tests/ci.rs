use assert_cmd::Command;
use regex::Regex;
use std::{env::remove_var, fs::read_to_string, path::Path};

#[ctor::ctor]
fn initialize() {
    unsafe {
        remove_var("CARGO_TERM_COLOR");
    }
}

#[test]
fn clippy() {
    Command::new("cargo")
        .args([
            "+nightly",
            "clippy",
            "--all-features",
            "--all-targets",
            "--",
            "--deny=warnings",
        ])
        .assert()
        .success();
}

#[test]
fn dylint() {
    Command::new("cargo")
        .args(["dylint", "--all", "--", "--all-features", "--all-targets"])
        .env("DYLINT_RUSTFLAGS", "--deny warnings")
        .assert()
        .success();
}

#[test]
fn hack_feature_powerset_udeps() {
    Command::new("rustup")
        .env("RUSTFLAGS", "-D warnings")
        .args([
            "run",
            "nightly",
            "cargo",
            "hack",
            "--feature-powerset",
            "udeps",
        ])
        .assert()
        .success();
}

#[test]
fn markdown_link_check() {
    let tempdir = tempfile::tempdir().unwrap();

    Command::new("npm")
        .args(["install", "markdown-link-check"])
        .current_dir(&tempdir)
        .assert()
        .success();

    let readme_md = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");

    Command::new("npx")
        .args(["markdown-link-check", &readme_md.to_string_lossy()])
        .current_dir(&tempdir)
        .assert()
        .success();
}

#[test]
fn readme_reference_links_are_sorted() {
    let re = Regex::new(r"^\[[^\]]*\]:").unwrap();
    let readme = read_to_string("README.md").unwrap();
    let links = readme
        .lines()
        .filter(|line| re.is_match(line))
        .collect::<Vec<_>>();
    let mut links_sorted = links.clone();
    links_sorted.sort_unstable();
    assert_eq!(links_sorted, links);
}
