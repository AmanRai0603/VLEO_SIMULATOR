//! The inputs a person sets, and the CSV they travel in.
//!
//! One case, every input it has: each declared, published row with room to
//! move, its default — the value declared on its own sheet — and whether it
//! describes what the CUSTOMER chooses or the CONDITION the design flies in
//! (`cases/multipayload.toml` lists the conditions; everything else is
//! customer).
//!
//! The values a person works with never live in the repository. They are a
//! CSV: downloaded from the application, filled in anywhere a table can be
//! edited, uploaded again, and stored by the face that received it. This
//! module is the one reading and the one writing of that CSV, so the daemon
//! and the command line cannot disagree about what a file means.
//!
//! EVERY ROW IS CHECKED AND NONE IS CORRECTED. An unknown input, a unit that is
//! not the row's, a value outside the declared range: each is refused by name,
//! with the reason, and a file with any refusal is not applied at all. Keeping
//! the good rows of a bad file would be a run on values nobody asked for, and
//! the fifth rule says a refusal is never a substitution.
//!
//! A FILE FROM AN OLDER TOOL IS CARRIED OVER, NOT REFUSED. The inputs change as
//! the tree grows — a row is added, retired, re-ranged — so a CSV kept from last
//! month may name an input this tree no longer has. Every file the tool writes
//! says which set of inputs it was written for, in a `#! template` line. When
//! that is not the set the tree has now, the file is read as an upgrade: every
//! value that still applies is carried, every input the file never knew runs
//! at its default, and every value that cannot be carried is SET ASIDE — named,
//! with why, and written back into the upgraded file so nothing is lost without
//! a record. A file with no template line, or with this tree's, is read
//! strictly: there a name that is not an input is a mistake in the file, and a
//! mistake is refused rather than set aside.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use vleo_core::graph::{Kind, State};
use vleo_units::Unit;

use crate::tables::{self, CaseDef, NODES, VARS};
use crate::Vleo;

/// Which half of the case an input sits in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Customer,
    Condition,
}

impl Group {
    pub fn name(self) -> &'static str {
        match self {
            Group::Customer => "customer",
            Group::Condition => "condition",
        }
    }
}

/// One input: what it is, where it may move, and what it is when nobody moved it.
#[derive(Clone, Debug)]
pub struct Input {
    pub var: u16,
    pub id: &'static str,
    pub label: &'static str,
    pub symbol: &'static str,
    pub group: Group,
    /// The unit a person reads and types, and the factor that takes it to SI.
    pub unit: &'static str,
    pub factor: f64,
    pub lo: f64,
    pub hi: f64,
    pub why_lo: &'static str,
    pub why_hi: &'static str,
    /// SI. The value declared on the row's own sheet.
    pub default: f64,
}

impl Input {
    /// SI to the unit a person reads.
    pub fn shown(&self, si: f64) -> f64 {
        si / self.factor
    }
    /// Whether an SI value is inside the declared range, and if not, why not.
    pub fn refusal(&self, si: f64) -> Option<String> {
        if !si.is_finite() {
            return Some("not a number".to_string());
        }
        if si < self.lo {
            return Some(format!(
                "below {}{}{}",
                num(self.shown(self.lo)),
                self.unit_after(),
                reason(self.why_lo)
            ));
        }
        if si > self.hi {
            return Some(format!(
                "above {}{}{}",
                num(self.shown(self.hi)),
                self.unit_after(),
                reason(self.why_hi)
            ));
        }
        None
    }
    /// The unit as it follows a number in a sentence: nothing for a ratio.
    fn unit_after(&self) -> String {
        match self.unit {
            "" | "-" => String::new(),
            u => format!(" {u}"),
        }
    }
}

fn reason(r: &str) -> String {
    if r.is_empty() {
        String::new()
    } else {
        format!(" — {r}")
    }
}

/// A number as it is written into the CSV: the shortest form that reads back
/// to the same f64, so a download uploaded unchanged changes nothing.
pub fn num(v: f64) -> String {
    format!("{v}")
}

/// Every input of a case, customers first, each group in id order.
///
/// The same test the face applies to offer a field: declared, published, and
/// with room to move. A retired row is not an input any more, and a row whose
/// upper bound does not exceed its lower has nowhere to go.
pub fn inputs(case: &CaseDef) -> Vec<Input> {
    let mut out = Vec::new();
    for (i, def) in NODES.iter().enumerate() {
        if def.kind != Kind::Declared || def.state != State::Published {
            continue;
        }
        let Some(&o) = def.outputs.first() else {
            continue;
        };
        let var = &VARS[o as usize];
        if var.limit.upper <= var.limit.lower {
            continue;
        }
        let mut val = [0.0f64; tables::MAX_OUTPUTS];
        let default = match (tables::DISPATCH[i])(&[], &mut val[..def.outputs.len()]) {
            Ok(_) => val[0],
            Err(_) => continue,
        };
        out.push(Input {
            var: o,
            id: var.id,
            label: var.label,
            symbol: var.symbol,
            group: if case.conditions.contains(&o) {
                Group::Condition
            } else {
                Group::Customer
            },
            unit: var.unit.symbol(),
            factor: var.unit.si_factor(),
            lo: var.limit.lower,
            hi: var.limit.upper,
            why_lo: var.limit.reason_lower,
            why_hi: var.limit.reason_upper,
            default,
        });
    }
    out.sort_by(|a, b| {
        (a.group == Group::Condition, a.id).cmp(&(b.group == Group::Condition, b.id))
    });
    out
}

/// The inputs of the one case.
///
/// FIXED FOR THE LIFE OF THE PROGRAM, so worked out once where there is a
/// standard library to hold it. The daemon reads the saved case on every run
/// it answers, and walking all 663 declared rows three times per request made
/// each run slow enough that a panel making hundreds of them visibly lagged.
pub fn case_inputs() -> Vec<Input> {
    #[cfg(feature = "std")]
    {
        static ALL: std::sync::OnceLock<Vec<Input>> = std::sync::OnceLock::new();
        ALL.get_or_init(|| Vleo::default_case().map(inputs).unwrap_or_default())
            .clone()
    }
    #[cfg(not(feature = "std"))]
    {
        Vleo::default_case().map(inputs).unwrap_or_default()
    }
}

/// Which set of inputs a file was written for: every input's id, unit and
/// range, hashed. It moves when an input is added, retired, re-ranged or
/// changes its unit — exactly the changes that can stop a saved value applying
/// — and not when a default or a group moves, which a saved value survives.
pub fn template_of(inputs: &[Input]) -> String {
    let mut rows: Vec<String> = inputs
        .iter()
        .map(|i| format!("{}\t{}\t{}\t{}\n", i.id, i.unit, num(i.lo), num(i.hi)))
        .collect();
    rows.sort();
    // FNV-1a: small, stable across platforms and toolchains, and enough to
    // tell two sets of inputs apart. Nothing trusts it for more than that.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in rows.concat().bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{:012x}", h & 0xffff_ffff_ffff)
}

/// The template the tree has now. Fixed for the life of the program, like
/// the inputs it is worked out from.
pub fn template() -> String {
    #[cfg(feature = "std")]
    {
        static NOW: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        NOW.get_or_init(|| template_of(&case_inputs())).clone()
    }
    #[cfg(not(feature = "std"))]
    {
        template_of(&case_inputs())
    }
}

/// A value an upgrade could not carry into the inputs the tree has now.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SetAside {
    pub id: String,
    /// As the old file wrote it, in its own unit, so it can be typed back.
    pub value: String,
    pub unit: String,
    pub why: String,
}

/// What an upgrade did: which template the values were written for, what could
/// not be carried, which inputs are new, and where the file as it was is kept.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Upgrade {
    /// The template the file was written for, or `unversioned`.
    pub from: String,
    /// Inputs this tree has that the file did not name; they run at their defaults.
    pub new: Vec<String>,
    pub set_aside: Vec<SetAside>,
    /// The file as it was before it was carried over, when a stored case was.
    pub backup: Option<String>,
}

// ---------------------------------------------------------------------------
// writing

/// The case as a CSV: every input, with the value it runs at when one is set
/// and a blank when it runs at its default.
///
/// Blank rather than the default repeated, so a file says at a glance which
/// inputs somebody changed — and a default that later moves on the sheet is
/// followed, rather than frozen at whatever it was on the day of the download.
pub fn csv(values: &[(String, f64)]) -> String {
    csv_with(values, None)
}

/// The same, with the record of an upgrade written into it: what was carried
/// from, what is new, and every value set aside, so a value that could not be
/// carried is still in the file a person keeps.
pub fn csv_with(values: &[(String, f64)], upgrade: Option<&Upgrade>) -> String {
    let mut o = String::new();
    o.push_str("# VLEO multipayload — the case, one row per input.\n");
    o.push_str("# Fill in `value` in the unit shown; leave it blank for the default.\n");
    o.push_str("# Upload this file on the Inputs page, or pass it with `vleo run --inputs`.\n");
    o.push_str("# Lines starting with # are ignored. Columns may be in any order; id and value are needed.\n");
    o.push_str("# Keep the `#! template` line: it is how a newer tool carries this file over.\n");
    o.push_str(&format!("#! template {}\n", template()));
    if let Some(u) = upgrade {
        o.push_str(&format!("#! upgraded-from {}\n", u.from));
        if let Some(b) = &u.backup {
            o.push_str(&format!("#! backup {b}\n"));
        }
        for n in &u.new {
            o.push_str(&format!("#! new {n}\n"));
        }
        for a in &u.set_aside {
            o.push_str(&format!(
                "#! set-aside {},{},{},{}\n",
                field(&a.id),
                field(&a.value),
                field(&a.unit),
                field(&a.why)
            ));
        }
    }
    o.push_str("group,id,name,value,unit,default,min,max\n");
    for i in case_inputs() {
        let set = values.iter().find(|(k, _)| k == i.id).map(|(_, v)| *v);
        o.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            i.group.name(),
            i.id,
            field(i.label),
            set.map(|v| num(i.shown(v))).unwrap_or_default(),
            field(i.unit),
            num(i.shown(i.default)),
            num(i.shown(i.lo)),
            num(i.shown(i.hi)),
        ));
    }
    o
}

/// A CSV field, quoted when it has to be.
fn field(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

// ---------------------------------------------------------------------------
// reading

/// What a file says, row by row, before anything is applied.
#[derive(Clone, Debug, Default)]
pub struct Reading {
    /// Inputs the file sets, in SI, in file order.
    pub set: Vec<(String, f64)>,
    /// Of those, how many differ from their default.
    pub changed: usize,
    /// Inputs the file leaves at their default: blank, or not in the file.
    pub defaulted: usize,
    /// `(line, id, why)`, for every row that cannot be applied.
    pub refused: Vec<(usize, String, String)>,
    /// The `#! template` the file names, if it names one.
    pub template: Option<String>,
    /// Written for another set of inputs than the tree has now: read as an
    /// upgrade, and — for a stored case — to be written again in this one.
    pub outdated: bool,
    /// What an upgrade did: this read's, when `outdated`, or the record of an
    /// earlier one that the file carries.
    pub upgrade: Option<Upgrade>,
}

impl Reading {
    pub fn ok(&self) -> bool {
        self.refused.is_empty()
    }
}

/// Read a case CSV against the inputs the tree has now.
pub fn read_csv(text: &str) -> Reading {
    let all = case_inputs();
    let now = template();
    let mut r = Reading::default();
    let mut header: Option<Vec<String>> = None;
    let mut note = Upgrade::default();
    let mut noted = false;
    let mut named: Vec<String> = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        let line = n + 1;
        let t = raw.trim().trim_start_matches('\u{feff}');
        // The tool's own lines: which template, and the record of an upgrade.
        if let Some(m) = t.strip_prefix("#!") {
            let m = m.trim();
            let (k, v) = m.split_once(' ').unwrap_or((m, ""));
            let v = v.trim();
            match k {
                "template" => {
                    r.template = Some(v.to_string());
                    r.outdated = v != now;
                }
                "upgraded-from" => {
                    note.from = v.to_string();
                    noted = true;
                }
                "backup" => note.backup = Some(v.to_string()),
                "new" => note.new.push(v.to_string()),
                "set-aside" => {
                    let c = split(v);
                    let at = |k: usize| c.get(k).map(|s| s.trim().to_string()).unwrap_or_default();
                    note.set_aside.push(SetAside {
                        id: at(0),
                        value: at(1),
                        unit: at(2),
                        why: at(3),
                    });
                }
                _ => {}
            }
            continue;
        }
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let cells = split(t);
        let Some(h) = &header else {
            let h: Vec<String> = cells
                .iter()
                .map(|c| c.trim().to_ascii_lowercase())
                .collect();
            if !h.iter().any(|c| c == "id") || !h.iter().any(|c| c == "value") {
                r.refused.push((
                    line,
                    String::new(),
                    "the first row that is not a comment must name the columns, including `id` \
                     and `value` — download the template to see the format"
                        .to_string(),
                ));
                return r;
            }
            header = Some(h);
            // A record of an earlier upgrade describes the file it is in; a
            // file being upgraded now starts a record of its own.
            if r.outdated {
                note = Upgrade {
                    from: r.template.clone().unwrap_or_default(),
                    ..Default::default()
                };
                noted = true;
            }
            continue;
        };
        let col = |name: &str| {
            h.iter()
                .position(|c| c == name)
                .and_then(|k| cells.get(k))
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        };
        let id = col("id");
        if id.is_empty() {
            r.refused.push((line, id, "a row with no id".to_string()));
            continue;
        }
        named.push(id.clone());
        let unit = col("unit");
        let v = col("value");
        // In an upgrade, what cannot be carried is set aside with its value;
        // in a strict read it is refused.
        let mut aside = |r: &mut Reading, id: String, why: String| {
            if r.outdated {
                note.set_aside.push(SetAside {
                    id,
                    value: v.clone(),
                    unit: unit.clone(),
                    why,
                });
            } else {
                r.refused.push((line, id, why));
            }
        };
        let Some(inp) = all.iter().find(|i| i.id == id) else {
            let why = match Vleo::find(&id) {
                Some(v) => match NODES[VARS[v as usize].producer as usize].kind {
                    Kind::Declared => "is not an input that can be set: it is retired, or has no \
                                       range to move in"
                        .to_string(),
                    _ => "is computed from its inputs, so a value for it would be overwritten \
                          the moment it is evaluated"
                        .to_string(),
                },
                None => "is not a row in this tree".to_string(),
            };
            // A blank value in an old file asked for nothing, so an input it
            // names that has since gone loses nothing: it is not set aside.
            if r.outdated && v.is_empty() {
                continue;
            }
            aside(&mut r, id, why);
            continue;
        };
        if r.set.iter().any(|(k, _)| k == &id) {
            r.refused.push((
                line,
                id,
                "appears twice; which one is meant cannot be guessed".into(),
            ));
            continue;
        }
        // The factor from the file's unit to SI: the row's own, or — in an
        // upgrade, where the tool itself may have changed the row's unit — the
        // unit the file names, when it measures the same thing. Never between
        // two dimensionless units: a ratio and a decibel share a dimension and
        // are not interchangeable.
        let mut factor = inp.factor;
        if r.outdated && v.is_empty() {
            continue;
        }
        if !unit.is_empty() && unit != inp.unit {
            let same_kind = r
                .outdated
                .then(|| unit_named(&unit))
                .flatten()
                .zip(unit_named(inp.unit))
                .filter(|(a, b)| a.dim() == b.dim() && a.dim() != vleo_units::unit::Dim::NONE);
            match same_kind {
                Some((from, _)) => factor = from.si_factor(),
                None => {
                    let shown = if inp.unit.is_empty() {
                        "no unit"
                    } else {
                        inp.unit
                    };
                    aside(
                        &mut r,
                        id,
                        format!(
                            "is in {shown} here and the file says {unit}; values are read in the \
                             row's own unit"
                        ),
                    );
                    continue;
                }
            }
        }
        if v.is_empty() {
            continue;
        }
        let Ok(shown) = v.parse::<f64>() else {
            r.refused
                .push((line, id, format!("value '{v}' is not a number")));
            continue;
        };
        let si = shown * factor;
        if let Some(why) = inp.refusal(si) {
            let why = if r.outdated {
                format!("{why}; the range has changed since the file was written")
            } else {
                why
            };
            aside(&mut r, id, why);
            continue;
        }
        if si != inp.default {
            r.changed += 1;
        }
        r.set.push((id, si));
    }
    if header.is_none() {
        r.refused.push((
            0,
            String::new(),
            "the file has no rows — download the template to see the format".to_string(),
        ));
    }
    if r.outdated {
        note.new = all
            .iter()
            .filter(|i| !named.iter().any(|n| n == i.id))
            .map(|i| i.id.to_string())
            .collect();
    }
    if noted {
        r.upgrade = Some(note);
    }
    r.defaulted = all.len() - r.set.len().min(all.len());
    r
}

/// A unit by the symbol a file writes it with.
fn unit_named(symbol: &str) -> Option<Unit> {
    Unit::NAMES
        .iter()
        .filter_map(|n| Unit::from_name(n))
        .find(|u| u.symbol() == symbol)
}

/// Values already in SI — from the Inputs page or a stored case — checked the
/// same way a file is. Stored values are re-checked on every read, because a
/// row can be retired or re-ranged after the case was saved.
pub fn check_values(values: &[(String, f64)]) -> Reading {
    let all = case_inputs();
    let mut r = Reading::default();
    for (n, (id, si)) in values.iter().enumerate() {
        match all.iter().find(|i| i.id == id) {
            None => r.refused.push((
                n + 1,
                id.clone(),
                "is no longer an input that can be set".to_string(),
            )),
            Some(inp) => match inp.refusal(*si) {
                Some(why) => r.refused.push((n + 1, id.clone(), why)),
                None => {
                    if *si != inp.default {
                        r.changed += 1;
                    }
                    r.set.push((id.clone(), *si));
                }
            },
        }
    }
    r.defaulted = all.len() - r.set.len().min(all.len());
    r
}

/// One CSV line into its fields, with quoted fields and doubled quotes.
pub fn split_csv(line: &str) -> Vec<String> {
    split(line)
}

fn split(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                cur.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => out.push(core::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}

// ---------------------------------------------------------------------------
// the stored case

/// The saved case, as the daemon and the command line both read it.
#[cfg(feature = "std")]
pub mod saved {
    use super::*;
    use std::path::{Path, PathBuf};

    /// The case a run is on, and what happened to it on the way in.
    #[derive(Clone, Debug, Default)]
    pub struct Saved {
        /// Whether a case is saved at all. None is every input at its default.
        pub stored: bool,
        pub reading: Reading,
        /// When the case needed carrying over and could not be written back —
        /// its values still apply for this run, and this says why the file
        /// on disk is still the old one.
        pub error: Option<String>,
    }

    /// Read the saved case, carrying it over first if the tool has changed
    /// under it.
    ///
    /// AFTER AN UPDATE A CASE STILL RUNS, AND NOTHING IN IT IS LOST QUIETLY.
    /// When the file was written for another set of inputs — or a row of it no
    /// longer applies — it is copied aside, untouched, beside itself, and
    /// written again in this tree's template: every value that still applies
    /// carried, every new input at its default, and every value that could not
    /// be carried set aside in the file by name, with why. The record stays in
    /// the file until the case is next saved, so every face can say what the
    /// update did, and the copy is there to go back to.
    pub fn load(path: &Path) -> Saved {
        // One request at a time: two reading an old case at once would each
        // carry it over, keep two copies aside, and one could read the file
        // while the other was half way through writing it back.
        let _one = one_at_a_time();
        // No case saved is every input at its default. Said directly rather
        // than by writing the blank template and reading it back, which cost a
        // millisecond on every run the daemon answered.
        let Ok(text) = std::fs::read_to_string(path) else {
            return Saved {
                stored: false,
                reading: Reading {
                    defaulted: case_inputs().len(),
                    template: Some(template()),
                    ..Default::default()
                },
                error: None,
            };
        };
        let r = read_csv(&text);
        if !r.outdated && r.refused.is_empty() {
            return Saved {
                stored: true,
                reading: r,
                error: None,
            };
        }
        let mut note = r.upgrade.clone().filter(|_| r.outdated).unwrap_or(Upgrade {
            from: r.template.clone().unwrap_or_else(|| "unversioned".into()),
            ..Default::default()
        });
        // A row refused in a stored file — a value edited by hand, or one from
        // before files carried a template — is set aside the same way.
        for (_, id, why) in &r.refused {
            note.set_aside.push(SetAside {
                id: id.clone(),
                value: String::new(),
                unit: String::new(),
                why: why.clone(),
            });
        }
        let backup = backup_path(path, &note.from);
        let kept = write_whole(&backup, &text);
        note.backup = Some(backup.display().to_string());
        let upgraded = csv_with(&r.set, Some(&note));
        let written = kept.and_then(|_| write_whole(path, &upgraded));
        let mut reading = read_csv(&upgraded);
        match written {
            Ok(()) => Saved {
                stored: true,
                reading,
                error: None,
            },
            Err(e) => {
                reading.upgrade = Some(note);
                Saved {
                    stored: true,
                    reading,
                    error: Some(format!(
                        "the case needed carrying over to this version of the tool and could \
                         not be written back to {}: {e}",
                        path.display()
                    )),
                }
            }
        }
    }

    /// Save a case — the text of a CSV this module wrote. Under the same lock
    /// as `load`, and whole: a reader sees the old file or the new one, never
    /// a file half way through being written.
    pub fn store(path: &Path, csv: &str) -> std::io::Result<()> {
        let _one = one_at_a_time();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        write_whole(path, csv)
    }

    /// Remove the saved case: every input back to its default. No case saved
    /// already is not an error.
    pub fn clear(path: &Path) -> std::io::Result<()> {
        let _one = one_at_a_time();
        match std::fs::remove_file(path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }

    fn one_at_a_time() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        // A request that panicked while holding it left nothing half done that
        // the next one needs to know about: every write below is whole.
        LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Written beside itself and renamed over, which replaces it in one step —
    /// the tool's one way of writing a person's data (vleo_data::write_whole).
    fn write_whole(path: &Path, text: &str) -> std::io::Result<()> {
        vleo_data::write_whole(path, text)
    }

    /// Beside the case, named for the template it was written for, and never
    /// over an earlier copy.
    fn backup_path(path: &Path, from: &str) -> PathBuf {
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "inputs".into());
        let dir = path.parent().unwrap_or(Path::new("."));
        let tag: String = from.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        let mut n = 1;
        loop {
            let name = if n == 1 {
                format!("{stem}.before-{tag}.csv")
            } else {
                format!("{stem}.before-{tag}-{n}.csv")
            };
            let p = dir.join(name);
            if !p.exists() {
                return p;
            }
            n += 1;
        }
    }
}
