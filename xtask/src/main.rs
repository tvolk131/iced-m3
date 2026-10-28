//! Development-only documentation and release checks. No command uploads.
mod generate;
mod material_symbols;
mod media;
mod package;
mod registry;
#[cfg(test)]
mod tests;
mod util;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

const HELP: &str = "Usage:
  cargo xtask doc-media build
  cargo xtask doc-media check
  cargo xtask doc-media verify [--root PATH]
  cargo xtask doc-media check-archive --archive PATH
  cargo xtask check-package [--source PATH] [--offline] [--require-doc-media]
  cargo xtask material-symbols verify [--root PATH]
  cargo xtask material-symbols generate --archive PATH [--root PATH]
  cargo xtask prepare-release

build renders and checks rustdoc in staging; check additionally renders twice,
checks the extracted archive, and dry-runs publication. prepare-release requires
a clean merged master checkout. No command uploads. See docs/RELEASING.md.";
#[derive(Debug, PartialEq)]
enum Task {
    Help,
    Media(String),
    Verify(PathBuf),
    Archive(PathBuf),
    Symbols {
        root: PathBuf,
        archive: Option<PathBuf>,
    },
    Package {
        source: PathBuf,
        offline: bool,
        require_media: bool,
    },
}
fn parse(args: &[String], repo: &Path) -> Result<Task> {
    let strings: Vec<_> = args.iter().map(String::as_str).collect();
    match strings.as_slice() {
        [] | ["--help"] | ["-h"] => Ok(Task::Help),
        ["prepare-release"] => Ok(Task::Media("prepare-release".into())),
        ["doc-media", mode @ ("build" | "check")] => Ok(Task::Media((*mode).into())),
        ["doc-media", "verify"] => Ok(Task::Verify(repo.into())),
        ["doc-media", "verify", "--root", root] => Ok(Task::Verify(PathBuf::from(root))),
        ["doc-media", "check-archive", "--archive", archive] => {
            Ok(Task::Archive(PathBuf::from(archive)))
        }
        [
            "material-symbols",
            mode @ ("verify" | "generate"),
            rest @ ..,
        ] => {
            let (mut root, mut archive) = (None, None);
            let mut args = rest.iter();
            while let Some(arg) = args.next() {
                match *arg {
                    "--root" if root.is_none() => {
                        root = Some(PathBuf::from(
                            args.next().context("--root requires a path")?,
                        ))
                    }
                    "--archive" if *mode == "generate" && archive.is_none() => {
                        archive = Some(PathBuf::from(
                            args.next().context("--archive requires a path")?,
                        ))
                    }
                    _ => bail!("Unknown/duplicate argument {arg}\n{HELP}"),
                }
            }
            anyhow::ensure!(
                *mode != "generate" || archive.is_some(),
                "generate requires --archive PATH"
            );
            Ok(Task::Symbols {
                root: root.unwrap_or_else(|| repo.join("crates/material-symbols")),
                archive,
            })
        }
        ["check-package", rest @ ..] => {
            let mut source = None;
            let mut offline = false;
            let mut require_media = false;
            let mut iter = rest.iter();
            while let Some(arg) = iter.next() {
                match *arg {
                    "--source" if source.is_none() => {
                        source =
                            Some(PathBuf::from(iter.next().ok_or_else(|| {
                                anyhow::anyhow!("--source requires a path")
                            })?))
                    }
                    "--offline" if !offline => offline = true,
                    "--require-doc-media" if !require_media => require_media = true,
                    _ => bail!("Unknown/duplicate argument {arg}\n{HELP}"),
                }
            }
            Ok(Task::Package {
                source: source.unwrap_or_else(|| repo.into()),
                offline,
                require_media,
            })
        }
        _ => bail!("Invalid command\n{HELP}"),
    }
}
fn main() -> Result<()> {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let task = parse(&std::env::args().skip(1).collect::<Vec<_>>(), repo)?;
    match task {
        Task::Help => println!("{HELP}"),
        Task::Media(mode) => generate::pipeline(repo, &mode)?,
        Task::Verify(root) => {
            media::validate(&root, &root.join(media::GENERATED))?;
            println!("Documentation assets match source inputs, generator, and registry.");
        }
        Task::Archive(archive) => {
            package::check_archive(repo, &archive)?;
        }
        Task::Symbols { root, archive } => {
            if let Some(archive) = archive {
                material_symbols::generate(&root, &archive)?;
            } else {
                material_symbols::verify(&root)?;
            }
        }
        Task::Package {
            source,
            offline,
            require_media,
        } => package::check_package(repo, &source.canonicalize()?, offline, require_media)?,
    }
    Ok(())
}
