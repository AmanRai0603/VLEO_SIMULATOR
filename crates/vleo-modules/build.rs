//! The engine's own identity: every source file of the kernel, units, bus and
//! facade, hashed. The design is not compiled in; it is read from its files
//! when a face opens it (`opened`).

use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let root = manifest
        .parent()
        .and_then(Path::parent)
        .expect("the repository root")
        .to_path_buf();
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));

    // THE ENGINE'S OWN CODE, HASHED. A node's arithmetic is the kernel
    // relations its method calls, and the resolver that walks them. A saved
    // result is reused when the engine that would answer is the one that
    // answered, so a change to any of that code has to change the engine's
    // identity — or an answer from before a formula was corrected would be
    // shown for the corrected engine.
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
