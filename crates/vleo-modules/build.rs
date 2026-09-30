//! Compile the graph into the engine.
//!
//! The resolver needs all three graphs. If it read them from a file at run time
//! the engine and the graph could disagree — an engine built on Tuesday walking
//! Wednesday's graph — so they are generated as tables and compiled in. Adding
//! an edge is therefore a rebuild, which is correct: it changes what the engine
//! computes, so it should go through the gate.
//!
//! This is the same `vleo-sheet` the generators use. Two implementations of
//! what a sheet means would be two standards.

use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let root = manifest
        .parent()
        .and_then(Path::parent)
        .expect("the repository root")
        .to_path_buf();
    println!("cargo:rerun-if-changed={}", root.join("layers").display());
    println!("cargo:rerun-if-changed={}", root.join("cases").display());
    println!("cargo:rerun-if-changed={}", root.join("sources").display());
    for e in std::fs::read_dir(root.join("crates")).expect("crates/") {
        let p = e.expect("dir entry").path();
        if p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("vleo-mod-"))
            .unwrap_or(false)
        {
            println!("cargo:rerun-if-changed={}", p.join("nodes").display());
        }
    }

    let tree = match vleo_sheet::load_all(&root) {
        Ok(t) => t,
        Err(e) => {
            println!("cargo:warning=the sheets did not load: {e}");
            std::process::exit(1);
        }
    };
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    std::fs::write(out.join("tables.rs"), vleo_sheet::emit::tables_rs(&tree))
        .expect("write tables.rs");
}
