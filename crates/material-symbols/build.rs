mod codegen;

use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=codegen.rs");
    println!("cargo::rerun-if-changed=assets/catalog.json");

    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("Missing package directory")?);
    let output = PathBuf::from(env::var_os("OUT_DIR").ok_or("Missing Cargo output directory")?);
    let metadata = fs::read(root.join("assets/catalog.json"))?;
    let source = codegen::from_json(&metadata)?;
    fs::write(output.join("catalog.rs"), source)?;
    Ok(())
}
