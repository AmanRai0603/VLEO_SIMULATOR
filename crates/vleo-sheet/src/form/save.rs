//! The declaration form: checking an answer, saving it, and regenerating the row.

use super::*;

/// The value as it must be written into the file, or why it cannot be.
///
/// THE GATE DOES NOT DO THIS. It asks whether a field is blank, and a sheet's
/// `[output] type` is emitted verbatim into the generated signature — so a type
/// that is not a real quantity produces `Result<Nonsense, Fault>`, which does
/// not compile. A face with a text box could write that, regenerate, pass the
/// gate and report success, leaving the repository not building. The gate cannot
/// catch it because the gate never compiles anything.
///
/// It also NORMALISES, which is why it gives back the value rather than a
/// verdict. A bound typed as `40` is written `40.0`: TOML reads the first as an
/// integer, and while the loader takes either, a form that turned every float in
/// the tree into an integer on the way past would be rewriting 1396 sheets for
/// nothing. A number that does not parse is refused here rather than written and
/// silently read back as zero.
pub fn normalise(field: &str, value: &str) -> Result<String, String> {
    let Some(f) = self::field(field) else {
        return Err(format!("'{field}' is not a field this form writes"));
    };
    let v = value.trim();
    match f.shape {
        Shape::Number => {
            let n: f64 = v.parse().map_err(|_| {
                format!(
                    "'{v}' is not a number. It is written into the sheet unquoted and into the \
                     generated guard as an f64; anything else would be read back as zero"
                )
            })?;
            if !n.is_finite() {
                return Err(format!(
                    "'{v}' is not finite. A bound that is not a number cannot guard anything"
                ));
            }
            Ok(format!("{n:?}"))
        }
        Shape::Count => v
            .parse::<u32>()
            .map(|n| n.to_string())
            .map_err(|_| format!("'{v}' is not a whole number of at least zero")),
        Shape::Choice(options) => {
            if options.contains(&v) {
                Ok(v.to_string())
            } else {
                Err(format!(
                    "'{v}' is not one of: {}. This is a closed set, not a label",
                    options.join(", ")
                ))
            }
        }
        Shape::Quantity => {
            if crate::is_quantity_name(v) {
                Ok(v.to_string())
            } else {
                Err(format!(
                    "'{v}' is not a quantity this system has. The type is written straight into \
                     the generated signature, so one that does not exist stops the tree \
                     compiling. One of: {}",
                    vleo_units::QUANTITIES.join(", ")
                ))
            }
        }
        Shape::UnitName => {
            if crate::unit_exists(v) {
                Ok(v.to_string())
            } else {
                Err(format!(
                    "'{v}' is not a unit this system knows, so nothing could convert it at a \
                     face boundary. See vleo_units::Unit for the ones that exist."
                ))
            }
        }
        // A row id is checked against the tree by the gate, which is the only
        // place the whole graph is visible. Refusing the obviously impossible
        // here saves a write and a rollback for a typo with a space in it.
        Shape::RowId => {
            if v.is_empty() || v.contains(char::is_whitespace) {
                Err(format!(
                    "'{v}' is not a row id — an id is one word, and whether it resolves is the \
                     gate's question"
                ))
            } else {
                Ok(v.to_string())
            }
        }
        Shape::Line => {
            if v.contains('\n') {
                Err(format!(
                    "'{field}' is one line. What was sent has {} of them; if the answer needs a \
                     paragraph it belongs in a field that holds one",
                    v.lines().count()
                ))
            } else {
                Ok(v.to_string())
            }
        }
        // Prose keeps its internal newlines and loses only trailing blank ones,
        // because the layout is somebody's.
        Shape::Prose => Ok(value.trim_end().trim_start_matches('\n').to_string()),
        // Code keeps even the first line's indentation.
        Shape::Code => Ok(code_text(value)),
        Shape::Inputs => inputs_text(value),
    }
}

/// Code as it is kept: every line as typed, blank lines at either end gone,
/// line endings made one kind.
pub(super) fn code_text(value: &str) -> String {
    let v = value.replace("\r\n", "\n");
    let v = v.trim_end();
    let first = v
        .char_indices()
        .find(|(_, c)| *c != '\n')
        .map(|(i, _)| i)
        .unwrap_or(v.len());
    // Back to the start of the first non-blank line, keeping its indent.
    let start = v[..first].rfind('\n').map(|i| i + 1).unwrap_or(0);
    v[start..].to_string()
}

/// A case's inputs as the inline table the sheet holds: `{ h = 250000.0 }`,
/// each value a number, the names in order.
pub(super) fn inputs_text(value: &str) -> Result<String, String> {
    let v = value.trim();
    let doc: toml::Value = format!("x = {v}").parse().map_err(|_| {
        format!("'{v}' is not a set of inputs — write them as {{ name = number, … }}")
    })?;
    let t = doc
        .get("x")
        .and_then(|x| x.as_table())
        .ok_or_else(|| format!("'{v}' is not a set of inputs"))?;
    let mut parts = Vec::new();
    for (k, x) in t {
        let n = x
            .as_float()
            .or_else(|| x.as_integer().map(|i| i as f64))
            .ok_or_else(|| format!("the input «{k}» is not a number"))?;
        if !n.is_finite() {
            return Err(format!("the input «{k}» is not finite"));
        }
        parts.push(format!("{k} = {n:?}"));
    }
    Ok(format!("{{ {} }}", parts.join(", ")))
}

/// Whether a value is one this field may hold. `normalise` without the value.
pub fn value_allowed(field: &str, value: &str) -> Result<(), String> {
    normalise(field, value).map(|_| ())
}

/// What a save did, or why it did nothing.
pub enum Saved {
    /// Written, regenerated and gated. Carries the row's new sheet hash.
    Ok {
        /// What the next save must send back.
        file_hash: String,
        /// Whether the row's MEANING moved — a caption or a bound's reason can
        /// change without this changing, and that is deliberate.
        sheet_hash: String,
        regenerated: usize,
    },
    /// The editor started from a version that is no longer current. Carries the
    /// hash it should have started from, so the face can show what changed
    /// rather than overwrite it.
    Stale { current: String },
    /// Refused, and nothing was written.
    Refused(String),
}

/// Change one field of one sheet, and leave the tree consistent or untouched.
///
/// The whole transaction, so it can be tested without an HTTP server:
///
///   1  the field is one the form writes, and is not structural
///   2  `base` is the hash the editor started from — a stale one is refused
///      rather than overwritten, which is what makes two editors safe
///   3  an edit to the relation carries an attribution, and that attribution is
///      not an assistant's
///   4  the sheet is written atomically: a temporary file, then a rename, so a
///      reader never sees half a sheet
///   5  the row's artefacts are regenerated and the gate is run on it
///   6  ANY failure after the write restores the previous sheet. A tree left
///      half-edited by a browser is the thing this must never do
///
/// `rustfmt` must be on the path, because the generated Rust is formatted before
/// it is compared and a fallback to unformatted text would leave the tree
/// failing its own regeneration diff. Refused up front rather than discovered
/// afterwards.
pub fn save(root: &std::path::Path, id: &str, field: &str, value: &str, base: &str) -> Saved {
    if let Some(why) = structural(field) {
        return Saved::Refused(format!("'{field}' is not editable here: {why}"));
    }
    if place(field).is_none() {
        return Saved::Refused(format!("'{field}' is not a field this form writes"));
    }
    if let Err(e) = value_allowed(field, value) {
        return Saved::Refused(e);
    }
    if let Some(why) = rustfmt_refusal() {
        return Saved::Refused(why);
    }

    let tree = match crate::load::load_all(root) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(format!("the tree does not load: {e}")),
    };
    let Some(sh) = tree.sheets.get(id) else {
        return Saved::Refused(format!("no node '{id}'"));
    };
    let path = sh.dir.join("node.toml");
    let before = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(format!("{}: {e}", path.display())),
    };
    let current = file_hash(&before);
    if base != current {
        return Saved::Stale { current };
    }
    // An assistant may never supply mathematics, at any face. The name is the
    // checkout's own — see `git_identity` — so it is the same one the commit
    // will carry rather than whatever was typed into a box.
    if field == "expression" || field == "confirmed_by" {
        let who = match git_identity(root) {
            Ok(w) => w,
            Err(e) => return Saved::Refused(e),
        };
        if let Err(e) = refuse_agent_attribution(root, &who) {
            return Saved::Refused(e);
        }
    }

    let after = match set(&before, field, value) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(e),
    };
    // AND THE NAME IS WRITTEN, NOT ONLY CHECKED. The identity was verified
    // above and then went nowhere, so a relation saved through the face came
    // out with `confirmed_by` still blank — a relation with nobody against it,
    // indistinguishable from one an assistant wrote, which is the exact thing that
    // field exists to tell apart.
    let after = if field == "expression" {
        let who = match git_identity(root) {
            Ok(w) => w,
            Err(e) => return Saved::Refused(e),
        };
        match stamp_relation(&after, &who) {
            Ok(t) => t,
            Err(e) => return Saved::Refused(e),
        }
    } else {
        after
    };
    commit_edit(root, id, &path, &before, after, false)
}

/// Why an edit cannot be written here, if `rustfmt` is not on the path.
///
/// The generated Rust is formatted before it is compared, and a fallback to
/// unformatted text would leave the tree failing its own regeneration diff. So
/// every edit refuses up front rather than discovering it afterwards.
pub(crate) fn rustfmt_refusal() -> Option<String> {
    let missing = std::process::Command::new("rustfmt")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| !s.success())
        .unwrap_or(true);
    missing.then(|| {
        "rustfmt is not on the path. The generated Rust is formatted before it is compared, \
         so saving without it would leave the tree failing its own regeneration check. \
         Nothing was written."
            .to_string()
    })
}

/// Write the edit, regenerate the row, gate it — or put everything back.
///
/// The tail of every edit, whether it moved one field or a whole repeated
/// block. Shared because the failure path is the part that matters and a second
/// copy of it would be a second chance to get the restore wrong.
///
/// `whole_tree` runs the assembly validations as well. They are not free — they
/// walk 1396 rows — so they run for the one edit that can break a row which is
/// not this one: an input is an edge, and whether an edge closes a loop in the
/// derivation graph is a question only the whole graph can answer.
///
/// ANY FAILURE AFTER THE WRITE RESTORES THE PREVIOUS SHEET. A tree left
/// half-edited by a browser is the thing this must never do.
pub(crate) fn commit_edit(
    root: &std::path::Path,
    id: &str,
    path: &std::path::Path,
    before: &str,
    after: String,
    whole_tree: bool,
) -> Saved {
    // Which generated files the row had BEFORE the edit. Taken before anything
    // is written, because it is the answer to "what may a restore delete".
    let dir = path.parent().unwrap_or(std::path::Path::new("."));
    let existed: Vec<std::path::PathBuf> = GENERATED
        .iter()
        .map(|n| dir.join(n))
        .filter(|p| p.exists())
        .collect();

    if let Err(e) = write_atomic(path, &after) {
        return Saved::Refused(e);
    }

    // Restoring the sheet is not enough on its own: once the artefacts have
    // been regenerated from the rejected edit, putting only node.toml back
    // leaves the tree failing its own regeneration check — the exact state this
    // whole path exists to avoid. So the artefacts are regenerated from the
    // restored sheet too.
    //
    // AND WHAT THE EDIT GENERATED THAT WAS NOT THERE BEFORE IS REMOVED. A
    // refused publish showed why: publishing a seeded row makes the generator
    // write its model, contract, module and evidence for the first time, and
    // regenerating the restored — still seeded — sheet writes only its page. The
    // four new files stayed behind under a message saying nothing had changed,
    // and the next "put these edits on a branch" would have committed them.
    // Only a generated file the row did not have before is removed; one that
    // existed is regenerated, which keeps whatever Rust its holes hold.
    let restore = |e: String| -> Saved {
        let _ = write_atomic(path, before);
        for n in GENERATED {
            let p = dir.join(n);
            if p.exists() && !existed.contains(&p) {
                let _ = std::fs::remove_file(&p);
            }
        }
        let put_back = crate::load::load_all(root)
            .ok()
            .and_then(|t| t.sheets.get(id).map(|s| regenerate(s, &t)));
        let lost = match put_back {
            Some(Err(w)) => format!(" — AND THE ARTEFACTS COULD NOT BE PUT BACK: {w}"),
            None => " — AND THE TREE WOULD NOT RELOAD TO PUT THE ARTEFACTS BACK".into(),
            Some(Ok(_)) => String::new(),
        };
        Saved::Refused(format!(
            "{e} — the sheet was restored, nothing changed{lost}"
        ))
    };
    let tree = match crate::load::load_all(root) {
        Ok(t) => t,
        Err(e) => return restore(format!("the edit does not parse: {e}")),
    };
    let Some(sh) = tree.sheets.get(id) else {
        return restore("the row vanished from the tree after the edit".into());
    };
    // REGENERATE BEFORE GATING, not after. One of the gate's own checks is that
    // every artefact matches what the sheet generates, so gating a freshly
    // written sheet whose artefacts are still the old ones fails every time —
    // and fails for a reason that has nothing to do with the edit.
    let n = match regenerate(sh, &tree) {
        Ok(n) => n,
        Err(e) => return restore(e),
    };
    let mut failed: Vec<String> = crate::gate::gate_node(sh, &tree)
        .iter()
        .filter(|c| c.failed())
        .map(|c| match &c.verdict {
            crate::gate::Verdict::Fail(w) => format!("{}: {w}", c.name),
            _ => c.name.to_string(),
        })
        .collect();
    if whole_tree && failed.is_empty() {
        failed = crate::gate::validate_tree(&tree)
            .iter()
            .filter(|c| c.failed())
            .map(|c| match &c.verdict {
                crate::gate::Verdict::Fail(w) => format!("{}: {w}", c.name),
                _ => c.name.to_string(),
            })
            .collect();
    }
    if !failed.is_empty() {
        return restore(format!("the gate refuses it — {}", failed.join("; ")));
    }
    Saved::Ok {
        file_hash: std::fs::read_to_string(path)
            .map(|t| file_hash(&t))
            .unwrap_or_default(),
        sheet_hash: crate::short_hex(sh.sheet_hash),
        regenerated: n,
    }
}

/// Every file the per-row generators write. `regenerate` writes the first four
/// only for a published row; the last two for every row.
const GENERATED: &[&str] = &[
    "model.rs",
    "contract.rs",
    "mod.rs",
    "evidence.rs",
    "page.html",
    "meta.json",
];

/// A temporary file then a rename, so a reader never sees half a sheet.
fn write_atomic(path: &std::path::Path, text: &str) -> Result<(), String> {
    let tmp = path.with_extension("toml.writing");
    std::fs::write(&tmp, text).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
}

/// `regenerate`, for a test that has to put a row back after editing it.
pub fn regenerate_for_test(
    sh: &crate::model::Sheet,
    tree: &crate::load::Tree,
) -> Result<usize, String> {
    regenerate(sh, tree)
}

/// The six per-node generators, for one row. The same set `xtask docs` writes.
fn regenerate(sh: &crate::model::Sheet, tree: &crate::load::Tree) -> Result<usize, String> {
    let holes = crate::load::read_holes(&sh.dir);
    let gaps = crate::emit::gap_pass(sh, &holes);
    let artefacts: Vec<(&str, String)> = if sh.is_seeded() {
        vec![
            ("page.html", crate::page::fragment(sh, &holes, tree)),
            ("meta.json", crate::emit::meta_json(sh, &gaps)),
        ]
    } else {
        vec![
            ("model.rs", crate::emit::model_rs(sh, &holes)),
            ("contract.rs", crate::emit::contract_rs(sh)),
            ("mod.rs", crate::emit::mod_rs(sh)),
            ("evidence.rs", crate::emit::evidence_rs(sh)),
            ("page.html", crate::page::fragment(sh, &holes, tree)),
            ("meta.json", crate::emit::meta_json(sh, &gaps)),
        ]
    };
    let mut n = 0;
    for (name, text) in artefacts {
        let text = if name.ends_with(".rs") {
            crate::gate::formatted(&text)
        } else {
            text
        };
        let p = sh.dir.join(name);
        let same = std::fs::read_to_string(&p)
            .map(|o| o == text)
            .unwrap_or(false);
        if !same {
            std::fs::write(&p, &text).map_err(|e| format!("{}: {e}", p.display()))?;
            n += 1;
        }
    }
    // The node's method lives in the kernel, not beside the sheet, and the
    // model this just wrote calls it — so it is written in the same step.
    n += crate::emit::sync_methods(tree)?;
    Ok(n)
}
