//! Releasing: stamping versions, the reference-data bundles, the kit a team
//! member unzips, and pointing git at the hooks.

use super::*;

/// Install the hooks on this clone.
///
/// Git will not follow a path committed to the repository on its own: a hook
/// that ran because it was cloned would be arbitrary code from a pull request.
/// So it is one command per person, and this is that command rather than a
/// line of prose someone has to find.
pub(crate) fn cmd_setup(root: &Path) -> Result<(), String> {
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

pub(crate) fn cmd_bundle(root: &Path, args: &[&str]) -> Result<(), String> {
    match args.first().copied() {
        Some("publish") => {
            let dir = args
                .get(1)
                .map(PathBuf::from)
                .ok_or("usage: cargo xtask bundle publish <dir>")?;
            let b = vleo_data::load_bundle(&dir)?;
            let computed = vleo_data::hash_files(&dir, &b.manifest.files)?;
            let mp = dir.join("manifest.toml");
            let text = fs::read_to_string(&mp).map_err(|e| e.to_string())?;
            let mut out = String::new();
            for line in text.lines() {
                if line.trim_start().starts_with("content_hash") {
                    out.push_str(&format!("content_hash = \"{computed}\"\n"));
                } else {
                    out.push_str(line);
                    out.push('\n');
                }
            }
            fs::write(&mp, out).map_err(|e| e.to_string())?;
            println!(
                "published {}@{} — {}",
                b.manifest.name, b.manifest.version, computed
            );
            println!("A published version is never edited. A correction is a new version.");
            Ok(())
        }
        Some("verify") | None => {
            let dir = root.join("bundles");
            let mut n = 0;
            let mut bad = 0;
            for e in fs::read_dir(&dir).map_err(|e| e.to_string())? {
                let p = e.map_err(|e| e.to_string())?.path();
                if !p.is_dir() {
                    continue;
                }
                for v in fs::read_dir(&p).map_err(|e| e.to_string())? {
                    let vp = v.map_err(|e| e.to_string())?.path();
                    if !vp.join("manifest.toml").is_file() {
                        continue;
                    }
                    n += 1;
                    let b = vleo_data::load_bundle(&vp)?;
                    if b.verified {
                        println!(
                            "  \x1b[32mok\x1b[0m   {}@{} {}",
                            b.manifest.name, b.manifest.version, b.manifest.content_hash
                        );
                    } else {
                        bad += 1;
                        println!(
                            "  \x1b[31mFAIL\x1b[0m {}@{} — {}",
                            b.manifest.name,
                            b.manifest.version,
                            b.refusal.unwrap_or_default()
                        );
                    }
                }
            }
            println!("{n} bundle(s), {bad} refused");
            if bad > 0 {
                return Err("a tampered byte is refused, never warned about".into());
            }
            Ok(())
        }
        Some(other) => Err(format!("unknown bundle command '{other}'")),
    }
}

/// The workspace version, as `Cargo.toml` states it.
pub(crate) fn workspace_version(root: &Path) -> Result<String, String> {
    let text =
        fs::read_to_string(root.join("Cargo.toml")).map_err(|e| format!("Cargo.toml: {e}"))?;
    let v: toml::Value = text.parse().map_err(|e| format!("Cargo.toml: {e}"))?;
    v.get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .ok_or_else(|| "Cargo.toml has no [workspace.package] version".into())
}

/// A RELEASE STAMPS THE BELIEFS IT CARRIES. Every node version recorded since
/// the last release says `next`; this names them with the release that ships
/// them, so a node's page and a saved result can say which release first held
/// each belief — and why the one before it was replaced.
/// THE TOOL, FOR A TEAM MEMBER, WITHOUT THE REPOSITORY.
///
/// The daemon reads its web face, the tree and every node's page at run time,
/// so the two programs alone do not run. A kit is those programs beside exactly
/// the files they read — nothing else: no git history, no generators, no
/// kernel source. The team uses it and sends back node forms; nothing in it is
/// ever edited, and their case and results live under `~/.vleo/`, outside it,
/// so the next kit replaces this one without losing either.
pub(crate) fn cmd_kit(root: &Path, args: &[&str]) -> Result<(), String> {
    let after = |flag: &str| {
        args.iter()
            .position(|a| *a == flag)
            .and_then(|i| args.get(i + 1))
            .map(PathBuf::from)
    };
    let version = workspace_version(root)?;
    let bin = after("--bin").unwrap_or_else(|| root.join("target").join("release"));
    let out = after("--out").unwrap_or_else(|| root.join("dist").join(format!("vleo-{version}")));
    let exe = |name: &str| {
        [name.to_string(), format!("{name}.exe")]
            .into_iter()
            .map(|n| bin.join(n))
            .find(|p| p.is_file())
    };
    // `--files-only`: the files the tool reads and nothing that runs — what
    // the Python package carries beside its own engine.
    let files_only = args.contains(&"--files-only");
    let wanted: &[&str] = if files_only {
        &[]
    } else {
        &["vleo-daemon", "vleo"]
    };
    let programs: Vec<PathBuf> = wanted
        .iter()
        .map(|n| {
            exe(n).ok_or_else(|| {
                format!(
                    "no {n} in {} — build the programs first: \
                     cargo build --release -p vleo-daemon -p vleo-cli",
                    bin.display()
                )
            })
        })
        .collect::<Result<_, _>>()?;
    if out.exists() {
        fs::remove_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;

    fn copy_tree(from: &Path, to: &Path) -> Result<usize, String> {
        let mut n = 0;
        fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
        for e in fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
            let p = e.map_err(|e| e.to_string())?.path();
            let dest = to.join(p.file_name().unwrap());
            if p.is_dir() {
                n += copy_tree(&p, &dest)?;
            } else {
                fs::copy(&p, &dest).map_err(|e| format!("{}: {e}", p.display()))?;
                n += 1;
            }
        }
        Ok(n)
    }
    // What the daemon reads, and only that.
    let mut files = 0;
    for dir in [
        "web",
        "layers",
        "cases",
        "sources",
        "bundles",
        "matlab/reference",
    ] {
        let from = root.join(dir);
        if from.is_dir() {
            files += copy_tree(&from, &out.join(dir))?;
        }
    }
    fs::create_dir_all(out.join("docs")).map_err(|e| e.to_string())?;
    fs::copy(root.join("docs/manual.toml"), out.join("docs/manual.toml"))
        .map_err(|e| format!("docs/manual.toml: {e}"))?;
    files += 1;
    // The role guides travel with the tool: a user reads docs/roles/user.html
    // without the repository.
    if root.join("docs/roles").is_dir() {
        files += copy_tree(&root.join("docs/roles"), &out.join("docs/roles"))?;
    }
    let tree = load(root)?;
    let mut crates: BTreeSet<&str> = BTreeSet::new();
    for sh in tree.ordered() {
        crates.insert(sh.crate_name.as_str());
    }
    for c in &crates {
        let from = root.join("crates").join(c).join("nodes");
        files += copy_tree(&from, &out.join("crates").join(c).join("nodes"))?;
    }
    // On Windows the daemon ships as `Start VLEO.exe`: the program itself is
    // what a person double-clicks, and it opens the browser because of its
    // name. No script starts it — a script launching a program is one more
    // thing an antivirus heuristic weighs against an unknown file.
    let windows = programs
        .iter()
        .any(|p| p.extension().is_some_and(|e| e == "exe"));
    for p in &programs {
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        let as_named = if name == "vleo-daemon.exe" {
            "Start VLEO.exe".to_string()
        } else {
            name
        };
        fs::copy(p, out.join(as_named)).map_err(|e| e.to_string())?;
    }
    let guide = fs::read_to_string(root.join("docs/TEAM_GUIDE.md"))
        .map_err(|e| format!("docs/TEAM_GUIDE.md: {e}"))?;
    fs::write(out.join("START_HERE.md"), guide).map_err(|e| e.to_string())?;
    let commit = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    fs::write(
        out.join("VERSION"),
        format!(
            "vleo {version}\ncommit {commit}\n{} rows\n",
            tree.sheets.len()
        ),
    )
    .map_err(|e| e.to_string())?;
    if !files_only && !windows {
        fs::write(
            out.join("start.sh"),
            "#!/bin/sh\n# Start the tool and open it in the browser. Stop it with Ctrl-C.\n\
             cd \"$(dirname \"$0\")\" || exit 1\nexec ./vleo-daemon --open\n",
        )
        .map_err(|e| e.to_string())?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for f in ["start.sh", "vleo-daemon", "vleo"] {
            let p = out.join(f);
            if p.is_file() {
                let _ = fs::set_permissions(&p, fs::Permissions::from_mode(0o755));
            }
        }
    }
    let start = if files_only {
        "installs the Python package that carries it"
    } else if windows {
        "double-clicks Start VLEO.exe"
    } else {
        "runs start.sh"
    };
    println!(
        "kit: {} — vleo {version}, {} rows, {files} files beside {} program(s).\n\
         Zip that folder and share it. A team member unzips it, {start} \
         and reads START_HERE.md; nothing in it is edited, \
         and their case and results stay under ~/.vleo/ when the next kit replaces it.",
        out.display(),
        tree.sheets.len(),
        programs.len()
    );
    Ok(())
}

pub(crate) fn cmd_release(root: &Path, args: &[&str]) -> Result<(), String> {
    use vleo_sheet::derisk::{release_key, stamp, NEXT};
    let v = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .copied()
        .ok_or("usage: xtask release <version> [--check] — e.g. `release 0.2.0`")?;
    let key = release_key(v)
        .filter(|_| v != NEXT)
        .ok_or_else(|| format!("«{v}» is not a version: three numbers, like 0.2.0"))?;
    let now = workspace_version(root)?;
    let now_key =
        release_key(&now).ok_or_else(|| format!("Cargo.toml's version «{now}» is not x.y.z"))?;
    if key < now_key {
        return Err(format!(
            "{v} is older than the workspace's {now}; a release only moves forward"
        ));
    }
    let tree = load(root)?;
    let mut newer = Vec::new();
    let mut pending = Vec::new();
    for sh in tree.ordered() {
        for ver in &sh.versions {
            if ver.release == NEXT {
                pending.push(format!("{} v{}", sh.id, ver.n));
            } else if release_key(&ver.release) > Some(key) {
                newer.push(format!("{} v{} says {}", sh.id, ver.n, ver.release));
            }
        }
    }
    if !newer.is_empty() {
        return Err(format!(
            "these versions already name a later release than {v}: {}",
            newer.join(", ")
        ));
    }
    if args.contains(&"--check") {
        if v != now {
            return Err(format!("the workspace says {now}, and the release is {v}. Run `cargo xtask release {v}` and commit"));
        }
        if !pending.is_empty() {
            return Err(format!(
                "{} version(s) are still `next`, so this release would ship beliefs it does not name: {}. \
                 Run `cargo xtask release {v}` and commit",
                pending.len(),
                pending.join(", ")
            ));
        }
        println!("release {v}: every node version is stamped, and the workspace says {v}");
        return Ok(());
    }
    let mut stamped = 0;
    let mut nodes = 0;
    for sh in tree.ordered() {
        if !sh.versions.iter().any(|x| x.release == NEXT) {
            continue;
        }
        let path = sh.dir.join("node.toml");
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let (out, n) = stamp(&text, v);
        if n > 0 {
            fs::write(&path, out).map_err(|e| format!("{}: {e}", path.display()))?;
            stamped += n;
            nodes += 1;
        }
    }
    if v != now {
        let path = root.join("Cargo.toml");
        let text = fs::read_to_string(&path).map_err(|e| format!("Cargo.toml: {e}"))?;
        let mut in_pkg = false;
        let mut done = false;
        let mut out = String::with_capacity(text.len());
        for line in text.split_inclusive('\n') {
            let l = line.trim();
            if l.starts_with('[') {
                in_pkg = l == "[workspace.package]";
            }
            if in_pkg && !done && l.starts_with("version") && l.contains('=') {
                out.push_str(&format!("version = \"{v}\"\n"));
                done = true;
            } else {
                out.push_str(line);
            }
        }
        fs::write(&path, out).map_err(|e| format!("Cargo.toml: {e}"))?;
        // The lock file names the workspace's own version; cargo rewrites it,
        // offline, the moment anything asks it about the workspace.
        let st = std::process::Command::new("cargo")
            .args(["metadata", "--offline", "--format-version", "1"])
            .current_dir(root)
            .stdout(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("cargo metadata: {e}"))?;
        if !st.success() {
            return Err("cargo could not update Cargo.lock for the new version".into());
        }
        // THE CRATES OUTSIDE THE WORKSPACE KEEP LOCK FILES OF THEIR OWN, and
        // each names the workspace crates it depends on at their version. Left
        // alone they still said 0.1.0 after release 0.1.1, and the pipeline's
        // `--locked` build of the wasm face refused. Every one is refreshed the
        // same way, offline.
        let crates = fs::read_dir(root.join("crates")).map_err(|e| format!("crates/: {e}"))?;
        for dir in crates.filter_map(|e| e.ok()).map(|e| e.path()) {
            if !dir.join("Cargo.lock").is_file() {
                continue;
            }
            let st = std::process::Command::new("cargo")
                .args(["metadata", "--offline", "--format-version", "1"])
                .current_dir(&dir)
                .stdout(std::process::Stdio::null())
                .status()
                .map_err(|e| format!("cargo metadata: {e}"))?;
            if !st.success() {
                return Err(format!(
                    "cargo could not update {}/Cargo.lock for the new version",
                    dir.display()
                ));
            }
        }
    }
    cmd_docs(root, &[])?;
    cmd_derisk(root, &[])?;
    println!(
        "release {v}: {stamped} version(s) stamped on {nodes} node(s); workspace {now} -> {v}.\n\
         Next: `cargo run -p xtask -- gate && cargo test`, commit, then tag v{v}."
    );
    Ok(())
}
