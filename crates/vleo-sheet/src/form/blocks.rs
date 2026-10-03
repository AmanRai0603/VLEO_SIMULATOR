//! The declaration form: the repeated blocks: inputs, steps and assumptions, added and removed.

use super::*;

// ─── the repeated blocks ─────────────────────────────────────────────────────
//
// A sheet's inputs, algorithm steps, theory steps and assumptions are
// `[[table]]` arrays, and they are not fields. A field is replaced; a block is
// ADDED and REMOVED as well, and adding or removing one can break something a
// scalar edit never could — a numbered hole in generated Rust, a hand-written
// body that binds a name, an edge that closes a loop in the derivation graph.
//
// So they have their own table, their own operations and their own refusals.

/// One key inside a repeated block.
pub struct Column {
    pub key: &'static str,
    pub shape: Shape,
    pub ask: &'static str,
    /// A block with this blank says nothing.
    pub required: bool,
    /// Written by this module and never typed. A step's `number` is the
    /// identity of a numbered hole in `model.rs`; a person retyping it would
    /// reattach somebody's Rust to a different step.
    pub managed: bool,
}

/// Where a block may be added and removed.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Blocks {
    /// Added, edited and removed, subject to the checks in `save_block`.
    Free,
    /// Added at the end and removed from the end only.
    ///
    /// The blocks are numbered and the number is a hole identifier in generated
    /// Rust. Appending leaves every existing number where it is; inserting in
    /// the middle renumbers the ones after it, which moves each hand-written
    /// body onto a different step without changing a line of it. That is not an
    /// edit anybody would notice in review.
    EndOnly,
}

/// One repeated block of a sheet.
pub struct Array {
    pub name: &'static str,
    /// Its TOML path, which is what `[[…]]` carries.
    pub path: &'static str,
    pub label: &'static str,
    pub why: &'static str,
    pub columns: &'static [Column],
    pub blocks: Blocks,
    /// The table a first block goes after, when the sheet has none yet.
    pub after: &'static str,
    /// The group whose rows alone carry this block, or empty for every row.
    /// A risk is registered on a risk-register row and nowhere else.
    pub only_under: &'static str,
}

impl Array {
    /// Whether this block belongs on this row's form.
    pub fn applies(&self, sh: &Sheet) -> bool {
        self.only_under.is_empty() || sh.parent == self.only_under
    }
}

/// EVERY REPEATED BLOCK THIS FORM WRITES.
pub const ARRAYS: &[Array] = &[
    Array {
        name: "input",
        path: "input",
        label: "what this row reads",
        why: "the consumer declares what it expects, and assembly refuses a mismatch against \
              what the producer publishes. It is also the derivation graph: an input is an \
              edge, and the edges are what decide what runs before what",
        columns: &[
            Column {
                key: "binding",
                shape: Shape::Line,
                ask: "the name it goes by inside this row",
                required: true,
                managed: false,
            },
            Column {
                key: "var",
                shape: Shape::RowId,
                ask: "which row answers it",
                required: true,
                managed: false,
            },
            Column {
                key: "type",
                shape: Shape::Quantity,
                ask: "the quantity this row expects it to be",
                required: true,
                managed: false,
            },
        ],
        blocks: Blocks::Free,
        after: "output",
        only_under: "",
    },
    Array {
        name: "algorithm",
        path: "algorithm.step",
        label: "the steps",
        why: "each step becomes one numbered hole in the generated model, and the hole is \
              where a person writes the few typed lines that compose relations from the \
              kernel. The step's text is the comment above it",
        columns: &[
            Column {
                key: "number",
                shape: Shape::Count,
                ask: "",
                required: true,
                managed: true,
            },
            Column {
                key: "text",
                shape: Shape::Prose,
                ask: "what this step does",
                required: true,
                managed: false,
            },
            Column {
                key: "binds",
                shape: Shape::Line,
                ask: "the name its result is bound to",
                required: true,
                managed: false,
            },
            Column {
                key: "type",
                shape: Shape::Quantity,
                ask: "and its quantity",
                required: true,
                managed: false,
            },
        ],
        blocks: Blocks::EndOnly,
        after: "output",
        only_under: "",
    },
    Array {
        name: "theory",
        path: "theory.step",
        label: "how the relation was arrived at",
        why: "the expression says what the relation is; these say how it was got to. A \
              relation an assistant invented carries a citation just as convincingly, and the \
              derivation is what a reviewer reads instead of taking the citation's word",
        columns: &[
            Column {
                key: "text",
                shape: Shape::Prose,
                ask: "the step of the argument",
                required: true,
                managed: false,
            },
            Column {
                key: "math",
                shape: Shape::Line,
                ask: "and the line of maths, if there is one",
                required: false,
                managed: false,
            },
        ],
        blocks: Blocks::Free,
        after: "theory",
        only_under: "",
    },
    Array {
        name: "assumption",
        path: "assumption",
        label: "what has to be true for this to hold",
        why: "an assumption nobody wrote down is one nobody can check. Each says what it \
              assumes AND the case it fails in, because an assumption with no failure case \
              is a sentence rather than a warning",
        columns: &[
            Column {
                key: "text",
                shape: Shape::Prose,
                ask: "what is assumed",
                required: true,
                managed: false,
            },
            Column {
                key: "fails_when",
                shape: Shape::Prose,
                ask: "and when that stops being true",
                required: true,
                managed: false,
            },
        ],
        blocks: Blocks::Free,
        // NOT `theory`. 1322 of 1396 rows have no `[theory]` at all, so a first
        // assumption on one of them had nothing to go after and was refused —
        // for a table that is optional. `[maths]` is on every row, and
        // `insert_point` still prefers the derivation when there is one, so the
        // reading order of an authored sheet is kept where it exists.
        after: "maths",
        only_under: "",
    },
    Array {
        name: "risk",
        path: "risk",
        label: "the risks this row registers",
        why: "a risk is registered once, here, and moved only by node versions — a risk is \
              reduced because something was tested, and the version that tested it is the \
              record of how. Which risk-register row holds a risk says what kind it is",
        columns: &[
            Column {
                key: "id",
                shape: Shape::Line,
                ask: "its id: R- and a number, never reused",
                required: true,
                managed: false,
            },
            Column {
                key: "title",
                shape: Shape::Line,
                ask: "what could go wrong, in a line",
                required: true,
                managed: false,
            },
            Column {
                key: "level",
                shape: Shape::Choice(crate::derisk::LEVELS),
                ask: "how serious it is now, L1 (least) to L5",
                required: true,
                managed: false,
            },
            Column {
                key: "owner",
                shape: Shape::Line,
                ask: "who owns it",
                required: true,
                managed: false,
            },
            Column {
                key: "since",
                shape: Shape::Line,
                ask: "registered on (YYYY-MM-DD)",
                required: true,
                managed: false,
            },
            Column {
                key: "why",
                shape: Shape::Prose,
                ask: "what it would cost if it happened",
                required: true,
                managed: false,
            },
        ],
        blocks: Blocks::Free,
        after: "maths",
        only_under: crate::derisk::REGISTER,
    },
    Array {
        name: "case",
        path: "case",
        label: "your test cases",
        why: "inputs and the answer YOUR code gave, or that your code refuses them. They \
              decide: the method is run on each one before you send the form, and the code \
              the tool generates must reproduce every one before the node is connected to \
              anything. At least three answers and one refusal",
        columns: &[
            Column {
                key: "label",
                shape: Shape::Line,
                ask: "what this case is, in a few words — 'lowest altitude', 'a storm'",
                required: true,
                managed: false,
            },
            Column {
                key: "refuse",
                shape: Shape::Choice(YES_NO),
                ask: "must the node refuse this case",
                required: true,
                managed: false,
            },
            Column {
                key: "expect",
                shape: Shape::Number,
                ask: "the answer your code gave, in SI",
                required: false,
                managed: false,
            },
            Column {
                key: "tolerance",
                shape: Shape::Number,
                ask: "how close is close enough, as a fraction (1e-6 is one part in a million)",
                required: false,
                managed: false,
            },
            Column {
                key: "inputs",
                shape: Shape::Inputs,
                ask: "each input's value, in SI",
                required: true,
                managed: false,
            },
            Column {
                key: "also",
                shape: Shape::Inputs,
                ask: "only for a node that publishes several values: each published member's \
                      value your code gave, by its symbol, in SI",
                required: false,
                managed: false,
            },
            Column {
                key: "origin",
                shape: Shape::Choice(ORIGINS),
                ask: "where the answer came from: your own code, a hand calculation, a \
                      spreadsheet or a paper. Left out, it is your code",
                required: false,
                managed: false,
            },
        ],
        blocks: Blocks::Free,
        after: "maths",
        only_under: "",
    },
    Array {
        name: "flight",
        path: "flight",
        label: "flight software kept with this node",
        why: "on-board code that implements this node, stored with its test beside the \
              relation it implements, so the two are reviewed and versioned together",
        columns: &[
            Column {
                key: "name",
                shape: Shape::Line,
                ask: "its file name, as in the flight tree",
                required: true,
                managed: false,
            },
            Column {
                key: "language",
                shape: Shape::Line,
                ask: "its language",
                required: true,
                managed: false,
            },
            Column {
                key: "purpose",
                shape: Shape::Prose,
                ask: "what it does on board, and which part of this node it implements",
                required: true,
                managed: false,
            },
            Column {
                key: "code",
                shape: Shape::Code,
                ask: "the code",
                required: true,
                managed: false,
            },
            Column {
                key: "test_code",
                shape: Shape::Code,
                ask: "its test",
                required: false,
                managed: false,
            },
            Column {
                key: "test_result",
                shape: Shape::Prose,
                ask: "what the test gave when it was last run, and where",
                required: false,
                managed: false,
            },
        ],
        blocks: Blocks::Free,
        after: "maths",
        only_under: "",
    },
];

/// One repeated block, by name.
pub fn array(name: &str) -> Option<&'static Array> {
    ARRAYS.iter().find(|a| a.name == name)
}

/// One line of a sheet, and whether it is inside a multi-line string.
///
/// EVERYTHING TEXTUAL HERE NEEDS THIS. A line beginning `[` inside somebody's
/// paragraph is not a table header and a line reading `x = 1` inside one is not
/// an assignment; treating either as what it looks like edits a reason and
/// leaves the field alone.
pub(super) struct Row {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) prose: bool,
}

pub(super) fn scan(text: &str) -> Vec<Row> {
    let mut out = Vec::new();
    let mut at = 0;
    let mut open = false;
    for line in text.split_inclusive('\n') {
        let end = at + line.len();
        if open {
            out.push(Row {
                start: at,
                end,
                prose: true,
            });
            if line.contains("\"\"\"") {
                open = false;
            }
        } else {
            out.push(Row {
                start: at,
                end,
                prose: false,
            });
            open = key_and_rhs(line)
                .map(|(_, r)| opens_prose(r))
                .unwrap_or(false);
        }
        at = end;
    }
    out
}

/// The byte range of each `[[path]]` block, in the order they appear.
///
/// A block runs from its own header line to the next table header of any kind,
/// so whatever trails it — a blank line, a comment about the next one — travels
/// with it. That is what makes removing one leave a file somebody would have
/// written.
pub(super) fn array_blocks(text: &str, path: &str) -> Vec<(usize, usize)> {
    let header = format!("[[{path}]]");
    let mut out: Vec<(usize, usize)> = Vec::new();
    let mut open: Option<usize> = None;
    for r in scan(text) {
        if r.prose {
            continue;
        }
        let t = text[r.start..r.end].trim();
        if !t.starts_with('[') {
            continue;
        }
        if let Some(s) = open.take() {
            out.push((s, r.start));
        }
        if t == header {
            open = Some(r.start);
        }
    }
    if let Some(s) = open {
        out.push((s, text.len()));
    }
    out
}

/// Where a new block of this array goes.
///
/// After the last one of its kind, so a sheet reads in the order it was written.
/// When there are none, after the table the array belongs under — which for a
/// theory step means creating `[theory]` first, exactly as a scalar theory field
/// does.
fn insert_point(text: &str, a: &Array) -> Result<usize, Error> {
    if let Some((_, end)) = array_blocks(text, a.path).last() {
        return Ok(*end);
    }
    // An assumption reads after the derivation it qualifies, when there is one.
    if a.path == "assumption" {
        if let Some((_, end)) = array_blocks(text, "theory.step").last() {
            return Ok(*end);
        }
        if let Some((_, end)) = window(text, "theory") {
            return Ok(end);
        }
    }
    // The author's cases read after their code, which produced them; flight
    // software after the cases.
    if a.path == "case" || a.path == "flight" {
        if a.path == "flight" {
            if let Some((_, end)) = array_blocks(text, "case").last() {
                return Ok(*end);
            }
        }
        for t in ["author", "method"] {
            if let Some((_, end)) = window(text, t) {
                return Ok(end);
            }
        }
        if let Some((_, end)) = array_blocks(text, "theory.step").last() {
            return Ok(*end);
        }
    }
    // An algorithm step reads after the inputs it consumes.
    if a.path == "algorithm.step" {
        if let Some((_, end)) = array_blocks(text, "input").last() {
            return Ok(*end);
        }
    }
    let (_, end) = window(text, a.after).ok_or_else(|| {
        Error::new(
            ErrorKind::Malformed,
            format!("this sheet has no [{}] table to put it after", a.after),
        )
    })?;
    // `window` ends at the newline before the next header, so a block written at
    // that offset needs the newline back in front of it. `write_block` adds it.
    Ok(end)
}

/// One block's text, from its keys.
fn write_block(a: &Array, values: &[(&str, String)]) -> String {
    let mut o = format!("\n\n[[{}]]\n", a.path);
    for c in a.columns {
        if let Some((_, v)) = values.iter().find(|(k, _)| *k == c.key) {
            if v.trim().is_empty() && !c.required {
                continue;
            }
            o.push_str(&format!("{} = {}\n", c.key, written(&c.shape, v)));
        }
    }
    o
}

/// Change one repeated block, and leave the tree consistent or untouched.
///
/// `op` is `add`, `set` or `remove`. `values` carries the keys to write for an
/// add, or the one key and its value for a set.
///
/// This is the half that needs the tree: what the sheet already reads, and what
/// the hand-written hole bodies in `model.rs` depend on. The textual half is
/// `block_text`, which can be tested against a sheet in a string. The
/// transaction is `commit_edit`'s — the same stale-hash check, atomic write,
/// regenerate-then-gate and restore-on-any-failure that a scalar edit gets.
pub fn save_block(
    root: &std::path::Path,
    id: &str,
    name: &str,
    op: &str,
    index: usize,
    values: &[(&str, String)],
    base: &str,
) -> Saved {
    let Some(a) = array(name) else {
        return Saved::Refused(format!("'{name}' is not a repeated block this form writes"));
    };
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
    if base != file_hash(&before) {
        return Saved::Stale {
            current: file_hash(&before),
        };
    }

    // THE HOLES ARE THE THING A BLOCK EDIT CAN BREAK. A hole body is Rust
    // somebody wrote by hand, and the generated code around it is written from
    // these blocks: an input's `binding` is a parameter name, a step's `binds`
    // and `type` are the `let` the body has to satisfy. Change either under a
    // filled hole and the row stops compiling — which the gate cannot see,
    // because the gate never compiles anything.
    let holes = crate::load::read_holes(&sh.dir);
    let filled: Vec<(u32, &String)> = holes
        .iter()
        .filter(|(_, b)| !b.trim().is_empty())
        .map(|(n, b)| (*n, b))
        .collect();
    let mentions = |word: &str| -> Option<u32> {
        let word = word.trim();
        if word.is_empty() {
            return None;
        }
        filled
            .iter()
            .find(|(_, b)| {
                b.split(|c: char| !c.is_alphanumeric() && c != '_')
                    .any(|w| w == word)
            })
            .map(|(n, _)| *n)
    };
    let blocks = array_blocks(&before, a.path);
    let at = |i: usize| -> Option<(usize, usize)> { blocks.get(i).copied() };

    match op {
        "add" if a.path == "input" => {
            // V2 — each edge is declared once, by the end it changes. That is an
            // assembly validation, and this refuses it here so the reason names
            // the row rather than arriving as a rolled-back tree failure.
            if let Some((_, v)) = values.iter().find(|(k, _)| *k == "var") {
                if sh.inputs.iter().any(|i| &i.var == v) {
                    return Saved::Refused(format!(
                        "this row already reads '{v}'. Each edge is declared once, by the end it \
                         changes — a second declaration is two edges the graph cannot tell apart"
                    ));
                }
            }
        }
        "set" => {
            let Some((s0, s1)) = at(index) else {
                return Saved::Refused(no_such_block(a, blocks.len(), index));
            };
            // The keys a filled hole is written around.
            if let Some((key, _)) = values.first() {
                let guarded = (a.path == "input" && *key == "binding")
                    || (a.path == "algorithm.step" && matches!(*key, "binds" | "type"));
                if guarded {
                    let old = block_value(&before, s0, s1, key).unwrap_or_default();
                    if let Some(n) = mentions(&old) {
                        return Saved::Refused(format!(
                            "hole {n} is filled and its body uses `{}`. Changing it here would \
                             move the generated code around a body nobody has read since, and \
                             the gate never compiles anything, so nothing would catch it. Edit \
                             the hole and this together, in a checkout",
                            old.trim()
                        ));
                    }
                }
            }
        }
        "remove" => {
            let Some((s0, s1)) = at(index) else {
                return Saved::Refused(no_such_block(a, blocks.len(), index));
            };
            if a.path == "algorithm.step" {
                let n = (index + 1) as u32;
                if holes.get(&n).map(|b| !b.trim().is_empty()).unwrap_or(false) {
                    return Saved::Refused(format!(
                        "hole {n} has a body. Removing the step would delete the generated \
                         model's only home for it. Empty the hole first, in a checkout, so \
                         losing the Rust is its own reviewable change"
                    ));
                }
            }
            if a.path == "input" {
                let old = block_value(&before, s0, s1, "binding").unwrap_or_default();
                if let Some(n) = mentions(&old) {
                    return Saved::Refused(format!(
                        "hole {n} is filled and its body uses `{}`. Removing the input would \
                         take the parameter out of the generated signature and leave that body \
                         referring to nothing",
                        old.trim()
                    ));
                }
            }
        }
        _ => {}
    }

    let after = match block_text(&before, name, op, index, values) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(e.into()),
    };
    // AN EDGE IS THE ONE EDIT THAT CAN BREAK A ROW THAT IS NOT THIS ONE. The
    // per-node gate asks whether each input resolves and agrees on type; whether
    // the edge closes a loop in the derivation graph is a question about the
    // whole graph, and only the assembly validations can ask it.
    commit_edit(root, id, &path, &before, after, a.path == "input")
}

fn no_such_block(a: &Array, have: usize, index: usize) -> String {
    format!(
        "this row has {have} {} block(s); there is no number {}",
        a.name,
        index + 1
    )
}

/// The textual half of a block edit: what the sheet becomes.
///
/// Separate from `save_block` so it can be tested against a sheet in a string.
/// What is here is everything decidable from the file itself — the shapes, the
/// required keys, the managed number, where a new block goes. What needs the
/// tree or the hole bodies stays in `save_block`.
pub fn block_text(
    text: &str,
    name: &str,
    op: &str,
    index: usize,
    values: &[(&str, String)],
) -> Result<String, Error> {
    let a = array(name).ok_or_else(|| {
        Error::new(
            ErrorKind::Refused,
            format!("'{name}' is not a repeated block this form writes"),
        )
    })?;
    let blocks = array_blocks(text, a.path);
    match op {
        // ADDED AT THE END, ALWAYS, whatever the array. Inserting in the middle
        // is not offered rather than refused: for a numbered array it would move
        // every hand-written hole body after the insertion onto a different
        // step, and for the others the order is a reading order somebody can
        // rearrange in a checkout where the diff is legible.
        "add" => {
            // A case the node must refuse may give an input no node takes.
            let refusal = a.name == "case"
                && values
                    .iter()
                    .any(|(k, v)| *k == "refuse" && v.trim() == "yes");
            let mut vals: Vec<(&str, String)> = Vec::new();
            for c in a.columns {
                let v = if c.managed {
                    // The step number is this module's to assign: one past the
                    // last, which is the only value that leaves every existing
                    // hole where it is.
                    (blocks.len() + 1).to_string()
                } else {
                    values
                        .iter()
                        .find(|(k, _)| *k == c.key)
                        .map(|(_, v)| v.clone())
                        .unwrap_or_default()
                };
                if v.trim().is_empty() {
                    if c.required {
                        return Err(Error::new(
                            ErrorKind::Refused,
                            format!("a {} block needs `{}`: {}", a.name, c.key, c.ask),
                        ));
                    }
                    continue;
                }
                vals.push((
                    c.key,
                    if refusal && matches!(c.shape, Shape::Inputs) {
                        refused_inputs_text(&v)?
                    } else {
                        normalise_column(c, &v)?
                    },
                ));
            }
            // A theory step needs the table its path sits under, exactly as a
            // scalar theory field does.
            let text = if a.path.starts_with("theory.") {
                ensure_table(text, "theory")?
            } else {
                text.to_string()
            };
            let at = insert_point(&text, a)?;
            let mut o = String::with_capacity(text.len() + 256);
            o.push_str(text[..at].trim_end_matches('\n'));
            o.push_str(&write_block(a, &vals));
            let tail = text[at..].trim_start_matches('\n');
            if !tail.is_empty() {
                o.push('\n');
                o.push_str(tail);
            }
            Ok(o)
        }
        "set" => {
            let &(s0, s1) = blocks.get(index).ok_or_else(|| {
                Error::new(ErrorKind::Refused, no_such_block(a, blocks.len(), index))
            })?;
            let (key, value) = values
                .first()
                .ok_or_else(|| Error::new(ErrorKind::Refused, "nothing to set"))?;
            let c = a.columns.iter().find(|c| c.key == *key).ok_or_else(|| {
                Error::new(
                    ErrorKind::Refused,
                    format!("'{key}' is not a key of a {} block", a.name),
                )
            })?;
            if c.managed {
                return Err(Error::new(
                    ErrorKind::Refused,
                    format!(
                        "`{}` is written by the form and never typed: it is the identity of a \
                     numbered hole in the generated model, and retyping it would reattach \
                     somebody's Rust to a different step",
                        c.key
                    ),
                ));
            }
            if c.required && value.trim().is_empty() {
                return Err(Error::new(
                    ErrorKind::Refused,
                    format!(
                        "a {} block needs `{}`: {}. Remove the whole block rather than emptying \
                     one of its keys",
                        a.name, c.key, c.ask
                    ),
                ));
            }
            let refusal =
                a.name == "case" && block_value(text, s0, s1, "refuse").as_deref() == Some("yes");
            let v = if refusal && matches!(c.shape, Shape::Inputs) {
                refused_inputs_text(value)?
            } else {
                normalise_column(c, value)?
            };
            // A case stops being a refusal only with inputs a node can take.
            if a.name == "case" && c.key == "refuse" && v != "yes" {
                if let Some(k) = non_finite_input(text, s0, s1) {
                    return Err(Error::new(
                        ErrorKind::Refused,
                        format!(
                            "this case gives «{k}» a value that is not finite, which only a case \
                             the node must refuse may do. Give it a number first"
                        ),
                    ));
                }
            }
            set_in_block(text, s0, s1, c, &v)
        }
        "remove" => {
            let &(s0, s1) = blocks.get(index).ok_or_else(|| {
                Error::new(ErrorKind::Refused, no_such_block(a, blocks.len(), index))
            })?;
            if a.blocks == Blocks::EndOnly && index + 1 != blocks.len() {
                return Err(Error::new(
                    ErrorKind::Refused,
                    format!(
                        "only the last {} block can be removed. Removing one from the middle \
                     renumbers every step after it, and each number is a hole holding \
                     somebody's Rust",
                        a.name
                    ),
                ));
            }
            let mut o = String::with_capacity(text.len());
            o.push_str(text[..s0].trim_end_matches('\n'));
            o.push('\n');
            let tail = text[s1..].trim_start_matches('\n');
            if !tail.is_empty() {
                o.push('\n');
                o.push_str(tail);
            }
            Ok(o)
        }
        _ => Err(Error::new(
            ErrorKind::Refused,
            format!("'{op}' is not add, set or remove"),
        )),
    }
}

/// What one block currently says for a key.
///
/// Read through the TOML parser rather than by line, because this is a READ and
/// the parser is the authority on what a block says. It is writing that has to
/// be textual, to keep the comments a serialiser would drop.
fn block_value(text: &str, s0: usize, s1: usize, key: &str) -> Option<String> {
    let body = text[s0..s1].split_once('\n')?.1;
    let v: toml::Value = body.parse().ok()?;
    v.get(key).and_then(|x| x.as_str()).map(String::from)
}

/// The first of a block's inputs that is not a finite number, if one is.
fn non_finite_input(text: &str, s0: usize, s1: usize) -> Option<String> {
    let body = text[s0..s1].split_once('\n')?.1;
    let v: toml::Value = body.parse().ok()?;
    v.get("inputs")?
        .as_table()?
        .iter()
        .find(|(_, x)| x.as_float().is_some_and(|f| !f.is_finite()))
        .map(|(k, _)| k.clone())
}

/// Replace or add one key inside one block.
fn set_in_block(
    text: &str,
    s0: usize,
    s1: usize,
    c: &Column,
    value: &str,
) -> Result<String, Error> {
    let body = &text[s0..s1];
    // The block's own lines, minus its header, with prose bodies skipped.
    let mut hit: Option<(usize, usize, String)> = None;
    let mut open: Option<(usize, bool)> = None;
    for r in scan(body) {
        let line = &body[r.start..r.end];
        match open {
            Some((start, wanted)) => {
                if let Some(i) = line.find("\"\"\"") {
                    if wanted {
                        hit = Some((start, r.end, trailing_comment(&line[i + 3..])));
                    }
                    open = None;
                }
            }
            None => {
                if let Some((name, rhs)) = key_and_rhs(line) {
                    if opens_prose(rhs) {
                        open = Some((r.start, name == c.key));
                    } else if name == c.key {
                        if hit.is_some() {
                            return Err(Error::new(
                                ErrorKind::Malformed,
                                format!(
                                "`{}` is assigned twice in this block — refusing to guess which \
                                 one is meant",
                                c.key
                            ),
                            ));
                        }
                        hit = Some((r.start, r.end, trailing_comment(rhs)));
                    }
                }
            }
        }
    }
    let written = format!("{} = {}", c.key, written(&c.shape, value));
    let mut o = String::with_capacity(text.len() + value.len());
    match hit {
        Some((a, b, comment)) => {
            o.push_str(&text[..s0 + a]);
            o.push_str(&written);
            o.push_str(&comment);
            o.push('\n');
            o.push_str(&text[s0 + b..]);
        }
        None => {
            // Absent and optional — `math` on a theory step is the case. Added
            // directly under the block's header, which is the one place in a
            // block that is unambiguous.
            let head = body.find('\n').map(|i| s0 + i + 1).ok_or_else(|| {
                Error::new(ErrorKind::Refused, "this block has no body to write into")
            })?;
            o.push_str(&text[..head]);
            o.push_str(&written);
            o.push('\n');
            o.push_str(&text[head..]);
        }
    }
    Ok(o)
}

/// A column's value as it must be written, or why it cannot be.
fn normalise_column(c: &Column, value: &str) -> Result<String, Error> {
    let v = value.trim();
    match c.shape {
        Shape::Count => v.parse::<u32>().map(|n| n.to_string()).map_err(|_| {
            Error::new(
                ErrorKind::Refused,
                format!("`{}` is a whole number; '{v}' is not", c.key),
            )
        }),
        Shape::Quantity if !crate::is_quantity_name(v) => Err(Error::new(
            ErrorKind::Missing,
            format!(
                "'{v}' is not a quantity this system has. It is written straight into the \
             generated signature, so one that does not exist stops the tree compiling. One \
             of: {}",
                vleo_units::QUANTITIES.join(", ")
            ),
        )),
        Shape::RowId if v.is_empty() || v.contains(char::is_whitespace) => Err(Error::new(
            ErrorKind::Refused,
            format!(
                "'{v}' is not a row id — an id is one word, and whether it resolves is the gate's \
             question"
            ),
        )),
        Shape::Line if v.contains('\n') => Err(Error::new(
            ErrorKind::Refused,
            format!("`{}` is one line; what was sent is not", c.key),
        )),
        Shape::Prose => Ok(value.trim_end().trim_start_matches('\n').to_string()),
        Shape::Code => Ok(code_text(value)),
        Shape::Inputs => inputs_text(value),
        Shape::Number => {
            let n: f64 = v.parse().map_err(|_| {
                Error::new(
                    ErrorKind::Refused,
                    format!("`{}` is a number; '{v}' is not", c.key),
                )
            })?;
            if !n.is_finite() {
                return Err(Error::new(
                    ErrorKind::Refused,
                    format!("`{}` must be finite", c.key),
                ));
            }
            Ok(format!("{n:?}"))
        }
        Shape::Choice(options) if !options.contains(&v) => Err(Error::new(
            ErrorKind::Refused,
            format!("`{}` is one of: {} — not '{v}'", c.key, options.join(", ")),
        )),
        _ => Ok(v.to_string()),
    }
}
