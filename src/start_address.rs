//! Gets an ELF's start address by reading its `Elf64Hdr`.

use anyhow::Result;
use std::{fs::File, io::Read, path::Path, slice::from_raw_parts_mut};

const EI_NIDENT: usize = 16;

#[allow(clippy::struct_field_names)]
#[derive(Default)]
#[repr(C)]
pub struct Elf64Hdr {
    pub e_ident: [u8; EI_NIDENT],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

pub fn start_address(path: impl AsRef<Path>) -> Result<u64> {
    let mut file = File::open(path)?;
    let mut elf64_hdr = Elf64Hdr::default();
    let buf: &mut [u8] =
        unsafe { from_raw_parts_mut((&raw mut elf64_hdr).cast::<u8>(), size_of::<Elf64Hdr>()) };
    file.read_exact(buf)?;
    Ok(elf64_hdr.e_entry)
}
