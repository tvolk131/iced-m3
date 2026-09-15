"""Verify the distributable and build an independent client against its contents.

This never publishes. Run from any directory with Python 3.9+ and Cargo on PATH.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--offline", action="store_true", help="Use already cached dependencies")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[1]
    flags = ["--locked"] + (["--offline"] if args.offline else [])
    metadata = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps", *flags],
        cwd=repo, capture_output=True, text=True, check=True,
    )
    package = next(p for p in json.loads(metadata.stdout)["packages"]
                   if Path(p["manifest_path"]) == repo / "Cargo.toml")
    stem = f'{package["name"]}-{package["version"]}'
    target = repo / "target"
    target.mkdir(exist_ok=True)
    env = dict(os.environ, CARGO_TARGET_DIR=str(target))

    def cargo(*arguments):
        subprocess.run(["cargo", *arguments, *flags], cwd=repo, env=env, check=True)

    # Cargo itself extracts and compiles the package before this returns.
    cargo("package", "--allow-dirty")
    archive = target / "package" / f"{stem}.crate"
    with tarfile.open(archive, "r:gz") as tar:
        members = tar.getmembers()
        names = {member.name.removeprefix(f"{stem}/") for member in members}
        required = {
            "src/lib.rs", "README.md", "LICENSE", "NOTICE", "Cargo.lock",
            "assets/fonts/Roboto-Variable.ttf", "assets/fonts/OFL.txt",
            "assets/loading/morphs.bin", "assets/loading/NOTICE",
            "assets/loading/LICENSE-APACHE-2.0.txt", "assets/loading/README.md",
            "assets/loading/SHA256SUMS", "tools/generate_loading_shapes.py",
            "tests/visual/mod.rs", "tests/visual/progress-preview.html",
            "assets/loading/reference/soft-burst.svg", "examples/minimal.rs",
            "examples/gallery/icons.rs",
            "src/guide.rs", "docs/GETTING_STARTED.md", "docs/COOKBOOK.md",
            "docs/THEMING.md", "docs/README.md", "docs/desktop-beta.png",
        }
        missing = required - names
        if missing:
            raise RuntimeError(f"Missing package inputs/notices: {sorted(missing)}")
        forbidden = [name for name in names if name.startswith(("target/", "consumers/", ".github/", "tests/visual/references/"))]
        if forbidden:
            raise RuntimeError(f"Development-only files leaked into package: {forbidden[:10]}")
        if archive.stat().st_size > 5_000_000:
            raise RuntimeError("Package unexpectedly exceeds 5 MB compressed")
        total_bytes = sum(member.size for member in members)
        for member in members:
            path = Path(member.name)
            if path.is_absolute() or ".." in path.parts or path.parts[0] != stem or not (member.isfile() or member.isdir()):
                raise RuntimeError(f"Unexpected package entry: {member.name}")

        with tempfile.TemporaryDirectory(prefix="consumer-package-", dir=target) as temporary:
            temporary = Path(temporary)
            # Entries above are constrained to ordinary files/directories below stem.
            tar.extractall(temporary, members=members)
            packaged = temporary / stem
            consumer = temporary / "consumer"
            shutil.copytree(repo / "consumers/desktop", consumer, ignore=shutil.ignore_patterns("target"))
            manifest = consumer / "Cargo.toml"
            original = manifest.read_text()
            needle = 'path = "../.."'
            if original.count(needle) != 1:
                raise RuntimeError("Consumer path dependency changed; update the package check")
            manifest.write_text(original.replace(needle, "path = " + json.dumps(packaged.as_posix())))
            # This builds normal tests from the tarball, then tests a separate app
            # using that tarball with its own lockfile and feature selection.
            cargo("test", "--manifest-path", str(packaged / "Cargo.toml"), "--all-targets")
            cargo("test", "--manifest-path", str(packaged / "Cargo.toml"), "--doc")
            cargo("test", "--manifest-path", str(consumer / "Cargo.toml"), "--all-targets")
            cargo("test", "--manifest-path", str(consumer / "Cargo.toml"), "--all-targets", "--no-default-features")
            cargo("tree", "--manifest-path", str(consumer / "Cargo.toml"), "-i", "iced-material")

    result = {
        "archive": str(archive.relative_to(repo)),
        "files": len(names),
        "uncompressed_bytes": total_bytes,
        "compressed_bytes": archive.stat().st_size,
        "packaged_tests": "passed",
        "packaged_doctests": "passed",
        "independent_consumer_default_and_software": "passed",
        "published": False,
    }
    (target / "beta-package-report.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
