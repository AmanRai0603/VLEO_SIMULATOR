//! A new row started from a sibling, and a row's lesson form, made, checked
//! and applied.

use super::*;

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
