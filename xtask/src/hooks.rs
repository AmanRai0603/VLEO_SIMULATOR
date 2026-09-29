//! The git hooks this repository runs, and whether this clone runs them.

use super::*;

/// Where the hooks live, relative to the repository root.
pub(super) const HOOKS_PATH: &str = "tools/githooks";

/// What `core.hooksPath` is set to on this clone, if anything.
///
/// Read through git rather than by parsing `.git/config`: the setting can come
/// from the repository, the user or the system, and only git knows which one
/// won.
pub(super) fn configured_hooks_path(root: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .current_dir(root)
        .args(["config", "--get", "core.hooksPath"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

/// Whether the commit-message hook will actually run here.
pub(super) fn hooks_are_wired(root: &Path) -> bool {
    configured_hooks_path(root).is_some_and(|p| p == HOOKS_PATH)
}

/// Say so, once, on the commands a person runs by hand.
///
/// A warning rather than a refusal: the pipeline has no hooks and does not need
/// them — it re-checks every rule a hook checks, which is the point of a hook
/// being a convenience and not a control. But a person whose hook never ran
/// finds out at review, and that is the expensive place to find out.
pub(super) fn warn_if_hooks_are_not_wired(root: &Path) {
    if std::env::var_os("CI").is_some() || hooks_are_wired(root) {
        return;
    }
    eprintln!(
        "\x1b[33mnote: the commit-message hook is not installed on this clone.\n      \
         Run `cargo xtask setup` once. Without it a bad commit subject is\n      \
         caught in the pipeline instead of before the commit.\x1b[0m"
    );
}

/// Install the hooks on this clone.
///
/// Git will not follow a path committed to the repository on its own: a hook
/// that ran because it was cloned would be arbitrary code from a pull request.
/// So it is one command per person, and this is that command rather than a
/// line of prose someone has to find.
pub(super) fn cmd_setup(root: &Path) -> Result<(), String> {
    let hooks = root.join(HOOKS_PATH);
    if !hooks.is_dir() {
        return Err(format!("{} is not a directory", hooks.display()));
    }

    match configured_hooks_path(root) {
        Some(p) if p == HOOKS_PATH => {
            println!("core.hooksPath is already {HOOKS_PATH}");
        }
        other => {
            if let Some(p) = &other {
                println!("core.hooksPath was {p}");
            }
            let st = std::process::Command::new("git")
                .current_dir(root)
                .args(["config", "core.hooksPath", HOOKS_PATH])
                .status()
                .map_err(|e| format!("running git: {e}"))?;
            if !st.success() {
                return Err("git config core.hooksPath failed".into());
            }
            // Read it back. Setting a value and reporting success without
            // looking is how a setup command comes to be trusted wrongly.
            if !hooks_are_wired(root) {
                return Err(format!(
                    "git config accepted core.hooksPath={HOOKS_PATH} and reading it back \
                     gave something else — a user or system setting is overriding it"
                ));
            }
            println!("core.hooksPath is now {HOOKS_PATH}");
        }
    }

    let mut listed = 0usize;
    for e in fs::read_dir(&hooks).map_err(|e| format!("{}: {e}", hooks.display()))? {
        let e = e.map_err(|e| format!("{e}"))?;
        let name = e.file_name().to_string_lossy().to_string();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let m = e.metadata().map_err(|e| format!("{e}"))?;
            if m.permissions().mode() & 0o111 == 0 {
                return Err(format!(
                    "{name} is not executable — git runs hooks by executing them, so this \
                     one would be skipped silently"
                ));
            }
        }
        println!("  hook: {name}");
        listed += 1;
    }
    if listed == 0 {
        return Err(format!("{} is empty", hooks.display()));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
