//! A hack to get an ELF's start address by calling `objdump` on the command line.
//!
//! A proper solution would use [`gimli`] or something similar.
//!
//! [`gimli`]: https://crates.io/crates/gimli

use anyhow::{Result, bail, ensure};
use assert_cmd::output::OutputError;
use std::{path::Path, process::Command};

pub fn start_address(path: impl AsRef<Path>) -> Result<u64> {
    let mut command = Command::new("objdump");
    command.arg("-f");
    command.arg(path.as_ref());
    let output = command.output()?;
    ensure!(
        output.status.success(),
        "command failed `{command:?}`: {}",
        OutputError::new(output)
    );
    let stdout = std::str::from_utf8(&output.stdout)?;
    for line in stdout.lines() {
        if let Some(suffix) = line.strip_prefix("start address: 0x") {
            if let Ok(address) = u64::from_str_radix(suffix, 16) {
                return Ok(address);
            }
        }
    }
    bail!(
        "failed to determine start address for `{}`",
        path.as_ref().display()
    );
}
