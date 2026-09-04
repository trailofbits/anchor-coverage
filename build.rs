use std::env::var_os;

fn main() {
    // smoelius: Do not allow Dylint to cause the nested workspaces to be built. Doing so can cause
    // "incompatible version of rustc" errors.
    if var_os("RUSTC_WORKSPACE_WRAPPER").is_none() {
        nested_workspace::build().unwrap();
    }
}
