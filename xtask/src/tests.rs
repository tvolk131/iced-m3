use super::*;
use crate::{media::*, util::*};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::{collections::BTreeMap, fs, io::Write};

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], payload: &[u8]) {
    out.extend((payload.len() as u32).to_be_bytes());
    out.extend(kind);
    out.extend(payload);
    let mut hash = crc32fast::Hasher::new();
    hash.update(kind);
    hash.update(payload);
    out.extend(hash.finalize().to_be_bytes());
}
fn png(animated: bool, seconds: u16) -> Vec<u8> {
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    chunk(&mut out, b"IHDR", &[0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0]);
    if animated {
        chunk(&mut out, b"acTL", &[0, 0, 0, 1, 0, 0, 0, 0]);
        let mut control = vec![0; 26];
        control[7] = 1;
        control[11] = 1;
        control[20..22].copy_from_slice(&seconds.to_be_bytes());
        control[23] = 1;
        chunk(&mut out, b"fcTL", &control);
    }
    let mut compressed =
        flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    compressed.write_all(&[0, 255, 0, 0, 255]).unwrap();
    chunk(&mut out, b"IDAT", &compressed.finish().unwrap());
    chunk(&mut out, b"IEND", &[]);
    out
}
struct Fixture {
    temp: tempfile::TempDir,
    cases: Vec<registry::Example>,
}
impl Fixture {
    fn new(animated: bool) -> Self {
        let f = Self {
            temp: tempfile::tempdir().unwrap(),
            cases: vec![registry::Example {
                id: "switch".into(),
                title: "Switch".into(),
                width: 1,
                height: 1,
                scale: 1,
                budget: 1000,
                compare: false,
                animated,
            }],
        };
        for (name, data) in [
            ("Cargo.toml", "[package]"),
            ("Cargo.lock", "lock"),
            ("src/lib.rs", "// component"),
            ("tests/visual/harness.rs", "// harness"),
            (CATALOG, "// registry"),
            ("assets/font.ttf", "font"),
        ] {
            let p = f.root().join(name);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, data).unwrap();
        }
        fs::create_dir_all(f.generated()).unwrap();
        f.fragment("primary", png(animated, 3));
        f.fragment("variants", png(false, 0));
        f.manifest();
        f
    }
    fn root(&self) -> &Path {
        self.temp.path()
    }
    fn generated(&self) -> PathBuf {
        self.root().join(GENERATED)
    }
    fn fragment(&self, part: &str, image: Vec<u8>) {
        fs::write(
            self.generated().join(format!("switch-{part}.md")),
            format!("<img src=\"{DATA_URI}{}\">", STANDARD.encode(image)),
        )
        .unwrap();
    }
    fn manifest(&self) {
        let mut records = BTreeMap::new();
        for part in ["primary", "variants"] {
            let name = format!("switch-{part}.md");
            records.insert(
                name.clone(),
                record(&fs::read_to_string(self.generated().join(name)).unwrap()).unwrap(),
            );
        }
        json(
            &self.generated().join("manifest.json"),
            &Manifest {
                schema: 2,
                generator: generator_hash(),
                inputs: inputs(self.root()).unwrap(),
                files: records,
                renderer: "test".into(),
                rustc: "test".into(),
                rustfmt: "test".into(),
            },
        )
        .unwrap();
    }
    fn check(&self) -> Result<Manifest> {
        validate_with(self.root(), &self.generated(), &self.cases)
    }
}
#[test]
fn valid_and_cargo_normalized_manifest() {
    let f = Fixture::new(true);
    f.check().unwrap();
    fs::rename(
        f.root().join("Cargo.toml"),
        f.root().join("Cargo.toml.orig"),
    )
    .unwrap();
    fs::write(f.root().join("Cargo.toml"), "normalized").unwrap();
    f.check().unwrap();
}
#[test]
fn static_preview_contract() {
    let f = Fixture::new(false);
    f.check().unwrap();
    f.fragment("primary", png(true, 3));
    f.manifest();
    assert!(f.check().is_err());
}
#[test]
fn static_comparison_and_duplicate_or_empty_registry_rejected() {
    let mut f = Fixture::new(false);
    f.cases[0].compare = true;
    assert!(f.check().is_err());
    f.cases[0].compare = false;
    f.cases.push(f.cases[0].clone());
    assert!(f.check().is_err());
    f.cases.clear();
    assert!(f.check().is_err());
}
#[test]
fn missing_and_orphan_files_rejected() {
    let f = Fixture::new(true);
    fs::write(f.generated().join("orphan.png"), png(false, 0)).unwrap();
    assert!(f.check().is_err());
    fs::remove_file(f.generated().join("orphan.png")).unwrap();
    fs::remove_file(f.generated().join("switch-primary.md")).unwrap();
    assert!(f.check().is_err());
}
#[test]
fn added_and_deleted_examples_rejected() {
    let mut f = Fixture::new(true);
    f.cases[0].id = "other".into();
    assert!(f.check().is_err());
    f.cases.push(registry::Example {
        id: "switch".into(),
        ..f.cases[0].clone()
    });
    assert!(f.check().is_err());
}
#[test]
fn stale_component_font_lockfile_and_registry_rejected() {
    for name in [
        "src/lib.rs",
        "assets/font.ttf",
        "Cargo.lock",
        CATALOG,
        "tests/visual/harness.rs",
    ] {
        let f = Fixture::new(true);
        fs::write(f.root().join(name), "changed").unwrap();
        assert!(f.check().is_err(), "{name}");
    }
}
#[test]
fn stale_generator_and_manifest_schema_rejected() {
    for field in ["generator", "schema"] {
        let f = Fixture::new(true);
        let path = f.generated().join("manifest.json");
        let mut m: serde_json::Value = serde_json::from_slice(&read(&path).unwrap()).unwrap();
        m[field] = if field == "schema" {
            serde_json::json!(1)
        } else {
            serde_json::json!("old-tool")
        };
        json(&path, &m).unwrap();
        assert!(f.check().is_err());
    }
}
#[test]
fn edited_fragment_and_image_hash_rejected() {
    let f = Fixture::new(true);
    f.fragment("primary", png(false, 0));
    assert!(f.check().is_err());
    f.manifest();
    let path = f.generated().join("manifest.json");
    let mut m: Manifest = serde_json::from_slice(&read(&path).unwrap()).unwrap();
    m.files.get_mut("switch-primary.md").unwrap().images[0] = "wrong".into();
    json(&path, &m).unwrap();
    assert!(f.check().is_err());
}
#[test]
fn missing_animation_even_with_updated_manifest_rejected() {
    let f = Fixture::new(true);
    f.fragment("primary", png(false, 0));
    f.manifest();
    assert!(f.check().is_err());
}
#[test]
fn png_corruption_truncation_and_timing_rejected() {
    let data = png(true, 3);
    png_info(&data).unwrap();
    for length in 0..data.len() {
        assert!(png_info(&data[..length]).is_err(), "length {length}");
    }
    let mut corrupt = data.clone();
    corrupt[20] ^= 1;
    assert!(png_info(&corrupt).is_err());
    assert!(png_info(&png(true, 2)).is_err());
    let mut trailing = data;
    trailing.push(0);
    assert!(png_info(&trailing).is_err());
}
#[test]
fn missing_manifest_and_wrong_catalog_rejected() {
    let f = Fixture::new(true);
    assert!(validate(f.root(), &f.generated()).is_err());
    fs::remove_file(f.generated().join("manifest.json")).unwrap();
    assert!(f.check().is_err());
}
#[test]
fn rustdoc_parser_checks_real_tags_not_literal_markdown_or_code() {
    let data = STANDARD.encode(png(false, 0));
    let html = format!(
        "<!DOCTYPE html><html><head><script>var name = 'example';</script></head><body>## Motion schemes\n<pre>&lt;h2 id=\"motion-schemes\"&gt;</pre><!-- <h2 id=\"motion-schemes\"> --><h2 id=\"example\">Example</h2><picture><source srcset=\"{DATA_URI}{data}\"><img src=\"{DATA_URI}{data}\"></picture><br></body></html>"
    );
    let parsed = DocContent::parse(&html).unwrap();
    assert!(parsed.headings.contains("example"));
    assert!(!parsed.headings.contains("motion-schemes"));
    assert_eq!(parsed.images, vec![sha(png(false, 0)); 2]);
    assert!(
        DocContent::parse("<h3 id=\"motion-schemes\">Motion</h3>")
            .unwrap()
            .headings
            .contains("motion-schemes")
    );
}
#[test]
fn cleanup_is_confined_to_owned_children() {
    let temp = tempfile::tempdir().unwrap();
    let work = temp.path().join("work");
    let outside = temp.path().join("outside");
    fs::create_dir(&outside).unwrap();
    assert!(fresh_dir(&work, &outside).is_err());
    assert!(outside.exists());
    assert!(fresh_dir(&work, &work).is_err());
    assert!(fresh_dir(&work, &work.join("../outside")).is_err());
    fresh_dir(&work, &work.join("render")).unwrap();
    fs::write(work.join("render/old"), "old").unwrap();
    fresh_dir(&work, &work.join("render")).unwrap();
    assert!(!work.join("render/old").exists());
}
#[cfg(unix)]
#[test]
fn cleanup_and_validation_reject_symlinks() {
    let temp = tempfile::tempdir().unwrap();
    let work = temp.path().join("work");
    let outside = temp.path().join("outside");
    fs::create_dir(&work).unwrap();
    fs::create_dir(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, work.join("linked")).unwrap();
    assert!(fresh_dir(&work, &work.join("linked/child")).is_err());
    assert!(outside.exists());
    let f = Fixture::new(true);
    std::os::unix::fs::symlink(f.root().join("Cargo.toml"), f.generated().join("link")).unwrap();
    assert!(f.check().is_err());
}
fn archive(entries: &[(&str, tar::EntryType)]) -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("test.crate");
    let encoder = flate2::write::GzEncoder::new(
        fs::File::create(&path).unwrap(),
        flate2::Compression::default(),
    );
    let mut tar = tar::Builder::new(encoder);
    for (name, kind) in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(0);
        header.set_mode(0o644);
        header.set_entry_type(*kind);
        // Raw name bytes let tests exercise traversal that the tar builder forbids.
        header.as_mut_bytes()[..name.len()].copy_from_slice(name.as_bytes());
        header.set_cksum();
        tar.append(&header, &[][..]).unwrap();
    }
    tar.into_inner().unwrap().finish().unwrap();
    (temp, path)
}
#[test]
fn archive_rejects_traversal_links_duplicates_and_multiple_roots() {
    for entries in [
        vec![("../escaped", tar::EntryType::Regular)],
        vec![("/absolute", tar::EntryType::Regular)],
        vec![("root/../../escaped", tar::EntryType::Regular)],
        vec![("root\\file", tar::EntryType::Regular)],
        vec![("root/C:/file", tar::EntryType::Regular)],
        vec![("root/link", tar::EntryType::Symlink)],
        vec![("root/link", tar::EntryType::Link)],
        vec![
            ("a/file", tar::EntryType::Regular),
            ("b/file", tar::EntryType::Regular),
        ],
        vec![
            ("a/file", tar::EntryType::Regular),
            ("a/file", tar::EntryType::Regular),
        ],
    ] {
        let (temp, path) = archive(&entries);
        let output = temp.path().join("out");
        fs::create_dir(&output).unwrap();
        assert!(package::extract(&path, &output).is_err(), "{entries:?}");
        assert!(fs::read_dir(output).unwrap().next().is_none());
    }
}
#[test]
fn ordinary_archive_extracts() {
    let (temp, path) = archive(&[("root/file", tar::EntryType::Regular)]);
    let output = temp.path().join("out");
    fs::create_dir(&output).unwrap();
    let contents = package::extract(&path, &output).unwrap();
    assert_eq!(contents.stem, "root");
    assert!(output.join("root/file").is_file());
}
#[test]
fn cli_rejects_missing_values_unknown_flags_and_publish() {
    let root = Path::new("/repo");
    for args in [
        vec!["publish"],
        vec!["doc-media", "check-archive"],
        vec!["doc-media", "build", "--root", "/elsewhere"],
        vec!["check-package", "--source"],
        vec!["check-package", "--offline", "--offline"],
    ] {
        assert!(
            parse(
                &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                root
            )
            .is_err()
        );
    }
    assert_eq!(
        parse(&["doc-media".into(), "build".into()], root).unwrap(),
        Task::Media("build".into())
    );
}
