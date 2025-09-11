use crate::util::files_with_extension;
use anyhow::{ensure, Result};
use assert_cmd::cargo::CommandCargoExt;
use std::{
    collections::HashSet,
    env::current_dir,
    fs::read_to_string,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};

const BASIC_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/basic");
const MULTIPLE_TEST_CONFIGS_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/fixtures/multiple_test_configs"
);
const EXTERNAL_CALL_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/external_call");
const MULTIPLE_PROGRAMS_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/multiple_programs");

// smoelius: Only one Anchor test can be run at a time.
static MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn basic() {
    let _lock = MUTEX.lock().unwrap();

    yarn(BASIC_DIR).unwrap();

    let mut command = anchor_coverage_command(BASIC_DIR);
    let status = command.status().unwrap();
    assert!(status.success(), "command failed: {command:?}");

    let lcovs = files_with_extension(Path::new(BASIC_DIR).join("sbf_trace_dir"), "lcov").unwrap();
    let source_files = lcovs
        .iter()
        .map(|lcov| {
            let contents = read_to_string(lcov).unwrap();
            let source_file = contents
                .lines()
                .next()
                .and_then(|line| line.strip_prefix("SF:"))
                .unwrap();
            PathBuf::from(source_file)
        })
        .collect::<HashSet<_>>();

    // smoelius: Verify lcov was generated.
    let lib_rs_path = Path::new(BASIC_DIR).join("programs/basic/src/lib.rs");
    assert!(source_files.contains(&lib_rs_path));
}

#[test]
fn multiple_test_configs() {
    let _lock = MUTEX.lock().unwrap();

    yarn(MULTIPLE_TEST_CONFIGS_DIR).unwrap();

    for test_config in ["full", "just_increment_x", "just_increment_y"] {
        let mut command = anchor_coverage_command(MULTIPLE_TEST_CONFIGS_DIR);
        command.args(["--run", &format!("test_configs/{test_config}")]);
        let status = command.status().unwrap();
        assert!(status.success(), "command failed: {command:?}");

        // smoelius: We cannot use `dir-diff` here. ELF files built on different platforms may
        // differ and hence so will their `pcs` files. So we instead just compare the sets of lcov
        // files generated.
        //
        // smoelius: We might additionally consider sanitizing and comparing their `insns` files.
        //
        // smoelius: Updates to the Rust compiler and Solana platform tools have necessitated
        // changes to the lcov files. However, the following aspects of those files should _not_
        // change. The `is_initialized` lcov files should show that these lines were not executed:
        //
        // ```rs
        //         ctx.accounts.storage.x += 1;
        //         ...
        //         ctx.accounts.storage.y += 1;
        // ```
        //
        // The `should_increment_x` lcov files should show that the former was executed but the
        // latter wasn't. For `should_increment_y`, it should be the other way around. The above
        // lines are 15 and 20 in
        // fixtures/multiple_test_configs/programs/multiple_test_configs/src/lib.rs. So, concretely,
        // the `is_initialized`, `should_increment_x`, and `should_increment_y` lcov files should
        // contain the following:
        //
        // is_initialized:
        // ```
        // DA:15,0
        // DA:20,0
        // ```
        //
        // should_increment_x:
        // ```
        // DA:15,3
        // DA:20,0
        // ```
        //
        // should_increment_y:
        // ```
        // DA:15,0
        // DA:20,3
        // ```

        let snapshots_subdir = format!("multiple_test_configs_{test_config}");
        let expected_lcov_paths =
            files_with_extension(Path::new("snapshots").join(snapshots_subdir), "lcov").unwrap();
        let expected_lcov_contents = expected_lcov_paths
            .into_iter()
            .map(|path| read_to_string(path).unwrap())
            .collect::<HashSet<_>>();

        let current_dir = current_dir().unwrap();
        let actual_lcov_paths = files_with_extension(
            Path::new(MULTIPLE_TEST_CONFIGS_DIR).join("sbf_trace_dir"),
            "lcov",
        )
        .unwrap();
        let actual_lcov_contents = actual_lcov_paths
            .into_iter()
            .map(|path| {
                read_to_string(path)
                    .unwrap()
                    .replace(&*current_dir.to_string_lossy(), "$DIR")
            })
            .collect::<HashSet<_>>();

        assert_eq!(expected_lcov_contents, actual_lcov_contents);
    }
}

#[test]
fn include_cargo_does_not_change_line_hits() {
    let _lock = MUTEX.lock().unwrap();

    yarn(EXTERNAL_CALL_DIR).unwrap();

    let report_without_cargo = run_anchor_coverage_and_read_lcov(EXTERNAL_CALL_DIR, false).unwrap();

    let report_with_cargo = run_anchor_coverage_and_read_lcov(EXTERNAL_CALL_DIR, true).unwrap();

    for (file_key, file_without_cargo) in report_without_cargo.sections {
        let file_with_cargo = report_with_cargo.sections.get(&file_key).unwrap();
        for (line_key, lines_without_cargo) in file_without_cargo.lines {
            let lines_with_cargo = file_with_cargo.lines.get(&line_key).unwrap();
            assert!(
                lines_without_cargo.count == lines_with_cargo.count,
                "{}:{}: {} != {}",
                file_key.source_file.display(),
                line_key.line,
                lines_without_cargo.count,
                lines_with_cargo.count
            );
        }
    }
}

#[test]
fn multiple_programs() {
    let _lock = MUTEX.lock().unwrap();

    yarn(MULTIPLE_PROGRAMS_DIR).unwrap();

    let mut command = anchor_coverage_command(MULTIPLE_PROGRAMS_DIR);
    let status = command.status().unwrap();
    assert!(status.success(), "command failed: {command:?}");

    let lcovs = files_with_extension(
        Path::new(MULTIPLE_PROGRAMS_DIR).join("sbf_trace_dir"),
        "lcov",
    )
    .unwrap();
    let source_files = lcovs
        .iter()
        .map(|lcov| {
            let contents = read_to_string(lcov).unwrap();
            let source_file = contents
                .lines()
                .next()
                .and_then(|line| line.strip_prefix("SF:"))
                .unwrap();
            PathBuf::from(source_file)
        })
        .collect::<HashSet<_>>();

    for program in ["foo", "bar"] {
        // smoelius: Verify IDL was generated.
        let idl_json_path = Path::new(MULTIPLE_PROGRAMS_DIR)
            .join("target/idl")
            .join(program)
            .with_extension("json");
        assert!(idl_json_path.try_exists().unwrap());
        // smoelius: Verify lcov was generated.
        let lib_rs_path = Path::new(MULTIPLE_PROGRAMS_DIR)
            .join("programs")
            .join(program)
            .join("src/lib.rs");
        assert!(source_files.contains(&lib_rs_path));
    }
}

fn yarn(dir: &str) -> Result<()> {
    let mut command = Command::new("yarn");
    command.current_dir(dir);
    let status = command.status().unwrap();
    ensure!(status.success(), "command failed: {command:?}");
    Ok(())
}

fn run_anchor_coverage_and_read_lcov(dir: &str, include_cargo: bool) -> Result<lcov::Report> {
    let mut command = anchor_coverage_command(dir);
    if include_cargo {
        command.env("INCLUDE_CARGO", "1");
    }
    let status = command.status().unwrap();
    ensure!(status.success(), "command failed: {command:?}");
    let lcovs = files_with_extension(Path::new(dir).join("sbf_trace_dir"), "lcov")?;
    let [lcov] = lcovs.as_slice() else {
        panic!("multiple or no lcov files found");
    };
    lcov::Report::from_file(lcov).map_err(Into::into)
}

fn anchor_coverage_command(dir: impl AsRef<Path>) -> Command {
    let mut command = Command::cargo_bin("anchor-coverage").unwrap();
    command.current_dir(dir);
    command
}
