//! The design's files as the rows a page hands the engine compiled for the
//! browser (`crates/vleo-kernel-wasm`, `vleo_open`), for
//! `tools/kernel_check.mjs`.
//!
//!     cargo run -p vleo-files --example design_rows -- <file>
//!
//! writes `design/`, read as everything reads it, encoded by
//! `vleo_files::rows::encode_design`.

use std::path::Path;

fn main() {
    let out = std::env::args()
        .nth(1)
        .expect("usage: design_rows <file>");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (files, _) = vleo_files::convert::read_folder(&root.join("design"))
        .unwrap_or_else(|e| panic!("the design folder does not open: {e}"));
    std::fs::write(&out, vleo_files::rows::encode_design(&files))
        .unwrap_or_else(|e| panic!("{out}: {e}"));
    println!("{} files", files.len());
}
