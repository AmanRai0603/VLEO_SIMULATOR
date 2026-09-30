//! Saving an edit: who may make it, writing it whole, regenerating and gating
//! as one change, putting everything back on a refusal, and publishing.

use super::*;

/// Every name that is an assistant, not a person, lowercased.
///
/// A relation is confirmed by a person who has read it against its source — the
/// developer who applies a node form, or who writes the relation in. An
/// assistant may implement a relation a person supplied; it may never be the one
/// who supplied it. These are the names an assistant arrives under.
pub fn agent_identities(_root: &std::path::Path) -> Vec<String> {
    [
        "claude",
        "agent",
        "assistant",
        "copilot",
        "chatgpt",
        "gpt",
        "gemini",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Who this checkout says it is, from `git config user.name`.
///
/// NOT typed into the form. An attribution a person types is a name they chose
/// for that box; this is the name their commits already carry, so the sheet and
/// the history agree about who did it and nobody can put a colleague's name on
/// their own work by typing it.
///
/// It is not authentication and this does not pretend otherwise: anyone who can
/// edit a checkout can edit its git config. What it removes is the casual case —
/// typing somebody else's name into a text box — and it makes the sheet's
/// attribution and the commit's author the same claim rather than two.
pub fn git_identity(root: &std::path::Path) -> Result<String, String> {
    let out = std::process::Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(["config", "user.name"])
        .output()
        .map_err(|e| format!("git could not be run: {e}"))?;
    let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if name.is_empty() {
        return Err(
            "this checkout has no `git config user.name`, so there is no name to put against \
             the relation. Set it — `git config user.name \"Your Name\"` — and the sheet will \
             carry the same name your commits do. Nothing was written."
                .into(),
        );
    }
    Ok(name)
}

/// Whether this attribution is an assistant's, and so must never be written.
///
/// An assistant may never supply mathematics. Stated as a sentence that is a hope;
/// here it is a fact about what can reach the file — and it has to hold at every
/// face, or the browser becomes the way round a rule the terminal enforces.
pub fn refuse_agent_attribution(root: &std::path::Path, who: &str) -> Result<(), String> {
    let lower = who.trim().to_lowercase();
    if lower.is_empty() {
        return Err(
            "an attribution cannot be blank — it takes the name of a person who has \
                    read the relation against its source and is prepared to own it"
                .into(),
        );
    }
    for bad in agent_identities(root) {
        if lower == bad
            || lower.starts_with(&format!("{bad} "))
            || lower.contains(&format!("{bad}/"))
        {
            return Err(format!(
                "refused: '{who}' is an assistant's name. An assistant may never supply mathematics, and this \
                 field is the only thing that can tell whether one did. It takes the name of a \
                 person who has read the relation against its source and is prepared to own it. \
                 Nothing was written."
            ));
        }
    }
    Ok(())
}

/// Today, as the sheets write it (vleo_units::clock).
pub(crate) fn today() -> String {
    vleo_units::clock::today()
}

/// Put a name against the relation, replacing whatever was there.
///
/// WHEN THE RELATION CHANGES THE ATTRIBUTION MUST MOVE WITH IT. The old name
/// was against the old mathematics; leaving it on the new attributes work to
/// somebody who never saw it, which is worse than either having no name or
/// having the editor's.
pub(crate) fn stamp_relation(text: &str, who: &str) -> Result<String, String> {
    set(text, "confirmed_by", &format!("{who} / {}", today()))
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
pub(crate) const GENERATED: &[&str] = &[
    "model.rs",
    "contract.rs",
    "mod.rs",
    "evidence.rs",
    "page.html",
    "meta.json",
];

/// A temporary file then a rename, so a reader never sees half a sheet.
pub(crate) fn write_atomic(path: &std::path::Path, text: &str) -> Result<(), String> {
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
pub(crate) fn regenerate(
    sh: &crate::model::Sheet,
    tree: &crate::load::Tree,
) -> Result<usize, String> {
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

// ─── the repeated blocks ─────────────────────────────────────────────────────
//
// A sheet's inputs, algorithm steps, theory steps and assumptions are
// `[[table]]` arrays, and they are not fields. A field is replaced; a block is
// ADDED and REMOVED as well, and adding or removing one can break something a
// scalar edit never could — a numbered hole in generated Rust, a hand-written
// body that binds a name, an edge that closes a loop in the derivation graph.
//
// So they have their own table, their own operations and their own refusals.

/// Why this row cannot be published yet, or nothing.
///
/// A row's `state` is structural and may not be typed into a box: it decides
/// whether anything is generated from the row at all, and a row flipped to
/// `published` half-written generates four files that do not say anything.
/// Moving it is an ACTION with preconditions rather than a field with a value,
/// and this is the list of them.
pub fn unpublishable(sh: &Sheet) -> Vec<String> {
    let mut why = Vec::new();
    if !sh.is_seeded() {
        why.push(format!(
            "this row is already '{}'; publishing moves a seeded row and nothing else",
            sh.state
        ));
        return why;
    }
    let open = unfilled(sh);
    if !open.is_empty() {
        why.push(format!(
            "{} question(s) still block generation: {}",
            open.len(),
            open.join(", ")
        ));
    }
    if !sh.is_declared() {
        // V5, which is an assembly validation and so cannot be seen from the
        // per-node gate this save runs. A computed row with no inputs would
        // publish, pass its own gate, and fail the whole tree's.
        if sh.inputs.is_empty() {
            why.push(
                "a computed row reads something. With no inputs it is a constant written as a \
                 function, and the assembly validations refuse it"
                    .into(),
            );
        }
        if sh.steps.is_empty() {
            why.push(
                "a computed row has at least one algorithm step. With none, the generator has \
                 no hole to put the computation in and emits the row as a declared value of \
                 zero"
                    .into(),
            );
        }
    }
    why
}

/// Move a seeded row to published.
///
/// The one structural field this writes, and only through here. What makes it
/// safe is not that the check list is long but that it is the SAME check the
/// tree runs afterwards: the preconditions above are the assembly validations a
/// per-node gate cannot see, and the save runs the assembly validations too. A
/// row that gets through both is a row that would have got through a terminal.
pub fn publish(root: &std::path::Path, id: &str, base: &str) -> Saved {
    let tree = match crate::load::load_all(root) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(format!("the tree does not load: {e}")),
    };
    let Some(sh) = tree.sheets.get(id) else {
        return Saved::Refused(format!("no node '{id}'"));
    };
    let why = unpublishable(sh);
    if !why.is_empty() {
        return Saved::Refused(format!(
            "this row is not ready to publish — {}. Nothing was written",
            why.join("; ")
        ));
    }
    let path = sh.dir.join("node.toml");
    let before = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(format!("{}: {e}", path.display())),
    };
    if base != file_hash(&before) {
        return Saved::Stale {
            current: file_hash(&before),
        };
    }
    // Textual, like everything else here, and through `assignments` so a
    // `state` inside somebody's paragraph is not mistaken for the key.
    let hits = assignments(&before, "", "state");
    let Some((s0, s1, comment)) = hits.into_iter().next() else {
        return Saved::Refused(
            "this sheet has no `state =` line to move. Every seeded row has one, so this sheet \
             is malformed and not something a form should paper over"
                .into(),
        );
    };
    let mut after = String::with_capacity(before.len() + 16);
    after.push_str(&before[..s0]);
    after.push_str(&format!("state = \"published\"{comment}\n"));
    after.push_str(&before[s1..]);
    // The whole tree, because publishing a row puts it in front of every
    // assembly validation for the first time.
    commit_edit(root, id, &path, &before, after, true)
}
