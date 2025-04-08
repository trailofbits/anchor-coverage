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
        // smoelius: "start address" may (LLVM `objdump`) or may not (GNU `objdump`) be followed by
        // a colon (':'). Hence, we cannot simply use `strip_prefix`.
        if !line.starts_with("start address") {
            continue;
        }
        let Some(position) = line.rfind("0x") else {
            continue;
        };
        if let Ok(address) = u64::from_str_radix(&line[position + 2..], 16) {
            return Ok(address);
        }
    }
    bail!(
        "failed to determine start address for `{}`",
        path.as_ref().display()
    );
}
