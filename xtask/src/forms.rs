//! Node forms: made, checked, applied, and new nodes built from them.

use super::*;

/// One node's form, to fill anywhere and send back. See `vleo_sheet::template`.
pub(super) fn cmd_form(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = load(root)?;
    let html = if args.contains(&"--new") {
        vleo_sheet::template::document_new(&tree)
    } else {
        let id = args
            .first()
            .filter(|a| !a.starts_with("--"))
            .ok_or("usage: cargo xtask form <node>|--new [--example] [--out <file.html>]")?;
        let sh = tree.sheets.get(*id).ok_or_else(|| {
            format!("no node '{id}'. For a node the design does not have yet: `form --new`")
        })?;
        if args.contains(&"--example") {
            vleo_sheet::template::document_example(sh, &tree)?
        } else {
            vleo_sheet::template::document(sh, &tree)
        }
    };
    match args
        .iter()
        .position(|a| *a == "--out")
        .and_then(|i| args.get(i + 1))
    {
        Some(out) => {
            fs::write(out, &html).map_err(|e| format!("{out}: {e}"))?;
            eprintln!(
                "wrote {out} — open it in a browser, fill it, save a filled copy, and send that \
                 back. Apply it with `cargo run -p xtask -- intake <file> --apply`."
            );
        }
        None => print!("{html}"),
    }
    Ok(())
}

/// What a filled form would do to its node, and with `--apply`, do it.
pub(super) fn cmd_intake(root: &Path, args: &[&str]) -> Result<(), String> {
    use crate::pipeline::{OnStop, Run};
    use vleo_sheet::template;
    let file = *args
        .first()
        .filter(|a| !a.starts_with("--"))
        .ok_or("usage: cargo xtask intake <file.html> [--apply [--partial]]")?;
    let apply = args.contains(&"--apply");
    // Only an apply writes, so only an apply runs as numbered steps and
    // leaves a trace. The check alone is a report.
    let mut run = apply.then(|| Run::start(root, "intake", args, 3));
    let untouched = || {
        OnStop::new(
            "unchanged — nothing was written",
            format!("cargo run -p xtask -- intake {file}"),
        )
    };
    let read = || -> Result<(template::Plan, String), String> {
        let html = fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
        Ok((template::plan(root, &html)?, file.to_string()))
    };
    let (p, _) = match run.as_mut() {
        Some(r) => r.step("read the form", untouched(), || {
            let (p, f) = read()?;
            let said = format!(
                "{} — filled by {}",
                p.new
                    .as_ref()
                    .map(|n| n.id.clone())
                    .unwrap_or_else(|| p.form.node.clone()),
                if p.form.name.is_empty() {
                    "(nobody named)"
                } else {
                    &p.form.name
                }
            );
            Ok(((p, f), said))
        })?,
        None => read()?,
    };
    if let (Some(r), Some(_)) = (run.as_mut(), p.new.as_ref()) {
        r.set_total(4);
    }
    let f = &p.form;
    let who = format!(
        "{}{}{}; assistant: {}",
        if f.name.is_empty() {
            "(nobody named)"
        } else {
            &f.name
        },
        if f.team.is_empty() {
            String::new()
        } else {
            format!(" ({})", f.team)
        },
        if f.date.is_empty() {
            String::new()
        } else {
            format!(", {}", f.date)
        },
        f.ai
    );
    let report = || {
        match &p.new {
            Some(n) => println!(
                "\x1b[1ma new node\x1b[0m — {} under {}, a {} row — a node form filled by {who}",
                if n.id.is_empty() { "(no id)" } else { &n.id },
                if n.parent.is_empty() {
                    "(no group)"
                } else {
                    &n.parent
                },
                n.kind
            ),
            None => {
                println!("\x1b[1m{}\x1b[0m — a node form filled by {who}", f.node);
                println!(
                    "  {}",
                    if p.base_current {
                        "node.toml is still the version the form was made from"
                    } else {
                        "node.toml has changed since the form was made — a change to the same thing \
                         is a conflict, named below"
                    }
                );
            }
        }
        print_plan(&p);
    };
    let Some(mut run) = run else {
        report();
        if p.applicable() > 0 {
            println!("\napply with: cargo run -p xtask -- intake {file} --apply");
        }
        return Ok(());
    };
    run.step("check every change", untouched(), || {
        report();
        if p.blocked() > 0 && !args.contains(&"--partial") {
            return Err(format!(
                "{} change(s) cannot be applied (listed above). Resolve them, or apply the rest \
                 with --apply --partial",
                p.blocked()
            ));
        }
        Ok((
            (),
            format!("{} to apply, {} blocked", p.applicable(), p.blocked()),
        ))
    })?;
    let named = if f.name.is_empty() {
        "the filler".to_string()
    } else {
        f.name.clone()
    };
    let put_back = || {
        OnStop::new(
            "as they were — a refused apply is put back whole",
            format!("send the lines above back to {named}; or fix and `cargo run -p xtask -- intake {file} --apply`"),
        )
    };
    match &p.new {
        Some(n) => {
            let (dir, p2) = run.step("build the new node", put_back(), || {
                if p.items
                    .iter()
                    .any(|i| i.what.starts_with("new ·") && i.verdict != template::Verdict::Apply)
                {
                    return Err(
                        "a new node needs a free id, a group that exists and a kind the tree \
                         holds before it can be built"
                            .into(),
                    );
                }
                let (dir, like) = build_new_node(root, n)?;
                let mut p2 = match template::plan_onto(root, f, &n.id, &like) {
                    Ok(p2) => p2,
                    Err(e) => {
                        unbuild_new_node(root, &dir)?;
                        return Err(e.into());
                    }
                };
                // WHAT THE FORM DOES NOT ASK, the model row decides — and says
                // so. These are the developer's: how many reviewers the node
                // needs, its tier, what reference data it reads.
                let made = fs::read_to_string(dir.join("node.toml")).unwrap_or_default();
                for key in ["subsystem", "owner", "criticality", "tier", "bundles"] {
                    if let Some(l) = made
                        .lines()
                        .find(|l| l.trim_start().starts_with(&format!("{key} = ")))
                    {
                        p2.open.push(format!(
                            "`{}` is taken from {like} — the form does not ask it; confirm it",
                            l.split('#').next().unwrap_or(l).trim()
                        ));
                    }
                }
                if load(root)?
                    .sheets
                    .get(&like)
                    .is_some_and(|l| !l.kpis.is_empty())
                {
                    p2.open.push(format!(
                        "it contributes to no KPI: {like}'s [contributes] is {like}'s own contract \
                         edge and is not inherited — add one if this row should move a KPI"
                    ));
                }
                print_plan(&p2);
                Ok(((dir, p2), format!("{} built on the shape of {like}", n.id)))
            })?;
            run.step("apply to the sheet", put_back(), || match template::apply(root, &p2) {
                vleo_sheet::form::Saved::Ok { regenerated, .. } => {
                    cmd_codeowners(root)?;
                    Ok((
                        (),
                        format!(
                            "node.toml written, {regenerated} artefact(s) generated, the whole tree \
                             gated, CODEOWNERS regenerated"
                        ),
                    ))
                }
                vleo_sheet::form::Saved::Stale { .. } => {
                    unbuild_new_node(root, &dir)?;
                    Err("the new node changed while it was being built".into())
                }
                vleo_sheet::form::Saved::Refused(e) => {
                    unbuild_new_node(root, &dir)?;
                    Err(format!("{e} — the new node was removed again"))
                }
            })?;
            run.done(&format!(
                "added: {} under {}. It is seeded: `cargo run -p xtask -- declare {}` says what is \
                 still open, and `cargo run -p xtask -- publish {}` generates its code once it is \
                 complete. Review with `git status` and `git diff`, and name {named} in the commit.",
                n.id, n.parent, n.id, n.id
            ));
            Ok(())
        }
        None => {
            run.step("apply to the sheet", put_back(), || match template::apply(root, &p) {
                vleo_sheet::form::Saved::Ok { regenerated, .. } => Ok((
                    (),
                    format!("node.toml written, {regenerated} artefact(s) regenerated, the gate passed"),
                )),
                vleo_sheet::form::Saved::Stale { .. } => Err(
                    "node.toml changed while the form was being checked. Run intake again".into(),
                ),
                vleo_sheet::form::Saved::Refused(e) => Err(e),
            })?;
            run.done(&format!(
                "applied: review with `git diff`, and name {named} in the commit — the form is theirs."
            ));
            Ok(())
        }
    }
}

/// The checker's report: every change with its verdict, every interface, what
/// is still open, the filler's note and the known values.
pub(super) fn print_plan(p: &vleo_sheet::template::Plan) {
    use vleo_sheet::template::Verdict;
    println!();
    if p.items.is_empty() {
        println!("  the form changes nothing in the node");
    }
    let clip = |v: &str| {
        let one = v.split_whitespace().collect::<Vec<_>>().join(" ");
        if one.chars().count() > 70 {
            format!("{}…", one.chars().take(69).collect::<String>())
        } else {
            one
        }
    };
    for i in &p.items {
        let (tag, why) = match &i.verdict {
            Verdict::Apply => ("\x1b[32mAPPLY   \x1b[0m", String::new()),
            Verdict::Already => ("already ", " — the node already says this".to_string()),
            Verdict::Conflict(w) => ("\x1b[33mCONFLICT\x1b[0m", format!(" — {w}")),
            Verdict::Refused(w) => ("\x1b[31mREFUSED \x1b[0m", format!(" — {w}")),
        };
        println!(
            "  {tag} {:<28} «{}» → «{}»{why}",
            i.what,
            clip(&i.from),
            clip(&i.to)
        );
    }
    if !p.interfaces.is_empty() {
        println!("\ninterfaces — what each input reads:");
        for i in &p.interfaces {
            if i.ok() {
                println!(
                    "  \x1b[32mconnects\x1b[0m {:<16} ← {:<34} {} in {}",
                    i.binding, i.var, i.have, i.unit
                );
            } else {
                println!(
                    "  \x1b[31mREFUSED \x1b[0m {:<16} ← {:<34} {}",
                    i.binding, i.var, i.why
                );
            }
        }
    }
    if !p.about.is_empty() {
        let f = &p.form;
        println!(
            "\nwhy it is changing — the decisions it moves: {}",
            p.about.join(", ")
        );
        for (k, ask, _) in vleo_sheet::template::DERISK {
            let v = f.derisk.get(k);
            if !v.trim().is_empty() {
                println!("  {:<40} {}", ask, clip(v));
            }
        }
        match p.version {
            Some(n) => println!("  → recorded as version {n}, released with the next release"),
            None => println!(
                "  → no version recorded: the record is incomplete, so these decisions are withheld"
            ),
        }
    }
    // THE METHOD, AGAINST ITS AUTHOR'S CASES, on the node as it would be after
    // this form — the same check the author saw in the form and the gate runs
    // on apply, so a refusal here is one the author could already see.
    if let Some(text) = &p.text {
        if let Ok(r) = vleo_sheet::method::report_toml(text) {
            if text.contains("\n[method]") || !r.cases.is_empty() {
                println!("\nthe method, against the author's cases:");
                for d in &r.diags {
                    println!("  {d}");
                }
                for (c, v) in &r.cases {
                    println!(
                        "  {} {}: {}",
                        if v.agrees() { "ok  " } else { "FAIL" },
                        c.label,
                        v.text(c)
                    );
                }
                for sft in &r.shortfall {
                    println!("  missing: {sft}");
                }
                println!(
                    "  {}",
                    if r.sound() {
                        "sound — the method checks and agrees with every case"
                    } else {
                        "NOT SOUND — the gate will refuse this on apply; send it back with the lines above"
                    }
                );
            }
        }
    }
    if !p.open.is_empty() {
        println!("\nstill for the developer to settle:");
        for o in &p.open {
            println!("  · {o}");
        }
    }
    println!();
    println!(
        "{} change(s) can be applied, {} cannot.",
        p.applicable(),
        p.blocked()
    );
    let f = &p.form;
    if !f.notes.trim().is_empty() {
        println!(
            "\nfrom the filler:\n  {}",
            f.notes.trim().replace('\n', "\n  ")
        );
    }
    if !f.known.is_empty() {
        println!(
            "\nknown values the form supplies — a request for whoever records fixtures, never \
             applied by this command:\n"
        );
        print!("{}", vleo_sheet::template::fixture_request(f));
    }
}

/// Build a new node's folder on the shape of an existing row of the same kind,
/// placed last under its group. Returns the folder and the row it was built on.
///
/// The shape comes from a row in the same part of the tree, so the new row is
/// in the right crate, subsystem and owner; its own content is then applied by
/// the form. It is SEEDED whatever its model was: a row is published on
/// purpose, with `publish`, once it is complete.
pub(super) fn build_new_node(
    root: &Path,
    n: &vleo_sheet::template::NewNode,
) -> Result<(PathBuf, String), String> {
    let tree = load(root)?;
    // Everything under the group, and the group's own ancestors in order, so a
    // group with no rows of its own still finds its part of the tree.
    let under = |s: &vleo_sheet::model::Sheet, g: &str| {
        let mut p = s.parent.clone();
        let mut hops = 0;
        while !p.is_empty() && hops < 64 {
            if p == g {
                return true;
            }
            p = tree
                .groups
                .get(&p)
                .map(|x| x.parent.clone())
                .unwrap_or_default();
            hops += 1;
        }
        false
    };
    let mut scope = n.parent.clone();
    let mut like = None;
    for _ in 0..64 {
        let rows: Vec<&vleo_sheet::model::Sheet> = tree
            .ordered()
            .into_iter()
            .filter(|s| under(s, &scope) && s.state != "deprecated")
            .collect();
        // The closest shape: a row of the same kind, under the same group, whose
        // id begins as the new one does — `orbit_…` is built on an `orbit_` row,
        // not on whichever row of the group happens to come last, and so takes
        // that row's subsystem and owner rather than a neighbour's.
        let stem = |id: &str| id.split('_').next().unwrap_or("").to_string();
        if let Some(s) = rows
            .iter()
            .filter(|s| s.kind == n.kind)
            .max_by_key(|s| (s.parent == n.parent, stem(&s.id) == stem(&n.id), s.order))
        {
            like = Some(*s);
            break;
        }
        match tree.groups.get(&scope).map(|g| g.parent.clone()) {
            Some(p) if !p.is_empty() => scope = p,
            _ => break,
        }
    }
    let like = like.ok_or_else(|| {
        format!(
            "no {} row anywhere above {} to take the shape from — a node of a kind its part of \
             the tree has never held is a decision about the tree, not a form",
            n.kind, n.parent
        )
    })?;
    // Last under its group: after the highest order among the group's rows, or
    // after the row it is built on.
    let after = tree
        .ordered()
        .into_iter()
        .filter(|s| s.parent == n.parent)
        .map(|s| s.order)
        .max()
        .unwrap_or(like.order);
    let dir = root
        .join("crates")
        .join(&like.crate_name)
        .join("nodes")
        .join(&n.id);
    if dir.exists() {
        return Err(format!("{} already exists", dir.display()));
    }
    let sheet = fs::read_to_string(like.dir.join("node.toml")).map_err(|e| e.to_string())?;
    let (out, _) = clone_sheet(&sheet, &n.id, &n.id, after);
    let mut text = String::new();
    // The clone marks every blanked line "re-decide, do not inherit". Here the
    // form re-decides them, and what it leaves blank intake reports by name —
    // so the marker would only be a stale instruction on an answered line.
    let mark = "   # REQUIRED — re-decide, do not inherit";
    for line in out.lines() {
        let line = line.strip_suffix(mark).unwrap_or(line);
        let l = line.trim_start();
        if l.starts_with("parent = ") {
            text.push_str(&format!("parent = \"{}\"\n", n.parent));
        } else if l.starts_with("state = ") {
            text.push_str("state = \"empty\"\n");
        } else {
            text.push_str(line);
            text.push('\n');
        }
    }
    // Make room, exactly as `new` does: every row after the new one moves up.
    for sh in tree.ordered() {
        if sh.order <= after {
            continue;
        }
        let f = sh.dir.join("node.toml");
        let t = fs::read_to_string(&f).map_err(|e| e.to_string())?;
        let mut w = String::new();
        for line in t.lines() {
            if line.trim_start().starts_with("order = ") {
                w.push_str(&format!("order = {}\n", sh.order + 1));
            } else {
                w.push_str(line);
                w.push('\n');
            }
        }
        fs::write(&f, w).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join("node.toml"), text).map_err(|e| e.to_string())?;
    fs::write(
        dir.join("fixtures.toml"),
        "# Known-good values, and where each came from. An expected value may\n# never be produced by the code under test.\n",
    )
    .map_err(|e| e.to_string())?;
    Ok((dir, like.id.clone()))
}

/// Take a new node back out: its folder, and the room made for it.
pub(super) fn unbuild_new_node(root: &Path, dir: &Path) -> Result<(), String> {
    let order = fs::read_to_string(dir.join("node.toml"))
        .ok()
        .and_then(|t| {
            t.lines().find_map(|l| {
                l.trim_start()
                    .strip_prefix("order = ")
                    .map(|v| v.trim().to_string())
            })
        })
        .and_then(|v| v.parse::<u32>().ok());
    fs::remove_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    if let Some(at) = order {
        let tree = load(root)?;
        for sh in tree.ordered() {
            if sh.order <= at {
                continue;
            }
            let f = sh.dir.join("node.toml");
            let t = fs::read_to_string(&f).map_err(|e| e.to_string())?;
            let mut w = String::new();
            for line in t.lines() {
                if line.trim_start().starts_with("order = ") {
                    w.push_str(&format!("order = {}\n", sh.order - 1));
                } else {
                    w.push_str(line);
                    w.push('\n');
                }
            }
            fs::write(&f, w).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub(super) fn cmd_new(root: &Path, args: &[&str]) -> Result<(), String> {
    let id = args
        .first()
        .ok_or("usage: cargo xtask new <id> --like <sibling>")?;
    let like = args
        .iter()
        .position(|a| *a == "--like")
        .and_then(|i| args.get(i + 1))
        .ok_or("a new node clones the shape of a sibling: --like <sibling id>")?;
    let tree = load(root)?;
    let src = tree
        .sheets
        .get(*like)
        .ok_or_else(|| format!("no node '{like}' to clone the shape of"))?;
    // The folder is the identifier. This used to strip the subsystem prefix,
    // which is how it came to disagree with the seeder: `prop_throat_area`
    // landed in `nodes/throat_area` while the tree already held it under its
    // own name, and two folders then claimed one id. The gate caught it, but
    // the rule only has to exist once for that not to happen at all.
    let folder = id.to_string();
    let dir = root
        .join("crates")
        .join(&src.crate_name)
        .join("nodes")
        .join(&folder);
    if dir.exists() {
        return Err(format!("{} already exists", dir.display()));
    }
    let sheet = fs::read_to_string(src.dir.join("node.toml")).map_err(|e| e.to_string())?;
    let src_order = src.order;
    let (out, carried) = clone_sheet(&sheet, id, &folder, src_order);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join("node.toml"), out).map_err(|e| e.to_string())?;
    if !carried.is_empty() {
        println!(
            "{} comment block line(s) were carried from {like} and may be about that row \
             rather than this one. Read them before `docs`:",
            carried.len()
        );
        for c in carried.iter().take(4) {
            println!("   {}", &c[..c.len().min(96)]);
        }
        if carried.len() > 4 {
            println!("   ... and {} more", carried.len() - 4);
        }
    }
    // Make room. Every sheet already at or beyond the new row's place moves up
    // one, so the tree stays contiguous and the assembly check stays quiet.
    // `order` is outside the sheet hash, so this rewrites no generated artefact.
    let mut shifted = 0usize;
    for sh in tree.ordered() {
        if sh.id.as_str() == *id || sh.order <= src_order {
            continue;
        }
        let f = sh.dir.join("node.toml");
        let t = fs::read_to_string(&f).map_err(|e| e.to_string())?;
        let mut w = String::new();
        for line in t.lines() {
            if line.trim_start().starts_with("order = ") {
                w.push_str(&format!("order = {}\n", sh.order + 1));
            } else {
                w.push_str(line);
                w.push('\n');
            }
        }
        fs::write(&f, w).map_err(|e| e.to_string())?;
        shifted += 1;
    }
    if shifted > 0 {
        println!(
            "made room at {}: {} row(s) moved up one",
            src_order + 1,
            shifted
        );
    }
    fs::write(
        dir.join("fixtures.toml"),
        "# Known-good values, and where each came from. An expected value may\n# never be produced by the code under test.\n",
    )
    .map_err(|e| e.to_string())?;
    println!("new: {}", dir.display());
    println!("`cargo xtask declare {id}` lists what must be answered first.");
    println!("Then `cargo xtask docs {id}`.");
    println!("Generation refuses while any field is open, and names the fields. That refusal is the mechanism: ambiguity becomes a blocking item on an engineer's screen rather than something an implementer resolves silently.");
    println!("When it generates, `cargo xtask ready {id}` says whether a person should be asked to look yet.");
    Ok(())
}

/// A row's lesson: its form made, a filled one checked, or applied beside the
/// row. See `vleo_sheet::lesson_form` and docs/LESSONS.md.
pub(super) fn cmd_lesson(root: &Path, args: &[&str]) -> Result<(), String> {
    use crate::pipeline::{OnStop, Run};
    const USAGE: &str = "usage: cargo xtask lesson form <node> [--out <file.html>] | \
                         lesson check <file> [--for <node>] | lesson apply <file> [--for <node>]";
    let flag = |f: &str| {
        args.iter()
            .position(|a| *a == f)
            .and_then(|i| args.get(i + 1))
            .copied()
    };
    let (sub, target) = match args {
        [s, t, ..] if !t.starts_with("--") => (*s, *t),
        _ => return Err(USAGE.into()),
    };
    // `--check` is what `--dry-run` passes: the check, and nothing written.
    let sub = match (sub, args.contains(&"--check")) {
        ("form", true) => {
            let tree = load(root)?;
            if !tree.sheets.contains_key(target) {
                return Err(format!("no row '{target}'"));
            }
            println!("would write {target}'s lesson form; nothing written");
            return Ok(());
        }
        ("apply", true) => "check",
        (s, _) => s,
    };
    match sub {
        "form" => {
            let tree = load(root)?;
            let sh = tree
                .sheets
                .get(target)
                .ok_or_else(|| format!("no row '{target}'"))?;
            let html = vleo_sheet::lesson_form::document(sh, &tree)?;
            match flag("--out") {
                Some(out) => {
                    fs::write(out, &html).map_err(|e| format!("{out}: {e}"))?;
                    eprintln!(
                        "wrote {out} — send it to whoever knows {target}. They fill it in a \
                         browser, save a filled copy and send that back; place it with \
                         `cargo run -p xtask -- lesson apply <file>`."
                    );
                }
                None => print!("{html}"),
            }
            Ok(())
        }
        "check" => {
            let (node, l, problems) = read_lesson(root, target, flag("--for"))?;
            print_lesson(&node, &l, &problems);
            if problems.is_empty() {
                let with_for = flag("--for")
                    .map(|n| format!(" --for {n}"))
                    .unwrap_or_default();
                println!("\napply with: cargo run -p xtask -- lesson apply {target}{with_for}");
                Ok(())
            } else {
                Err(format!(
                    "{} reason(s) the lesson would be refused, listed above",
                    problems.len()
                ))
            }
        }
        "apply" => {
            let mut run = Run::start(root, "lesson", args, 3);
            let untouched = || {
                OnStop::new(
                    "unchanged — nothing was written",
                    format!("cargo run -p xtask -- lesson check {target}"),
                )
            };
            let (node, l, toml_text) = run.step("check the lesson", untouched(), || {
                let (node, l, problems) = read_lesson(root, target, flag("--for"))?;
                print_lesson(&node, &l, &problems);
                if !problems.is_empty() {
                    return Err(format!(
                        "{} reason(s) the lesson would be refused, listed above — send them back",
                        problems.len()
                    ));
                }
                let text = fs::read_to_string(target).map_err(|e| format!("{target}: {e}"))?;
                let (_, toml_text) = vleo_sheet::lesson_form::from_file(&text, Some(&node))?;
                let said = format!("{node} — written by {}", l.by);
                Ok(((node, l, toml_text), said))
            })?;
            let tree = load(root)?;
            let dir = tree.sheets[&node].dir.clone();
            let path = dir.join(vleo_sheet::lesson::FILE);
            let before = fs::read_to_string(&path).ok();
            let put_back = || {
                OnStop::new(
                    "put back: the row's lesson.toml is as it was",
                    format!("cargo run -p xtask -- lesson apply {target}"),
                )
            };
            run.step("write it beside the row", put_back(), || {
                fs::write(&path, &toml_text).map_err(|e| format!("{}: {e}", path.display()))?;
                Ok((
                    (),
                    format!(
                        "{} {}",
                        path.strip_prefix(root).unwrap_or(&path).display(),
                        if before.is_some() {
                            "replaced"
                        } else {
                            "written"
                        }
                    ),
                ))
            })?;
            let gated = run.step("gate the row", put_back(), || {
                let tree = load(root)?;
                let bad: Vec<String> = vleo_sheet::gate::gate_node(&tree.sheets[&node], &tree)
                    .into_iter()
                    .filter_map(|c| match c.verdict {
                        vleo_sheet::gate::Verdict::Fail(why) if c.name == "lesson" => Some(why),
                        _ => None,
                    })
                    .collect();
                if bad.is_empty() {
                    Ok(((), "check `lesson` passes".to_string()))
                } else {
                    Err(bad.join("; "))
                }
            });
            if let Err(e) = gated {
                match &before {
                    Some(t) => fs::write(&path, t),
                    None => fs::remove_file(&path),
                }
                .map_err(|x| format!("{e}; and putting it back failed: {x}"))?;
                return Err(e);
            }
            run.done(&format!(
                "applied: {node}'s lesson, \"{}\" by {} — commit naming {}, and it ships with the row",
                l.title, l.by, l.by
            ));
            Ok(())
        }
        _ => Err(USAGE.into()),
    }
}

/// A filled lesson form or a bare lesson.toml, read and checked against the
/// tree as the gate checks it.
fn read_lesson(
    root: &Path,
    file: &str,
    node: Option<&str>,
) -> Result<(String, vleo_sheet::lesson::Lesson, Vec<String>), String> {
    let text = fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
    let (node, toml_text) = vleo_sheet::lesson_form::from_file(&text, node)?;
    let tree = load(root)?;
    if !tree.sheets.contains_key(&node) {
        return Err(format!("the lesson is for '{node}', which is not a row"));
    }
    let l = vleo_sheet::lesson::read(&toml_text, &node)?;
    let p = vleo_sheet::lesson::problems(&l, &tree);
    Ok((node, l, p))
}

fn print_lesson(node: &str, l: &vleo_sheet::lesson::Lesson, problems: &[String]) {
    println!(
        "{node}: \"{}\" by {} — {} station(s), {} equation(s), {} widget(s), {} check(s)",
        l.title,
        if l.by.is_empty() {
            "(nobody named)"
        } else {
            &l.by
        },
        l.stations.len(),
        l.equations.len(),
        l.widgets.len(),
        l.checks.len()
    );
    if problems.is_empty() {
        println!("  passes the check the gate runs");
    }
    for p in problems {
        println!("  \x1b[31mrefused\x1b[0m  {p}");
    }
}
