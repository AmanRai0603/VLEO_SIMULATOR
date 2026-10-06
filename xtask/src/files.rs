//! The design-file library, built for the page.
//!
//! `vleo_files` is the one implementation of what a design file is and what a
//! signature means. The installed application links it; a page opened from a
//! file carries it compiled to WebAssembly, as `web/files.wasm.gz`, so the
//! page and the application refuse the same files by the same code. This
//! builds that file from `crates/vleo-files-wasm`, and stamps it with the
//! fingerprint of what it was built from (`vleo_files::page::WASM_SOURCES`).

use std::fs;
use std::path::Path;
use std::process::Command;

/// Where the page's build lives, and the fingerprint of what it was built from.
pub const WASM: &str = "web/files.wasm.gz";
pub const STAMP: &str = "web/files.wasm.stamp";

fn recorded(root: &Path) -> String {
    fs::read_to_string(root.join(STAMP))
        .unwrap_or_default()
        .lines()
        .find_map(|l| l.strip_prefix("fingerprint = "))
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_default()
}

/// `files-wasm [--check]` — rebuild the library the pages carry, or only say
/// whether it is current.
pub fn cmd_files_wasm(root: &Path, args: &[&str]) -> Result<(), String> {
    let now = vleo_files::page::sources_fingerprint(root).map_err(|e| e.to_string())?;
    if args.contains(&"--check") {
        if recorded(root) == now {
            println!("files-wasm: {WASM} is built from the sources as they are ({now})");
            return Ok(());
        }
        return Err(format!(
            "{WASM} was built from different sources ({:?}, now {now}). A page carries it, so \
             the page would check files by the old rules while the application checks them by \
             the new. Run `cargo run -p xtask -- files-wasm` and commit both files.",
            recorded(root)
        ));
    }
    let manifest = root.join("crates/vleo-files-wasm/Cargo.toml");
    let ok = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--locked",
            "--target",
            "wasm32-unknown-unknown",
            "--manifest-path",
        ])
        .arg(&manifest)
        .current_dir(root)
        .status()
        .map_err(|e| e.to_string())?
        .success();
    if !ok {
        return Err(
            "the library did not build for the page — is the wasm32-unknown-unknown target \
             installed? `rustup target add wasm32-unknown-unknown`"
                .into(),
        );
    }
    let built = root
        .join("crates/vleo-files-wasm/target/wasm32-unknown-unknown/release/vleo_files_wasm.wasm");
    // Gzipped with no name or time in the header, so the same build gives the
    // same bytes.
    let gz = Command::new("gzip")
        .args(["-9", "-n", "-c"])
        .arg(&built)
        .output()
        .map_err(|e| format!("gzip could not be run: {e}"))?;
    if !gz.status.success() {
        return Err("gzip failed on the built library".into());
    }
    fs::write(root.join(WASM), &gz.stdout).map_err(|e| e.to_string())?;
    fs::write(
        root.join(STAMP),
        format!(
            "# The design-file library the pages carry: web/files.wasm.gz, built by\n\
             # `cargo run -p xtask -- files-wasm` from vleo_files::page::WASM_SOURCES.\n\
             # A test refuses a checkout whose sources have moved on from it.\n\
             fingerprint = \"{now}\"\nformat = {}\n",
            vleo_files::meta::FORMAT
        ),
    )
    .map_err(|e| e.to_string())?;
    println!(
        "files-wasm: {WASM} — {} KB, fingerprint {now}",
        gz.stdout.len() / 1024
    );
    Ok(())
}
