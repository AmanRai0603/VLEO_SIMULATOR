//! The pages carry this library compiled to WebAssembly, as
//! `web/files.wasm.gz`, and check files with it. If the library's sources move
//! on and that file is not rebuilt, a page checks signatures by the old rules
//! while the installed application checks them by the new — the one
//! disagreement a single implementation exists to prevent.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_page_carries_the_library_built_from_the_current_sources() {
    let root = root();
    let now = vleo_files::page::sources_fingerprint(&root).unwrap();
    let stamp = std::fs::read_to_string(root.join("web/files.wasm.stamp")).unwrap_or_default();
    let recorded = stamp
        .lines()
        .find_map(|l| l.strip_prefix("fingerprint = "))
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_default();
    assert_eq!(
        recorded, now,
        "web/files.wasm.gz was built from other sources than these. Run \
         `cargo run -p xtask -- files-wasm` and commit web/files.wasm.gz and its stamp."
    );
    let gz = std::fs::read(root.join("web/files.wasm.gz")).unwrap();
    assert_eq!(&gz[..2], &[0x1f, 0x8b], "web/files.wasm.gz is not gzip");
}

#[test]
fn the_stamp_moves_when_a_source_does() {
    // A copy of the sources with one byte changed gives another fingerprint.
    let root = root();
    let tmp = std::env::temp_dir().join(format!("vleo-files-stamp-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    for s in vleo_files::page::WASM_SOURCES {
        let from = root.join(s);
        let to = tmp.join(s);
        if from.is_dir() {
            std::fs::create_dir_all(&to).unwrap();
            for e in std::fs::read_dir(&from).unwrap() {
                let e = e.unwrap();
                std::fs::copy(e.path(), to.join(e.file_name())).unwrap();
            }
        } else {
            std::fs::create_dir_all(to.parent().unwrap()).unwrap();
            std::fs::copy(&from, &to).unwrap();
        }
    }
    let before = vleo_files::page::sources_fingerprint(&tmp).unwrap();
    assert_eq!(
        before,
        vleo_files::page::sources_fingerprint(&root).unwrap()
    );
    let chain = tmp.join("crates/vleo-files/src/chain.rs");
    let mut text = std::fs::read_to_string(&chain).unwrap();
    text.push(' ');
    std::fs::write(&chain, text).unwrap();
    assert_ne!(before, vleo_files::page::sources_fingerprint(&tmp).unwrap());
    std::fs::remove_dir_all(&tmp).unwrap();
}
