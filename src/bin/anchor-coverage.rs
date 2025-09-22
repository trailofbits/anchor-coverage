use anchor_coverage::util::StripCurrentDir;
use anyhow::{bail, ensure, Result};
use std::{
    env::{args, current_dir},
    fs::{canonicalize, create_dir_all, remove_dir_all},
    path::{Path, PathBuf},
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

    anchor_test_with_debug(&options.args, &sbf_trace_dir)?;

    let pcs_paths = anchor_coverage::util::files_with_extension(&sbf_trace_dir, "pcs")?;

    if pcs_paths.is_empty() {
        let try_running_message = grep_command()
            .map(|command| {
                format!(
                    " Try running the following command:
    {command}"
                )
            })
            .unwrap_or_default();

        bail!(
            "Found no program counter files in: {}
Are you sure your `solana-test-validator` is patched?{}",
            sbf_trace_dir.strip_current_dir().display(),
            try_running_message
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

fn anchor_test_with_debug(args: &[String], sbf_trace_dir: &Path) -> Result<()> {
    #[cfg(feature = "__anchor_cli")]
    anchor_coverage::__build_with_debug(
        &anchor_coverage::ConfigOverride::default(),
        false, // no_idl
        None,
        None,
        false,
        true, // skip_lint
        None,
        None,
        None,
        anchor_coverage::BootstrapMode::None,
        None,
        None,
        Vec::new(),
        Vec::new(),
        true, // no_docs
        anchor_coverage::ProgramArch::Sbf,
    )?;

    anchor_test_skip_build(args, sbf_trace_dir)?;

    Ok(())
}

fn anchor_test_skip_build(args: &[String], sbf_trace_dir: &Path) -> Result<()> {
    let mut command = Command::new("anchor");
    command.args(["test", "--skip-build"]);
    command.args(args);
    command.env(SBF_TRACE_DIR, sbf_trace_dir);
    let status = command.status()?;
    ensure!(status.success(), "command failed: {command:?}");
    Ok(())
}

fn grep_command() -> Result<String> {
    let path = which("solana-test-validator")?;
    Ok(format!(
        "grep SBF_TRACE_DIR {} || echo 'solana-test-validator is not patched'",
        path.display()
    ))
}

fn which(filename: &str) -> Result<PathBuf> {
    let mut command = Command::new("which");
    let output = command.arg(filename).output()?;
    ensure!(output.status.success(), "command failed: {command:?}");
    let stdout = std::str::from_utf8(&output.stdout)?;
    let path = canonicalize(stdout.trim_end())?;
    Ok(path)
}
