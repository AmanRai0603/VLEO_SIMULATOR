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

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use vleo_core::graph::{Kind, State};

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
pub fn case_inputs() -> Vec<Input> {
    Vleo::default_case().map(inputs).unwrap_or_default()
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
    let mut o = String::new();
    o.push_str("# VLEO multipayload — the case, one row per input.\n");
    o.push_str("# Fill in `value` in the unit shown; leave it blank for the default.\n");
    o.push_str("# Upload this file on the Inputs page, or pass it with `vleo run --inputs`.\n");
    o.push_str("# Lines starting with # are ignored. Columns may be in any order; id and value are needed.\n");
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
}

impl Reading {
    pub fn ok(&self) -> bool {
        self.refused.is_empty()
    }
}

/// Read a case CSV against the inputs the tree has now.
pub fn read_csv(text: &str) -> Reading {
    let all = case_inputs();
    let mut r = Reading::default();
    let mut header: Option<Vec<String>> = None;
    for (n, raw) in text.lines().enumerate() {
        let line = n + 1;
        let t = raw.trim().trim_start_matches('\u{feff}');
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
            r.refused.push((line, id, why));
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
        let unit = col("unit");
        if !unit.is_empty() && unit != inp.unit {
            r.refused.push((
                line,
                id,
                format!(
                    "is in {} here and the file says {unit}; values are read in the row's own unit",
                    if inp.unit.is_empty() {
                        "no unit"
                    } else {
                        inp.unit
                    }
                ),
            ));
            continue;
        }
        let v = col("value");
        if v.is_empty() {
            continue;
        }
        let Ok(shown) = v.parse::<f64>() else {
            r.refused
                .push((line, id, format!("value '{v}' is not a number")));
            continue;
        };
        let si = shown * inp.factor;
        if let Some(why) = inp.refusal(si) {
            r.refused.push((line, id, why));
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
    r.defaulted = all.len() - r.set.len().min(all.len());
    r
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
