use crate::util::files_with_extension;
use anyhow::{Result, ensure};
use assert_cmd::cargo::CommandCargoExt;
use std::{path::Path, process::Command};

const CALL_EXTERNAL_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/call_external");

#[test]
fn include_cargo_does_not_change_line_hits() {
    let report_without_cargo = run_anchor_coverage_and_read_lcov(CALL_EXTERNAL_DIR, false).unwrap();

    let report_with_cargo = run_anchor_coverage_and_read_lcov(CALL_EXTERNAL_DIR, true).unwrap();

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
