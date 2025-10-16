use anyhow::{bail, Result};
use std::{
    env::current_dir,
    ffi::OsStr,
    fs::read_dir,
    path::{Path, PathBuf},
};

pub mod var_guard;

pub fn files_with_extension(dir: impl AsRef<Path>, extension: &str) -> Result<Vec<PathBuf>> {
    let mut pcs_paths = Vec::new();
    for result in read_dir(dir)? {
        let entry = result?;
        let path = entry.path();
        if path.is_file() && path.extension() == Some(OsStr::new(extension)) {
            pcs_paths.push(path);
        }
    }
    Ok(pcs_paths)
}

pub fn patched_agave_tools(path: impl AsRef<Path>) -> Result<Option<PathBuf>> {
    let mut path_bufs = Vec::new();
    for result in read_dir(path)? {
        let entry = result?;
        let path = entry.path();
        let Some(file_name) = path.file_name() else {
            continue;
        };
        if file_name
            .to_str()
            .is_none_or(|s| !s.starts_with("patched-agave-tools-"))
        {
            continue;
        }
        if !path.is_dir() {
            eprintln!(
                "Warning: Found `{}` but it is not a directory. If it contains patched Agave \
                 tools that you want to use, please unzip and untar it.",
                path.display()
            );
            continue;
        }
        path_bufs.push(path);
    }
    let mut iter = path_bufs.into_iter();
    let Some(path_buf) = iter.next() else {
        return Ok(None);
    };
    if iter.next().is_some() {
        bail!("Found multiple patched Agave tools directories");
    }
    Ok(Some(path_buf))
}

pub trait StripCurrentDir {
    fn strip_current_dir(&self) -> &Self;
}

impl StripCurrentDir for Path {
    fn strip_current_dir(&self) -> &Self {
        let Ok(current_dir) = current_dir() else {
            return self;
        };
        self.strip_prefix(current_dir).unwrap_or(self)
    }
}
