use crate::{media::*, util::*};
use anyhow::{Context, Result, ensure};
use flate2::read::GzDecoder;
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path},
};

pub fn metadata(root: &Path, offline: bool) -> Result<Value> {
    let mut args = vec!["metadata", "--locked", "--no-deps", "--format-version=1"];
    if offline {
        args.push("--offline");
    }
    let value: Value = serde_json::from_str(&command(root, "cargo", &args, &[], true)?)?;
    let manifest = root.join("Cargo.toml").canonicalize()?;
    value["packages"]
        .as_array()
        .context("Missing packages")?
        .iter()
        .find(|p| {
            p["manifest_path"]
                .as_str()
                .and_then(|p| Path::new(p).canonicalize().ok())
                .as_ref()
                == Some(&manifest)
        })
        .cloned()
        .context("Source package not in cargo metadata")
}
pub fn stem(package: &Value) -> Result<String> {
    Ok(format!(
        "{}-{}",
        package["name"].as_str().context("Missing name")?,
        package["version"].as_str().context("Missing version")?
    ))
}
#[derive(Debug)]
pub struct Contents {
    pub stem: String,
    pub names: BTreeSet<String>,
    pub bytes: u64,
}
/// Validate every entry before extracting anything. No links, special files,
/// duplicate paths, absolute paths or traversal are accepted on any platform.
pub fn extract(archive: &Path, destination: &Path) -> Result<Contents> {
    let mut tar = tar::Archive::new(GzDecoder::new(fs::File::open(archive)?));
    let mut names = BTreeSet::new();
    let mut stems = BTreeSet::new();
    let mut bytes = 0;
    let mut full_names = BTreeSet::new();
    for entry in tar.entries()? {
        let entry = entry?;
        let path = entry.path()?;
        let path_str = path.to_str().context("Non-UTF8 archive path")?;
        ensure!(
            !path_str.contains(['\\', ':'])
                && path.components().all(|p| matches!(p, Component::Normal(_))),
            "Unsafe archive path {path_str}"
        );
        ensure!(
            entry.header().entry_type().is_file() || entry.header().entry_type().is_dir(),
            "Unsafe archive entry {path_str}"
        );
        ensure!(
            full_names.insert(path.to_path_buf()),
            "Duplicate archive entry {path_str}"
        );
        let mut parts = path.components();
        let first = parts
            .next()
            .context("Empty archive path")?
            .as_os_str()
            .to_str()
            .unwrap();
        stems.insert(first.to_owned());
        let rest = parts.as_path().to_string_lossy().replace('\\', "/");
        if entry.header().entry_type().is_file() {
            ensure!(!rest.is_empty(), "File at archive root");
            names.insert(rest);
        }
        bytes += entry.header().size()?;
    }
    ensure!(stems.len() == 1, "Unexpected archive roots");
    let mut tar = tar::Archive::new(GzDecoder::new(fs::File::open(archive)?));
    tar.unpack(destination)?;
    Ok(Contents {
        stem: stems.into_iter().next().unwrap(),
        names,
        bytes,
    })
}
pub fn check_archive(repo: &Path, archive: &Path) -> Result<String> {
    ensure!(
        fs::metadata(archive)?.len() <= MAX_PACKAGE_BYTES,
        "Illustrated package exceeds the reviewed 9.8 MB compressed budget"
    );
    let destination = repo.join("target/doc-media/extracted");
    fresh_dir(&repo.join("target/doc-media"), &destination)?;
    let contents = extract(archive, &destination)?;
    let root = destination.join(contents.stem);
    build_docs(&root, &repo.join("target"))?;
    Ok(sha(read(archive)?))
}
const REQUIRED: &[&str] = &[
    "src/lib.rs",
    "README.md",
    "CHANGELOG.md",
    "LICENSE",
    "NOTICE",
    "Cargo.lock",
    "assets/fonts/Roboto-Variable.ttf",
    "assets/fonts/OFL.txt",
    "assets/loading/morphs.bin",
    "assets/loading/NOTICE",
    "assets/loading/LICENSE-APACHE-2.0.txt",
    "assets/loading/README.md",
    "assets/loading/SHA256SUMS",
    "tools/generate_loading_shapes.py",
    "tests/visual/mod.rs",
    "tests/visual/progress-preview.html",
    "assets/loading/reference/soft-burst.svg",
    "examples/minimal.rs",
    "examples/gallery/icons.rs",
    "src/guide.rs",
    "docs/GETTING_STARTED.md",
    "docs/COOKBOOK.md",
    "docs/THEMING.md",
    "docs/README.md",
    "docs/desktop-beta.png",
    "tests/visual/doc_media/catalog.rs",
];
pub fn check_contents(contents: &Contents, compressed: u64, require_media: bool) -> Result<bool> {
    for name in REQUIRED {
        ensure!(
            contents.names.contains(*name),
            "Missing package input/notice: {name}"
        );
    }
    for name in &contents.names {
        ensure!(
            ![
                "target/",
                "consumers/",
                ".github/",
                "xtask/",
                "tests/visual/references/"
            ]
            .iter()
            .any(|p| name.starts_with(p)),
            "Development-only file leaked into package: {name}"
        );
    }
    let illustrated = contents.names.contains("docs/generated/manifest.json");
    ensure!(
        !require_media || illustrated,
        "Prepared package is missing generated documentation"
    );
    ensure!(
        compressed
            <= if illustrated {
                MAX_PACKAGE_BYTES
            } else {
                5_000_000
            },
        "Package exceeds compressed byte budget"
    );
    Ok(illustrated)
}
pub fn check_package(repo: &Path, source: &Path, offline: bool, require_media: bool) -> Result<()> {
    let package = metadata(source, offline)?;
    let stem = stem(&package)?;
    let target = repo.join("target");
    fs::create_dir_all(&target)?;
    let target_str = target.to_string_lossy();
    let cargo = |mut args: Vec<&str>| -> Result<()> {
        args.push("--locked");
        if offline {
            args.push("--offline");
        }
        command(
            source,
            "cargo",
            &args,
            &[("CARGO_TARGET_DIR", &target_str)],
            false,
        )?;
        Ok(())
    };
    cargo(vec!["package", "--allow-dirty"])?;
    let archive = target.join(format!("package/{stem}.crate"));
    let temp = tempfile::Builder::new()
        .prefix("consumer-package-")
        .tempdir_in(&target)?;
    let contents = extract(&archive, temp.path())?;
    ensure!(contents.stem == stem, "Unexpected package name");
    let compressed = fs::metadata(&archive)?.len();
    let illustrated = check_contents(&contents, compressed, require_media)?;
    let packaged = temp.path().join(&stem);
    if illustrated {
        validate(&packaged, &packaged.join(GENERATED))?;
    }
    let consumer = temp.path().join("consumer");
    let consumer_source = repo.join("consumers/desktop");
    for entry in walkdir::WalkDir::new(&consumer_source)
        .into_iter()
        .filter_entry(|e| e.file_name() != "target")
    {
        let entry = entry?;
        ensure!(
            !entry.file_type().is_symlink(),
            "Symlink in consumer source"
        );
        if entry.file_type().is_file() {
            let dest = consumer.join(entry.path().strip_prefix(&consumer_source)?);
            fs::create_dir_all(dest.parent().unwrap())?;
            fs::copy(entry.path(), dest)?;
        }
    }
    let manifest = consumer.join("Cargo.toml");
    let original = fs::read_to_string(&manifest)?;
    let needle = "path = \"../..\"";
    ensure!(
        original.matches(needle).count() == 1,
        "Consumer path dependency changed; update the package check"
    );
    // JSON strings are valid TOML basic strings; forward slashes also work on Windows.
    let replacement = format!(
        "path = {}",
        serde_json::to_string(&packaged.to_string_lossy().replace('\\', "/"))?
    );
    fs::write(&manifest, original.replace(needle, &replacement))?;
    let packaged_manifest = packaged.join("Cargo.toml");
    let packaged_manifest = packaged_manifest.to_string_lossy();
    let consumer_manifest = manifest.to_string_lossy();
    cargo(vec![
        "test",
        "--manifest-path",
        &packaged_manifest,
        "--all-targets",
    ])?;
    cargo(vec!["test", "--manifest-path", &packaged_manifest, "--doc"])?;
    cargo(vec![
        "test",
        "--manifest-path",
        &consumer_manifest,
        "--all-targets",
    ])?;
    cargo(vec![
        "test",
        "--manifest-path",
        &consumer_manifest,
        "--all-targets",
        "--no-default-features",
    ])?;
    cargo(vec![
        "tree",
        "--manifest-path",
        &consumer_manifest,
        "-i",
        package["name"].as_str().unwrap(),
    ])?;
    let report = serde_json::json!({"archive": archive.strip_prefix(repo)?, "files": contents.names.len(), "uncompressed_bytes": contents.bytes, "compressed_bytes": compressed, "packaged_tests": "passed", "packaged_doctests": "passed", "independent_consumer_default_and_software": "passed", "published": false});
    json(&target.join("beta-package-report.json"), &report)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn required_files_budgets_and_development_exclusions() {
        let mut contents = Contents {
            stem: "crate-1.0.0".into(),
            names: REQUIRED.iter().map(|s| s.to_string()).collect(),
            bytes: 0,
        };
        assert!(!check_contents(&contents, 5_000_000, false).unwrap());
        assert!(check_contents(&contents, 5_000_001, false).is_err());
        assert!(check_contents(&contents, 1, true).is_err());
        contents.names.insert("docs/generated/manifest.json".into());
        assert!(check_contents(&contents, MAX_PACKAGE_BYTES, true).unwrap());
        assert!(check_contents(&contents, MAX_PACKAGE_BYTES + 1, true).is_err());
        for name in [
            "target/file",
            "consumers/file",
            ".github/workflow",
            "xtask/Cargo.toml",
            "tests/visual/references/file",
        ] {
            contents.names.insert(name.into());
            assert!(check_contents(&contents, 1, true).is_err());
            contents.names.remove(name);
        }
        contents.names.remove("LICENSE");
        assert!(check_contents(&contents, 1, true).is_err());
    }
}
