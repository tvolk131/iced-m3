use crate::{media::*, registry, util::*};
use anyhow::{Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const SCAFFOLD: &str = "# use iced_m3::*;\n# use iced::widget;\n# #[derive(Clone)]\n# enum Message { Toggle(bool), Action, Input(String), Dismiss, Select(u8), Value(f32), Segments(SegmentSelection<u8>), Range((f32, f32)), Date(DateSelection), Time(Time), Part(TimePart), Index(usize) }\n";
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
fn uri(bytes: &[u8]) -> String {
    format!("{DATA_URI}{}", STANDARD.encode(bytes))
}
fn picture(animation: &[u8], poster: &[u8], width: u32, alt: &str) -> String {
    format!(
        "<picture><source media=\"(prefers-reduced-motion: reduce)\" srcset=\"{}\"><img alt=\"{}\" width=\"{width}\" style=\"max-width:100%;height:auto\" src=\"{}\"></picture>\n",
        uri(poster),
        escape(alt),
        uri(animation)
    )
}
pub fn generate(repo: &Path, destination: &Path) -> Result<PathBuf> {
    fresh_dir(&repo.join("target/doc-media"), destination)?;
    let raw = destination.join("raw");
    command(
        repo,
        "cargo",
        &[
            &format!("+{TOOLCHAIN}"),
            "test",
            "--locked",
            "--no-default-features",
            "--lib",
            "generate_component_doc_media",
            "--",
            "--ignored",
            "--nocapture",
        ],
        &[("ICED_M3_DOC_OUTPUT", &raw.to_string_lossy())],
        false,
    )?;
    fragments(repo, &raw, &destination.join("generated"))
}
/// Kept separate from capture so formatting can be tested against fixed captures.
pub fn fragments(repo: &Path, raw: &Path, generated: &Path) -> Result<PathBuf> {
    let cases: Vec<registry::Example> = serde_json::from_slice(&read(&raw.join("registry.json"))?)?;
    registry::validate(&cases)?;
    ensure!(
        cases == registry::catalog(),
        "Renderer and compiled registries disagree"
    );
    fs::create_dir_all(generated)?;
    for case in cases {
        let key = &case.id;
        let title = &case.title;
        let width = case.width;
        let source = rustfmt(&read(&raw.join(format!("{key}.rs")))?)?;
        let caption = fs::read_to_string(raw.join(format!("{key}.txt")))?;
        let animation = read(&raw.join(format!("{key}.png")))?;
        let poster = read(&raw.join(format!("{key}-poster.png")))?;
        let mut primary = format!("\n## Example\n\n{caption}\n\n");
        if case.animated {
            primary += &picture(&animation, &poster, width, &format!("{title}: {caption}"));
        } else {
            primary += &format!(
                "<img alt=\"{}\" width=\"{width}\" style=\"max-width:100%;height:auto\" src=\"{}\">\n",
                escape(title),
                uri(&poster)
            );
        }
        primary += &format!(
            "\nThe view below is compiled and rendered for this preview. Application messages are supplied by the example; the surrounding padding and theme belong to the capture harness.\n\n```rust\n{SCAFFOLD}{source}```\n"
        );
        fs::write(generated.join(format!("{key}-primary.md")), primary)?;
        let image = read(&raw.join(format!("{key}-variants.png")))?;
        let logical_width = png_info(&image)?.width / case.scale;
        let mut variants = format!(
            "\n## Appearance variants\n\nLight theme on the left; dark theme on the right. These are actual component renders, including disabled and structural variants where applicable.\n\n<img alt=\"{} variants in light and dark themes\" width=\"{logical_width}\" style=\"max-width:100%;height:auto\" src=\"{}\">\n",
            escape(title),
            uri(&image)
        );
        let variant_source = rustfmt(&read(&raw.join(format!("{key}-variants.rs")))?)?;
        let hidden_view: String = source.lines().map(|line| format!("# {line}\n")).collect();
        variants += &format!(
            "\n<details><summary>Code for this comparison</summary>\n\nThis composition uses the same `view` function shown above.\n\n```rust\n{SCAFFOLD}{hidden_view}{variant_source}```\n\n</details>\n"
        );
        fs::write(generated.join(format!("{key}-variants.md")), variants)?;
        if case.compare {
            let animation = read(&raw.join(format!("{key}-comparison.png")))?;
            let poster = read(&raw.join(format!("{key}-comparison-poster.png")))?;
            let mut comparison = String::from(
                "\n## Motion schemes\n\nStandard on the left; Expressive on the right. Identical input timing. Reduced-motion readers see the initial resting state.\n\n",
            );
            comparison += &picture(
                &animation,
                &poster,
                width * 2,
                &format!("{title}: synchronized Standard and Expressive motion"),
            );
            fs::write(generated.join(format!("{key}-comparison.md")), comparison)?;
        }
    }
    let mut records = BTreeMap::new();
    for (name, path) in files(generated)? {
        records.insert(name, record(&fs::read_to_string(path)?)?);
    }
    let manifest = Manifest {
        schema: 2,
        generator: generator_hash(),
        inputs: inputs(repo)?,
        files: records,
        renderer: "TinySkia; Roboto; 60 fps; lossless RGBA8 sparse APNG".into(),
        rustc: command(
            repo,
            "rustc",
            &[&format!("+{TOOLCHAIN}"), "--version"],
            &[],
            true,
        )?
        .trim()
        .into(),
        rustfmt: command(repo, "rustfmt", &["--version"], &[], true)?
            .trim()
            .into(),
    };
    json(&generated.join("manifest.json"), &manifest)?;
    validate(repo, generated)?;
    Ok(generated.into())
}
pub fn stage(repo: &Path, generated: &Path) -> Result<PathBuf> {
    let destination = repo.join("target/doc-media/stage");
    fresh_dir(&repo.join("target/doc-media"), &destination)?;
    let listing = command(
        repo,
        "cargo",
        &["package", "--list", "--locked", "--allow-dirty"],
        &[],
        true,
    )?;
    for line in listing.lines() {
        if !repo.join(line).is_file()
            || matches!(line, "Cargo.toml.orig" | ".cargo_vcs_info.json")
            || line.starts_with("docs/generated/")
        {
            continue;
        }
        let dest = destination.join(line);
        fs::create_dir_all(dest.parent().unwrap())?;
        fs::copy(repo.join(line), dest)?;
    }
    copy_tree(generated, &destination.join(GENERATED))?;
    validate(&destination, &destination.join(GENERATED))?;
    Ok(destination)
}
pub fn pipeline(repo: &Path, mode: &str) -> Result<()> {
    if mode == "prepare-release" {
        ensure!(
            command(repo, "git", &["status", "--porcelain"], &[], true)?
                .trim()
                .is_empty(),
            "Release preparation requires a clean checkout of the merged, CI-approved commit"
        );
        ensure!(
            command(repo, "git", &["branch", "--show-current"], &[], true)?.trim() == "master",
            "Prepare the merged release from master"
        );
        ensure!(
            command(repo, "git", &["rev-parse", "HEAD"], &[], true)?
                == command(repo, "git", &["rev-parse", "origin/master"], &[], true)?,
            "Fetch/pull origin/master and prepare its exact CI-approved commit"
        );
    }
    let work = repo.join("target/doc-media");
    let generated = generate(repo, &work.join("render"))?;
    if mode == "check" {
        let repeated = generate(repo, &work.join("repeat"))?;
        let hashes = |dir: &Path| -> Result<BTreeMap<String, String>> {
            files(dir)?
                .into_iter()
                .map(|(name, path)| Ok((name, sha(read(&path)?))))
                .collect()
        };
        ensure!(
            hashes(&generated)? == hashes(&repeated)?,
            "Documentation generation is not deterministic"
        );
    }
    let staged = stage(repo, &generated)?;
    let target = repo.join("target");
    build_docs(&staged, &target)?;
    if matches!(mode, "check" | "prepare-release") {
        let env = [("CARGO_TARGET_DIR", target.to_str().unwrap())];
        command(
            &staged,
            "cargo",
            &["package", "--locked", "--allow-dirty"],
            &env,
            false,
        )?;
        let package = crate::package::metadata(&staged, false)?;
        let archive = target.join(format!("package/{}.crate", crate::package::stem(&package)?));
        let digest = crate::package::check_archive(repo, &archive)?;
        command(
            &staged,
            "cargo",
            &["publish", "--dry-run", "--locked", "--allow-dirty"],
            &env,
            false,
        )?;
        validate(&staged, &staged.join(GENERATED))?;
        ensure!(
            sha(read(&archive)?) == digest,
            "Publication dry run changed the verified archive"
        );
        json(
            &work.join("release.json"),
            &serde_json::json!({"commit": command(repo, "git", &["rev-parse", "HEAD"], &[], true)?.trim(), "source_inputs": inputs(repo)?, "generator": generator_hash(), "archive_sha256": digest, "stage": staged, "release_candidate": mode == "prepare-release", "published": false}),
        )?;
    }
    println!(
        "Generated documentation: {}",
        target
            .join("doc/iced_m3/guide/components/index.html")
            .display()
    );
    println!("Prepared sources: {} (nothing uploaded)", staged.display());
    Ok(())
}
