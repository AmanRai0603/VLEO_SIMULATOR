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
    let wrong = vleo_sheet::wiring::errors(&tree);
    if !wrong.is_empty() {
        for w in &wrong {
            println!("cargo:warning={w}");
        }
        println!(
            "cargo:warning={} name(s) in the sheets resolve to nothing; the engine is not built on a guess",
            wrong.len()
        );
        std::process::exit(1);
    }
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    std::fs::write(out.join("tables.rs"), vleo_sheet::emit::tables_rs(&tree))
        .expect("write tables.rs");

    // THE ENGINE'S OWN CODE, HASHED. Each node's hash covers its HOLE code,
    // but a node's arithmetic is mostly the kernel relations it calls, and the
    // resolver that walks them. A saved result is reused when the engine that
    // would answer is the one that answered, so a change to any of that code
    // has to change the engine's identity — or an answer from before a formula
    // was corrected would be shown for the corrected engine.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for dir in [
        "vleo-core/src",
        "vleo-units/src",
        "vleo-bus/src",
        "vleo-modules/src",
    ] {
        let d = root.join("crates").join(dir);
        println!("cargo:rerun-if-changed={}", d.display());
        let mut files = Vec::new();
        rust_files(&d, &mut files);
        files.sort();
        for f in files {
            let rel = f
                .strip_prefix(&root)
                .unwrap_or(&f)
                .to_string_lossy()
                .replace('\\', "/");
            // Line endings are not code: a Windows checkout and a Linux one
            // are the same engine.
            let text = std::fs::read(&f).unwrap_or_default();
            for b in rel
                .bytes()
                .chain([0u8])
                .chain(text.into_iter().filter(|&b| b != b'\r'))
            {
                h ^= b as u64;
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
    }
    std::fs::write(
        out.join("engine_source.rs"),
        format!(
            "/// Every source file of the kernel, units, bus and facade, hashed.\n\
             pub const ENGINE_SOURCE: u64 = 0x{h:016x};\n"
        ),
    )
    .expect("write engine_source.rs");
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}
