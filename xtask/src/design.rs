//! `design.vleo`, written from the tree and checked against it.
//!
//! The tool a team runs reads the design from this one file rather than from
//! seven thousand files in folders (`vleo-design`). It is written here, from
//! the tree as it is committed, and `--check` holds a file to the tree: every
//! file's bytes against its SHA-256, the fingerprint, and the file against the
//! folders it was written from, one difference at a time.

use super::*;

/// Where `design` writes when not told: beside the build, never committed.
pub(super) fn default_out(root: &Path) -> PathBuf {
    root.join("target").join(vleo_design::FILE)
}

/// The commit the tree is at, or "" outside a checkout.
pub(super) fn head(root: &Path) -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// Write the design file for the tree at `root` to `out`.
pub(super) fn write(root: &Path, out: &Path) -> Result<vleo_design::Written, String> {
    let stamp = vleo_design::Stamp {
        tool: release::workspace_version(root)?,
        commit: head(root),
        built: fills::today(),
    };
    vleo_design::write(root, out, &stamp).map_err(String::from)
}

pub(super) fn cmd_design(root: &Path, args: &[&str]) -> Result<(), String> {
    let flag = |f: &str| {
        args.iter()
            .position(|a| *a == f)
            .and_then(|i| args.get(i + 1))
            .map(PathBuf::from)
    };
    if let Some(file) = flag("--check") {
        let d = vleo_design::Design::open(&file, root).map_err(String::from)?;
        d.verify().map_err(String::from)?;
        let diff = d.compare(root).map_err(String::from)?;
        if !diff.is_empty() {
            for line in diff.iter().take(20) {
                println!("  {line}");
            }
            if diff.len() > 20 {
                println!("  … and {} more", diff.len() - 20);
            }
            return Err(format!(
                "{} is not the tree: {} difference(s). Write it again with \
                 `cargo run -p xtask -- design`",
                file.display(),
                diff.len()
            ));
        }
        println!(
            "{} is the tree: {} files, {} rows, fingerprint {}…, written by {} from {}",
            file.display(),
            d.meta("files"),
            d.meta("rows"),
            &d.meta("fingerprint")[..16.min(d.meta("fingerprint").len())],
            d.meta("tool"),
            if d.meta("commit").is_empty() {
                "an unknown commit"
            } else {
                d.meta("commit")
            }
        );
        return Ok(());
    }
    let out = flag("--out").unwrap_or_else(|| default_out(root));
    let w = write(root, &out)?;
    println!(
        "wrote {}: {} rows, {} files, {:.1} MB, fingerprint {}…",
        out.display(),
        w.rows,
        w.files,
        w.bytes as f64 / 1e6,
        &w.fingerprint[..16]
    );
    Ok(())
}
