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
    install_symbols(archive, &root)?;
    build_docs(&root, &repo.join("target"))?;
    Ok(sha(read(archive)?))
}

const SYMBOLS_PACKAGE: &str = "iced-m3-material-symbols";
const SYMBOLS_LOCK_NAME: &str = "name = \"iced-m3-material-symbols\"";

/// Cargo adds a registry source/checksum when packaging the workspace's asset
/// dependency. Normalize only that sourcing metadata for documentation freshness;
/// its version and every other locked dependency remain part of the input hash.
pub fn canonical_lock(lock: &str) -> String {
    let mut symbols = false;
    lock.split_inclusive('\n')
        .filter(|line| {
            if line.trim() == "[[package]]" {
                symbols = false;
            } else if line.trim() == SYMBOLS_LOCK_NAME {
                symbols = true;
            }
            !(symbols && (line.starts_with("source = ") || line.starts_with("checksum = ")))
        })
        .collect()
}

/// Before publication, consume the exact companion archive produced by Cargo's
/// workspace packaging. Its hash must equal the dependency checksum in iced-m3's
/// archived lockfile. The patch and adjusted lock exist only in the extracted
/// test copy, never in an uploaded archive.
fn install_symbols(archive: &Path, root: &Path) -> Result<u64> {
    let package = metadata(root, true)?;
    let dependency = package["dependencies"]
        .as_array()
        .context("Missing dependencies")?
        .iter()
        .find(|dep| dep["name"] == SYMBOLS_PACKAGE)
        .context("Missing optional symbols dependency")?;
    let version = dependency["req"]
        .as_str()
        .and_then(|req| req.strip_prefix('='))
        .context("Symbols dependency must use an exact version")?;
    let paired = archive.with_file_name(format!("{SYMBOLS_PACKAGE}-{version}.crate"));
    let bytes = fs::metadata(&paired)?.len();
    ensure!(bytes <= MAX_PACKAGE_BYTES, "Symbols package exceeds 9.8 MB");
    let lock_path = root.join("Cargo.lock");
    let lock = fs::read_to_string(&lock_path)?;
    let block = lock
        .split("[[package]]")
        .find(|block| block.lines().any(|line| line == SYMBOLS_LOCK_NAME))
        .context("Missing packaged symbols lock entry")?;
    let expected_checksum = format!("checksum = \"{}\"", sha(read(&paired)?));
    ensure!(
        block.lines().any(|line| line == expected_checksum),
        "Companion archive differs from iced-m3's locked checksum"
    );
    let config_dir = root.join(".cargo");
    let destination = config_dir.join("paired-symbols");
    fs::create_dir_all(&destination)?;
    let contents = extract(&paired, &destination)?;
    for name in [
        "assets/symbols.bin",
        "assets/catalog.json",
        "assets/LICENSE",
        "assets/NOTICE",
        "assets/SHA256SUMS",
        "src/lib.rs",
        "src/storage.rs",
        "build.rs",
        "codegen.rs",
    ] {
        ensure!(
            contents.names.contains(name),
            "Missing companion package input: {name}"
        );
    }
    crate::material_symbols::verify(&destination.join(&contents.stem))?;
    let path = destination.join(contents.stem).canonicalize()?;
    let config = config_dir.join("config.toml");
    ensure!(!config.exists(), "Unexpected packaged Cargo configuration");
    fs::write(
        config,
        format!(
            "[patch.crates-io]\n{SYMBOLS_PACKAGE} = {{ path = {} }}\n",
            serde_json::to_string(&path.to_string_lossy().replace('\\', "/"))?
        ),
    )?;
    fs::write(lock_path, canonical_lock(&lock))?;
    Ok(bytes)
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
    "src/material_symbols.rs",
    "examples/material_symbols.rs",
    "docs/ICONS.md",
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
                "crates/",
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
    cargo(vec!["package", "--workspace", "--allow-dirty"])?;
    let archive = target.join(format!("package/{stem}.crate"));
    let temp = tempfile::Builder::new()
        .prefix("consumer-package-")
        .tempdir_in(&target)?;
    let contents = extract(&archive, temp.path())?;
    ensure!(contents.stem == stem, "Unexpected package name");
    let compressed = fs::metadata(&archive)?.len();
    let illustrated = check_contents(&contents, compressed, require_media)?;
    let packaged = temp.path().join(&stem);
    let symbols_bytes = install_symbols(&archive, &packaged)?;
    // Cargo reads the temporary patch from this directory even when checking
    // the separate consumer's manifest. Only the paired, extracted asset crate
    // is patched; all third-party dependencies keep their locked registry pins.
    let cargo = |mut args: Vec<&str>| -> Result<()> {
        args.push("--locked");
        if offline {
            args.push("--offline");
        }
        command(
            &packaged,
            "cargo",
            &args,
            &[("CARGO_TARGET_DIR", &target_str)],
            false,
        )?;
        Ok(())
    };
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
        "--all-features",
    ])?;
    cargo(vec![
        "test",
        "--manifest-path",
        &packaged_manifest,
        "--doc",
        "--all-features",
    ])?;
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
        "test",
        "--manifest-path",
        &consumer_manifest,
        "--all-targets",
        "--no-default-features",
        "--features",
        "material-symbols",
    ])?;
    cargo(vec![
        "tree",
        "--manifest-path",
        &consumer_manifest,
        "-i",
        package["name"].as_str().unwrap(),
    ])?;
    let report = serde_json::json!({"archive": archive.strip_prefix(repo)?, "files": contents.names.len(), "uncompressed_bytes": contents.bytes, "compressed_bytes": compressed, "symbols_compressed_bytes": symbols_bytes, "packaged_tests": "passed", "packaged_doctests": "passed", "independent_consumer_default_and_software": "passed", "independent_consumer_material_symbols": "passed", "published": false});
    json(&target.join("beta-package-report.json"), &report)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lock_normalization_preserves_versions_and_other_dependency_checksums() {
        let lock = "[[package]]\nname = \"iced-m3-material-symbols\"\nversion = \"0.1.0-beta.5\"\nsource = \"registry\"\nchecksum = \"assets\"\n\n[[package]]\nname = \"other\"\nversion = \"1.0.0\"\nsource = \"registry\"\nchecksum = \"other\"\n";
        let normalized = canonical_lock(lock);
        assert!(normalized.contains("version = \"0.1.0-beta.5\""));
        assert!(!normalized.contains("checksum = \"assets\""));
        assert!(normalized.contains("source = \"registry\"\nchecksum = \"other\""));
        assert_eq!(canonical_lock(&normalized), normalized);
        assert_ne!(
            canonical_lock(&lock.replace("beta.5", "beta.6")),
            normalized
        );
        assert_ne!(
            canonical_lock(&lock.replace("checksum = \"other\"", "checksum = \"changed\"")),
            normalized
        );
    }
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
