//! Wire the node folders into the crate.
//!
//! An aggregate, so it is built and never committed. `#[path]` is absolute
//! because the file lives in OUT_DIR; nothing generated into the repository
//! ever carries an absolute path, or the regeneration diff would fail on a
//! different machine.

use std::fs;
use std::path::Path;

fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let out = std::env::var("OUT_DIR").expect("OUT_DIR");
    let nodes = Path::new(&manifest).join("nodes");
    println!("cargo:rerun-if-changed={}", nodes.display());

    let mut dirs: Vec<String> = Vec::new();
    if nodes.is_dir() {
        for e in fs::read_dir(&nodes).expect("nodes/") {
            let p = e.expect("dir entry").path();
            if p.join("mod.rs").is_file() {
                let name = p.file_name().unwrap().to_string_lossy().to_string();
                println!("cargo:rerun-if-changed={}", p.display());
                dirs.push(name);
            }
        }
    }
    dirs.sort();

    let mut src = String::from(
        "// GENERATED at build time. An aggregate: built, never committed.\n\n",
    );
    for d in &dirs {
        let ident = d.replace(['-', '.'], "_");
        src.push_str(&format!(
            "#[path = \"{}/nodes/{}/mod.rs\"]\npub mod {};\n",
            manifest, d, ident
        ));
    }
    fs::write(Path::new(&out).join("nodes.rs"), src).expect("write nodes.rs");
}
