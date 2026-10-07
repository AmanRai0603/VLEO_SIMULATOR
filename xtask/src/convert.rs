//! `convert`: the design, written as its files (docs/PLAN_1_0.md, phase E).
//!
//! Every branch, row and case of the tree, in the one schema, laid out as the
//! shared drive holds them (docs/OPERATING_1_0.md, section 15): each group's
//! file and node files under `groups/<group>/`, each case under `cases/`. The
//! conversion is `vleo_files::convert`; this writes what it makes, and reads
//! it back through its inverse to show it is the tree. Nothing in the
//! repository changes: it writes beside the build, or to `--out`.

use super::*;

/// Where `convert` writes when not told: beside the build, never committed.
fn default_out(root: &Path) -> PathBuf {
    root.join("target").join("converted")
}

pub(super) fn cmd_convert(root: &Path, args: &[&str]) -> Result<(), String> {
    let out = args
        .iter()
        .position(|a| *a == "--out")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| default_out(root));
    if out.exists()
        && fs::read_dir(&out)
            .map_err(|e| e.to_string())?
            .next()
            .is_some()
    {
        if out != default_out(root) {
            return Err(format!(
                "{} is not empty. The conversion writes into an empty folder, so \
                 nothing it finds there is taken for part of the design; name another \
                 with --out",
                out.display()
            ));
        }
        fs::remove_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    write(root, &out)
}

/// The design converted again into `design/`, after a deliberate change to it.
///
/// `design/` is the design as its files, and a test holds it equal to the
/// sheets until they leave (AGENTS.md, *Until the switch-over*). A form
/// applied, or a group's release built, changes the sheets on purpose, so the
/// copy is written again here, beside today's answers, and the difference goes
/// into the same commit. What it says is how many files changed.
pub(crate) fn record_design(root: &Path) -> Result<String, String> {
    let out = root.join("design");
    if out.exists() {
        fs::remove_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    write(root, &out)?;
    let changed = std::process::Command::new("git")
        .args(["status", "--porcelain", "--", "design"])
        .current_dir(root)
        .output()
        .map_err(|e| e.to_string())?;
    let n = String::from_utf8_lossy(&changed.stdout).lines().count();
    Ok(if n == 0 {
        "no file of design/ changed".to_string()
    } else {
        format!("{n} file(s) of design/ written again: commit them with the change")
    })
}

/// The tree converted and written into `out`, which is empty or absent.
fn write(root: &Path, out: &Path) -> Result<(), String> {
    println!("1/3  reading the tree");
    let tree = load(root)?;
    println!(
        "2/3  converting {} rows and {} headings",
        tree.sheets.len(),
        tree.groups.len()
    );
    let app = format!("vleo {}", release::workspace_version(root)?);
    let files = vleo_files::convert::convert(&tree, &vleo_sheet::files::Disk, &app)
        .map_err(|e| e.to_string())?;
    // Read back as the folders before anything is written: a conversion whose
    // inverse does not load as a design is not written at all.
    let served = vleo_files::convert::Served::new(
        root,
        &files,
        std::sync::Arc::new(vleo_sheet::files::Disk),
    )
    .map_err(|e| e.to_string())?;
    let back = vleo_sheet::load::load_all_from(&served, root)?;
    println!("3/3  writing {} files to {}", files.len(), out.display());
    for (path, f) in &files {
        let p = out.join(path);
        if let Some(d) = p.parent() {
            fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
        }
        vleo_files::sqlite::write(f, &p).map_err(|e| format!("{}: {e}", p.display()))?;
    }
    let groups = files
        .iter()
        .filter(|(p, _)| p.ends_with(&format!(".{}", vleo_files::convert::GROUP_FILE)))
        .count();
    println!(
        "converted: {groups} groups, {} rows read back ({} of them open blocks proposed where the \
         breakdown holds none yet), {} cases. Read them back with `vleo_files::convert::Served`; delete {} to undo.",
        back.sheets.len(),
        back.sheets.len() - tree.sheets.len(),
        back.cases.len(),
        out.display()
    );
    Ok(())
}
