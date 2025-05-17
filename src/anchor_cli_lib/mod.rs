// smoelius: This file is essentially the portions of the following file needed to build an Anchor
// workspace:
//
//     https://github.com/solana-foundation/anchor/blob/v0.31.1/cli/src/lib.rs
//
// A small addition has been made to `_build_rust_cwd` to pass `--debug` to `cargo-build-sbf`.
//
// See the following issue for context: https://github.com/solana-foundation/anchor/issues/3643

#![allow(unused_imports)]
#![allow(clippy::all, clippy::pedantic)]
#![cfg_attr(
    dylint_lib = "inconsistent_qualification",
    allow(inconsistent_qualification)
)]

use crate::config::{
    get_default_ledger_path, AnchorPackage, BootstrapMode, BuildConfig, Config, ConfigOverride,
    Manifest, PackageManager, ProgramArch, ProgramDeployment, ProgramWorkspace, ScriptsConfig,
    TestValidator, WithPath, SHUTDOWN_WAIT, STARTUP_WAIT,
};
use anchor_client::Cluster;
use anchor_lang::idl::{IdlAccount, IdlInstruction, ERASED_AUTHORITY};
use anchor_lang::{AccountDeserialize, AnchorDeserialize, AnchorSerialize, Discriminator};
// use anchor_lang_idl::convert::convert_idl;
use anchor_lang_idl::types::{Idl, IdlArrayLen, IdlDefinedFields, IdlType, IdlTypeDefTy};
use anyhow::{anyhow, Context, Result};
// use checks::{check_anchor_version, check_deps, check_idl_build_feature, check_overflow};
use clap::{CommandFactory, Parser};
use dirs::home_dir;
// use flate2::read::GzDecoder;
// use flate2::read::ZlibDecoder;
// use flate2::write::{GzEncoder, ZlibEncoder};
// use flate2::Compression;
use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use regex::{Regex, RegexBuilder};
// use reqwest::blocking::multipart::{Form, Part};
// use reqwest::blocking::Client;
// use rust_template::{ProgramTemplate, TestTemplate};
// use semver::{Version, VersionReq};
use serde::Deserialize;
use serde_json::{json, Map, Value as JsonValue};
// use solana_client::rpc_client::RpcClient;
use solana_sdk::account_utils::StateMut;
use solana_sdk::bpf_loader;
use solana_sdk::bpf_loader_deprecated;
use solana_sdk::bpf_loader_upgradeable::{self, UpgradeableLoaderState};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use solana_sdk::signer::EncodableKey;
use solana_sdk::transaction::Transaction;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::str::FromStr;
use std::string::ToString;
// use tar::Archive;

mod hacks;
pub use hacks::*;

fn get_keypair(path: &str) -> Result<Keypair> {
    solana_sdk::signature::read_keypair_file(path)
        .map_err(|_| anyhow!("Unable to read keypair file ({path})"))
}

#[allow(clippy::too_many_arguments)]
pub fn build(
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
    // Change to the workspace member directory, if needed.
    if let Some(program_name) = program_name.as_ref() {
        cd_member(cfg_override, program_name)?;
    }
    let cfg = Config::discover(cfg_override)?.expect("Not in workspace.");
    let cfg_parent = cfg.path().parent().expect("Invalid Anchor.toml");

    // Require overflow checks
    let workspace_cargo_toml_path = cfg_parent.join("Cargo.toml");
    if workspace_cargo_toml_path.exists() {
        check_overflow(workspace_cargo_toml_path)?;
    }

    // Check whether there is a mismatch between CLI and crate/package versions
    check_anchor_version(&cfg).ok();
    check_deps(&cfg).ok();

    let idl_out = match idl {
        Some(idl) => Some(PathBuf::from(idl)),
        None => Some(cfg_parent.join("target").join("idl")),
    };
    fs::create_dir_all(idl_out.as_ref().unwrap())?;

    let idl_ts_out = match idl_ts {
        Some(idl_ts) => Some(PathBuf::from(idl_ts)),
        None => Some(cfg_parent.join("target").join("types")),
    };
    fs::create_dir_all(idl_ts_out.as_ref().unwrap())?;

    if !cfg.workspace.types.is_empty() {
        fs::create_dir_all(cfg_parent.join(&cfg.workspace.types))?;
    };

    let cargo = Manifest::discover()?;
    let build_config = BuildConfig {
        verifiable,
        solana_version: solana_version.or_else(|| cfg.toolchain.solana_version.clone()),
        docker_image: docker_image.unwrap_or_else(|| cfg.docker()),
        bootstrap,
    };
    match cargo {
        // No Cargo.toml so build the entire workspace.
        None => build_all(
            &cfg,
            cfg.path(),
            no_idl,
            idl_out,
            idl_ts_out,
            &build_config,
            stdout,
            stderr,
            env_vars,
            cargo_args,
            skip_lint,
            no_docs,
            arch,
        )?,
        // If the Cargo.toml is at the root, build the entire workspace.
        Some(cargo) if cargo.path().parent() == cfg.path().parent() => build_all(
            &cfg,
            cfg.path(),
            no_idl,
            idl_out,
            idl_ts_out,
            &build_config,
            stdout,
            stderr,
            env_vars,
            cargo_args,
            skip_lint,
            no_docs,
            arch,
        )?,
        // Cargo.toml represents a single package. Build it.
        Some(cargo) => build_rust_cwd(
            &cfg,
            cargo.path().to_path_buf(),
            no_idl,
            idl_out,
            idl_ts_out,
            &build_config,
            stdout,
            stderr,
            env_vars,
            cargo_args,
            skip_lint,
            no_docs,
            &arch,
        )?,
    }

    set_workspace_dir_or_exit();

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_all(
    cfg: &WithPath<Config>,
    cfg_path: &Path,
    no_idl: bool,
    idl_out: Option<PathBuf>,
    idl_ts_out: Option<PathBuf>,
    build_config: &BuildConfig,
    stdout: Option<File>, // Used for the package registry server.
    stderr: Option<File>, // Used for the package registry server.
    env_vars: Vec<String>,
    cargo_args: Vec<String>,
    skip_lint: bool,
    no_docs: bool,
    arch: ProgramArch,
) -> Result<()> {
    let cur_dir = std::env::current_dir()?;
    let r = match cfg_path.parent() {
        None => Err(anyhow!("Invalid Anchor.toml at {}", cfg_path.display())),
        Some(_parent) => {
            for p in cfg.get_rust_program_list()? {
                build_rust_cwd(
                    cfg,
                    p.join("Cargo.toml"),
                    no_idl,
                    idl_out.clone(),
                    idl_ts_out.clone(),
                    build_config,
                    stdout.as_ref().map(|f| f.try_clone()).transpose()?,
                    stderr.as_ref().map(|f| f.try_clone()).transpose()?,
                    env_vars.clone(),
                    cargo_args.clone(),
                    skip_lint,
                    no_docs,
                    &arch,
                )?;
            }
            for (name, path) in cfg.get_solidity_program_list()? {
                build_solidity_cwd(
                    cfg,
                    name,
                    path,
                    idl_out.clone(),
                    idl_ts_out.clone(),
                    build_config,
                    stdout.as_ref().map(|f| f.try_clone()).transpose()?,
                    stderr.as_ref().map(|f| f.try_clone()).transpose()?,
                    cargo_args.clone(),
                )?;
            }
            Ok(())
        }
    };
    std::env::set_current_dir(cur_dir)?;
    r
}

// Runs the build command outside of a workspace.
#[allow(clippy::too_many_arguments)]
fn build_rust_cwd(
    cfg: &WithPath<Config>,
    cargo_toml: PathBuf,
    no_idl: bool,
    idl_out: Option<PathBuf>,
    idl_ts_out: Option<PathBuf>,
    build_config: &BuildConfig,
    stdout: Option<File>,
    stderr: Option<File>,
    env_vars: Vec<String>,
    cargo_args: Vec<String>,
    skip_lint: bool,
    no_docs: bool,
    arch: &ProgramArch,
) -> Result<()> {
    match cargo_toml.parent() {
        None => return Err(anyhow!("Unable to find parent")),
        Some(p) => std::env::set_current_dir(p)?,
    };
    match build_config.verifiable {
        false => _build_rust_cwd(
            cfg, no_idl, idl_out, idl_ts_out, skip_lint, no_docs, arch, cargo_args,
        ),
        true => build_cwd_verifiable(
            cfg,
            cargo_toml,
            build_config,
            stdout,
            stderr,
            skip_lint,
            env_vars,
            cargo_args,
            no_docs,
            arch,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn _build_rust_cwd(
    cfg: &WithPath<Config>,
    no_idl: bool,
    idl_out: Option<PathBuf>,
    idl_ts_out: Option<PathBuf>,
    skip_lint: bool,
    no_docs: bool,
    arch: &ProgramArch,
    cargo_args: Vec<String>,
) -> Result<()> {
    let exit = std::process::Command::new("cargo")
        .arg(arch.build_subcommand())
        // smoelius: The next call to `arg` does not appear in the original.
        .arg("--debug")
        .args(cargo_args.clone())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| anyhow::format_err!("{}", e.to_string()))?;
    if !exit.status.success() {
        std::process::exit(exit.status.code().unwrap_or(1));
    }

    // Generate IDL
    if !no_idl {
        let idl = generate_idl(cfg, skip_lint, no_docs, &cargo_args)?;

        // JSON out path.
        let out = match idl_out {
            None => PathBuf::from(".")
                .join(&idl.metadata.name)
                .with_extension("json"),
            Some(o) => PathBuf::from(&o.join(&idl.metadata.name).with_extension("json")),
        };
        // TS out path.
        let ts_out = match idl_ts_out {
            None => PathBuf::from(".")
                .join(&idl.metadata.name)
                .with_extension("ts"),
            Some(o) => PathBuf::from(&o.join(&idl.metadata.name).with_extension("ts")),
        };

        // Write out the JSON file.
        write_idl(&idl, OutFile::File(out))?;
        // Write out the TypeScript type.
        fs::write(&ts_out, idl_ts(&idl)?)?;

        // Copy out the TypeScript type.
        let cfg_parent = cfg.path().parent().expect("Invalid Anchor.toml");
        if !&cfg.workspace.types.is_empty() {
            fs::copy(
                &ts_out,
                cfg_parent
                    .join(&cfg.workspace.types)
                    .join(&idl.metadata.name)
                    .with_extension("ts"),
            )?;
        }
    }

    Ok(())
}

/// Generate IDL with method decided by whether manifest file has `idl-build` feature or not.
fn generate_idl(
    cfg: &WithPath<Config>,
    skip_lint: bool,
    no_docs: bool,
    cargo_args: &[String],
) -> Result<Idl> {
    check_idl_build_feature()?;

    anchor_lang_idl::build::IdlBuilder::new()
        .resolution(cfg.features.resolution)
        .skip_lint(cfg.features.skip_lint || skip_lint)
        .no_docs(no_docs)
        .cargo_args(cargo_args.into())
        .build()
}

fn idl_ts(idl: &Idl) -> Result<String> {
    let idl_name = &idl.metadata.name;
    let type_name = idl_name.to_pascal_case();
    let idl = serde_json::to_string(idl)?;

    // Convert every field of the IDL to camelCase
    let camel_idl = Regex::new(r#""\w+":"([\w\d]+)""#)?
        .captures_iter(&idl)
        .fold(idl.clone(), |acc, cur| {
            let name = cur.get(1).unwrap().as_str();

            // Do not modify pubkeys
            if Pubkey::from_str(name).is_ok() {
                return acc;
            }

            let camel_name = name.to_lower_camel_case();
            acc.replace(&format!(r#""{name}""#), &format!(r#""{camel_name}""#))
        });

    // Pretty format
    let camel_idl = serde_json::to_string_pretty(&serde_json::from_str::<Idl>(&camel_idl)?)?;

    Ok(format!(
        r#"/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/{idl_name}.json`.
 */
export type {type_name} = {camel_idl};
"#
    ))
}

fn write_idl(idl: &Idl, out: OutFile) -> Result<()> {
    let idl_json = serde_json::to_string_pretty(idl)?;
    match out {
        OutFile::Stdout => println!("{idl_json}"),
        OutFile::File(out) => fs::write(out, idl_json)?,
    };

    Ok(())
}

enum OutFile {
    Stdout,
    File(PathBuf),
}

fn set_workspace_dir_or_exit() {
    let d = match Config::discover(&ConfigOverride::default()) {
        Err(err) => {
            println!("Workspace configuration error: {err}");
            std::process::exit(1);
        }
        Ok(d) => d,
    };
    match d {
        None => {
            println!("Not in anchor workspace.");
            std::process::exit(1);
        }
        Some(cfg) => {
            match cfg.path().parent() {
                None => {
                    println!("Unable to make new program");
                }
                Some(parent) => {
                    if std::env::set_current_dir(parent).is_err() {
                        println!("Not in anchor workspace.");
                        std::process::exit(1);
                    }
                }
            };
        }
    }
}

/// Sync program `declare_id!` pubkeys with the pubkey from `target/deploy/<KEYPAIR>.json`.
fn keys_sync(cfg_override: &ConfigOverride, program_name: Option<String>) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        let declare_id_regex = RegexBuilder::new(r#"^(([\w]+::)*)declare_id!\("(\w*)"\)"#)
            .multi_line(true)
            .build()
            .unwrap();

        let mut changed_src = false;
        for program in cfg.get_programs(program_name)? {
            // Get the pubkey from the keypair file
            let actual_program_id = program.pubkey()?.to_string();

            // Handle declaration in program files
            let src_path = program.path.join("src");
            let files_to_check = vec![src_path.join("lib.rs"), src_path.join("id.rs")];

            for path in files_to_check {
                let mut content = match fs::read_to_string(&path) {
                    Ok(content) => content,
                    Err(_) => continue,
                };

                let incorrect_program_id = declare_id_regex
                    .captures(&content)
                    .and_then(|captures| captures.get(3))
                    .filter(|program_id_match| program_id_match.as_str() != actual_program_id);
                if let Some(program_id_match) = incorrect_program_id {
                    println!("Found incorrect program id declaration in {path:?}");

                    // Update the program id
                    content.replace_range(program_id_match.range(), &actual_program_id);
                    fs::write(&path, content)?;

                    changed_src = true;
                    println!("Updated to {actual_program_id}\n");
                    break;
                }
            }

            // Handle declaration in Anchor.toml
            'outer: for programs in cfg.programs.values_mut() {
                for (name, deployment) in programs {
                    // Skip other programs
                    if name != &program.lib_name {
                        continue;
                    }

                    if deployment.address.to_string() != actual_program_id {
                        println!(
                            "Found incorrect program id declaration in Anchor.toml for the \
                             program `{name}`"
                        );

                        // Update the program id
                        deployment.address = Pubkey::from_str(&actual_program_id).unwrap();
                        fs::write(cfg.path(), cfg.to_string())?;

                        println!("Updated to {actual_program_id}\n");
                        break 'outer;
                    }
                }
            }
        }

        println!("All program id declarations are synced.");
        if changed_src {
            println!("Please rebuild the program to update the generated artifacts.")
        }

        Ok(())
    })
}

// with_workspace ensures the current working directory is always the top level
// workspace directory, i.e., where the `Anchor.toml` file is located, before
// and after the closure invocation.
//
// The closure passed into this function must never change the working directory
// to be outside the workspace. Doing so will have undefined behavior.
fn with_workspace<R>(
    cfg_override: &ConfigOverride,
    f: impl FnOnce(&mut WithPath<Config>) -> R,
) -> R {
    set_workspace_dir_or_exit();

    let mut cfg = Config::discover(cfg_override)
        .expect("Previously set the workspace dir")
        .expect("Anchor.toml must always exist");

    let r = f(&mut cfg);

    set_workspace_dir_or_exit();

    r
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s == "." || s.starts_with('.') || s == "target")
        .unwrap_or(false)
}
