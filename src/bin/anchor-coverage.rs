use anyhow::{Result, bail, ensure};
use std::{
    env::{args, current_dir},
    ffi::OsStr,
    fs::{create_dir_all, read_dir, remove_dir_all},
    path::{Path, PathBuf},
    process::Command,
};

const SBF_TRACE_DIR: &str = "SBF_TRACE_DIR";

fn main() -> Result<()> {
    let args = args().collect::<Vec<_>>();

    if args[1..]
        .iter()
        .any(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        eprintln!(
            "{} {}

A wrapper around `anchor test` for computing test coverage",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }

    let current_dir = current_dir()?;

    let sbf_trace_dir = current_dir.join("sbf_trace_dir");

    if sbf_trace_dir.try_exists()? {
        eprintln!("Warning: Removing `{}`", sbf_trace_dir.display());
        remove_dir_all(&sbf_trace_dir)?;
    }

    create_dir_all(&sbf_trace_dir)?;

    anchor_test(&args[1..], &sbf_trace_dir)?;

    let pcs_paths = collect_pcs_paths(&sbf_trace_dir)?;

    if pcs_paths.is_empty() {
        bail!(
            "`SBF_TRACE_DIR` contains no program counter files; are you sure `solana-validator` \
             is patched?"
        );
    }

    anchor_coverage::run(sbf_trace_dir)?;

    Ok(())
}

fn anchor_test(args: &[String], sbf_trace_dir: &Path) -> Result<()> {
    let mut command = Command::new("anchor");
    command.arg("test");
    command.args(args);
    if !args.iter().any(|arg| arg == "--") {
        command.arg("--");
    }
    command.arg("--debug");
    command.env(SBF_TRACE_DIR, sbf_trace_dir);
    let status = command.status()?;
    ensure!(status.success(), "command failed: {:?}", command);
    Ok(())
}

fn collect_pcs_paths(path: &Path) -> Result<Vec<PathBuf>> {
    let mut pcs_paths = Vec::new();
    for result in read_dir(path)? {
        let entry = result?;
        let path = entry.path();
        if entry.path().extension() == Some(OsStr::new("pcs")) {
            pcs_paths.push(path);
        }
    }
    Ok(pcs_paths)
}
