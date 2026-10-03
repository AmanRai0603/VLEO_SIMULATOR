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
    if apply && draft {
        return Err("an unsealed release is looked at, never applied: --draft and --apply do not go together".into());
    }
    let Release {
        group,
        version,
        sealed,
    } = Release::open(&dir, draft)?;

    // 2 · each node, as the form its author would have filled
    let tree = load(root)?;
    let nodes = read_csv(&dir.join("nodes.csv"))?;
    let derisk = latest_version(&dir)?;
    let (mut planned, mut applied, mut refused, mut takeable) = (0, 0, 0, 0);
    let mut changes: Vec<String> = Vec::new();
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
        let form = node_form(root, &dir, sh, &group, &version, &sealed, &derisk)?;
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
            changes.push(id.clone());
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
    // 3 · whom it reaches: every row of another group downstream of a node
    // this release changes, so the developer knows who to tell before merging.
    if !changes.is_empty() {
        println!(
            "\n\x1b[1mwhom it reaches\x1b[0m — {} node(s) this release changes:",
            changes.len()
        );
        let rows: Vec<&str> = changes.iter().map(String::as_str).collect();
        crate::catalogue::print_impact(&vleo_sheet::catalogue::impact(&tree, &rows));
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

/// What a written-out release says of itself, once its seal is checked.
pub(super) struct Release {
    pub group: String,
    pub version: String,
    /// When it was sealed; empty for a draft.
    pub sealed: String,
}

impl Release {
    /// Read RELEASE.toml and check every file against the sealed fingerprint.
    /// An unsealed release opens only as a draft, which is looked at and never
    /// taken; a sealed one whose files changed after signing does not open.
    pub(super) fn open(dir: &Path, draft: bool) -> Result<Release, String> {
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
            if !draft {
                return Err(
                    "this release is not sealed. Look at it with --draft; it is taken only once sealed"
                        .into(),
                );
            }
        } else {
            let have = fingerprint(dir)?;
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
        Ok(Release {
            group,
            version,
            sealed,
        })
    }
}

/// One node of the release as a node form: its sheet as it is now on one side,
/// what the release holds on the other.
fn node_form(
    root: &Path,
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
    // The author's code, which produced the isolation results: the gate holds
    // cases that came from code until the code is beside them.
    let how = fs::read_to_string(nd.join("results/how-run.md")).unwrap_or_default();
    if let Some((file, code, owner)) = author_code(dir, &nd, &how) {
        // Whose code it is: the author of the node whose folder holds it.
        let who = read_csv(&owner.join("declaration.csv"))
            .ok()
            .and_then(|r| r.into_iter().next())
            .and_then(|r| r.get("author").cloned())
            .unwrap_or_default();
        filled.fields.insert("author_name".into(), who);
        let ext = file.rsplit('.').next().unwrap_or("").to_lowercase();
        let language = match ext.as_str() {
            "py" => "Python",
            "m" => "MATLAB",
            "jl" => "Julia",
            "c" => "C",
            "cpp" | "cc" | "cxx" => "C++",
            "f" | "f90" | "f95" => "Fortran",
            "rs" => "Rust",
            "xlsx" | "xls" | "ods" => "Excel",
            _ => "other",
        };
        filled
            .fields
            .insert("author_language".into(), language.into());
        filled.fields.insert("author_code".into(), code.clone());
        // The script that ran the cases is the one how-run.md names; in a group
        // folder it is usually the same file that holds the code.
        filled.fields.insert("author_test_code".into(), code);
        filled
            .fields
            .insert("author_how_run".into(), how.trim().to_string());
        // The function rerun calls is named only when the author names it.
        let entry = how
            .lines()
            .find_map(|l| {
                l.split_once("**Entry:**")
                    .map(|(_, e)| e.trim().trim_matches('`').to_string())
            })
            .unwrap_or_default();
        filled.fields.insert("author_entry".into(), entry);
    }
    // How the relation was arrived at: without it the row is stated and never
    // derived, and the engine holds it unanswered.
    if let Ok(md) = fs::read_to_string(nd.join("theory.md")) {
        let (why, steps, reading) = theory(&md);
        if !why.is_empty() || !steps.is_empty() || !reading.is_empty() {
            filled.fields.insert("theory_why".into(), why);
            filled.fields.insert("theory_reading".into(), reading);
            filled.arrays.insert(
                "theory".into(),
                steps
                    .into_iter()
                    .map(|(text, math)| {
                        BTreeMap::from([("text".into(), text), ("math".into(), math)])
                    })
                    .collect(),
            );
        }
    }
    // Whose it is, and whether an assistant helped. Silence is not "none".
    let decl = read_csv(&nd.join("declaration.csv"))
        .ok()
        .and_then(|r| r.into_iter().next())
        .unwrap_or_default();
    let help = declared_help(root, &decl);
    if let Some(why) = &help.not_taken {
        println!("  transcribed, but {why}: taken as a relation an assistant supplied");
    }
    // A transcription is recorded on the node's version, so the sheet says
    // what the method was copied from and who read the copy against it.
    let mut derisk = derisk.clone();
    if let Some(t) = &help.transcribed {
        derisk.changed = if derisk.changed.trim().is_empty() {
            t.clone()
        } else {
            format!("{} {t}", derisk.changed.trim())
        };
    }
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
        ai: help.ai,
        notes: format!("from the group release {group} {version}"),
        original,
        filled,
        known: Vec::new(),
        derisk,
    })
}

/// How an assistant helped, as a node's declaration says.
pub(super) struct Help {
    /// `none`, `wording`, `relation` or `transcribed`, as the plan reads it.
    pub ai: String,
    /// For a transcription taken: what the version records of it.
    pub transcribed: Option<String>,
    /// For a transcription not taken: why.
    pub not_taken: Option<String>,
}

/// Read a declaration. Silence is not "none". A transcription is a person's
/// relation copied by an assistant, and is taken only when it names what it was
/// copied from and the person who read the copy against that — a person, not an
/// assistant. Without either it is one an assistant supplied.
pub(super) fn declared_help(root: &Path, decl: &BTreeMap<String, String>) -> Help {
    let plain = |ai: &str| Help {
        ai: ai.to_string(),
        transcribed: None,
        not_taken: None,
    };
    match decl.get("ai").map(String::as_str) {
        Some(a @ ("none" | "wording" | "relation")) => plain(a),
        Some("transcribed") => {
            let source = decl.get("source").map(|s| s.trim()).unwrap_or("");
            let checker = decl.get("checked_by").map(|s| s.trim()).unwrap_or("");
            let not_taken = if source.is_empty() {
                Some("it names no source it was copied from".to_string())
            } else if checker.is_empty() {
                Some("it names nobody who checked the copy against its source".to_string())
            } else if vleo_sheet::form::refuse_agent_attribution(root, checker).is_err() {
                Some(format!(
                    "'{checker}', who it says checked the copy, is an assistant"
                ))
            } else {
                None
            };
            match not_taken {
                Some(why) => Help {
                    not_taken: Some(why),
                    ..plain("relation")
                },
                None => Help {
                    ai: "transcribed".into(),
                    transcribed: Some(format!(
                        "The method was transcribed by an assistant from {source}; \
                         {checker} read the copy against it and signs it."
                    )),
                    not_taken: None,
                },
            }
        }
        _ => plain("relation"),
    }
}

/// The isolation results as the form's test cases: every value back in SI,
/// as the sheet holds it.
pub(super) fn cases(path: &Path) -> Result<Vec<BTreeMap<String, String>>, String> {
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

/// The code that produced a node's results: the node's own `code/` folder, or
/// the file its how-run.md names in another node's (`nodes/<id>/code/<file>`).
/// Several files are kept as one, each under its name.
fn author_code(dir: &Path, nd: &Path, how: &str) -> Option<(String, String, PathBuf)> {
    let read_dir = |d: &Path| -> Vec<(String, String)> {
        let mut v: Vec<(String, String)> = fs::read_dir(d)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_file())
            .filter_map(|p| {
                let name = p.file_name()?.to_string_lossy().to_string();
                fs::read_to_string(&p).ok().map(|t| (name, t))
            })
            .collect();
        v.sort();
        v
    };
    let mut files = read_dir(&nd.join("code"));
    let mut owner = nd.to_path_buf();
    if files.is_empty() {
        let named = how
            .split('`')
            .skip(1)
            .step_by(2)
            .flat_map(|span| span.split_whitespace())
            .find(|w| w.starts_with("nodes/") && w.contains("/code/"))?;
        let t = fs::read_to_string(dir.join(named)).ok()?;
        owner = dir.join(named.split("/code/").next()?);
        files.push((named.rsplit('/').next().unwrap_or(named).to_string(), t));
    }
    let first = files.first()?.0.clone();
    if files.len() == 1 {
        return Some((first, files.remove(0).1, owner));
    }
    let joined = files
        .iter()
        .map(|(n, t)| format!("# ── {n} ──\n{t}"))
        .collect::<Vec<_>>()
        .join("\n");
    Some((first, joined, owner))
}

/// A node's theory.md as the sheet's `[theory]`: the inverse of what
/// `group-export` writes. Under *Derivation*, prose before the numbered steps
/// is why; each step is its sentence, with a trailing `code span` as its line
/// of maths. Under *Validity*, everything but the sentence the export writes
/// from the output's bounds is the reading. Equations and assumptions stay in
/// the release.
fn theory(md: &str) -> (String, Vec<(String, String)>, String) {
    let mut sections: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    let mut at = String::new();
    for line in md.lines() {
        match line.strip_prefix("## ") {
            Some(h) => at = h.trim().to_lowercase(),
            None => sections.entry(at.clone()).or_default().push(line),
        }
    }
    let paragraphs = |name: &str| -> Vec<String> {
        sections
            .get(name)
            .map(|l| l.join("\n"))
            .unwrap_or_default()
            .split("\n\n")
            .map(|p| p.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|p| !p.is_empty())
            .collect()
    };
    let numbered = |l: &str| {
        let l = l.trim_start();
        let digits = l.chars().take_while(char::is_ascii_digit).count();
        (digits > 0 && l[digits..].starts_with(". ")).then(|| l[digits + 2..].to_string())
    };
    let (mut why, mut steps) = (Vec::new(), Vec::<String>::new());
    for line in sections
        .get("derivation")
        .cloned()
        .unwrap_or_default()
        .join("\n")
        .split("\n\n")
    {
        let mut prose = Vec::new();
        for l in line.lines() {
            match numbered(l) {
                Some(s) => steps.push(s),
                None if !steps.is_empty() && !l.trim().is_empty() && prose.is_empty() => {
                    let last = steps.last_mut().unwrap();
                    last.push(' ');
                    last.push_str(l.trim());
                }
                None => prose.push(l.trim()),
            }
        }
        let p = prose.join(" ");
        if !p.trim().is_empty() {
            why.push(p.split_whitespace().collect::<Vec<_>>().join(" "));
        }
    }
    let steps = steps
        .into_iter()
        .map(|s| {
            let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
            match s
                .strip_suffix('`')
                .and_then(|t| t.rfind(" `").map(|i| (t, i)))
            {
                Some((t, i)) => (t[..i].to_string(), t[i + 2..].to_string()),
                None => (s, String::new()),
            }
        })
        .collect();
    // The export's own sentence from the bounds: "From <lo> to <hi> <unit>." —
    // the bounds are the contract's, not the reading.
    let bounds = |p: &str| {
        let w: Vec<&str> = p.split_whitespace().collect();
        let n = |s: &str| s.parse::<f64>().is_ok() || s.contains("inf") || s.contains('∞');
        w.len() > 4 && w[0] == "From" && w[2] == "to" && n(w[1]) && n(w[3])
    };
    let reading: Vec<String> = paragraphs("validity")
        .into_iter()
        .filter(|p| !bounds(p))
        .collect();
    (why.join("\n\n"), steps, reading.join("\n\n"))
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
pub(super) fn read_csv(path: &Path) -> Result<Vec<BTreeMap<String, String>>, String> {
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
pub(super) fn parse_csv(text: &str) -> (Vec<String>, Vec<Vec<String>>) {
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

    fn declaration(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn a_transcription_is_taken_only_with_its_source_and_a_person_who_checked_it() {
        let root = repo();
        for a in ["none", "wording", "relation"] {
            assert_eq!(declared_help(&root, &declaration(&[("ai", a)])).ai, a);
        }
        // Silence, and a value nobody defined, are an assistant's relation.
        assert_eq!(declared_help(&root, &declaration(&[])).ai, "relation");
        assert_eq!(
            declared_help(&root, &declaration(&[("ai", "copied")])).ai,
            "relation"
        );
        let taken = declared_help(
            &root,
            &declaration(&[
                ("ai", "transcribed"),
                ("source", "crates/vleo-core/src/physics/solar.rs:120"),
                ("checked_by", "A. Person"),
            ]),
        );
        assert_eq!(taken.ai, "transcribed");
        let record = taken.transcribed.unwrap();
        assert!(
            record.contains("solar.rs:120") && record.contains("A. Person"),
            "{record}"
        );
        for (source, checker) in [
            ("", "A. Person"),
            ("x.rs:1", ""),
            ("x.rs:1", "  "),
            ("x.rs:1", "Claude"),
            ("x.rs:1", "claude/opus"),
            ("x.rs:1", "Copilot bot"),
        ] {
            let h = declared_help(
                &root,
                &declaration(&[
                    ("ai", "transcribed"),
                    ("source", source),
                    ("checked_by", checker),
                ]),
            );
            assert_eq!(h.ai, "relation", "{source:?} / {checker:?}");
            assert!(h.not_taken.is_some() && h.transcribed.is_none());
        }
    }

    #[test]
    fn the_spec_refuses_the_same_assistants_the_engine_does() {
        let root = repo();
        let spec: toml::Value = fs::read_to_string(root.join("groups/SPEC.toml"))
            .unwrap()
            .parse()
            .unwrap();
        let listed: Vec<String> = spec["file"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["path"].as_str() == Some("declaration.csv"))
            .and_then(|f| f.get("assistants"))
            .and_then(|a| a.as_array())
            .expect("declaration.csv lists the assistants' names")
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert_eq!(listed, vleo_sheet::form::agent_identities(&root));
    }

    #[test]
    fn a_transcribed_method_is_planned_where_an_assistants_relation_is_refused() {
        let root = repo();
        let tree = load(&root).unwrap();
        let sh = tree.sheets.get("sw_activity_band").unwrap();
        let original = template::content(sh);
        let mut filled = original.clone();
        let method = original
            .fields
            .get("method_text")
            .cloned()
            .unwrap_or_default();
        filled.fields.insert(
            "method_text".into(),
            format!("# copied from the running code\n{method}"),
        );
        let base = fs::read_to_string(sh.dir.join("node.toml"))
            .map(|t| vleo_sheet::form::file_hash(&t))
            .unwrap();
        let plan = |ai: &str| {
            let f = Form {
                node: sh.id.clone(),
                base: base.clone(),
                name: "A. Person".into(),
                date: "2026-10-03".into(),
                ai: ai.into(),
                original: original.clone(),
                filled: filled.clone(),
                ..Form::default()
            };
            let p = template::plan_form(&root, f).unwrap();
            p.items
                .into_iter()
                .find(|i| i.what == "method_text")
                .expect("the method is an item of the plan")
                .verdict
        };
        let refused_as_assistants = |v: &vleo_sheet::template::Verdict| matches!(v, vleo_sheet::template::Verdict::Refused(why) if why.contains("assistant"));
        assert!(refused_as_assistants(&plan("relation")));
        assert!(!refused_as_assistants(&plan("transcribed")));
    }

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
