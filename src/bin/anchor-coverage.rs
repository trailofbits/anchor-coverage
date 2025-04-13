use anchor_coverage::util::StripCurrentDir;
use anyhow::{Result, bail, ensure};
use std::{
    env::{args, current_dir},
    fs::{create_dir_all, remove_dir_all},
    path::Path,
    process::Command,
};

const SBF_TRACE_DIR: &str = "SBF_TRACE_DIR";

struct Options {
    args: Vec<String>,
    debug: bool,
    help: bool,
}

fn main() -> Result<()> {
    let options = parse_args();

    if options.help {
        println!(
            "{} {}

A wrapper around `anchor test` for computing test coverage

Usage: {0} [ANCHOR_TEST_ARGS]...
",
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

    anchor_test(&options.args, &sbf_trace_dir)?;

    let pcs_paths = anchor_coverage::util::files_with_extension(&sbf_trace_dir, "pcs")?;

    if pcs_paths.is_empty() {
        bail!(
            "Found no program counter files in: {}

Are you sure your `solana-test-validator` is patched?",
            sbf_trace_dir.strip_current_dir().display()
        );
    }

    anchor_coverage::run(sbf_trace_dir, options.debug)?;

    Ok(())
}

fn parse_args() -> Options {
    let mut debug = false;
    let mut help = false;
    let args = args()
        .skip(1)
        .filter_map(|arg| {
            if arg == "--debug" {
                debug = true;
                None
            } else if arg == "--help" || arg == "-h" {
                help = true;
                None
            } else {
                Some(arg)
            }
        })
        .collect::<Vec<_>>();
    Options { args, debug, help }
}

fn anchor_test(args: &[String], sbf_trace_dir: &Path) -> Result<()> {
    let mut command = Command::new("anchor");
    command.arg("test");
    command.args(args);
    // smoelius: Options after `--` are passed to `cargo-build-sbpf`. For our case, passing
    // `--debug` tells `cargo-build-sbpf` to enable debug symbols.
    if !args.iter().any(|arg| arg == "--") {
        command.arg("--");
    }
    command.arg("--debug");
    command.env(SBF_TRACE_DIR, sbf_trace_dir);
    let status = command.status()?;
    ensure!(status.success(), "command failed: {:?}", command);
    Ok(())
}
