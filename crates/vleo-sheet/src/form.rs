//! The declaration form — what a sheet still needs, and why each answer matters.
//!
//! Nine questions, in the order a person is asked them. `xtask declare` prints
//! them for a terminal and the daemon serves them to the web face, so a person
//! filling a row in a browser is asked exactly what a person at a terminal is
//! asked. A second copy of this list in JavaScript would drift the first week
//! somebody added a field, and the drift would be invisible: both forms would
//! look complete.
//!
//! It lives here because this crate is the single implementation of what a
//! sheet *means*. `unfilled` is the function the generators refuse on, so the
//! form and the generator cannot disagree about what a finished sheet is.

use crate::model::Sheet;

/// Which of the nine required fields are still blank.
///
/// `docs` refuses to emit a scaffold while any of these is open: the signature
/// needs the type and the unit, and a bound with no reason is a guard the next
/// person deletes. So this is the one list, read by the generator and by both
/// forms.
pub fn unfilled(sh: &Sheet) -> Vec<&'static str> {
    let mut missing = Vec::new();
    for (name, v) in [
        ("label", &sh.label),
        ("question", &sh.question),
        ("expression", &sh.expression),
        ("source", &sh.source),
        ("type", &sh.ty),
        ("unit", &sh.unit),
        ("symbol", &sh.symbol),
        ("reason_lower", &sh.reason_lower),
        ("reason_upper", &sh.reason_upper),
    ] {
        if v.trim().is_empty() {
            missing.push(name);
        }
    }
    missing
}

/// One question on the form.
pub struct Ask {
    pub field: &'static str,
    /// The question, as a person is asked it.
    pub ask: &'static str,
    /// What cannot be emitted without it. A field with no stated consequence is
    /// a field somebody fills with anything to make the form go green.
    pub why: &'static str,
    pub open: bool,
}

/// The nine questions, with the current answer's state.
///
/// Errors when a field is blank and does not block generation, which would mean
/// this form and `docs` disagree about what a finished sheet is. Two lists that
/// must agree are two lists that will not, so the disagreement is a hard error
/// rather than a difference nobody notices.
pub fn asks(sh: &Sheet) -> Result<Vec<Ask>, String> {
    const SPEC: &[(&str, &str, &str)] = &[
        (
            "label",
            "what is this row called, in the tree",
            "the page title and every reference to it",
        ),
        (
            "question",
            "what one question does it answer",
            "an equation with no question gets reused for the wrong thing",
        ),
        (
            "expression",
            "what is the relation",
            "the algorithm, and what a reviewer compares against the source",
        ),
        (
            "source",
            "cited where — book, paper, page",
            "this is the claim everything else rests on",
        ),
        (
            "symbol",
            "what is the answer's symbol",
            "the binding name in the generated signature",
        ),
        (
            "type",
            "what quantity is it",
            "the signature; a dimensional error has to fail to compile",
        ),
        (
            "unit",
            "in what unit",
            "the conversion at every face boundary",
        ),
        (
            "reason_lower",
            "why is the lower bound there",
            "a guard whose reason is not written gets deleted by the next person",
        ),
        (
            "reason_upper",
            "why is the upper bound there",
            "the same, at the other end",
        ),
    ];
    let blocking = unfilled(sh);
    let mut out = Vec::new();
    for (field, ask, why) in SPEC {
        let open = blocking.contains(field);
        if !open && value(sh, field).trim().is_empty() {
            return Err(format!(
                "'{field}' is blank and does not block generation — the form and docs \
                 disagree about what a finished sheet is"
            ));
        }
        out.push(Ask {
            field,
            ask,
            why,
            open,
        });
    }
    Ok(out)
}

/// What the sheet currently says for one form field.
pub fn value<'a>(sh: &'a Sheet, field: &str) -> &'a str {
    match field {
        "label" => &sh.label,
        "question" => &sh.question,
        "expression" => &sh.expression,
        "source" => &sh.source,
        "symbol" => &sh.symbol,
        "type" => &sh.ty,
        "unit" => &sh.unit,
        "reason_lower" => &sh.reason_lower,
        "reason_upper" => &sh.reason_upper,
        _ => "",
    }
}

/// A JSON string body, escaped. Smaller to write than to depend on.
fn jq(v: &str) -> String {
    let mut o = String::with_capacity(v.len() + 2);
    o.push('"');
    for c in v.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// The form as JSON, for a face to render.
///
/// `structural` is included so the form can SHOW those fields and refuse to
/// edit them. Changing a parent or an order moves the tree and renumbers its
/// neighbours; that is a developer's act at a terminal, not a text box.
///
/// `sheet_hash` is what a later save sends back to prove which version it
/// started from, so two people editing one row cannot overwrite each other
/// silently.
pub fn json(sh: &Sheet) -> Result<String, String> {
    let asks = asks(sh)?;
    let mut o = String::from("{\n");
    o.push_str(&format!("  \"id\": {},\n", jq(&sh.id)));
    o.push_str(&format!("  \"label\": {},\n", jq(&sh.label)));
    o.push_str(&format!(
        "  \"sheet_hash\": {},\n",
        jq(&crate::short_hex(sh.sheet_hash))
    ));
    // What a save must send back. See `file_hash`: the sheet hash is not it.
    o.push_str(&format!(
        "  \"file_hash\": {},\n",
        jq(&std::fs::read_to_string(sh.dir.join("node.toml"))
            .map(|t| file_hash(&t))
            .unwrap_or_default())
    ));
    o.push_str(&format!("  \"criticality\": {},\n", jq(&sh.criticality)));
    o.push_str("  \"structural\": {\n");
    let st = [
        ("kind", sh.kind.clone()),
        ("subsystem", sh.subsystem.clone()),
        ("parent", sh.parent.clone()),
        ("owner", sh.owner.clone()),
        ("tier", sh.tier.clone()),
        ("state", sh.state.clone()),
        ("layer", sh.layer.to_string()),
        ("order", sh.order.to_string()),
    ];
    for (i, (k, v)) in st.iter().enumerate() {
        o.push_str(&format!(
            "    {}: {}{}\n",
            jq(k),
            jq(v),
            if i + 1 == st.len() { "" } else { "," }
        ));
    }
    o.push_str("  },\n  \"fields\": [\n");
    for (i, a) in asks.iter().enumerate() {
        o.push_str(&format!(
            "    {{\"field\": {}, \"ask\": {}, \"why\": {}, \"value\": {}, \"open\": {}}}{}\n",
            jq(a.field),
            jq(a.ask),
            jq(a.why),
            jq(value(sh, a.field)),
            a.open,
            if i + 1 == asks.len() { "" } else { "," }
        ));
    }
    o.push_str("  ],\n");
    // Neither of these blocks generation, and both decide whether the row
    // answers at all, so they are reported apart from the nine rather than
    // mixed in with them.
    o.push_str(&format!(
        "  \"derivation\": {{\"wanted\": {}, \"present\": {}}},\n",
        !sh.steps.is_empty(),
        !sh.steps.is_empty() && !sh.theory.is_empty()
    ));
    o.push_str(&format!(
        "  \"attribution\": {{\"wanted\": {}, \"who\": {}}},\n",
        !sh.expression.trim().is_empty(),
        jq(&sh.relation_by)
    ));
    o.push_str(&format!(
        "  \"open\": {}\n}}\n",
        asks.iter().filter(|a| a.open).count()
    ));
    Ok(o)
}

/// A hash of the sheet's bytes, for detecting a concurrent edit.
///
/// NOT `sheet_hash`, and the difference matters. `sheet_hash` covers what a
/// reader would call the node's meaning and deliberately leaves out formatting,
/// comments and notes, so tidying a sentence does not invalidate every artefact
/// downstream. That makes it exactly wrong for this job: of the nine fields this
/// form writes, `label`, `symbol`, `reason_lower` and `reason_upper` are all
/// outside it, so two people editing a bound's reason would both see the same
/// `sheet_hash` and the second would overwrite the first without either being
/// told. This moves whenever the file moves.
pub fn file_hash(text: &str) -> String {
    crate::short_hex(crate::fnv1a(text))
}

/// Where each form field lives in the file: its table, and its key.
///
/// An empty table means a top-level key.
pub fn place(field: &str) -> Option<(&'static str, &'static str)> {
    Some(match field {
        "label" => ("", "label"),
        "question" => ("question", "text"),
        "expression" => ("maths", "expression"),
        "source" => ("maths", "source"),
        "confirmed_by" => ("maths", "confirmed_by"),
        "symbol" => ("output", "symbol"),
        "type" => ("output", "type"),
        "unit" => ("output", "unit"),
        "reason_lower" => ("output", "reason_lower"),
        "reason_upper" => ("output", "reason_upper"),
        _ => return None,
    })
}

/// The fields a face may never write, and why.
///
/// Not a blocklist of names but the reason each one is refused, because a
/// reader who is told "no" and not "why" goes looking for a way round.
pub fn structural(field: &str) -> Option<&'static str> {
    Some(match field {
        "id" | "folder" => "the identifier is the folder and the variable name; renaming it is a move",
        "parent" => "the parent is the tree's shape — moving a row moves everyone who reads it",
        "order" => "order decides position among siblings, and the block may be packed solid; \
                    inserting renumbers its neighbours",
        "layer" => "the layer decides which contract the row sits under",
        "kind" => "kind decides whether the row declares a value or computes one, which changes \
                   what is generated for it",
        "subsystem" | "owner" => "ownership is generated into CODEOWNERS and decides who reviews it",
        "tier" | "state" => "both change what the gate demands of the row",
        _ => return None,
    })
}

/// Replace one field's value in a sheet's text, leaving everything else alone.
///
/// TEXTUAL, never a TOML round-trip. These sheets carry more comment than
/// content and every comment is somebody's reason; a serialiser would silently
/// drop the lot. The same choice `xtask confirm` makes, for the same reason.
///
/// Refuses rather than guesses: an absent key, a duplicated one, or a value
/// written as a multi-line string are all reported instead of being patched
/// approximately. A sheet edited approximately is worse than one not edited.
pub fn set(text: &str, field: &str, value: &str) -> Result<String, String> {
    let Some((table, key)) = place(field) else {
        return Err(format!("'{field}' is not a field this form writes"));
    };
    // The window this key must be found in: from its table header to the next
    // one. Without it, `source` under [maths] and a `source` under some other
    // table are the same search.
    let (from, to) = if table.is_empty() {
        (0, text.find("\n[").unwrap_or(text.len()))
    } else {
        let header = format!("\n[{table}]\n");
        let Some(h) = text.find(&header) else {
            return Err(format!("this sheet has no [{table}] table to write {key} into"));
        };
        let start = h + header.len();
        let end = text[start..]
            .find("\n[")
            .map(|i| start + i)
            .unwrap_or(text.len());
        (start, end)
    };
    let window = &text[from..to];

    // Every line in the window that assigns this key.
    //
    // A commented-out assignment needs no guard: the key extracted from
    // `# unit = "Foot"` is `# unit`, which is not `unit`, so it never matches. A
    // check for it was here and nothing could reach it — removed rather than
    // left looking load-bearing.
    //
    // A key-looking line INSIDE a multi-line body does match, and that is what
    // the duplicate refusal below is really protecting: it counts two and
    // refuses, rather than replacing a line in somebody's prose.
    let mut hits = Vec::new();
    let mut at = from;
    for line in window.split_inclusive('\n') {
        let t = line.trim_start();
        if let Some(eq) = t.find('=') {
            if t[..eq].trim() == key {
                hits.push((at, at + line.len(), line));
            }
        }
        at += line.len();
    }
    if hits.is_empty() {
        return Err(format!(
            "no `{key} =` in {} — this form replaces a value that is already there, it does not \
             decide where a new key belongs",
            if table.is_empty() { "the sheet's head" } else { table }
        ));
    }
    if hits.len() > 1 {
        return Err(format!(
            "`{key}` is assigned {} times in [{table}] — refusing to guess which one is meant",
            hits.len()
        ));
    }
    let (s0, s1, line) = hits[0];
    // A multi-line string is a body somebody laid out on purpose. Replacing it
    // by line would truncate it at the first newline and leave the rest as
    // stray TOML, so it is refused by name.
    let rhs = line.split_once('=').map(|x| x.1.trim()).unwrap_or("");
    if rhs.starts_with("\"\"\"") && !(rhs.len() > 5 && rhs.ends_with("\"\"\"")) {
        return Err(format!(
            "`{key}` is a multi-line string. Editing one through this form would truncate it; \
             edit the sheet directly"
        ));
    }
    let mut o = String::with_capacity(text.len() + value.len());
    o.push_str(&text[..s0]);
    o.push_str(&format!("{key} = {}\n", toml_quote(value)));
    o.push_str(&text[s1..]);
    Ok(o)
}

/// A TOML string literal. Prefers a basic string, falls back to a literal one
/// where the value has backslashes worth keeping as typed.
fn toml_quote(v: &str) -> String {
    if v.contains('\n') {
        // A form field that has become multi-line is written as one, so the
        // file stays parseable rather than losing the tail.
        return format!("\"\"\"\n{}\"\"\"", if v.ends_with('\n') { v.to_string() } else { format!("{v}\n") });
    }
    let mut o = String::from("\"");
    for c in v.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// Every name that is an agent, not a person, lowercased.
///
/// Read from `agents/provenance.toml` so adding an agent to the roster adds it
/// here, plus the two generic words no attribution should ever be.
pub fn agent_identities(root: &std::path::Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(root.join("agents/provenance.toml")) else {
        return vec!["claude".into(), "agent".into()];
    };
    let Ok(v) = text.parse::<toml::Value>() else {
        return vec!["claude".into(), "agent".into()];
    };
    let mut out = Vec::new();
    for a in v
        .get("agent")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
    {
        for k in ["name", "id"] {
            if let Some(x) = a.get(k).and_then(|x| x.as_str()) {
                out.push(x.to_lowercase());
            }
        }
    }
    out.push("claude".into());
    out.push("agent".into());
    out
}

/// Whether this attribution is an agent's, and so must never be written.
///
/// An agent may never supply mathematics. Stated as a sentence that is a hope;
/// here it is a fact about what can reach the file — and it has to hold at every
/// face, or the browser becomes the way round a rule the terminal enforces.
pub fn refuse_agent_attribution(root: &std::path::Path, who: &str) -> Result<(), String> {
    let lower = who.trim().to_lowercase();
    if lower.is_empty() {
        return Err("an attribution cannot be blank — it takes the name of a person who has \
                    read the relation against its source and is prepared to own it"
            .into());
    }
    for bad in agent_identities(root) {
        if lower == bad
            || lower.starts_with(&format!("{bad} "))
            || lower.contains(&format!("{bad}/"))
        {
            return Err(format!(
                "refused: '{who}' is an agent. An agent may never supply mathematics, and this \
                 field is the only thing that can tell whether one did. It takes the name of a \
                 person who has read the relation against its source and is prepared to own it. \
                 Nothing was written."
            ));
        }
    }
    Ok(())
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
///      not an agent's
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
pub fn save(
    root: &std::path::Path,
    id: &str,
    field: &str,
    value: &str,
    base: &str,
    by: &str,
) -> Saved {
    if let Some(why) = structural(field) {
        return Saved::Refused(format!("'{field}' is not editable here: {why}"));
    }
    if place(field).is_none() {
        return Saved::Refused(format!("'{field}' is not a field this form writes"));
    }
    if std::process::Command::new("rustfmt")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| !s.success())
        .unwrap_or(true)
    {
        return Saved::Refused(
            "rustfmt is not on the path. The generated Rust is formatted before it is compared, \
             so saving without it would leave the tree failing its own regeneration check. \
             Nothing was written."
                .into(),
        );
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
    // An agent may never supply mathematics, at any face.
    if field == "expression" || field == "confirmed_by" {
        if let Err(e) = refuse_agent_attribution(root, by) {
            return Saved::Refused(e);
        }
    }

    let after = match set(&before, field, value) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(e),
    };
    if let Err(e) = write_atomic(&path, &after) {
        return Saved::Refused(e);
    }

    // From here on, a failure has to put the old sheet back.
    // Restoring the sheet is not enough on its own: once the artefacts have
    // been regenerated from the rejected edit, putting only node.toml back
    // leaves the tree failing its own regeneration check — the exact state this
    // whole path exists to avoid. So the artefacts are regenerated from the
    // restored sheet too.
    let restore = |e: String| -> Saved {
        let _ = write_atomic(&path, &before);
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
    let failed: Vec<String> = crate::gate::gate_node(sh, &tree)
        .iter()
        .filter(|c| c.failed())
        .map(|c| match &c.verdict {
            crate::gate::Verdict::Fail(w) => format!("{}: {w}", c.name),
            _ => c.name.to_string(),
        })
        .collect();
    if !failed.is_empty() {
        return restore(format!("the gate refuses it — {}", failed.join("; ")));
    }
    Saved::Ok {
        file_hash: std::fs::read_to_string(&path)
            .map(|t| file_hash(&t))
            .unwrap_or_default(),
        sheet_hash: crate::short_hex(sh.sheet_hash),
        regenerated: n,
    }
}

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
        let same = std::fs::read_to_string(&p).map(|o| o == text).unwrap_or(false);
        if !same {
            std::fs::write(&p, &text).map_err(|e| format!("{}: {e}", p.display()))?;
            n += 1;
        }
    }
    Ok(n)
}
