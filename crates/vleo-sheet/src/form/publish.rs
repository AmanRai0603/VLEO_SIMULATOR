//! The declaration form: publishing a row.

use super::*;

// ─── publishing a row ────────────────────────────────────────────────────────

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
