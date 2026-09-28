//! De-risking: why a node is what it is, and why it changed.
//!
//! Every decision in the tree rests on a belief — that this relation holds
//! here, that this input is the one that matters, that this picture shows what
//! it claims to. A node changes because one of those beliefs BROKE: somebody
//! tested it and learned otherwise. So every change is recorded the way the
//! programme's de-risking narrative is written, one row per change:
//!
//!   what we believed · what we tested · what we now know · what it cost ·
//!   what changed · which risks it opened or closed
//!
//! and every version says what IT rests on and what would break it, so the
//! next change is anticipated rather than discovered.
//!
//! The rows add up. A risk is registered once, on one of the risk-register rows
//! of the management layer, and moved only by node versions — so each
//! risk-register row can say, from the whole tree, how the work below it has
//! bought its risks down, and what it cost.
//!
//! Nothing here decides anything. It reads what the sheets record, checks that
//! the record is complete and consistent, and lays it out.

use std::collections::BTreeMap;

use crate::load::Tree;
use crate::model::{Sheet, Version};

/// The kinds of decision a version can move. A form's changes are sorted into
/// these by what they touch — see [`about_of`] — so the record says what kind
/// of belief broke without anybody having to classify it.
pub const ABOUT: &[&str] = &[
    "node",
    "input",
    "output",
    "model",
    "math",
    "algorithm",
    "visualisation",
];

/// Risk levels, least to most.
pub const LEVELS: &[&str] = &["L1", "L2", "L3", "L4", "L5"];

/// The group whose rows hold the risk register. Which of its rows holds a risk
/// says what kind of risk it is: technical, schedule, supply, regulatory.
pub const REGISTER: &str = "mgt_risk_register";

/// What a version released but not yet cut says in its `release`.
pub const NEXT: &str = "next";

/// Which kind of decision a form field or block is.
///
/// Wording — the note, the plain-words explanation, the prose of the
/// derivation — is not a decision and moves no version: correcting a sentence
/// must never need a de-risking record, or nobody corrects the sentence.
pub fn about_of(what: &str) -> Option<&'static str> {
    Some(match what {
        "label" | "question" | "new" => "node",
        "input" | "declared_value" => "input",
        "symbol" | "type" | "unit" | "lower" | "upper" | "reason_lower" | "reason_upper"
        | "sense" | "publishes" => "output",
        "source" | "assumption" => "model",
        "expression" | "method_text" => "math",
        "algorithm" => "algorithm",
        "view" => "visualisation",
        _ => return None,
    })
}

/// One move a version makes on one risk.
#[derive(Clone, Debug, PartialEq)]
pub enum Move {
    Opened,
    Closed,
    Level { from: String, to: String },
}

/// Read `R-01 L5->L4`, `R-09 closed` or `R-12 opened`.
pub fn parse_move(s: &str) -> Result<(String, Move), String> {
    let s = s.trim();
    let (id, rest) = s.split_once(char::is_whitespace).ok_or_else(|| {
        format!("«{s}» is not a risk move: write `R-01 L5->L4`, `R-09 closed` or `R-12 opened`")
    })?;
    if !risk_id_ok(id) {
        return Err(format!(
            "«{id}» is not a risk id: R- and a number, like R-07"
        ));
    }
    let rest = rest.trim();
    let m = match rest {
        "opened" => Move::Opened,
        "closed" => Move::Closed,
        _ => {
            let (a, b) = rest
                .split_once("->")
                .ok_or_else(|| format!("«{s}»: a level move is written `L5->L4`"))?;
            let (a, b) = (a.trim(), b.trim());
            if !LEVELS.contains(&a) || !LEVELS.contains(&b) {
                return Err(format!("«{s}»: a level is one of {}", LEVELS.join(", ")));
            }
            if a == b {
                return Err(format!("«{s}» moves nothing"));
            }
            Move::Level {
                from: a.into(),
                to: b.into(),
            }
        }
    };
    Ok((id.to_string(), m))
}

pub fn risk_id_ok(id: &str) -> bool {
    id.strip_prefix("R-")
        .map(|n| !n.is_empty() && n.len() <= 4 && n.chars().all(|c| c.is_ascii_digit()))
        .unwrap_or(false)
}

/// `0.2.0` → (0, 2, 0); `next` sorts after every release.
pub fn release_key(r: &str) -> Option<(u32, u32, u32)> {
    if r == NEXT {
        return Some((u32::MAX, 0, 0));
    }
    let mut it = r.split('.').map(|x| x.parse::<u32>().ok());
    let k = (it.next()??, it.next()??, it.next()??);
    if it.next().is_some() {
        return None;
    }
    Some(k)
}

pub fn date_ok(d: &str) -> bool {
    let b = d.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && d.chars()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

/// `2026-09-14` → `Q3-26`, the quarter the narrative files it under.
pub fn quarter(date: &str) -> String {
    if !date_ok(date) {
        return String::new();
    }
    let m: u32 = date[5..7].parse().unwrap_or(1);
    format!("Q{}-{}", (m.max(1) - 1) / 3 + 1, &date[2..4])
}

/// Everything wrong with one node's record, by version.
pub fn version_problems(sh: &Sheet) -> Vec<String> {
    let mut bad = Vec::new();
    let mut last = (0u32, 0u32, 0u32);
    for (i, v) in sh.versions.iter().enumerate() {
        let at = format!("version {}", i + 1);
        if v.n as usize != i + 1 {
            bad.push(format!(
                "{at} is numbered {} — versions count 1, 2, 3 in order",
                v.n
            ));
        }
        match release_key(&v.release) {
            None => bad.push(format!(
                "{at}: release «{}» is neither `next` nor a version like 0.2.0",
                v.release
            )),
            Some(k) if k < last => bad.push(format!(
                "{at}: release {} is older than the version before it",
                v.release
            )),
            Some(k) => last = k,
        }
        if !date_ok(&v.date) {
            bad.push(format!("{at}: date «{}» is not YYYY-MM-DD", v.date));
        }
        if v.by.trim().is_empty() {
            bad.push(format!("{at}: nobody's name — a record says who made it"));
        }
        if v.about.is_empty() {
            bad.push(format!("{at}: says no kind of decision moved"));
        }
        for a in &v.about {
            if !ABOUT.contains(&a.as_str()) {
                bad.push(format!("{at}: «{a}» is not one of {}", ABOUT.join(", ")));
            }
        }
        if v.rests_on.trim().is_empty() || v.breaks_if.trim().is_empty() {
            bad.push(format!(
                "{at}: every version says what it rests on and what would break it"
            ));
        }
        let told = [&v.believed, &v.tested, &v.learned];
        if (i > 0 || told.iter().any(|x| !x.trim().is_empty()))
            && (told.iter().any(|x| x.trim().is_empty()) || v.changed.trim().is_empty())
        {
            bad.push(format!(
                "{at}: a change says what we believed, what we tested, what we now know, and \
                 what changed — a version without its reason is a change nobody can review"
            ));
        }
        for m in &v.risks {
            if let Err(e) = parse_move(m) {
                bad.push(format!("{at}: {e}"));
            }
        }
    }
    bad
}

/// The version a node is at: 0 when nothing is recorded.
pub fn current(sh: &Sheet) -> u32 {
    sh.versions.last().map(|v| v.n).unwrap_or(0)
}

// ---------------------------------------------------------------------------
// the register, read across the tree

/// One risk as it stands, and every version that moved it.
#[derive(Clone, Debug)]
pub struct RiskState {
    pub id: String,
    pub title: String,
    pub owner: String,
    pub why: String,
    /// The risk-register row that holds it, which is its kind.
    pub row: String,
    pub row_label: String,
    pub registered: String,
    /// The level now, or `closed`.
    pub now: String,
    pub moves: Vec<MoveRecord>,
}

#[derive(Clone, Debug)]
pub struct MoveRecord {
    pub node: String,
    pub label: String,
    pub n: u32,
    pub date: String,
    pub release: String,
    pub what: String,
    pub learned: String,
}

/// Every registered risk, moved by every version that names it, in date order.
pub fn register(tree: &Tree) -> Vec<RiskState> {
    let mut out: BTreeMap<String, RiskState> = BTreeMap::new();
    for sh in tree.ordered() {
        for r in &sh.risks {
            out.insert(
                r.id.clone(),
                RiskState {
                    id: r.id.clone(),
                    title: r.title.clone(),
                    owner: r.owner.clone(),
                    why: r.why.clone(),
                    row: sh.id.clone(),
                    row_label: sh.label.clone(),
                    registered: r.level.clone(),
                    now: r.level.clone(),
                    moves: Vec::new(),
                },
            );
        }
    }
    let mut moves: Vec<(String, String, u32, String, MoveRecord)> = Vec::new();
    for sh in tree.ordered() {
        for v in &sh.versions {
            for m in &v.risks {
                if let Ok((id, _)) = parse_move(m) {
                    moves.push((
                        v.date.clone(),
                        sh.id.clone(),
                        v.n,
                        id,
                        MoveRecord {
                            node: sh.id.clone(),
                            label: sh.label.clone(),
                            n: v.n,
                            date: v.date.clone(),
                            release: v.release.clone(),
                            what: m.clone(),
                            learned: v.learned.clone(),
                        },
                    ));
                }
            }
        }
    }
    moves.sort_by(|a, b| (&a.0, &a.1, a.2).cmp(&(&b.0, &b.1, b.2)));
    for (_, _, _, id, rec) in moves {
        if let Some(r) = out.get_mut(&id) {
            if let Ok((_, m)) = parse_move(&rec.what) {
                match m {
                    Move::Opened => {}
                    Move::Closed => r.now = "closed".into(),
                    Move::Level { to, .. } => r.now = to,
                }
            }
            r.moves.push(rec);
        }
    }
    out.into_values().collect()
}

/// Every risk move that names a risk nobody registered.
pub fn unregistered_moves(tree: &Tree) -> Vec<String> {
    let known: std::collections::BTreeSet<&str> = tree
        .sheets
        .values()
        .flat_map(|s| s.risks.iter().map(|r| r.id.as_str()))
        .collect();
    let mut bad = Vec::new();
    for sh in tree.ordered() {
        for v in &sh.versions {
            for m in &v.risks {
                if let Ok((id, _)) = parse_move(m) {
                    if !known.contains(id.as_str()) {
                        bad.push(format!(
                            "{} version {} moves {id}, which no risk-register row registers",
                            sh.id, v.n
                        ));
                    }
                }
            }
        }
    }
    bad
}

/// Everything wrong with the register itself.
pub fn register_problems(tree: &Tree) -> Vec<String> {
    let mut bad = Vec::new();
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
    for sh in tree.ordered() {
        if !sh.risks.is_empty() && sh.parent != REGISTER {
            bad.push(format!(
                "{} registers risks and is not a row of the risk register ({REGISTER})",
                sh.id
            ));
        }
        for r in &sh.risks {
            if !risk_id_ok(&r.id) {
                bad.push(format!(
                    "{}: «{}» is not a risk id: R- and a number",
                    sh.id, r.id
                ));
            }
            if let Some(other) = seen.insert(r.id.as_str(), sh.id.as_str()) {
                bad.push(format!(
                    "{} is registered twice: on {other} and on {}",
                    r.id, sh.id
                ));
            }
            if !LEVELS.contains(&r.level.as_str()) {
                bad.push(format!(
                    "{} {}: level «{}» is not one of {}",
                    sh.id,
                    r.id,
                    r.level,
                    LEVELS.join(", ")
                ));
            }
            if r.title.trim().is_empty() || r.owner.trim().is_empty() || r.why.trim().is_empty() {
                bad.push(format!(
                    "{} {}: a risk says what it is, who owns it and what it would cost",
                    sh.id, r.id
                ));
            }
            if !date_ok(&r.since) {
                bad.push(format!(
                    "{} {}: since «{}» is not YYYY-MM-DD",
                    sh.id, r.id, r.since
                ));
            }
        }
    }
    bad.extend(unregistered_moves(tree));
    bad
}

// ---------------------------------------------------------------------------
// the narrative

/// One row of the de-risking narrative: one version of one node.
#[derive(Clone, Debug)]
pub struct Row {
    pub quarter: String,
    pub release: String,
    pub date: String,
    pub node: String,
    pub label: String,
    pub v: Version,
}

/// Every recorded change, oldest first. A first version with nothing tested
/// yet is a starting belief rather than a change, and is listed separately.
pub fn narrative(tree: &Tree) -> (Vec<Row>, Vec<Row>) {
    let mut changes = Vec::new();
    let mut starts = Vec::new();
    for sh in tree.ordered() {
        for v in &sh.versions {
            let row = Row {
                quarter: quarter(&v.date),
                release: v.release.clone(),
                date: v.date.clone(),
                node: sh.id.clone(),
                label: sh.label.clone(),
                v: v.clone(),
            };
            if v.tested.trim().is_empty() && v.learned.trim().is_empty() {
                starts.push(row);
            } else {
                changes.push(row);
            }
        }
    }
    let key = |r: &Row| {
        (
            release_key(&r.release),
            r.date.clone(),
            r.node.clone(),
            r.v.n,
        )
    };
    changes.sort_by_key(key);
    starts.sort_by_key(key);
    (changes, starts)
}

fn csv_cell(s: &str) -> String {
    let one = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.contains([',', '"']) {
        format!("\"{}\"", one.replace('"', "\"\""))
    } else {
        one
    }
}

/// The narrative as a spreadsheet: the columns of the quarterly de-risking
/// narrative, then where each row comes from. Generated; opens in any
/// spreadsheet.
pub fn narrative_csv(tree: &Tree) -> String {
    let (changes, _) = narrative(tree);
    let mut o = String::from(
        "Quarter,What we believed,What we tested,What we now know,What it cost,\
         What changed in the plan,Risks opened / closed,Release,Date,Node,Version,Rests on now,Would break if\n",
    );
    for r in &changes {
        let cells = [
            r.quarter.clone(),
            r.v.believed.clone(),
            r.v.tested.clone(),
            r.v.learned.clone(),
            r.v.cost.clone(),
            r.v.changed.clone(),
            r.v.risks.join("; "),
            r.release.clone(),
            r.date.clone(),
            r.node.clone(),
            r.v.n.to_string(),
            r.v.rests_on.clone(),
            r.v.breaks_if.clone(),
        ];
        o.push_str(
            &cells
                .iter()
                .map(|c| csv_cell(c))
                .collect::<Vec<_>>()
                .join(","),
        );
        o.push('\n');
    }
    o
}

/// The narrative as a page to read.
pub fn narrative_md(tree: &Tree) -> String {
    let (changes, starts) = narrative(tree);
    let reg = register(tree);
    let versioned = tree
        .sheets
        .values()
        .filter(|s| !s.versions.is_empty())
        .count();
    let published = tree.sheets.values().filter(|s| !s.is_seeded()).count();
    let mut o = String::new();
    o.push_str("# The de-risking narrative\n\n");
    o.push_str(&format!(
        "> **Answer first.** {} change{} to the design {} been recorded, each because a belief \
         broke; {} of {} published rows state what they rest on; {} risk{} {} registered, {} \
         still open. Generated by `cargo run -p xtask -- derisk` from every node's `[[version]]` \
         record and the risk-register rows — edit those, never this.\n>\n> **Kind:** reference · \
         **For:** everyone\n\n",
        changes.len(),
        if changes.len() == 1 { "" } else { "s" },
        if changes.len() == 1 { "has" } else { "have" },
        versioned,
        published,
        reg.len(),
        if reg.len() == 1 { "" } else { "s" },
        if reg.len() == 1 { "is" } else { "are" },
        reg.iter().filter(|r| r.now != "closed").count()
    ));
    o.push_str(
        "Why this exists, and how a change gets here, is in `docs/DERISKING.md`. The same rows \
         as a spreadsheet are in `docs/derisking.csv`.\n\n",
    );
    o.push_str("## The risks, as they stand\n\n");
    if reg.is_empty() {
        o.push_str(
            "No risk is registered yet. A risk is registered on one of the risk-register rows \
             of the management layer, through that row's form.\n\n",
        );
    } else {
        o.push_str("| risk | kind | what it is | registered | now | moved by |\n|---|---|---|---|---|---|\n");
        for r in &reg {
            o.push_str(&format!(
                "| {} | {} | {} | {} | **{}** | {} |\n",
                r.id,
                r.row_label,
                r.title.replace('|', "/"),
                r.registered,
                r.now,
                if r.moves.is_empty() {
                    "nothing yet".to_string()
                } else {
                    r.moves
                        .iter()
                        .map(|m| format!("`{}` v{} ({})", m.node, m.n, m.what))
                        .collect::<Vec<_>>()
                        .join("; ")
                }
            ));
        }
        o.push('\n');
    }
    o.push_str("## Every change, by release\n\n");
    if changes.is_empty() {
        o.push_str("No change has been recorded yet.\n\n");
    }
    let mut last = String::new();
    for r in &changes {
        if r.release != last {
            last = r.release.clone();
            o.push_str(&format!(
                "### {}\n\n",
                if last == NEXT {
                    "Not yet released".to_string()
                } else {
                    format!("Release {last}")
                }
            ));
        }
        o.push_str(&format!(
            "**`{}` — {}, version {}** · {} · {} · {}\n\n",
            r.node,
            r.label,
            r.v.n,
            r.quarter,
            r.date,
            r.v.about.join(", ")
        ));
        for (k, v) in [
            ("What we believed", &r.v.believed),
            ("What we tested", &r.v.tested),
            ("What we now know", &r.v.learned),
            ("What it cost", &r.v.cost),
            ("What changed", &r.v.changed),
            ("Rests on now", &r.v.rests_on),
            ("Would break if", &r.v.breaks_if),
        ] {
            if !v.trim().is_empty() {
                o.push_str(&format!("- *{k}:* {}\n", v.replace('\n', " ")));
            }
        }
        if !r.v.risks.is_empty() {
            o.push_str(&format!("- *Risks:* {}\n", r.v.risks.join("; ")));
        }
        o.push('\n');
    }
    o.push_str("## Starting beliefs\n\n");
    if starts.is_empty() {
        o.push_str("None recorded.\n");
    } else {
        o.push_str(
            "The first version of a row: what it rests on before anything has tested it.\n\n",
        );
        for r in &starts {
            o.push_str(&format!(
                "- `{}` v{} ({}) — rests on: {} *Would break if:* {}\n",
                r.node,
                r.v.n,
                r.release,
                r.v.rests_on.replace('\n', " "),
                r.v.breaks_if.replace('\n', " ")
            ));
        }
    }
    o
}

// ---------------------------------------------------------------------------
// writing a version, and stamping a release

fn q(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => o.push_str("\\\\"),
            '"' => o.push_str("\\\""),
            '\n' => o.push_str("\\n"),
            '\t' => o.push_str("\\t"),
            '\r' => {}
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04X}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// A `[[version]]` block, to be appended to a sheet.
pub fn version_toml(v: &Version) -> String {
    let list = |xs: &[String]| {
        format!(
            "[{}]",
            xs.iter().map(|x| q(x)).collect::<Vec<_>>().join(", ")
        )
    };
    let mut o = String::from("\n[[version]]\n");
    o.push_str(&format!("n = {}\n", v.n));
    for (k, x) in [("release", &v.release), ("date", &v.date), ("by", &v.by)] {
        o.push_str(&format!("{k} = {}\n", q(x)));
    }
    o.push_str(&format!("about = {}\n", list(&v.about)));
    for (k, x) in [
        ("believed", &v.believed),
        ("tested", &v.tested),
        ("learned", &v.learned),
        ("cost", &v.cost),
        ("changed", &v.changed),
    ] {
        if !x.trim().is_empty() {
            o.push_str(&format!("{k} = {}\n", q(x)));
        }
    }
    if !v.risks.is_empty() {
        o.push_str(&format!("risks = {}\n", list(&v.risks)));
    }
    for (k, x) in [
        ("rests_on", &v.rests_on),
        ("breaks_if", &v.breaks_if),
        ("relation", &v.relation),
        ("source", &v.source),
    ] {
        o.push_str(&format!("{k} = {}\n", q(x)));
    }
    o
}

/// A sheet's text with every unreleased version stamped as `release`. Only the
/// `release` key inside a `[[version]]` block is touched; nothing else in the
/// file moves.
pub fn stamp(text: &str, release: &str) -> (String, usize) {
    let mut out = String::with_capacity(text.len());
    let mut in_version = false;
    let mut n = 0;
    for line in text.split_inclusive('\n') {
        let t = line.trim();
        if t.starts_with('[') {
            in_version = t == "[[version]]";
        }
        if in_version && t.replace(' ', "") == format!("release=\"{NEXT}\"") {
            out.push_str(&format!("release = \"{release}\"\n"));
            n += 1;
        } else {
            out.push_str(line);
        }
    }
    (out, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_risk_move_reads_or_is_refused_by_name() {
        assert_eq!(
            parse_move("R-01 L5->L4").unwrap(),
            (
                "R-01".into(),
                Move::Level {
                    from: "L5".into(),
                    to: "L4".into()
                }
            )
        );
        assert_eq!(parse_move("R-09 closed").unwrap().1, Move::Closed);
        assert_eq!(parse_move("R-12  opened").unwrap().1, Move::Opened);
        for bad in [
            "R01 closed",
            "R-01",
            "R-01 L6->L4",
            "R-01 L4->L4",
            "R-01 reduced",
            "X-1 closed",
        ] {
            assert!(parse_move(bad).is_err(), "{bad} was read");
        }
    }

    #[test]
    fn releases_order_and_next_comes_last() {
        assert!(release_key("0.2.0") < release_key("0.10.0"));
        assert!(release_key("9.9.9") < release_key(NEXT));
        assert!(release_key("0.2").is_none() && release_key("v0.2.0").is_none());
        assert_eq!(quarter("2026-09-14"), "Q3-26");
        assert_eq!(quarter("2027-01-02"), "Q1-27");
    }

    #[test]
    fn stamping_touches_only_unreleased_versions() {
        let text = "release = \"next\"\n[[version]]\nn = 1\nrelease = \"0.1.0\"\n[[version]]\nn = 2\nrelease = \"next\"\n[view]\nrelease = \"next\"\n";
        let (out, n) = stamp(text, "0.2.0");
        assert_eq!(n, 1);
        assert_eq!(
            out,
            "release = \"next\"\n[[version]]\nn = 1\nrelease = \"0.1.0\"\n[[version]]\nn = 2\nrelease = \"0.2.0\"\n[view]\nrelease = \"next\"\n"
        );
    }

    #[test]
    fn a_version_block_reads_back_as_it_was_written() {
        let v = Version {
            n: 2,
            release: NEXT.into(),
            date: "2026-09-27".into(),
            by: "A. Person \"Ops\"".into(),
            about: vec!["math".into(), "model".into()],
            believed: "one thing".into(),
            tested: "a \\ test".into(),
            learned: "another,\nthing".into(),
            cost: "2 days".into(),
            changed: "the relation".into(),
            risks: vec!["R-01 L5->L4".into()],
            rests_on: "the new source".into(),
            breaks_if: "flight data disagrees".into(),
            relation: "y = a * x".into(),
            source: "somebody2026".into(),
        };
        let t: toml::Value = version_toml(&v).parse().unwrap();
        let b = &t["version"][0];
        assert_eq!(b["by"].as_str(), Some("A. Person \"Ops\""));
        assert_eq!(b["learned"].as_str(), Some("another,\nthing"));
        assert_eq!(b["tested"].as_str(), Some("a \\ test"));
        assert_eq!(b["risks"][0].as_str(), Some("R-01 L5->L4"));
    }
}
