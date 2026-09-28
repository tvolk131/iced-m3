//! Provenance, PNG/APNG integrity, and embedded rustdoc verification.
use crate::{
    registry::{self, Example},
    util::*,
};
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use quick_xml::{Reader, events::Event};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub const GENERATED: &str = "docs/generated";
pub const CATALOG: &str = "tests/visual/doc_media/catalog.rs";
pub const MAX_PACKAGE_BYTES: u64 = 9_800_000;
pub const TOOLCHAIN: &str = "1.92.0";
pub const DATA_URI: &str = "data:image/png;base64,";

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub sha256: String,
    pub bytes: usize,
    pub images: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub schema: u32,
    pub generator: String,
    pub inputs: BTreeMap<String, String>,
    pub files: BTreeMap<String, Record>,
    pub renderer: String,
    pub rustc: String,
    pub rustfmt: String,
}
/// Embedded source makes verification independent of where the tool is executed.
/// Changing any generator code or dependency invalidates its prior manifests.
pub fn generator_hash() -> String {
    sha(concat!(
        include_str!("../Cargo.toml"),
        include_str!("../Cargo.lock"),
        include_str!("main.rs"),
        include_str!("util.rs"),
        include_str!("registry.rs"),
        include_str!("media.rs"),
        include_str!("generate.rs"),
        include_str!("package.rs"),
        include_str!("tests.rs"),
        include_str!("../../.cargo/config.toml")
    ))
}
pub fn inputs(root: &Path) -> Result<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    for name in ["Cargo.toml", "Cargo.lock"] {
        let orig = root.join("Cargo.toml.orig");
        let path = if name == "Cargo.toml" && orig.exists() {
            orig
        } else {
            root.join(name)
        };
        result.insert(name.into(), sha(read(&path)?));
    }
    for dir in ["src", "tests/visual", "assets"] {
        for (name, path) in files(&root.join(dir))? {
            if dir == "assets" || name.ends_with(".rs") {
                result.insert(format!("{dir}/{name}"), sha(read(&path)?));
            }
        }
    }
    Ok(result)
}
#[derive(Debug)]
pub struct PngInfo {
    pub width: u32,
    pub frames: u32,
}
fn u32be(data: &[u8]) -> u32 {
    u32::from_be_bytes(data[..4].try_into().unwrap())
}
pub fn png_info(data: &[u8]) -> Result<PngInfo> {
    ensure!(
        data.starts_with(b"\x89PNG\r\n\x1a\n"),
        "Invalid PNG signature"
    );
    let mut offset = 8;
    let (mut width, mut height, mut frames, mut controls, mut duration) = (0, 0, 0, 0, 0.0_f64);
    let (mut ihdr, mut iend, mut idat, mut actl) = (false, false, false, false);
    while offset < data.len() {
        ensure!(!iend && data.len() - offset >= 12, "Truncated/trailing PNG");
        let length = u32be(&data[offset..]) as usize;
        ensure!(length <= data.len() - offset - 12, "Truncated PNG chunk");
        let kind = &data[offset + 4..offset + 8];
        let payload = &data[offset + 8..offset + 8 + length];
        let end = offset + 12 + length;
        ensure!(
            crc32fast::hash(&data[offset + 4..end - 4]) == u32be(&data[end - 4..end]),
            "Invalid PNG chunk CRC"
        );
        ensure!(ihdr || kind == b"IHDR", "Missing first IHDR");
        match kind {
            b"IHDR" => {
                ensure!(!ihdr && length == 13, "Invalid IHDR");
                ihdr = true;
                width = u32be(payload);
                height = u32be(&payload[4..]);
            }
            b"acTL" => {
                ensure!(!actl && !idat && length == 8, "Invalid acTL");
                actl = true;
                frames = u32be(payload);
                ensure!(frames > 0, "Empty APNG");
            }
            b"fcTL" => {
                ensure!(actl && length == 26, "Invalid fcTL");
                controls += 1;
                let num = u16::from_be_bytes(payload[20..22].try_into().unwrap());
                let den = u16::from_be_bytes(payload[22..24].try_into().unwrap());
                duration += f64::from(num) / f64::from(if den == 0 { 100 } else { den });
            }
            b"IDAT" => idat = true,
            b"IEND" => {
                ensure!(length == 0, "Invalid IEND");
                iend = true;
            }
            _ => {}
        }
        offset = end;
    }
    ensure!(
        ihdr && iend && idat && width > 0 && height > 0,
        "Incomplete PNG"
    );
    ensure!(
        !actl || (frames == controls && (duration - 3.0).abs() <= 0.00001),
        "Incorrect APNG frame count or duration"
    );
    Ok(PngInfo { width, frames })
}
pub fn embedded_images(text: &str) -> Result<Vec<Vec<u8>>> {
    text.split(DATA_URI)
        .skip(1)
        .map(|s| {
            let end = s
                .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '=')))
                .unwrap_or(s.len());
            STANDARD
                .decode(&s[..end])
                .context("Invalid embedded PNG base64")
        })
        .collect()
}
pub fn record(data: &str) -> Result<Record> {
    Ok(Record {
        sha256: sha(data),
        bytes: data.len(),
        images: embedded_images(data)?.iter().map(sha).collect(),
    })
}
pub fn validate(root: &Path, generated: &Path) -> Result<Manifest> {
    ensure!(
        read(&root.join(CATALOG))? == include_bytes!("../../tests/visual/doc_media/catalog.rs"),
        "Catalog differs from this tool; run xtask from the matching source revision"
    );
    validate_with(root, generated, &registry::catalog())
}
pub fn validate_with(root: &Path, generated: &Path, cases: &[Example]) -> Result<Manifest> {
    registry::validate(cases)?;
    let manifest: Manifest = serde_json::from_slice(&read(&generated.join("manifest.json"))?)?;
    let expected: BTreeSet<_> = cases
        .iter()
        .flat_map(|c| {
            c.parts()
                .into_iter()
                .map(move |p| format!("{}-{p}.md", c.id))
        })
        .collect();
    let mut owned = expected.clone();
    owned.insert("manifest.json".into());
    ensure!(
        files(generated)?.into_keys().collect::<BTreeSet<_>>() == owned,
        "Missing/orphan documentation files"
    );
    ensure!(
        manifest.schema == 2 && manifest.files.keys().cloned().collect::<BTreeSet<_>>() == expected,
        "Manifest does not match the example registry"
    );
    ensure!(
        manifest.generator == generator_hash(),
        "Stale documentation: generator changed; use the matching source revision or regenerate"
    );
    ensure!(
        manifest.inputs == inputs(root)?,
        "Stale documentation: rendering inputs changed; regenerate"
    );
    for case in cases {
        for part in case.parts() {
            let name = format!("{}-{part}.md", case.id);
            let data = fs::read_to_string(generated.join(&name))?;
            let record = &manifest.files[&name];
            ensure!(
                sha(&data) == record.sha256 && data.len() == record.bytes,
                "Changed/stale documentation fragment: {name}"
            );
            let images = embedded_images(&data)?;
            ensure!(
                images.iter().map(sha).collect::<Vec<_>>() == record.images,
                "Documentation images do not match manifest: {name}"
            );
            let metadata = images
                .iter()
                .map(|i| png_info(i))
                .collect::<Result<Vec<_>>>()?;
            let want_animation = part != "variants" && case.animated;
            ensure!(
                !metadata.is_empty() && metadata.iter().any(|p| p.frames > 0) == want_animation,
                "Missing expected static/animated preview: {name}"
            );
        }
    }
    Ok(manifest)
}
#[derive(Default, Debug)]
pub struct DocContent {
    pub images: Vec<String>,
    pub headings: BTreeSet<String>,
}
impl DocContent {
    /// Rustdoc emits quoted attributes and HTML void tags. Do not interpret
    /// escaped code snippets, comments, or JavaScript as visible page content.
    pub fn parse(html: &str) -> Result<Self> {
        let mut reader = Reader::from_str(html);
        reader.config_mut().check_end_names = false;
        reader.config_mut().allow_unmatched_ends = true;
        reader.config_mut().allow_dangling_amp = true;
        let mut result = Self::default();
        loop {
            match reader.read_event()? {
                Event::Start(e) | Event::Empty(e) => {
                    let name = e.name();
                    let tag = name.as_ref();
                    if tag == b"script" || tag == b"style" {
                        reader.read_to_end(name)?;
                        continue;
                    }
                    if !matches!(
                        tag,
                        b"h1" | b"h2" | b"h3" | b"h4" | b"h5" | b"h6" | b"img" | b"source"
                    ) {
                        continue;
                    }
                    for attr in e.attributes() {
                        let attr = attr?;
                        let key = attr.key.as_ref();
                        let value = std::str::from_utf8(&attr.value)?;
                        if tag.starts_with(b"h") && key == b"id" {
                            result.headings.insert(value.into());
                        }
                        if matches!(tag, b"img" | b"source")
                            && matches!(key, b"src" | b"srcset")
                            && let Some(data) = value.strip_prefix(DATA_URI)
                        {
                            result.images.push(sha(STANDARD.decode(data)?));
                        }
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }
        Ok(result)
    }
}
pub fn build_docs(root: &Path, target: &Path) -> Result<()> {
    let manifest = validate(root, &root.join(GENERATED))?;
    let package = crate::package::metadata(root, false)?;
    let settings = &package["metadata"]["docs"]["rs"];
    ensure!(
        settings["rustdoc-args"] == serde_json::json!(["--cfg", "iced_m3_doc_media"])
            && settings["no-default-features"] == true,
        "docs.rs metadata must enable previews and disable the GPU backend"
    );
    let target_string = target.to_string_lossy();
    let env = [
        ("CARGO_TARGET_DIR", target_string.as_ref()),
        ("RUSTDOCFLAGS", "-D warnings --cfg iced_m3_doc_media"),
    ];
    command(
        root,
        "cargo",
        &["doc", "--locked", "--no-deps", "--no-default-features"],
        &env,
        false,
    )?;
    command(
        root,
        "cargo",
        &["test", "--locked", "--no-default-features", "--doc"],
        &env,
        false,
    )?;
    let docs = target.join("doc/iced_m3");
    let doc_files = files(&docs)?;
    for case in registry::catalog() {
        let filename = format!("fn.{}.html", case.id);
        let constructors: Vec<_> = doc_files
            .values()
            .filter(|p| p.file_name().is_some_and(|n| n == filename.as_str()))
            .collect();
        ensure!(
            constructors.len() == 1,
            "Expected one documented constructor for {}",
            case.id
        );
        let guide = docs.join(format!("guide/components/{}/index.html", case.id));
        for (page, parts) in [(constructors[0], vec!["primary"]), (&guide, case.parts())] {
            let mut actual = DocContent::parse(&fs::read_to_string(page)?)?;
            let mut expected: Vec<_> = parts
                .iter()
                .flat_map(|part| {
                    manifest.files[&format!("{}-{part}.md", case.id)]
                        .images
                        .clone()
                })
                .collect();
            actual.images.sort();
            expected.sort();
            ensure!(
                actual.images == expected,
                "Rustdoc dropped/changed embedded media: {}",
                page.display()
            );
            for part in parts {
                let heading = match part {
                    "primary" => "example",
                    "variants" => "appearance-variants",
                    _ => "motion-schemes",
                };
                ensure!(
                    actual.headings.contains(heading),
                    "Rustdoc dropped heading {heading}: {}",
                    page.display()
                );
            }
        }
    }
    println!(
        "Verified embedded previews: {}",
        docs.join("guide/components/index.html").display()
    );
    Ok(())
}
