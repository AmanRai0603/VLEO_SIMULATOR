//! The method language, from the terminal: a node's method checked against its
//! author's cases, and the checker every node form carries rebuilt.
//!
//! Both read `vleo_sheet::method`, the one implementation — the form runs the
//! same function compiled to WebAssembly, so what `xtask method` says about a
//! node is what the author saw in their form before they sent it.

use std::fs;
use std::path::Path;
use std::process::Command;

/// Where the checker lives, and the fingerprint of what it was built from.
pub const WASM: &str = "web/method.wasm";
pub const STAMP: &str = "web/method.wasm.stamp";

/// `method <node>` — the node's method, checked, and each of its author's
/// cases run through it.
pub fn cmd_method(root: &Path, args: &[&str]) -> Result<(), String> {
    let id = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .ok_or("usage: cargo run -p xtask -- method <node>")?;
    let tree = vleo_sheet::load_all(root)?;
    let sh = tree
        .sheets
        .get(*id)
        .ok_or_else(|| format!("no node '{id}'"))?;
    let text = fs::read_to_string(sh.dir.join("node.toml")).map_err(|e| e.to_string())?;
    let r = vleo_sheet::method::report_toml(&text)?;
    if sh.method.text.trim().is_empty() {
        println!("{id}: no method yet — the node keeps its hand-written holes until its owner sends one.");
        return Ok(());
    }
    println!(
        "{id}: the method (language {})",
        vleo_sheet::method::LANGUAGE_VERSION
    );
    for d in &r.diags {
        println!("  {d}");
    }
    for (i, (c, v)) in r.cases.iter().enumerate() {
        let mark = if v.agrees() { "ok  " } else { "FAIL" };
        let got = r.got.get(i).copied().flatten();
        let shown = match (c.expect, got) {
            (Some(e), Some(g)) => format!(" — your code {e:e}, the method {g:e}"),
            _ => String::new(),
        };
        println!("  {mark} {}: {}{shown}", c.label, v.text(c));
    }
    for s in &r.shortfall {
        println!("  missing: {s}");
    }
    if r.sound() {
        println!("sound: the method checks and agrees with every one of its author's cases.");
        Ok(())
    } else {
        Err(format!("{id}: the method is not sound yet — see above"))
    }
}

/// `method-wasm [--check]` — rebuild the checker the node form carries, or
/// only say whether it is current.
pub fn cmd_method_wasm(root: &Path, args: &[&str]) -> Result<(), String> {
    let now = vleo_sheet::method::checker_fingerprint(root)?;
    let stamp = fs::read_to_string(root.join(STAMP)).unwrap_or_default();
    let recorded = stamp
        .lines()
        .find_map(|l| l.strip_prefix("fingerprint = "))
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_default();
    if args.contains(&"--check") {
        if recorded == now {
            println!("method-wasm: {WASM} is built from the sources as they are ({now})");
            return Ok(());
        }
        return Err(format!(
            "{WASM} was built from different sources ({recorded:?}, now {now}). Every node form \
             carries it, so a form would check methods by the old rules. Run \
             `cargo run -p xtask -- method-wasm` and commit both files."
        ));
    }
    let manifest = root.join("crates/vleo-method-wasm/Cargo.toml");
    let mut cmd = Command::new("cargo");
    cmd.args([
        "build",
        "--release",
        "--target",
        "wasm32-unknown-unknown",
        "--manifest-path",
    ])
    .arg(&manifest)
    .current_dir(root);
    if root.join("crates/vleo-method-wasm/Cargo.lock").exists() {
        cmd.arg("--locked");
    }
    let ok = cmd.status().map_err(|e| e.to_string())?.success();
    if !ok {
        return Err(
            "the checker did not build — is the wasm32-unknown-unknown target installed? \
                    `rustup target add wasm32-unknown-unknown`"
                .into(),
        );
    }
    let built = root.join(
        "crates/vleo-method-wasm/target/wasm32-unknown-unknown/release/vleo_method_wasm.wasm",
    );
    fs::copy(&built, root.join(WASM)).map_err(|e| format!("{}: {e}", built.display()))?;
    fs::write(
        root.join(STAMP),
        format!(
            "# The node form's method checker: web/method.wasm, built by\n\
             # `cargo run -p xtask -- method-wasm` from vleo_sheet::method::CHECKER_SOURCES.\n\
             # A test refuses a checkout whose sources have moved on from it.\n\
             fingerprint = \"{now}\"\nlanguage = {}\n",
            vleo_sheet::method::LANGUAGE_VERSION
        ),
    )
    .map_err(|e| e.to_string())?;
    let size = fs::metadata(root.join(WASM)).map(|m| m.len()).unwrap_or(0);
    println!(
        "method-wasm: {WASM} — {} KB, fingerprint {now}",
        size / 1024
    );
    Ok(())
}
