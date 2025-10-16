// smoelius: `VarGuard` is based on a struct of the same name from Clippy's source code. The `set`
// method was modified to accept an `Option` so that variables could be cleared. Also, `unsafe`
// annotations were added to make the struct compile with Rust Edition 2024. See:
// https://github.com/rust-lang/rust-clippy/blob/c51472b4b09d22bdbb46027f08be54c4b285a725/tests/compile-test.rs#L267-L289

use std::{
    env::{remove_var, set_var, var_os},
    ffi::{OsStr, OsString},
    marker::PhantomData,
};

/// Restores an env var on drop
#[must_use]
pub struct VarGuard<T = OsString> {
    key: &'static str,
    value: Option<OsString>,
    _phantom: PhantomData<T>,
}

impl<T: AsRef<OsStr>> VarGuard<T> {
    pub fn set(key: &'static str, val: Option<T>) -> Self {
        let value = var_os(key);
        if let Some(val) = val {
            unsafe { set_var(key, val) };
        } else {
            unsafe { remove_var(key) };
        }
        Self {
            key,
            value,
            _phantom: PhantomData,
        }
    }
}

impl<T> Drop for VarGuard<T> {
    fn drop(&mut self) {
        match self.value.as_deref() {
            None => unsafe { remove_var(self.key) },
            Some(value) => unsafe { set_var(self.key, value) },
        }
    }
}
