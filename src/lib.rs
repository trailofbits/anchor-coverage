use addr2line::Loader;
use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use cargo_metadata::MetadataCommand;
use std::{
    collections::BTreeMap,
    env::var_os,
    fs::{metadata, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub const DOCKER_BUILDER_VERSION: &str = "0.0.0";

#[cfg(feature = "__anchor_cli")]
mod anchor_cli_lib;
#[cfg(feature = "__anchor_cli")]
pub use anchor_cli_lib::__build_with_debug;
#[cfg(feature = "__anchor_cli")]
pub use anchor_cli_lib::{
    __get_keypair as get_keypair, __is_hidden as is_hidden, __keys_sync as keys_sync,
};

#[cfg(feature = "__anchor_cli")]
mod anchor_cli_config;
#[cfg(feature = "__anchor_cli")]
use anchor_cli_config as config;
#[cfg(feature = "__anchor_cli")]
pub use anchor_cli_config::{BootstrapMode, ConfigOverride, ProgramArch};

mod insn;
use insn::Insn;

mod start_address;
use start_address::start_address;

pub mod util;
use util::{files_with_extension, StripCurrentDir};

mod vaddr;
use vaddr::Vaddr;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Entry<'a> {
    file: &'a str,
    line: u32,
}

struct Dwarf {
    path: PathBuf,
    start_address: u64,
    #[allow(dead_code, reason = "`vaddr` points into `loader`")]
    loader: &'static Loader,
    vaddr_entry_map: BTreeMap<u64, Entry<'static>>,
}

enum Outcome {
    Lcov(PathBuf),
    ClosestMatch(PathBuf),
}

type Vaddrs = Vec<u64>;

type VaddrEntryMap<'a> = BTreeMap<u64, Entry<'a>>;

#[allow(dead_code)]
#[derive(Debug)]
struct ClosestMatch<'a, 'b> {
    pcs_path: &'a Path,
    debug_path: &'b Path,
    mismatch: Mismatch,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default)]
struct Mismatch {
    index: usize,
    vaddr: Vaddr,
    expected: Insn,
    actual: Insn,
}

type FileLineCountMap<'a> = BTreeMap<&'a str, BTreeMap<u32, usize>>;

pub fn run(sbf_trace_dir: impl AsRef<Path>, debug: bool) -> Result<()> {
    let mut lcov_paths = Vec::new();
    let mut closest_match_paths = Vec::new();

    let debug_paths = debug_paths()?;

    let dwarfs = debug_paths
        .into_iter()
        .map(|path| build_dwarf(&path))
        .collect::<Result<Vec<_>>>()?;

    if dwarfs.is_empty() {
        eprintln!("Found no debug files");
        return Ok(());
    }

    if debug {
        for dwarf in dwarfs {
            dump_vaddr_entry_map(dwarf.vaddr_entry_map);
        }
        return Ok(());
    }

    let pcs_paths = files_with_extension(&sbf_trace_dir, "pcs")?;

    for pcs_path in &pcs_paths {
        match process_pcs_path(&dwarfs, pcs_path)? {
            Outcome::Lcov(lcov_path) => {
                lcov_paths.push(lcov_path.strip_current_dir().to_path_buf());
            }
            Outcome::ClosestMatch(closest_match_path) => {
                closest_match_paths.push(closest_match_path.strip_current_dir().to_path_buf());
            }
        }
    }

    eprintln!(
        "
Processed {} of {} program counter files

Lcov files written: {lcov_paths:#?}

Closest match files written: {closest_match_paths:#?}

If you are done generating lcov files, try running:

    genhtml --output-directory coverage {}/*.lcov && open coverage/index.html
",
        lcov_paths.len(),
        pcs_paths.len(),
        sbf_trace_dir.as_ref().strip_current_dir().display()
    );

    Ok(())
}

fn debug_paths() -> Result<Vec<PathBuf>> {
    let metadata = MetadataCommand::new().no_deps().exec()?;
    let target_directory = metadata.target_directory;
    files_with_extension(target_directory.join("deploy"), "debug")
}

fn build_dwarf(debug_path: &Path) -> Result<Dwarf> {
    let start_address = start_address(debug_path)?;

    let loader = Loader::new(debug_path).map_err(|error| {
        anyhow!(
            "failed to build loader for {}: {}",
            debug_path.display(),
            error.to_string()
        )
    })?;

    let loader = Box::leak(Box::new(loader));

    let vaddr_entry_map = build_vaddr_entry_map(loader, debug_path)?;

    Ok(Dwarf {
        path: debug_path.to_path_buf(),
        start_address,
        loader,
        vaddr_entry_map,
    })
}

fn process_pcs_path(dwarfs: &[Dwarf], pcs_path: &Path) -> Result<Outcome> {
    eprintln!();
    eprintln!(
        "Program counters file: {}",
        pcs_path.strip_current_dir().display()
    );

    let mut vaddrs = read_vaddrs(pcs_path)?;

    eprintln!("Program counters read: {}", vaddrs.len());

    let (dwarf, mismatch) = find_applicable_dwarf(dwarfs, pcs_path, &mut vaddrs)?;

    if let Some(mismatch) = mismatch {
        return write_closest_match(pcs_path, dwarf, mismatch).map(Outcome::ClosestMatch);
    }

    eprintln!(
        "Applicable dwarf: {}",
        dwarf.path.strip_current_dir().display()
    );

    assert!(vaddrs
        .first()
        .is_some_and(|&vaddr| vaddr == dwarf.start_address));

    // smoelius: If a sequence of program counters refer to the same file and line, treat them as
    // one hit to that file and line.
    vaddrs.dedup_by_key::<_, Option<&Entry>>(|vaddr| dwarf.vaddr_entry_map.get(vaddr));

    // smoelius: A `vaddr` could not have an entry because its file does not exist. Keep only those
    // `vaddr`s that have entries.
    let vaddrs = vaddrs
        .into_iter()
        .filter(|vaddr| dwarf.vaddr_entry_map.contains_key(vaddr))
        .collect::<Vec<_>>();

    eprintln!("Line hits: {}", vaddrs.len());

    let file_line_count_map = build_file_line_count_map(&dwarf.vaddr_entry_map, vaddrs);

    write_lcov_file(pcs_path, file_line_count_map).map(Outcome::Lcov)
}

static CARGO_HOME: std::sync::LazyLock<PathBuf> = std::sync::LazyLock::new(|| {
    if let Some(cargo_home) = var_os("CARGO_HOME") {
        PathBuf::from(cargo_home)
    } else {
        #[allow(deprecated)]
        #[cfg_attr(
            dylint_lib = "inconsistent_qualification",
            allow(inconsistent_qualification)
        )]
        std::env::home_dir().unwrap().join(".cargo")
    }
});

fn build_vaddr_entry_map<'a>(loader: &'a Loader, debug_path: &Path) -> Result<VaddrEntryMap<'a>> {
    let mut vaddr_entry_map = VaddrEntryMap::new();
    let metadata = metadata(debug_path)?;
    for vaddr in (0..metadata.len()).step_by(size_of::<u64>()) {
        let location = loader.find_location(vaddr).map_err(|error| {
            anyhow!(
                "failed to find location for address 0x{vaddr:x}: {}",
                error.to_string()
            )
        })?;
        let Some(location) = location else {
            continue;
        };
        let Some(file) = location.file else {
            continue;
        };
        // smoelius: Ignore files that do not exist.
        if !Path::new(file).try_exists()? {
            continue;
        }
        if !include_cargo() && file.starts_with(CARGO_HOME.to_string_lossy().as_ref()) {
            continue;
        }
        let Some(line) = location.line else {
            continue;
        };
        // smoelius: Even though we ignore columns, fetch them should we ever want to act on them.
        let Some(_column) = location.column else {
            continue;
        };
        let entry = vaddr_entry_map.entry(vaddr).or_default();
        entry.file = file;
        entry.line = line;
    }
    Ok(vaddr_entry_map)
}

fn dump_vaddr_entry_map(vaddr_entry_map: BTreeMap<u64, Entry<'_>>) {
    let mut prev = String::new();
    for (vaddr, Entry { file, line }) in vaddr_entry_map {
        let curr = format!("{file}:{line}");
        if prev != curr {
            eprintln!("0x{vaddr:x}: {curr}");
            prev = curr;
        }
    }
}

fn read_vaddrs(pcs_path: &Path) -> Result<Vaddrs> {
    let mut vaddrs = Vaddrs::new();
    let mut pcs_file = File::open(pcs_path)?;
    while let Ok(pc) = pcs_file.read_u64::<LittleEndian>() {
        let vaddr = pc << 3;
        vaddrs.push(vaddr);
    }
    Ok(vaddrs)
}

fn find_applicable_dwarf<'a>(
    dwarfs: &'a [Dwarf],
    pcs_path: &Path,
    vaddrs: &mut [u64],
) -> Result<(&'a Dwarf, Option<Mismatch>)> {
    let dwarf_mismatches = collect_dwarf_mismatches(dwarfs, pcs_path, vaddrs)?;

    if let Some((dwarf, _)) = dwarf_mismatches
        .iter()
        .find(|(_, mismatch)| mismatch.is_none())
    {
        let vaddr_first = *vaddrs.first().unwrap();

        assert!(dwarf.start_address >= vaddr_first);

        let shift = dwarf.start_address - vaddr_first;

        // smoelius: Make the shift "permanent".
        for vaddr in vaddrs.iter_mut() {
            *vaddr += shift;
        }

        return Ok((dwarf, None));
    }

    Ok(dwarf_mismatches
        .into_iter()
        .max_by_key(|(_, mismatch)| mismatch.as_ref().unwrap().index)
        .unwrap())
}

fn collect_dwarf_mismatches<'a>(
    dwarfs: &'a [Dwarf],
    pcs_path: &Path,
    vaddrs: &[u64],
) -> Result<Vec<(&'a Dwarf, Option<Mismatch>)>> {
    dwarfs
        .iter()
        .map(|dwarf| {
            let mismatch = dwarf_mismatch(vaddrs, dwarf, pcs_path)?;
            Ok((dwarf, mismatch))
        })
        .collect()
}

fn dwarf_mismatch(vaddrs: &[u64], dwarf: &Dwarf, pcs_path: &Path) -> Result<Option<Mismatch>> {
    use std::io::{Seek, SeekFrom};

    let Some(&vaddr_first) = vaddrs.first() else {
        return Ok(Some(Mismatch::default()));
    };

    if dwarf.start_address < vaddr_first {
        return Ok(Some(Mismatch::default()));
    }

    // smoelius: `start_address` is both an offset into the ELF file and a virtual address. The
    // current virtual addresses are offsets from the start of the text section. The current virtual
    // addresses must be shifted so that the first matches the start address.
    let shift = dwarf.start_address - vaddr_first;

    let mut so_file = File::open(dwarf.path.with_extension("so"))?;
    let mut insns_file = File::open(pcs_path.with_extension("insns"))?;

    for (index, &vaddr) in vaddrs.iter().enumerate() {
        let vaddr = vaddr + shift;

        so_file.seek(SeekFrom::Start(vaddr))?;
        let expected = so_file.read_u64::<LittleEndian>()?;

        let actual = insns_file.read_u64::<LittleEndian>()?;

        // smoelius: 0x85 is a function call. That they would be patched and differ is not
        // surprising.
        if expected & 0xff == 0x85 {
            continue;
        }

        if expected != actual {
            return Ok(Some(Mismatch {
                index,
                vaddr: Vaddr::from(vaddr),
                expected: Insn::from(expected),
                actual: Insn::from(actual),
            }));
        }
    }

    Ok(None)
}

fn write_closest_match(pcs_path: &Path, dwarf: &Dwarf, mismatch: Mismatch) -> Result<PathBuf> {
    let closest_match_path = pcs_path.with_extension("closest_match");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&closest_match_path)?;
    writeln!(
        file,
        "{:#?}",
        ClosestMatch {
            pcs_path,
            debug_path: &dwarf.path,
            mismatch
        }
    )?;
    Ok(closest_match_path)
}

fn build_file_line_count_map<'a>(
    vaddr_entry_map: &BTreeMap<u64, Entry<'a>>,
    vaddrs: Vaddrs,
) -> FileLineCountMap<'a> {
    let mut file_line_count_map = FileLineCountMap::new();
    for Entry { file, line } in vaddr_entry_map.values() {
        let line_count_map = file_line_count_map.entry(file).or_default();
        line_count_map.insert(*line, 0);
    }

    for vaddr in vaddrs {
        // smoelius: A `vaddr` could not have an entry because its file does not exist.
        let Some(entry) = vaddr_entry_map.get(&vaddr) else {
            continue;
        };
        let line_count_map = file_line_count_map.get_mut(entry.file).unwrap();
        let count = line_count_map.get_mut(&entry.line).unwrap();
        *count += 1;
    }

    file_line_count_map
}

fn write_lcov_file(pcs_path: &Path, file_line_count_map: FileLineCountMap<'_>) -> Result<PathBuf> {
    let lcov_path = Path::new(pcs_path).with_extension("lcov");

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&lcov_path)?;

    for (source_file, line_count_map) in file_line_count_map {
        // smoelius: Stripping `current_dir` from `source_file` has not effect on what's displayed.
        writeln!(file, "SF:{source_file}")?;
        for (line, count) in line_count_map {
            writeln!(file, "DA:{line},{count}")?;
        }
        writeln!(file, "end_of_record")?;
    }

    Ok(lcov_path)
}

fn include_cargo() -> bool {
    var_os("INCLUDE_CARGO").is_some()
}

#[cfg(test)]
mod tests;

#[test]
fn nested_workspace() {
    nested_workspace::test().unwrap();
}
