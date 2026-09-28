//! Every node form carries the method checker inside it, as `web/method.wasm.gz`,
//! and checks an author's method with it before anything is sent. If the
//! checker's sources move on and the file is not rebuilt, a form checks by the
//! old rules while intake and the gate check by the new ones — the one
//! disagreement a single implementation exists to prevent.
//!
//! The form reads the checker from the tree when it is made; no program carries
//! it built in. A program holding compressed WebAssembly beside the script that
//! unpacks and runs it is what browsers and antivirus block, and the first
//! 0.3.0 Windows kit was blocked by Chrome for exactly that.

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

/// The checker block of a form, decoded from base64.
fn carried(html: &str) -> Vec<u8> {
    let open = "id=\"vleo-method-wasm\">";
    let a = html.find(open).expect("the form has a checker block") + open.len();
    let b = a + html[a..].find("</script>").unwrap();
    let mut out = Vec::new();
    let mut acc = 0u32;
    let mut bits = 0;
    for c in html[a..b].trim().bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => continue,
        };
        acc = acc << 6 | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    out
}

#[test]
fn the_form_carries_the_checker_it_finds_beside_the_tree() {
    let mut tree = vleo_sheet::load::load_all(&root()).unwrap();
    let sh = tree.sheets.values().next().unwrap().clone();
    let gz = std::fs::read(root().join(vleo_sheet::template::CHECKER)).unwrap();
    assert_eq!(carried(&vleo_sheet::template::document(&sh, &tree)), gz);
    assert_eq!(carried(&vleo_sheet::template::document_new(&tree)), gz);
    // A tree without the file makes a form without a checker, which says so
    // on its check; it never carries one from anywhere else.
    tree.root = std::env::temp_dir().join("vleo-no-checker-here");
    assert!(carried(&vleo_sheet::template::document(&sh, &tree)).is_empty());
}
