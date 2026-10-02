//! A group's sealed release, taken into the design.
//!
//! A group seals its release in the group application (`web/group.html`). The
//! developer writes it out as its folder (`node tools/group_db.mjs --unpack
//! <file.vleo>`) and runs this. Nothing about a node is taken on trust:
//!
//! - the seal is checked first — every file's SHA-256, recomputed here, must
//!   give the fingerprint the release was sealed with, so a file changed after
//!   the people signed is refused before anything is read from it;
//! - each computed node becomes the node form its author would have filled —
//!   its pseudocode as the method, its isolation results, brought back to SI,
//!   as its test cases, its author and their declaration of any assistant's
//!   help — and goes through the same plan, check and apply as any form
//!   (`vleo_sheet::template`). So a change the repository made since is a
//!   conflict, named; a method or results an assistant supplied are refused,
//!   and a method is stamped with its author's name, refused under an
//!   assistant's; and an apply is regenerated and gated as one edit or put
//!   back whole.
//!
//! What the release holds beyond the method and its cases — its words, its
//! pictures, its evidence — stays in the release, which is what the group
//! signed; this command reports them, it does not move them into the sheets.

use super::*;
use vleo_sheet::template::{self, Derisk, Form};

pub(super) fn cmd_group_intake(root: &Path, args: &[&str]) -> Result<(), String> {
    let usage = "usage: group-intake <unpacked release folder> [--node <id>] [--apply [--partial]] [--draft]";
    let dir = PathBuf::from(*args.iter().find(|a| !a.starts_with("--")).ok_or(usage)?);
    let apply = args.contains(&"--apply");
    let partial = args.contains(&"--partial");
    let draft = args.contains(&"--draft");
    let only = args
        .iter()
        .position(|a| *a == "--node")
        .and_then(|i| args.get(i + 1))
        .copied();

    // 1 · what the release says of itself, and whether its files are the ones sealed
    let rel: toml::Value = fs::read_to_string(dir.join("RELEASE.toml"))
        .map_err(|_| {
            format!(
                "{} has no RELEASE.toml: it is not an unpacked release. Write the release out first: \
                 node tools/group_db.mjs --unpack <file.vleo> --out {}",
                dir.display(),
                dir.display()
            )
        })?
        .parse()
        .map_err(|e| format!("RELEASE.toml: {e}"))?;
    let meta = |k: &str| {
        rel.get(k)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let (group, version, sealed) = (meta("group_id"), meta("version"), meta("sealed"));
    println!(
        "\x1b[1m{group} {version}\x1b[0m — {}",
        if sealed.is_empty() {
            "NOT sealed".to_string()
        } else {
            format!(
                "sealed {} by {}",
                &sealed[..sealed.len().min(10)],
                meta("sealed_by")
            )
        }
    );
    if sealed.is_empty() {
        if apply {
            return Err(
                "an unsealed release is looked at, never applied: the group seals it first".into(),
            );
        }
        if !draft {
            return Err(
                "this release is not sealed. Look at it with --draft; it is taken only once sealed"
                    .into(),
            );
        }
    } else {
        let have = fingerprint(&dir)?;
        if have != meta("fingerprint") {
            return Err(format!(
                "the files are not the ones that were sealed: they give the fingerprint {have}, the seal says {}. \
                 Nothing is taken from a release changed after it was signed",
                meta("fingerprint")
            ));
        }
        println!(
            "  every file is the one sealed — fingerprint {}…",
            &have[..16]
        );
    }

    // 2 · each node, as the form its author would have filled
    let tree = load(root)?;
    let nodes = read_csv(&dir.join("nodes.csv"))?;
    let derisk = latest_version(&dir)?;
    let (mut planned, mut applied, mut refused, mut takeable) = (0, 0, 0, 0);
    for n in &nodes {
        let id = n.get("id").cloned().unwrap_or_default();
        if only.is_some_and(|o| o != id) {
            continue;
        }
        let Some(sh) = tree.sheets.get(&id) else {
            println!("\n\x1b[1m{id}\x1b[0m — not in the design: a new node arrives by its own form (`xtask form --new`)");
            continue;
        };
        if n.get("kind").map(String::as_str) != Some("computed") {
            continue;
        }
        let form = node_form(&dir, sh, &group, &version, &sealed, &derisk)?;
        println!(
            "\n\x1b[1m{id}\x1b[0m — by {}; assistant: {}",
            if form.name.is_empty() {
                "(nobody named)"
            } else {
                &form.name
            },
            form.ai
        );
        let p = template::plan_form(root, form)?;
        forms::print_plan(&p);
        planned += 1;
        if p.applicable() > 0 {
            takeable += 1;
        }
        if p.blocked() > 0 {
            refused += 1;
        }
        if apply && p.applicable() > 0 && (p.blocked() == 0 || partial) {
            match template::apply(root, &p) {
                vleo_sheet::form::Saved::Ok { regenerated, .. } => {
                    applied += 1;
                    println!("  applied — node.toml written, {regenerated} artefact(s) generated, the tree gated");
                }
                vleo_sheet::form::Saved::Stale { .. } => {
                    return Err(format!(
                        "{id}: node.toml changed while this ran — run it again"
                    ))
                }
                vleo_sheet::form::Saved::Refused(e) => {
                    return Err(format!("{id}: refused and put back — {e}"))
                }
            }
        }
    }
    println!(
        "\n{planned} computed node(s) read; {refused} with something that cannot be taken{}",
        if apply {
            format!("; {applied} applied")
        } else {
            String::new()
        }
    );
    if !apply && !draft && takeable > 0 {
        println!(
            "apply with: cargo run -p xtask -- group-intake {} --apply   (then `xtask build-node <node>` for each)",
            dir.display()
        );
    }
    Ok(())
}

/// One node of the release as a node form: its sheet as it is now on one side,
/// what the release holds on the other.
fn node_form(
    dir: &Path,
    sh: &vleo_sheet::model::Sheet,
    group: &str,
    version: &str,
    sealed: &str,
    derisk: &Derisk,
) -> Result<Form, String> {
    let nd = dir.join("nodes").join(&sh.id);
    let original = template::content(sh);
    let mut filled = original.clone();
    if let Ok(pc) = fs::read_to_string(nd.join("pseudocode.txt")) {
        filled
            .fields
            .insert("method_text".into(), pc.trim_end().to_string());
    }
    if let Ok(rows) = cases(&nd.join("results/isolation.csv")) {
        filled.arrays.insert("case".into(), rows);
    }
    // Whose it is, and whether an assistant helped. Silence is not "none".
    let decl = read_csv(&nd.join("declaration.csv"))
        .ok()
        .and_then(|r| r.into_iter().next())
        .unwrap_or_default();
    let ai = match decl.get("ai").map(String::as_str) {
        Some(a @ ("none" | "wording" | "relation")) => a.to_string(),
        _ => "relation".to_string(),
    };
    let base = fs::read_to_string(sh.dir.join("node.toml"))
        .map(|t| vleo_sheet::form::file_hash(&t))
        .unwrap_or_default();
    Ok(Form {
        node: sh.id.clone(),
        new: None,
        base,
        name: decl.get("author").cloned().unwrap_or_default(),
        team: group.to_string(),
        date: sealed.chars().take(10).collect(),
        ai,
        notes: format!("from the group release {group} {version}"),
        original,
        filled,
        known: Vec::new(),
        derisk: derisk.clone(),
    })
}

/// The isolation results as the form's test cases: every value back in SI,
/// as the sheet holds it.
fn cases(path: &Path) -> Result<Vec<BTreeMap<String, String>>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let (head, rows) = parse_csv(&text);
    let unit = |h: &str| -> Result<(String, f64), String> {
        match h.split_once('[') {
            Some((name, u)) => {
                let u = u.trim_end_matches(']').trim();
                let f = vleo_sheet::method::parse_unit(u)
                    .map_err(|e| format!("{}: {h}: {e}", path.display()))?
                    .0;
                Ok((name.trim().to_string(), f))
            }
            None => Ok((h.trim().to_string(), 1.0)),
        }
    };
    let cols: Vec<(String, f64)> = head.iter().map(|h| unit(h)).collect::<Result<_, _>>()?;
    let other = [
        "answer",
        "tolerance",
        "refuses",
        "origin",
        "says",
        "note",
        "label",
    ];
    let si = |v: &str, f: f64| -> Option<String> {
        let x: f64 = match v.trim().to_lowercase().as_str() {
            "nan" => f64::NAN,
            "inf" | "infinity" | "+inf" => f64::INFINITY,
            "-inf" | "-infinity" => f64::NEG_INFINITY,
            t => t.parse().ok()?,
        };
        let x = x * f;
        Some(if x.is_nan() {
            "nan".into()
        } else if x.is_infinite() {
            if x > 0.0 {
                "inf".into()
            } else {
                "-inf".into()
            }
        } else {
            format!("{x:?}")
        })
    };
    let mut out = Vec::new();
    for (k, r) in rows.iter().enumerate() {
        let get = |name: &str| {
            cols.iter()
                .position(|(c, _)| c == name)
                .and_then(|i| r.get(i))
                .cloned()
                .unwrap_or_default()
        };
        let refuse = get("refuses") == "yes";
        let inputs: Vec<String> = cols
            .iter()
            .enumerate()
            .filter(|(_, (c, _))| !other.contains(&c.as_str()))
            .filter_map(|(i, (c, f))| si(r.get(i)?, *f).map(|v| format!("{c} = {v}")))
            .collect();
        let answer = cols.iter().position(|(c, _)| c == "answer");
        let mut row = BTreeMap::new();
        let label = get("says");
        row.insert(
            "label".to_string(),
            if label.is_empty() {
                format!("case {}", k + 1)
            } else {
                label
            },
        );
        row.insert("refuse".into(), if refuse { "yes" } else { "no" }.into());
        if !refuse {
            if let Some(i) = answer {
                row.insert(
                    "expect".into(),
                    r.get(i).and_then(|v| si(v, cols[i].1)).unwrap_or_default(),
                );
            }
            row.insert("tolerance".into(), get("tolerance"));
        }
        row.insert("inputs".into(), format!("{{ {} }}", inputs.join(", ")));
        out.push(row);
    }
    Ok(out)
}

/// The group's latest version record, as the form's de-risking record.
fn latest_version(dir: &Path) -> Result<Derisk, String> {
    let rows = read_csv(&dir.join("versions.csv")).unwrap_or_default();
    let v = rows.last().cloned().unwrap_or_default();
    let g = |k: &str| v.get(k).cloned().unwrap_or_default();
    Ok(Derisk {
        believed: g("believed"),
        tested: g("tested"),
        learned: g("learned"),
        changed: g("changed"),
        risks: g("risks"),
        cost: g("cost"),
        rests_on: g("rests_on"),
        breaks_if: g("breaks_if"),
    })
}

/// The fingerprint the group application seals with (web/js/gseal.js): every
/// file but the folder's record of itself, by path, each by its SHA-256.
pub(super) fn fingerprint(dir: &Path) -> Result<String, String> {
    fn walk(base: &Path, d: &Path, out: &mut Vec<String>) -> Result<(), String> {
        for e in fs::read_dir(d).map_err(|e| format!("{}: {e}", d.display()))? {
            let p = e.map_err(|e| e.to_string())?.path();
            if p.is_dir() {
                walk(base, &p, out)?;
            } else {
                let rel = p.strip_prefix(base).map_err(|e| e.to_string())?;
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
        Ok(())
    }
    let mut paths = Vec::new();
    walk(dir, dir, &mut paths)?;
    paths.retain(|p| {
        p != "reviews.csv"
            && p != "RELEASE.toml"
            && !p.starts_with("packages/")
            && !p.starts_with("issues/")
    });
    paths.sort();
    let mut lines = Vec::new();
    for p in &paths {
        let bytes = fs::read(dir.join(p)).map_err(|e| format!("{p}: {e}"))?;
        lines.push(format!("{p}\u{0}{}", group::sha256_hex(&bytes)));
    }
    Ok(group::sha256_hex(lines.join("\n").as_bytes()))
}

/// A CSV file as records by header.
fn read_csv(path: &Path) -> Result<Vec<BTreeMap<String, String>>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let (head, rows) = parse_csv(&text);
    Ok(rows
        .into_iter()
        .map(|r| {
            head.iter()
                .cloned()
                .zip(r.into_iter().chain(std::iter::repeat(String::new())))
                .collect()
        })
        .collect())
}

/// CSV as the group pattern writes it: a header row, then rows; a field in
/// double quotes may hold commas, line breaks and doubled quotes.
fn parse_csv(text: &str) -> (Vec<String>, Vec<Vec<String>>) {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let (mut row, mut cell, mut quoted) = (Vec::new(), String::new(), false);
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match (quoted, c) {
            (true, '"') if chars.peek() == Some(&'"') => {
                cell.push('"');
                chars.next();
            }
            (true, '"') => quoted = false,
            (true, c) => cell.push(c),
            (false, '"') => quoted = true,
            (false, ',') => row.push(std::mem::take(&mut cell)),
            (false, '\r') => {}
            (false, '\n') => {
                row.push(std::mem::take(&mut cell));
                rows.push(std::mem::take(&mut row));
            }
            (false, c) => cell.push(c),
        }
    }
    if !cell.is_empty() || !row.is_empty() {
        row.push(cell);
        rows.push(row);
    }
    rows.retain(|r| r.iter().any(|c| !c.trim().is_empty()));
    let head = if rows.is_empty() {
        Vec::new()
    } else {
        rows.remove(0)
            .into_iter()
            .map(|h| h.trim().to_string())
            .collect()
    };
    (head, rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_reads_quotes_commas_and_line_breaks() {
        let (h, r) = parse_csv("a,b\n\"x, \"\"y\"\"\",\"two\nlines\"\n1,2\n");
        assert_eq!(h, ["a", "b"]);
        assert_eq!(r[0], ["x, \"y\"", "two\nlines"]);
        assert_eq!(r[1], ["1", "2"]);
    }

    #[test]
    fn the_seal_fingerprint_is_the_one_the_page_makes() {
        // Two files and the folder's record of itself; the expected value was
        // worked out with Python's hashlib, not with this code.
        let d = std::env::temp_dir().join(format!("vleo-gi-fp-{}", std::process::id()));
        fs::create_dir_all(d.join("nodes/a")).unwrap();
        fs::write(d.join("group.csv"), "id,name\nx,X\n").unwrap();
        fs::write(d.join("nodes/a/pseudocode.txt"), "return 1\n").unwrap();
        fs::write(d.join("reviews.csv"), "name,scope\nsomebody,group\n").unwrap();
        fs::write(d.join("RELEASE.toml"), "sealed = \"x\"\n").unwrap();
        assert_eq!(
            fingerprint(&d).unwrap(),
            "e7eda2b52f58711281855d3e2815334551728a2db3b862a724b7ba6f4dc93575"
        );
        // One byte changed after the seal, and it is another fingerprint.
        fs::write(d.join("nodes/a/pseudocode.txt"), "return 2\n").unwrap();
        assert_ne!(
            fingerprint(&d).unwrap(),
            "e7eda2b52f58711281855d3e2815334551728a2db3b862a724b7ba6f4dc93575"
        );
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn isolation_results_become_cases_in_si() {
        let d = std::env::temp_dir().join(format!("vleo-gi-{}", std::process::id()));
        fs::create_dir_all(&d).unwrap();
        let f = d.join("isolation.csv");
        fs::write(
            &f,
            "lead [d],answer [d],tolerance,refuses,origin,says\n2,1,1e-9,no,hand,two days\nNaN,,,yes,hand,\n",
        )
        .unwrap();
        let c = cases(&f).unwrap();
        assert_eq!(c[0]["inputs"], "{ lead = 172800.0 }");
        assert_eq!(c[0]["expect"], "86400.0");
        assert_eq!(c[0]["label"], "two days");
        assert_eq!(c[1]["refuse"], "yes");
        assert_eq!(c[1]["inputs"], "{ lead = nan }");
        assert!(!c[1].contains_key("expect"));
        fs::remove_dir_all(&d).ok();
    }
}
