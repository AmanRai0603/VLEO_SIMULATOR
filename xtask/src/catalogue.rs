//! `catalogue` and `impact`: what each group publishes to the others, and
//! which groups a change to it reaches (`vleo_sheet::catalogue`).

use super::*;
use vleo_sheet::catalogue::{catalogue, impact, Published, Reached};

/// The catalogue as CSV: one line per published row and reader.
pub(super) fn catalogue_csv(cat: &[Published]) -> String {
    let mut out = String::from(
        "group,node,label,type,unit,kind,state,version,crosses_to,read_by_group,read_by_node\n",
    );
    let q = |s: &str| {
        if s.contains([',', '"', '\n']) {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    };
    for p in cat {
        let readers: Vec<(&str, &str)> = if p.read_by.is_empty() {
            vec![("", "")]
        } else {
            p.read_by
                .iter()
                .map(|(g, n)| (g.as_str(), n.as_str()))
                .collect()
        };
        for (g, n) in readers {
            out.push_str(
                &[
                    q(&p.group),
                    q(&p.node),
                    q(&p.label),
                    q(&p.ty),
                    q(&p.unit),
                    q(&p.kind),
                    q(&p.state),
                    p.version.to_string(),
                    q(&p.crosses_to),
                    q(g),
                    q(n),
                ]
                .join(","),
            );
            out.push('\n');
        }
    }
    out
}

/// The group a command names: the one word that is not a flag — and not the
/// file `--csv` names, which is a path, not a group.
fn group_arg<'a>(args: &[&'a str]) -> Option<&'a str> {
    let csv_at = args.iter().position(|a| *a == "--csv");
    args.iter()
        .enumerate()
        .find(|(i, a)| !a.starts_with("--") && csv_at.map(|c| c + 1) != Some(*i))
        .map(|(_, a)| *a)
}

pub(super) fn cmd_catalogue(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = read(root)?;
    let only = group_arg(args);
    if let Some(g) = only {
        if !tree.groups.contains_key(g) {
            return Err(format!("no group called '{g}'"));
        }
    }
    let cat: Vec<Published> = catalogue(&tree)
        .into_iter()
        .filter(|p| only.is_none_or(|g| p.group == g))
        .collect();
    if let Some(i) = args.iter().position(|a| *a == "--csv") {
        let out = args.get(i + 1).ok_or("--csv needs a file to write")?;
        fs::write(out, catalogue_csv(&cat)).map_err(|e| format!("{out}: {e}"))?;
        println!("wrote {} published row(s) to {out}", cat.len());
        return Ok(());
    }
    let mut group = "";
    for p in &cat {
        if p.group != group {
            group = &p.group;
            let label = tree
                .groups
                .get(group)
                .map(|g| g.label.as_str())
                .unwrap_or("");
            println!("\n\x1b[1m{group}\x1b[0m {label}");
        }
        let unit = if p.unit.is_empty() || p.unit == "-" {
            String::new()
        } else {
            format!(" [{}]", p.unit)
        };
        let cross = if p.crosses_to.is_empty() {
            String::new()
        } else {
            format!(" — crosses to {}", p.crosses_to)
        };
        let v = if p.version == 0 {
            "no version".to_string()
        } else {
            format!("v{}", p.version)
        };
        println!("  {}{unit}  {} · {v}{cross}", p.node, p.state);
        let mut by: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for (g, n) in &p.read_by {
            by.entry(g).or_default().push(n);
        }
        for (g, ns) in by {
            println!("      read by {g}: {}", ns.join(", "));
        }
    }
    let groups: BTreeSet<&str> = cat.iter().map(|p| p.group.as_str()).collect();
    println!(
        "\n{} published row(s) in {} group(s), read {} time(s) across groups",
        cat.len(),
        groups.len(),
        cat.iter().map(|p| p.read_by.len()).sum::<usize>()
    );
    Ok(())
}

/// The rows named, a group named standing for all its rows.
pub(super) fn rows_named<'a>(tree: &'a Tree, names: &[&str]) -> Result<Vec<&'a str>, String> {
    let mut rows = Vec::new();
    for n in names {
        if let Some((k, _)) = tree.sheets.get_key_value(*n) {
            rows.push(k.as_str());
        } else if tree.groups.contains_key(*n) {
            rows.extend(
                tree.ordered()
                    .into_iter()
                    .filter(|s| s.parent == *n)
                    .map(|s| s.id.as_str()),
            );
        } else {
            return Err(format!("'{n}' is neither a row nor a group"));
        }
    }
    Ok(rows)
}

/// What a change to `rows` reaches in other groups, said by group.
pub(super) fn print_impact(reached: &[Reached]) {
    if reached.is_empty() {
        println!("  reaches no other group");
        return;
    }
    let mut by: BTreeMap<&str, Vec<&Reached>> = BTreeMap::new();
    for r in reached {
        by.entry(r.group.as_str()).or_default().push(r);
    }
    for (g, rs) in &by {
        let direct: Vec<&str> = rs
            .iter()
            .filter(|r| r.depth == 1)
            .map(|r| r.node.as_str())
            .collect();
        let further = rs.len() - direct.len();
        println!(
            "  {g}: {}{}",
            if direct.is_empty() {
                "nothing directly".to_string()
            } else {
                format!("reads it directly in {}", direct.join(", "))
            },
            if further > 0 {
                format!("; {further} row(s) further down")
            } else {
                String::new()
            }
        );
    }
    println!("  {} row(s) in {} other group(s)", reached.len(), by.len());
}

pub(super) fn cmd_impact(root: &Path, args: &[&str]) -> Result<(), String> {
    let names: Vec<&str> = args
        .iter()
        .filter(|a| !a.starts_with("--"))
        .copied()
        .collect();
    if names.is_empty() {
        return Err("usage: impact <node|group> ...".into());
    }
    let tree = read(root)?;
    let rows = rows_named(&tree, &names)?;
    println!("a change to {}:", names.join(", "));
    print_impact(&impact(&tree, &rows));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::group_arg;

    #[test]
    fn the_file_csv_names_is_never_taken_for_a_group() {
        assert_eq!(group_arg(&["--csv", "out.csv"]), None);
        assert_eq!(
            group_arg(&["l3_solar", "--csv", "out.csv"]),
            Some("l3_solar")
        );
        assert_eq!(
            group_arg(&["--csv", "out.csv", "l3_solar"]),
            Some("l3_solar")
        );
        assert_eq!(group_arg(&["l3_solar"]), Some("l3_solar"));
        assert_eq!(group_arg(&[]), None);
    }
}
