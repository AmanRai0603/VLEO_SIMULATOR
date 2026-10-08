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
