//! Every node form carries the method checker inside it, as `web/method.wasm.gz`,
//! and checks an author's method with it before anything is sent. If the
//! checker's sources move on and the file is not rebuilt, a form checks by the
//! old rules while intake and the gate check by the new ones — the one
//! disagreement a single implementation exists to prevent.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_form_carries_the_checker_built_from_the_current_sources() {
    let root = root();
    let now = vleo_sheet::method::checker_fingerprint(&root).unwrap();
    let stamp = std::fs::read_to_string(root.join("web/method.wasm.stamp")).unwrap_or_default();
    let recorded = stamp
        .lines()
        .find_map(|l| l.strip_prefix("fingerprint = "))
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_default();
    assert_eq!(
        recorded, now,
        "web/method.wasm.gz was built from other sources than these. Run \
         `cargo run -p xtask -- method-wasm` and commit web/method.wasm.gz and its stamp."
    );
    let gz = std::fs::read(root.join("web/method.wasm.gz")).unwrap();
    assert_eq!(&gz[..2], &[0x1f, 0x8b], "web/method.wasm.gz is not gzip");
}
