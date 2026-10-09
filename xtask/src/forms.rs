//! A row's lesson form, made and checked.

use super::*;

/// A row's lesson: its form made, or a filled one checked. See
/// `vleo_sheet::lesson_form` and docs/LESSONS.md.
pub(super) fn cmd_lesson(root: &Path, args: &[&str]) -> Result<(), String> {
    const USAGE: &str = "usage: cargo xtask lesson form <node> [--out <file.html>] | \
                         lesson check <file> [--for <node>]";
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
            let tree = read(root)?;
            if !tree.sheets.contains_key(target) {
                return Err(format!("no row '{target}'"));
            }
            println!("would write {target}'s lesson form; nothing written");
            return Ok(());
        }
        (s, _) => s,
    };
    match sub {
        "form" => {
            let tree = read(root)?;
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
                         browser, save a filled copy and send that back; check it with \
                         `cargo run -p xtask -- lesson check <file>`."
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
                Ok(())
            } else {
                Err(format!(
                    "{} reason(s) the lesson would be refused, listed above",
                    problems.len()
                ))
            }
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
    let tree = read(root)?;
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
