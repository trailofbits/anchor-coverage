use assert_cmd::Command;

#[test]
fn clippy() {
    Command::new("cargo")
        // smoelius: Remove `CARGO` environment variable to work around:
        // https://github.com/rust-lang/rust/pull/131729
        .env_remove("CARGO")
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
