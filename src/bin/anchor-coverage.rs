use anchor_coverage::util::{var_guard::VarGuard, StripCurrentDir};
use anyhow::{bail, ensure, Result};
use std::{
    env::{args, current_dir, join_paths, split_paths, var_os},
    ffi::OsString,
    fmt::Write,
    fs::{canonicalize, create_dir_all, read, read_to_string, remove_dir_all},
    path::{Path, PathBuf},
    process::Command,
};
use toml::{Table, Value};

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

    // smoelius: Set `PATH` now, once and for all. This way subsequent calls to `which` will return
    // paths to the tools actually used.
    let _guard: VarGuard;
    if let Some(path_buf) = anchor_coverage::util::patched_agave_tools(&current_dir)? {
        eprintln!(
            "Found patched Agave tools: {}",
            path_buf.strip_current_dir().display()
        );
        let prepended_paths = prepend_paths(path_buf.join("bin"))?;
        _guard = VarGuard::set("PATH", Some(prepended_paths));
    }

    if !profile_release_has_debug()? {
        eprintln!(
            "Warning: Could not find `debug = true` under `[profile.release]`; `anchor-coverage` \
             may not work correctly"
        );
    }

    let sbf_trace_dir = current_dir.join("sbf_trace_dir");

    if sbf_trace_dir.try_exists()? {
        eprintln!("Warning: Removing `{}`", sbf_trace_dir.display());
        remove_dir_all(&sbf_trace_dir)?;
    }

    create_dir_all(&sbf_trace_dir)?;

    anchor_test_with_debug(&options.args, &sbf_trace_dir)?;

    let pcs_paths = anchor_coverage::util::files_with_extension(&sbf_trace_dir, "pcs")?;

    if pcs_paths.is_empty() {
        let mut message = format!(
            "Found no program counter files in: {}",
            sbf_trace_dir.strip_current_dir().display()
        );
        let path = which("solana-test-validator")?;
        if !solana_test_validator_is_patched(&path)? {
            #[rustfmt::skip]
            write!(
                &mut message,
                "\n
`{}` does not appear to be patched.

Either download, unzip, and untar prebuilt patched binaries from:

    https://github.com/trail-of-forks/sbpf-coverage/releases

Or build patched binaries from source using the instructions at:

    https://github.com/trail-of-forks/sbpf-coverage",
                path.display()
            )
            .unwrap();
        }
        bail!(message);
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

fn prepend_paths(path: PathBuf) -> Result<OsString> {
    let Some(paths) = var_os("PATH") else {
        bail!("`PATH` is unset");
    };
    let paths_split = split_paths(&paths);
    let paths_chained = std::iter::once(path).chain(paths_split);
    let paths_joined = join_paths(paths_chained)?;
    Ok(paths_joined)
}

fn profile_release_has_debug() -> Result<bool> {
    // smoelius: If Cargo.toml cannot be found, proceed as if everything is okay.
    let Ok(contents) = read_to_string("Cargo.toml") else {
        return Ok(true);
    };
    let table = contents.parse::<Table>()?;
    Ok(table
        .get("profile")
        .and_then(Value::as_table)
        .and_then(|table| table.get("release"))
        .and_then(Value::as_table)
        .and_then(|table| table.get("debug"))
        .and_then(Value::as_bool)
        .unwrap_or(false))
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

fn solana_test_validator_is_patched(path: &Path) -> Result<bool> {
    let contents = read(path)?;
    let needle = "SBF_TRACE_DIR";
    Ok(contents
        .windows(needle.len())
        .any(|w| w == needle.as_bytes()))
}

fn which(filename: &str) -> Result<PathBuf> {
    let mut command = Command::new("which");
    let output = command.arg(filename).output()?;
    ensure!(output.status.success(), "command failed: {command:?}");
    let stdout = std::str::from_utf8(&output.stdout)?;
    let path = canonicalize(stdout.trim_end())?;
    Ok(path)
}
