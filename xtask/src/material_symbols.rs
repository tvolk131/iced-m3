//! Offline maintenance of the pinned official SVG catalog. No consumer build runs this.
use crate::util::{files, json, read, sha};
use anyhow::{Context, Result, ensure};
use flate2::{Compression, GzBuilder, bufread::GzDecoder};
use quick_xml::{NsReader, events::Event, name::ResolveResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File},
    io::{self, BufReader, Read, Write},
    path::Path,
};

#[path = "../../crates/material-symbols/codegen.rs"]
mod codegen;
use codegen::valid_name;

const REVISION: &str = "10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7";
const SVG_SHA256: &str = "d43104d54d6b4845cdddd11a7bb1030c0bde882ecb136a0c67831bf64ffcabee";
const FAMILIES: [&str; 2] = ["outlined", "rounded"];
const SIZES: [u16; 4] = [20, 24, 40, 48];
const CHUNK_BYTES: usize = 64 * 1024;
const NAME_COUNT: usize = 4157;
const VARIANTS: usize = 16;
const MAGIC: &[u8; 8] = b"M3SYM001";
const HEADER: usize = 20;
const RECORD: usize = 12;
const SVG_NS: &[u8] = b"http://www.w3.org/2000/svg";

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Metadata {
    revision: String,
    archive_sha256: String,
    families: Vec<String>,
    fills: Vec<u8>,
    optical_sizes: Vec<u16>,
    weight: u16,
    grade: i16,
    chunk_bytes: usize,
    variant_count: usize,
    names: Vec<String>,
    svg_sha256: String,
}

fn validate_svg(data: &[u8], size: u16) -> Result<()> {
    std::str::from_utf8(data).context("SVG must be UTF-8")?;
    let mut reader = NsReader::from_reader(data);
    let decoder = reader.decoder();
    let (mut root, mut depth, mut artwork) = (false, 0_usize, false);
    loop {
        let (namespace, event) = reader.read_resolved_event()?;
        let empty = matches!(event, Event::Empty(_));
        match event {
            Event::Start(element) | Event::Empty(element) => {
                let svg_namespace =
                    matches!(namespace, ResolveResult::Bound(ns) if ns.as_ref() == SVG_NS);
                if depth == 0 {
                    ensure!(
                        !root && svg_namespace && element.local_name().as_ref() == b"svg",
                        "Expected one SVG root in the official namespace"
                    );
                    root = true;
                    let mut dimensions = [None, None];
                    for attr in element.attributes() {
                        let attr = attr?;
                        let value = attr.decode_and_unescape_value(decoder)?;
                        match attr.key.as_ref() {
                            b"width" => dimensions[0] = Some(value.into_owned()),
                            b"height" => dimensions[1] = Some(value.into_owned()),
                            b"viewBox" => {
                                let values = value
                                    .split_whitespace()
                                    .map(str::parse::<i32>)
                                    .collect::<std::result::Result<Vec<_>, _>>()?;
                                ensure!(
                                    values.len() == 4 && values[2] > 0 && values[2] == values[3],
                                    "Expected a square SVG viewBox"
                                );
                            }
                            _ => {}
                        }
                    }
                    ensure!(
                        dimensions
                            .iter()
                            .all(|value| value.as_deref() == Some(size.to_string().as_str())),
                        "SVG dimensions differ from optical size {size}"
                    );
                } else if svg_namespace && element.local_name().as_ref() == b"path" {
                    artwork = true;
                }
                if !empty {
                    depth += 1;
                }
            }
            Event::End(_) => depth = depth.checked_sub(1).context("Unexpected closing SVG tag")?,
            Event::Text(text) if depth == 0 => ensure!(
                std::str::from_utf8(text.as_ref())?.trim().is_empty(),
                "Text outside SVG root"
            ),
            Event::Eof => break,
            _ => {}
        }
    }
    ensure!(
        root && depth == 0 && artwork,
        "Incomplete SVG or missing path artwork"
    );
    Ok(())
}

/// Only the implicit weight-400/grade-0 filenames match. Other axes are skipped.
fn source_key(path: &str) -> Option<(&str, usize)> {
    let mut parts = path.strip_prefix("symbols/web/")?.split('/');
    let (name, family, filename) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() || !valid_name(name) {
        return None;
    }
    let style = FAMILIES
        .iter()
        .position(|style| family == format!("materialsymbols{style}"))?;
    let suffix = filename.strip_prefix(name)?;
    let (fill, suffix) = if let Some(suffix) = suffix.strip_prefix("_fill1_") {
        (1, suffix)
    } else {
        (0, suffix.strip_prefix('_')?)
    };
    let size = suffix.strip_suffix("px.svg")?.parse::<u16>().ok()?;
    let optical = SIZES.iter().position(|value| *value == size)?;
    Some((name, style * 8 + fill * 4 + optical))
}

fn append_word(data: &mut Vec<u8>, value: usize) -> Result<()> {
    data.extend(u32::try_from(value)?.to_le_bytes());
    Ok(())
}

fn append_record(data: &mut Vec<u8>, values: [usize; 3]) -> Result<()> {
    for value in values {
        append_word(data, value)?;
    }
    Ok(())
}

fn pack(entries: &[&[u8]]) -> Result<Vec<u8>> {
    ensure!(!entries.is_empty(), "Cannot pack an empty catalog");
    let (mut chunks, mut current, mut index) = (Vec::new(), Vec::new(), Vec::new());
    let mut seen = HashMap::<&[u8], [usize; 3]>::new();
    for &svg in entries {
        ensure!(
            !svg.is_empty() && svg.len() <= CHUNK_BYTES,
            "SVG exceeds chunk budget or is empty"
        );
        let record = if let Some(record) = seen.get(svg) {
            *record
        } else {
            if current.len() + svg.len() > CHUNK_BYTES {
                chunks.push(std::mem::take(&mut current));
            }
            let record = [chunks.len(), current.len(), svg.len()];
            current.extend_from_slice(svg);
            seen.insert(svg, record);
            record
        };
        append_record(&mut index, record)?;
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    let (mut table, mut payload) = (Vec::new(), Vec::new());
    for chunk in &chunks {
        let mut gzip = GzBuilder::new()
            .mtime(0)
            .operating_system(255)
            .write(Vec::new(), Compression::best());
        gzip.write_all(chunk)?;
        let compressed = gzip.finish()?;
        append_record(&mut table, [payload.len(), compressed.len(), chunk.len()])?;
        payload.extend(compressed);
    }
    let mut result = MAGIC.to_vec();
    append_record(
        &mut result,
        [
            entries.len(),
            chunks.len(),
            HEADER + index.len() + table.len(),
        ],
    )?;
    result.extend(index);
    result.extend(table);
    result.extend(payload);
    Ok(result)
}

fn word(data: &[u8], offset: usize) -> Result<usize> {
    let bytes = data
        .get(offset..offset + 4)
        .context("Truncated symbol bundle")?;
    Ok(u32::from_le_bytes(bytes.try_into()?) as usize)
}

fn record(data: &[u8], offset: usize) -> Result<[usize; 3]> {
    Ok([
        word(data, offset)?,
        word(data, offset + 4)?,
        word(data, offset + 8)?,
    ])
}

struct Decoded {
    chunks: Vec<Vec<u8>>,
    entries: Vec<[usize; 3]>,
}

impl Decoded {
    fn new(data: &[u8], expected: usize) -> Result<Self> {
        ensure!(
            data.get(..8) == Some(MAGIC.as_slice()),
            "Invalid bundle magic/version"
        );
        let [count, chunk_count, payload] = record(data, 8)?;
        ensure!(
            count == expected && chunk_count > 0 && chunk_count <= count,
            "Unexpected variant or chunk count"
        );
        ensure!(
            payload == HEADER + (count + chunk_count) * RECORD,
            "Invalid bundle payload offset"
        );
        let (mut chunks, mut next) = (Vec::new(), 0);
        for chunk in 0..chunk_count {
            let [offset, length, decoded_length] = record(data, HEADER + (count + chunk) * RECORD)?;
            ensure!(
                offset == next && length > 0 && (1..=CHUNK_BYTES).contains(&decoded_length),
                "Invalid chunk table entry {chunk}"
            );
            let start = payload
                .checked_add(offset)
                .context("Chunk offset overflow")?;
            let end = start.checked_add(length).context("Chunk length overflow")?;
            let compressed = data.get(start..end).context("Truncated compressed chunk")?;
            let mut decoder = GzDecoder::new(compressed);
            let mut decoded = vec![0; decoded_length];
            decoder
                .read_exact(&mut decoded)
                .context("Invalid gzip chunk")?;
            ensure!(
                decoder.read(&mut [0])? == 0 && decoder.into_inner().is_empty(),
                "Unexpected extra chunk data"
            );
            chunks.push(decoded);
            next = offset
                .checked_add(length)
                .context("Chunk offset overflow")?;
        }
        ensure!(
            payload.checked_add(next) == Some(data.len()),
            "Trailing bundle bytes"
        );
        let mut entries = Vec::with_capacity(count);
        for index in 0..count {
            let entry @ [chunk, offset, length] = record(data, HEADER + index * RECORD)?;
            let bytes = chunks.get(chunk).context("Invalid chunk reference")?;
            ensure!(
                length > 0
                    && offset
                        .checked_add(length)
                        .is_some_and(|end| end <= bytes.len()),
                "Invalid SVG slice at variant {index}"
            );
            entries.push(entry);
        }
        Ok(Self { chunks, entries })
    }

    fn svg(&self, index: usize) -> &[u8] {
        let [chunk, offset, length] = self.entries[index];
        &self.chunks[chunk][offset..offset + length]
    }
}

fn checksums(root: &Path) -> Result<String> {
    let mut result = String::new();
    for name in [
        "assets/LICENSE",
        "assets/catalog.json",
        "assets/symbols.bin",
    ] {
        result.push_str(&format!("{}  {name}\n", sha(read(&root.join(name))?)));
    }
    Ok(result)
}

pub fn verify(root: &Path) -> Result<()> {
    let assets = root.join("assets");
    let metadata_bytes = read(&assets.join("catalog.json"))?;
    let metadata: Metadata = serde_json::from_slice(&metadata_bytes)?;
    ensure!(
        metadata.revision == REVISION
            && metadata.families == FAMILIES
            && metadata.fills == [0, 1]
            && metadata.optical_sizes == SIZES
            && metadata.weight == 400
            && metadata.grade == 0
            && metadata.chunk_bytes == CHUNK_BYTES
            && metadata.variant_count == NAME_COUNT * VARIANTS,
        "Unexpected catalog coordinates or revision"
    );
    ensure!(
        metadata.names.len() == NAME_COUNT
            && metadata.names.windows(2).all(|pair| pair[0] < pair[1]),
        "Names must be complete, sorted and unique"
    );
    ensure!(
        metadata.archive_sha256.len() == 64
            && metadata
                .archive_sha256
                .bytes()
                .all(|c| c.is_ascii_hexdigit()),
        "Invalid source archive checksum"
    );
    codegen::from_json(&metadata_bytes).map_err(anyhow::Error::msg)?;
    ensure!(
        !root.join("src/catalog.rs").exists(),
        "Typed catalog must be generated in OUT_DIR, not stored in src/catalog.rs"
    );
    let data = read(&assets.join("symbols.bin"))?;
    let decoded = Decoded::new(&data, metadata.variant_count)?;
    let mut digest = Sha256::new();
    for index in 0..metadata.variant_count {
        let svg = decoded.svg(index);
        validate_svg(svg, SIZES[index % SIZES.len()])
            .with_context(|| format!("Invalid SVG variant {index}"))?;
        digest.update(svg);
    }
    ensure!(
        format!("{:x}", digest.finalize()) == metadata.svg_sha256
            && metadata.svg_sha256 == SVG_SHA256,
        "Recovered SVGs differ from pinned originals"
    );
    ensure!(
        fs::read_to_string(assets.join("SHA256SUMS"))? == checksums(root)?,
        "Bundled assets differ from SHA256SUMS"
    );
    ensure!(
        !files(&assets)?.keys().any(|name| name.ends_with(".svg")),
        "Loose SVGs must not ship alongside the bundle"
    );
    println!(
        "Verified {} names / {} official SVG variants in {} chunks; {} bundle bytes, revision {REVISION}",
        metadata.names.len(),
        metadata.variant_count,
        decoded.chunks.len(),
        data.len()
    );
    Ok(())
}

fn archive_sha(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let length = file.read(&mut buffer)?;
        if length == 0 {
            break;
        }
        digest.update(&buffer[..length]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

pub fn generate(root: &Path, archive_path: &Path) -> Result<()> {
    let prefix = format!("material-design-icons-{REVISION}/");
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(BufReader::new(File::open(
        archive_path,
    )?)));
    let mut icons = BTreeMap::<String, [Option<Vec<u8>>; VARIANTS]>::new();
    let mut license = None;
    let mut variants = 0;
    println!("Reading pinned archive without extracting it...");
    for entry in archive.entries()? {
        let mut entry = entry?;
        // GitHub archives begin with a PAX metadata record outside the file tree.
        if entry.header().entry_type().is_pax_global_extensions() {
            continue;
        }
        let path = entry.path()?.into_owned();
        let path = path.to_str().context("Non-UTF8 upstream path")?;
        if path == prefix.trim_end_matches('/') {
            continue;
        }
        let relative = path
            .strip_prefix(&prefix)
            .with_context(|| format!("Wrong upstream archive revision in path {path:?}"))?;
        let key = source_key(relative);
        if relative != "LICENSE" && key.is_none() {
            continue;
        }
        ensure!(
            entry.header().entry_type().is_file() && entry.size() <= CHUNK_BYTES as u64,
            "Invalid upstream file {path}"
        );
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        if let Some((name, index)) = key {
            validate_svg(&data, SIZES[index % SIZES.len()])
                .with_context(|| format!("Invalid upstream SVG {relative}"))?;
            let slots = icons
                .entry(name.to_owned())
                .or_insert_with(|| std::array::from_fn(|_| None));
            ensure!(
                slots[index].replace(data).is_none(),
                "Duplicate upstream SVG {relative}"
            );
            variants += 1;
            if variants % 10_000 == 0 {
                println!("Read {variants} selected SVGs");
            }
        } else {
            ensure!(
                license.replace(data).is_none(),
                "Duplicate upstream license"
            );
        }
    }
    // Finish the gzip stream, including its CRC, after the tar end marker.
    io::copy(&mut archive.into_inner(), &mut io::sink())?;
    ensure!(
        icons.len() == NAME_COUNT && variants == NAME_COUNT * VARIANTS,
        "Incomplete upstream catalog: {} names / {variants} variants",
        icons.len()
    );
    let license = license.context("Missing upstream license")?;
    let mut entries = Vec::with_capacity(variants);
    for (name, slots) in &icons {
        for (index, data) in slots.iter().enumerate() {
            entries.push(
                data.as_deref()
                    .with_context(|| format!("Missing {name} variant {index}"))?,
            );
        }
    }
    let mut digest = Sha256::new();
    for svg in &entries {
        digest.update(svg);
    }
    let svg_sha256 = format!("{:x}", digest.finalize());
    ensure!(
        svg_sha256 == SVG_SHA256,
        "Upstream SVG corpus differs from the pinned original checksum"
    );
    let names: Vec<_> = icons.keys().cloned().collect();
    codegen::registry(&names).map_err(anyhow::Error::msg)?;
    let data = pack(&entries)?;
    // Validate every byte before replacing any committed artifact.
    let recovered = Decoded::new(&data, variants)?;
    for (index, expected) in entries.iter().enumerate() {
        ensure!(
            recovered.svg(index) == *expected,
            "Packing changed SVG variant {index}"
        );
    }
    let metadata = Metadata {
        revision: REVISION.into(),
        archive_sha256: archive_sha(archive_path)?,
        families: FAMILIES.map(String::from).to_vec(),
        fills: vec![0, 1],
        optical_sizes: SIZES.to_vec(),
        weight: 400,
        grade: 0,
        chunk_bytes: CHUNK_BYTES,
        variant_count: variants,
        names,
        svg_sha256,
    };
    fs::create_dir_all(root.join("assets"))?;
    fs::write(root.join("assets/symbols.bin"), data)?;
    fs::write(root.join("assets/LICENSE"), license)?;
    json(&root.join("assets/catalog.json"), &metadata)?;
    fs::write(root.join("assets/SHA256SUMS"), checksums(root)?)?;
    verify(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn svg(size: u16) -> Vec<u8> {
        format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{size}\" height=\"{size}\" viewBox=\"0 -960 960 960\"><path d=\"M0 0h10v10H0z\"/></svg>").into_bytes()
    }

    #[test]
    fn source_paths_select_only_the_documented_axes() {
        assert_eq!(
            source_key("symbols/web/home/materialsymbolsoutlined/home_20px.svg"),
            Some(("home", 0))
        );
        assert_eq!(
            source_key("symbols/web/home/materialsymbolsrounded/home_fill1_48px.svg"),
            Some(("home", 15))
        );
        for path in [
            "symbols/web/home/materialsymbolssharp/home_24px.svg",
            "symbols/web/home/materialsymbolsoutlined/home_wght100_24px.svg",
            "symbols/web/home/materialsymbolsoutlined/home_grad200_24px.svg",
            "symbols/web/home/materialsymbolsoutlined/house_24px.svg",
            "symbols/web/home/materialsymbolsoutlined/home_32px.svg",
            "symbols/web/home/materialsymbolsoutlined/sub/home_24px.svg",
        ] {
            assert_eq!(source_key(path), None, "{path}");
        }
    }

    #[test]
    fn names_preserve_upstream_spelling_and_reject_rust_collisions() {
        let text = codegen::registry(&["10k".into(), "arrow_back".into()]).unwrap();
        assert!(text.contains("Icon10k => \"10k\""));
        assert!(text.contains("ArrowBack => \"arrow_back\""));
        assert!(codegen::registry(&["a__b".into(), "a_b".into()]).is_err());
        assert!(codegen::registry(&["_".into()]).is_err());
        assert!(codegen::registry(&["bad\"name".into()]).is_err());
    }

    #[test]
    fn svg_validation_checks_namespace_dimensions_artwork_and_complete_xml() {
        validate_svg(&svg(24), 24).unwrap();
        // Some official icons use viewport dimensions without a viewBox.
        validate_svg(
            b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"20\" height=\"20\"><path/></svg>",
            20,
        )
        .unwrap();
        assert!(validate_svg(&svg(24), 20).is_err());
        for bad in [
            "<svg width=\"24\" height=\"24\"><path/></svg>",
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\"/>",
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\"><path/>",
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 20 24\"><path/></svg>",
        ] {
            assert!(validate_svg(bad.as_bytes(), 24).is_err(), "{bad}");
        }
        let mut trailing = svg(24);
        trailing.extend_from_slice(b"text");
        assert!(validate_svg(&trailing, 24).is_err());
        let mut two_roots = svg(24);
        two_roots.extend(svg(24));
        assert!(validate_svg(&two_roots, 24).is_err());
    }

    #[test]
    fn packing_is_deterministic_deduplicates_and_obeys_chunk_boundaries() {
        let first = vec![b'a'; CHUNK_BYTES - 20];
        let second = vec![b'b'; 40];
        let third = vec![b'c'; 40];
        let entries = [&first[..], &second, &first, &third];
        let data = pack(&entries).unwrap();
        assert_eq!(data, pack(&entries).unwrap());
        let decoded = Decoded::new(&data, entries.len()).unwrap();
        assert_eq!(decoded.chunks.len(), 2);
        assert_eq!(decoded.entries[0], decoded.entries[2]);
        for (index, expected) in entries.iter().enumerate() {
            assert_eq!(decoded.svg(index), *expected);
        }
        assert!(pack(&[&vec![0; CHUNK_BYTES + 1]]).is_err());
        assert!(pack(&[]).is_err());
    }

    #[test]
    fn malformed_offsets_lengths_streams_and_trailing_data_are_rejected() {
        let valid = pack(&[b"official SVG bytes"]).unwrap();
        assert!(Decoded::new(&valid, 2).is_err());
        for length in [0, 8, HEADER, valid.len() - 1] {
            assert!(Decoded::new(&valid[..length], 1).is_err());
        }
        for offset in [
            0,
            16,
            HEADER,
            HEADER + 4,
            HEADER + 8,
            HEADER + RECORD + 8,
            valid.len() - 1,
        ] {
            let mut bad = valid.clone();
            bad[offset] ^= 0xff;
            assert!(Decoded::new(&bad, 1).is_err(), "corruption at {offset}");
        }
        let mut trailing = valid;
        trailing.push(0);
        assert!(Decoded::new(&trailing, 1).is_err());
    }

    #[test]
    fn incorrect_or_incomplete_archives_do_not_replace_existing_artifacts() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("output");
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(root.join("assets/symbols.bin"), b"keep").unwrap();
        for prefix in [
            "wrong-revision".to_string(),
            format!("material-design-icons-{REVISION}"),
        ] {
            let path = temp.path().join("source.tar.gz");
            let gzip = GzBuilder::new()
                .mtime(0)
                .write(File::create(&path).unwrap(), Compression::fast());
            let mut tar = tar::Builder::new(gzip);
            let pax = format!("52 comment={REVISION}\n");
            let mut global = tar::Header::new_gnu();
            global.set_entry_type(tar::EntryType::XGlobalHeader);
            global.set_size(pax.len() as u64);
            global.set_cksum();
            tar.append_data(&mut global, "pax_global_header", pax.as_bytes())
                .unwrap();
            let data = svg(24);
            let mut header = tar::Header::new_gnu();
            header.set_mode(0o644);
            header.set_size(data.len() as u64);
            header.set_cksum();
            tar.append_data(
                &mut header,
                format!("{prefix}/symbols/web/home/materialsymbolsoutlined/home_24px.svg"),
                &data[..],
            )
            .unwrap();
            tar.into_inner().unwrap().finish().unwrap();
            let error = generate(&root, &path).unwrap_err().to_string();
            let expected = if prefix == "wrong-revision" {
                "Wrong upstream archive revision"
            } else {
                "Incomplete upstream catalog"
            };
            assert!(error.contains(expected), "{error}");
            assert_eq!(read(&root.join("assets/symbols.bin")).unwrap(), b"keep");
        }
    }

    #[test]
    fn committed_svg_corpus_remains_byte_identical_to_the_google_pin() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("crates/material-symbols");
        let data = read(&root.join("assets/symbols.bin")).unwrap();
        let decoded = Decoded::new(&data, NAME_COUNT * VARIANTS).unwrap();
        let mut digest = Sha256::new();
        for index in 0..NAME_COUNT * VARIANTS {
            digest.update(decoded.svg(index));
        }
        assert_eq!(format!("{:x}", digest.finalize()), SVG_SHA256);
    }
}
