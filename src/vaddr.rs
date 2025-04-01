#[derive(Clone, Copy, Default)]
pub struct Vaddr(u64);

impl std::fmt::Debug for Vaddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("0x{:x}", self.0))
    }
}

impl From<u64> for Vaddr {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
