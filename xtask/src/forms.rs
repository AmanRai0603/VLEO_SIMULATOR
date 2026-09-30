//! Taking a node form in: the form itself, the checker, building a new node
//! in its place, publishing it and putting a person's name against a relation.

use super::*;

/// One node's form, to fill anywhere and send back. See `vleo_sheet::template`.
pub(crate) fn cmd_form(root: &Path, args: &[&str]) -> Result<(), String> {
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
pub(crate) fn cmd_intake(root: &Path, args: &[&str]) -> Result<(), String> {
    use vleo_sheet::template;
    let file = args
        .first()
        .filter(|a| !a.starts_with("--"))
        .ok_or("usage: cargo xtask intake <file.html> [--apply [--partial]]")?;
    let html = fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
    let p = template::plan(root, &html)?;
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
    if !args.contains(&"--apply") {
        if p.applicable() > 0 {
            println!("\napply with: cargo run -p xtask -- intake {file} --apply");
        }
        return Ok(());
    }
    if p.blocked() > 0 && !args.contains(&"--partial") {
        return Err(format!(
            "{} change(s) cannot be applied (listed above). Resolve them, or apply the rest \
             with --apply --partial — nothing was written",
            p.blocked()
        ));
    }
    let named = if f.name.is_empty() {
        "the filler".to_string()
    } else {
        f.name.clone()
    };
    match &p.new {
        Some(n) => {
            if p.items
                .iter()
                .any(|i| i.what.starts_with("new ·") && i.verdict != template::Verdict::Apply)
            {
                return Err(
                    "a new node needs a free id, a group that exists and a kind \
                            the tree holds before it can be built — nothing was written"
                        .into(),
                );
            }
            let (dir, like) = build_new_node(root, n)?;
            let p2 = match template::plan_onto(root, f, &n.id, &like) {
                Ok(p2) => p2,
                Err(e) => {
                    unbuild_new_node(root, &dir)?;
                    return Err(e);
                }
            };
            println!(
                "\nbuilt {} on the shape of {like}; applying the form to it:",
                n.id
            );
            // WHAT THE FORM DOES NOT ASK, the model row decides — and says so.
            // These are the developer's: how many reviewers the node needs,
            // its tier, what reference data it reads.
            let made = fs::read_to_string(dir.join("node.toml")).unwrap_or_default();
            let mut p2 = p2;
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
            match template::apply(root, &p2) {
                vleo_sheet::form::Saved::Ok { regenerated, .. } => {
                    cmd_codeowners(root)?;
                    println!(
                        "\nadded: {} under {} — node.toml written, {regenerated} artefact(s) \
                         generated, the whole tree gated, CODEOWNERS regenerated. It is seeded: \
                         `cargo run -p xtask -- declare {}` says what is still open, and \
                         `cargo run -p xtask -- publish {}` generates its code once it is \
                         complete. Review with `git status` and `git diff`, and name {named} \
                         in the commit.",
                        n.id, n.parent, n.id, n.id
                    );
                    Ok(())
                }
                vleo_sheet::form::Saved::Stale { .. } => {
                    unbuild_new_node(root, &dir)?;
                    Err("the new node changed while it was being built — nothing was kept".into())
                }
                vleo_sheet::form::Saved::Refused(e) => {
                    unbuild_new_node(root, &dir)?;
                    Err(format!(
                        "{e} — the new node was removed again; nothing was kept"
                    ))
                }
            }
        }
        None => match template::apply(root, &p) {
            vleo_sheet::form::Saved::Ok { regenerated, .. } => {
                println!(
                    "\napplied: node.toml written, {regenerated} artefact(s) regenerated, the gate \
                     passed. Review with `git diff`, and name {named} in the commit — the form is \
                     theirs."
                );
                Ok(())
            }
            vleo_sheet::form::Saved::Stale { .. } => Err(
                "node.toml changed while the form was being checked. Run intake again — nothing \
                 was written"
                    .into(),
            ),
            vleo_sheet::form::Saved::Refused(e) => Err(e),
        },
    }
}

/// The checker's report: every change with its verdict, every interface, what
/// is still open, the filler's note and the known values.
pub(crate) fn print_plan(p: &vleo_sheet::template::Plan) {
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
pub(crate) fn build_new_node(
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
        set_order(&sh.dir.join("node.toml"), sh.order + 1)?;
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
pub(crate) fn unbuild_new_node(root: &Path, dir: &Path) -> Result<(), String> {
    let order = fs::read_to_string(dir.join("node.toml"))
        .ok()
        .and_then(|t| t.parse::<toml_edit::DocumentMut>().ok())
        .and_then(|d| d.get("order").and_then(|v| v.as_integer()))
        .and_then(|v| u32::try_from(v).ok());
    fs::remove_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    if let Some(at) = order {
        let tree = load(root)?;
        for sh in tree.ordered() {
            if sh.order <= at {
                continue;
            }
            set_order(&sh.dir.join("node.toml"), sh.order - 1)?;
        }
    }
    Ok(())
}

/// Move a seeded row to published, from a terminal.
pub(crate) fn cmd_publish(root: &Path, args: &[&str]) -> Result<(), String> {
    let id = args.first().ok_or("usage: cargo xtask publish <node>")?;
    let tree = load(root)?;
    let sh = tree
        .sheets
        .get(*id)
        .ok_or_else(|| format!("no node '{id}'"))?;
    let base = fs::read_to_string(sh.dir.join("node.toml"))
        .map(|t| vleo_sheet::form::file_hash(&t))
        .map_err(|e| e.to_string())?;
    match vleo_sheet::form::publish(root, id, &base) {
        vleo_sheet::form::Saved::Ok { regenerated, .. } => {
            println!(
                "published {id}: {regenerated} artefact(s) generated, the whole tree gated. Its \
                 holes are next — `cargo run -p xtask -- fill {id} --hole <n> --body <file>`."
            );
            Ok(())
        }
        vleo_sheet::form::Saved::Stale { .. } => {
            Err("the sheet changed while publishing — run it again".into())
        }
        vleo_sheet::form::Saved::Refused(e) => Err(e),
    }
}

/// The relations with nobody's name against them, and who owes each one.
///
/// Two thirds of the tree is declared values and every one of those already
/// carries a confirmation. What is missing is the other third: the relations,
/// where the name matters most, because a declared value that nobody confirmed
/// is a number somebody has to defend and a relation that nobody confirmed
/// might not have come from a person at all.
pub(crate) fn cmd_confirm(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = load(root)?;

    if args.first() == Some(&"--list") {
        let only = args.get(1).copied();
        let mut by_owner: BTreeMap<&str, Vec<&vleo_sheet::model::Sheet>> = BTreeMap::new();
        for sh in tree.ordered() {
            if sh.state != "published" || sh.is_declared() {
                continue;
            }
            if !sh.relation_by.trim().is_empty() {
                continue;
            }
            if only.is_some_and(|o| sh.subsystem != o) {
                continue;
            }
            by_owner.entry(sh.owner.as_str()).or_default().push(sh);
        }
        let total: usize = by_owner.values().map(Vec::len).sum();
        if total == 0 {
            println!("every written relation has a name against it");
            return Ok(());
        }
        for (owner, rows) in &by_owner {
            println!("\n\x1b[1m{owner}\x1b[0m — {} relation(s)", rows.len());
            for sh in rows {
                println!("  {:<32} {}", sh.id, sh.expression);
                println!("  {:<32} source: {}", "", sh.source);
            }
        }
        println!(
            "\n{total} relation(s) with nobody's name against them, across {} owner(s).",
            by_owner.len()
        );
        println!(
            "Each needs: cargo xtask confirm <node> --by \"<your name>\"\n\
             A name here is a person saying the relation is right and is theirs. It is not\n\
             a formatting step, and nothing else in the tool can supply it."
        );
        return Ok(());
    }

    let id = args
        .first()
        .ok_or("usage: cargo xtask confirm <node> --by \"<name>\"  |  confirm --list")?;
    let who = args
        .iter()
        .position(|a| *a == "--by")
        .and_then(|i| args.get(i + 1))
        .ok_or("who is confirming it: --by \"<name>\"")?
        .trim();

    let sh = tree
        .sheets
        .get(*id)
        .ok_or_else(|| format!("no node '{id}'"))?;
    if sh.is_declared() {
        return Err(format!(
            "'{id}' is a declared value. Its confirmation belongs under [value], not [maths], \
             and it already has one: {}",
            sh.confirmed_by
        ));
    }
    if sh.expression.trim().is_empty() {
        return Err(format!("'{id}' has no relation to confirm"));
    }
    if who.is_empty() {
        return Err("--by is empty".into());
    }

    // An assistant may never supply mathematics. Stated as a sentence it is a hope;
    // this makes it a fact about what can be written to the file.
    let lower = who.to_lowercase();
    for bad in vleo_sheet::form::agent_identities(root) {
        if lower == bad
            || lower.starts_with(&format!("{bad} "))
            || lower.contains(&format!("{bad}/"))
        {
            return Err(format!(
                "refused: '{who}' is an assistant's name. An assistant may never supply mathematics, and this \
                 field is the only thing that can tell whether one did. It takes the name of a \
                 person who has read the relation against its source and is prepared to own it. \
                 Nothing was written."
            ));
        }
    }
    if !sh.relation_by.trim().is_empty() {
        return Err(format!(
            "'{id}' is already confirmed by {}. Changing an attribution is a review decision, \
             not a command — edit the sheet and say why in the commit.",
            sh.relation_by
        ));
    }

    // What is being confirmed, in front of the person at the moment they
    // confirm it. A name put against a relation nobody re-read is a keystroke,
    // not a confirmation.
    println!("\n\x1b[1m{}\x1b[0m — {}", sh.id, sh.label);
    println!("  asks       {}", sh.question);
    println!("  relation   {}", sh.expression);
    println!("  source     {}", sh.source);
    if !sh.assumptions.is_empty() {
        for a in &sh.assumptions {
            println!("  assumes    {} — fails when {}", a.text, a.fails_when);
        }
    }
    println!(
        "  answers    {} in {} over {} … {}",
        sh.symbol, sh.unit, sh.lower, sh.upper
    );

    let stamp = format!("{who} / {}", today());
    let path = sh.dir.join("node.toml");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;

    // Textual insertion rather than a TOML round-trip: these sheets carry more
    // comment than content, and every one of those comments is somebody's
    // reason. A serialiser would silently drop the lot.
    let at = text
        .find("\n[maths]\n")
        .map(|i| i + "\n[maths]\n".len())
        .ok_or_else(|| format!("{}: no [maths] section", path.display()))?;
    let line =
        format!("confirmed_by = {stamp:?}   # a person read this relation against its source\n");
    let out = format!("{}{}{}", &text[..at], line, &text[at..]);
    fs::write(&path, &out).map_err(|e| format!("{}: {e}", path.display()))?;

    // Read it back through the loader, not the string we just wrote. Writing a
    // file and announcing success is how a malformed sheet gets found three
    // nodes later.
    let after = load(root)?;
    let got = after
        .sheets
        .get(*id)
        .map(|s| s.relation_by.clone())
        .unwrap_or_default();
    if got != stamp {
        fs::write(&path, &text).map_err(|e| format!("{}: {e}", path.display()))?;
        return Err(format!(
            "the sheet did not read back as expected (got {got:?}, wanted {stamp:?}). The file \
             has been put back as it was."
        ));
    }
    println!("\n  confirmed by {stamp}");
    println!("Now: cargo xtask gate {id} && cargo xtask ready {id}");
    Ok(())
}

// ---------------------------------------------------------------------------
