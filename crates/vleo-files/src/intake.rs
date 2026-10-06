//! A group's sealed release, read as the design takes it in.
//!
//! One implementation, used by `xtask group-intake`, which applies a release
//! to the sheets, and by today's design (`vleo_server::today`), which builds
//! the design from every group's latest sealed release when the application
//! opens. A release is read from the folder it unpacks to or from its file,
//! and nothing about it is taken on trust:
//!
//! - the seal is checked first ([`Release::check_seal`]) — every file's
//!   SHA-256, recomputed, must give the fingerprint the release was sealed
//!   with, so a file changed after the people signed is refused before
//!   anything is read from it;
//! - each computed node becomes the node form its author would have filled
//!   ([`node_form`]) — its pseudocode as the method, its isolation results,
//!   brought back to SI, as its test cases, its author and their declaration
//!   of any assistant's help — for the same plan, check and apply as any form
//!   (`vleo_sheet::template`).
//!
//! What the release holds beyond the method and its cases — its words, its
//! pictures, its evidence — stays in the release, which is what the group
//! signed.

use std::collections::BTreeMap;
use std::path::Path;

use vleo_sheet::template::{self, Derisk, Form};

use crate::format_1::Folder;

/// A sealed release: what it says of itself, and the folder it holds.
#[derive(Clone, Debug)]
pub struct Release {
    pub group: String,
    pub version: String,
    /// When it was sealed; empty for a draft.
    pub sealed: String,
    pub sealed_by: String,
    /// The fingerprint it was sealed with.
    pub fingerprint: String,
    /// Every file of the folder that was sealed, by its path in the folder.
    pub folder: Folder,
}

impl Release {
    /// A release from its folder and what it says of itself: `group_id`,
    /// `version`, `sealed`, `sealed_by` and `fingerprint`.
    pub fn new(meta: &BTreeMap<String, String>, folder: Folder) -> Release {
        let get = |k: &str| meta.get(k).cloned().unwrap_or_default();
        Release {
            group: get("group_id"),
            version: get("version"),
            sealed: get("sealed"),
            sealed_by: get("sealed_by"),
            fingerprint: get("fingerprint"),
            folder,
        }
    }

    /// A release from its file, as the group application seals it: format 1,
    /// read as it is. A file in any other format is not one today's releases
    /// are, and is refused by name.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open(path: &Path) -> Result<Release, String> {
        let old =
            crate::sqlite::read_format_1(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Release::new(&old.meta, old.folder()))
    }

    /// The release's files are the ones that were sealed. Returns the
    /// fingerprint they give; refused when the release is not sealed, or when
    /// any file changed after it was.
    pub fn check_seal(&self) -> Result<String, String> {
        if self.sealed.is_empty() {
            return Err("this release is not sealed".into());
        }
        crate::seal::check_sealed(&self.folder, &self.fingerprint)
    }

    /// One file of the folder, as text; none when it is not text.
    pub fn text(&self, path: &str) -> Option<String> {
        String::from_utf8(self.folder.get(path)?.clone()).ok()
    }

    /// One CSV file of the folder, as records by header.
    pub fn records(&self, path: &str) -> Result<Vec<BTreeMap<String, String>>, String> {
        self.text(path)
            .map(|t| records(&t))
            .ok_or_else(|| format!("the release has no {path}"))
    }

    /// The group's latest version record, as the form's de-risking record.
    pub fn latest_version(&self) -> Derisk {
        let rows = self.records("versions.csv").unwrap_or_default();
        let v = rows.last().cloned().unwrap_or_default();
        let g = |k: &str| v.get(k).cloned().unwrap_or_default();
        Derisk {
            believed: g("believed"),
            tested: g("tested"),
            learned: g("learned"),
            changed: g("changed"),
            risks: g("risks"),
            cost: g("cost"),
            rests_on: g("rests_on"),
            breaks_if: g("breaks_if"),
        }
    }
}

/// One node of the release as a node form: its sheet as it is now on one side,
/// what the release holds on the other. `base` is the hash of the sheet's
/// `node.toml` as the design holds it now (`vleo_sheet::form::file_hash`).
/// Also returns, for a transcription not taken, why.
pub fn node_form(
    root: &Path,
    release: &Release,
    sh: &vleo_sheet::model::Sheet,
    base: String,
    derisk: &Derisk,
) -> Result<(Form, Option<String>), String> {
    let nd = format!("nodes/{}", sh.id);
    let original = template::content(sh);
    let mut filled = original.clone();
    if let Some(pc) = release.text(&format!("{nd}/pseudocode.txt")) {
        filled
            .fields
            .insert("method_text".into(), pc.trim_end().to_string());
    }
    let isolation = format!("{nd}/results/isolation.csv");
    if let Some(text) = release.text(&isolation) {
        if let Ok(rows) = cases(&text, &isolation) {
            filled.arrays.insert("case".into(), rows);
        }
    }
    // The author's code, which produced the isolation results: the gate holds
    // cases that came from code until the code is beside them.
    let how = release
        .text(&format!("{nd}/results/how-run.md"))
        .unwrap_or_default();
    if let Some((file, code, owner)) = author_code(release, &nd, &how) {
        // Whose code it is: the author of the node whose folder holds it.
        let who = release
            .records(&format!("{owner}/declaration.csv"))
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
    if let Some(md) = release.text(&format!("{nd}/theory.md")) {
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
    let decl = release
        .records(&format!("{nd}/declaration.csv"))
        .ok()
        .and_then(|r| r.into_iter().next())
        .unwrap_or_default();
    let help = declared_help(root, &decl);
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
    let form = Form {
        node: sh.id.clone(),
        new: None,
        base,
        name: decl.get("author").cloned().unwrap_or_default(),
        team: release.group.clone(),
        date: release.sealed.chars().take(10).collect(),
        ai: help.ai,
        notes: format!(
            "from the group release {} {}",
            release.group, release.version
        ),
        original,
        filled,
        known: Vec::new(),
        derisk,
    };
    Ok((form, help.not_taken))
}

/// How an assistant helped, as a node's declaration says.
pub struct Help {
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
pub fn declared_help(root: &Path, decl: &BTreeMap<String, String>) -> Help {
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
/// as the sheet holds it. `name` says where the text came from, for a refusal.
pub fn cases(text: &str, name: &str) -> Result<Vec<BTreeMap<String, String>>, String> {
    let (head, rows) = parse_csv(text);
    let unit = |h: &str| -> Result<(String, f64), String> {
        match h.split_once('[') {
            Some((col, u)) => {
                let u = u.trim_end_matches(']').trim();
                let f = vleo_sheet::method::parse_unit(u)
                    .map_err(|e| format!("{name}: {h}: {e}"))?
                    .0;
                Ok((col.trim().to_string(), f))
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
        let get = |col: &str| {
            cols.iter()
                .position(|(c, _)| c == col)
                .and_then(|i| r.get(i))
                .cloned()
                .unwrap_or_default()
        };
        let refuse = get("refuses") == "yes";
        let inputs: Vec<String> = cols
            .iter()
            .enumerate()
            .filter(|(_, (c, _))| !other.contains(&c.as_str()) && !c.starts_with("answer."))
            .filter_map(|(i, (c, f))| si(r.get(i)?, *f).map(|v| format!("{c} = {v}")))
            .collect();
        // A node that publishes several values gives each member's answer in
        // its own `answer.<member> [unit]` column.
        let also: Vec<String> = cols
            .iter()
            .enumerate()
            .filter_map(|(i, (c, f))| {
                let m = c.strip_prefix("answer.")?;
                si(r.get(i)?, *f).map(|v| format!("{m} = {v}"))
            })
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
        // Where the answer came from, as the group's results say it; the gate
        // asks for the author's code only for a case that came from it.
        let origin = get("origin");
        if !origin.is_empty() {
            row.insert("origin".into(), origin);
        }
        if !refuse {
            if let Some(i) = answer {
                row.insert(
                    "expect".into(),
                    r.get(i).and_then(|v| si(v, cols[i].1)).unwrap_or_default(),
                );
            }
            row.insert("tolerance".into(), get("tolerance"));
            if !also.is_empty() {
                row.insert("also".into(), format!("{{ {} }}", also.join(", ")));
            }
        }
        row.insert("inputs".into(), format!("{{ {} }}", inputs.join(", ")));
        out.push(row);
    }
    Ok(out)
}

/// The code that produced a node's results: the node's own `code/` folder, or
/// the file its how-run.md names in another node's (`nodes/<id>/code/<file>`).
/// Several files are kept as one, each under its name. Returns the first
/// file's name, the code, and the folder of the node whose code it is.
fn author_code(release: &Release, nd: &str, how: &str) -> Option<(String, String, String)> {
    let in_code = |d: &str| -> Vec<(String, String)> {
        let prefix = format!("{d}/code/");
        release
            .folder
            .iter()
            .filter_map(|(p, b)| {
                let name = p.strip_prefix(&prefix)?;
                if name.contains('/') {
                    return None;
                }
                Some((name.to_string(), String::from_utf8(b.clone()).ok()?))
            })
            .collect()
    };
    let mut files = in_code(nd);
    let mut owner = nd.to_string();
    if files.is_empty() {
        let named = how
            .split('`')
            .skip(1)
            .step_by(2)
            .flat_map(|span| span.split_whitespace())
            .find(|w| w.starts_with("nodes/") && w.contains("/code/"))?;
        let t = release.text(named)?;
        owner = named.split("/code/").next()?.to_string();
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
pub fn theory(md: &str) -> (String, Vec<(String, String)>, String) {
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

/// A CSV text as records by header.
pub fn records(text: &str) -> Vec<BTreeMap<String, String>> {
    let (head, rows) = parse_csv(text);
    rows.into_iter()
        .map(|r| {
            head.iter()
                .cloned()
                .zip(r.into_iter().chain(std::iter::repeat(String::new())))
                .collect()
        })
        .collect()
}

/// A CSV text as its header and rows, as the group's folder writes it: quoted
/// cells may hold commas, quotes and line breaks; blank lines are dropped.
pub fn parse_csv(text: &str) -> (Vec<String>, Vec<Vec<String>>) {
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
