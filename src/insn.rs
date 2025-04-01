#[derive(Clone, Copy, Default)]
pub struct Insn(u64);

impl std::fmt::Debug for Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // smoelius: Reverse the instructions' bytes so that they appear as they would in a hex
        // dump of the file.
        f.write_fmt(format_args!("0x{:016x}", self.0.swap_bytes()))
    }
}

impl From<u64> for Insn {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
