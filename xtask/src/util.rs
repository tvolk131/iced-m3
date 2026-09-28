use anyhow::{Context, Result, ensure};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};
use walkdir::WalkDir;

pub fn sha(data: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(data.as_ref()))
}
pub fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).with_context(|| format!("Reading {}", path.display()))
}
pub fn json(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut data = serde_json::to_vec_pretty(value)?;
    data.push(b'\n');
    fs::write(path, data)?;
    Ok(())
}
pub fn command(
    root: &Path,
    program: &str,
    args: &[&str],
    env: &[(&str, &str)],
    capture: bool,
) -> Result<String> {
    let mut c = Command::new(program);
    c.args(args).current_dir(root).envs(env.iter().copied());
    if capture {
        c.stdout(Stdio::piped());
    }
    let output = c
        .spawn()
        .with_context(|| format!("Starting {program}"))?
        .wait_with_output()?;
    ensure!(
        output.status.success(),
        "{program} {} failed: {}",
        args.join(" "),
        output.status
    );
    Ok(String::from_utf8(output.stdout)?)
}
pub fn rustfmt(source: &[u8]) -> Result<String> {
    let mut c = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    c.stdin.take().context("rustfmt stdin")?.write_all(source)?;
    let out = c.wait_with_output()?;
    ensure!(out.status.success(), "rustfmt failed");
    Ok(String::from_utf8(out.stdout)?)
}
pub fn files(root: &Path) -> Result<BTreeMap<String, PathBuf>> {
    let mut result = BTreeMap::new();
    for entry in WalkDir::new(root) {
        let entry = entry?;
        ensure!(
            !entry.file_type().is_symlink(),
            "Symlink in input/output: {}",
            entry.path().display()
        );
        if entry.file_type().is_file() {
            result.insert(
                entry
                    .path()
                    .strip_prefix(root)?
                    .to_string_lossy()
                    .replace('\\', "/"),
                entry.path().into(),
            );
        }
    }
    Ok(result)
}
pub fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    for (name, path) in files(from)? {
        let dest = to.join(name);
        fs::create_dir_all(dest.parent().unwrap())?;
        fs::copy(path, dest)?;
    }
    Ok(())
}
/// Delete only descendants of the tool's workspace, rejecting symlinks and `..`.
pub fn fresh_dir(work: &Path, path: &Path) -> Result<()> {
    let relative = path
        .strip_prefix(work)
        .context("Not an owned generation directory")?;
    ensure!(
        relative.components().next().is_some()
            && relative
                .components()
                .all(|c| matches!(c, Component::Normal(_))),
        "Not an owned generation directory"
    );
    fs::create_dir_all(work)?;
    let base = work.canonicalize()?;
    let mut cursor = work.to_path_buf();
    for part in relative.components() {
        cursor.push(part);
        if let Ok(meta) = fs::symlink_metadata(&cursor) {
            ensure!(
                !meta.file_type().is_symlink() && cursor.canonicalize()?.starts_with(&base),
                "Not an owned generation directory"
            );
        }
    }
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)?;
    Ok(())
}
