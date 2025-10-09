use super::*;

// smoelius: Avoid "`Stdout` is never constructed" warning.
#[allow(dead_code)]
const STDOUT: OutFile = OutFile::Stdout;

// smoelius: `__build_with_debug` is the parent module's main export.

#[allow(clippy::too_many_arguments)]
pub fn __build_with_debug(
    cfg_override: &ConfigOverride,
    no_idl: bool,
    idl: Option<String>,
    idl_ts: Option<String>,
    verifiable: bool,
    skip_lint: bool,
    program_name: Option<String>,
    solana_version: Option<String>,
    docker_image: Option<String>,
    bootstrap: BootstrapMode,
    stdout: Option<File>, // Used for the package registry server.
    stderr: Option<File>, // Used for the package registry server.
    env_vars: Vec<String>,
    cargo_args: Vec<String>,
    no_docs: bool,
    arch: ProgramArch,
) -> Result<()> {
    build(
        cfg_override,
        no_idl,
        idl,
        idl_ts,
        verifiable,
        skip_lint,
        program_name,
        solana_version,
        docker_image,
        bootstrap,
        stdout,
        stderr,
        env_vars,
        cargo_args,
        no_docs,
        arch,
    )
}

// smoelius: The next three functions are needed by the `anchor_cli_config` module.

pub fn __get_keypair(path: &str) -> Result<Keypair> {
    get_keypair(path)
}

pub fn __is_hidden(entry: &walkdir::DirEntry) -> bool {
    is_hidden(entry)
}

pub fn __keys_sync(cfg_override: &ConfigOverride, program_name: Option<String>) -> Result<()> {
    keys_sync(cfg_override, program_name)
}

// smoelius: The remaining functions are stand-ins for Anchor functions with the same names.

pub fn cd_member(_cfg_override: &ConfigOverride, _program_name: &str) -> Result<()> {
    Ok(())
}

pub fn check_overflow(_cargo_toml_path: impl AsRef<Path>) -> Result<bool> {
    Ok(false)
}

pub fn check_anchor_version(_cfg: &WithPath<Config>) -> Result<()> {
    Ok(())
}

pub fn check_deps(_cfg: &WithPath<Config>) -> Result<()> {
    Ok(())
}

pub fn build_cwd_verifiable(
    _cfg: &WithPath<Config>,
    _cargo_toml: PathBuf,
    _build_config: &BuildConfig,
    _stdout: Option<File>,
    _stderr: Option<File>,
    _skip_lint: bool,
    _env_vars: Vec<String>,
    _cargo_args: Vec<String>,
    _no_docs: bool,
    _arch: &ProgramArch,
) -> Result<()> {
    Ok(())
}

pub fn check_idl_build_feature() -> Result<()> {
    Ok(())
}
