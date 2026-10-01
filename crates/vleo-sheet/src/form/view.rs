//! The declaration form: the row's view.

use super::*;

/// The `[view]` table, rewritten coherently.
///
/// NOT THREE FIELDS. A view's kind decides which other keys exist: a number has
/// none, a line has `over` and `points`, a bar has `y`. Offered as three
/// separate scalar fields the form would have to let somebody set `over` on a
/// row whose kind is `number`, where the key does not exist and means nothing,
/// and then set the kind afterwards — two saves, either order wrong. So the
/// whole table is written at once from the state the face sends.
///
/// `over` is the row the picture is drawn against, whatever the kind calls it:
/// the file says `over` for a line and `y` for a bar, and a face should not have
/// to know that.
///
/// It REFUSES A TABLE IT DOES NOT UNDERSTAND. A `[view]` carrying a comment, a
/// key outside the set, or a heatmap — which has two axes and a grid and is not
/// something this offers — is reported rather than flattened. This is the one
/// writer here that replaces a whole table instead of one value, so it is the
/// one place something can be lost without the loss appearing anywhere.
///
/// Whether the row being drawn against EXISTS is not asked here: that needs the
/// tree, and `save_view` asks it.
pub fn view_rewrite(text: &str, kind: &str, over: &str, points: &str) -> Result<String, Error> {
    if !VIEW_KINDS.contains(&kind) {
        return Err(Error::new(
            ErrorKind::Refused,
            format!(
                "'{kind}' is not a way this tool draws an answer. One of: {}",
                VIEW_KINDS.join(", ")
            ),
        ));
    }
    let (from, to) = window(text, "view").ok_or_else(|| {
        Error::new(
            ErrorKind::Malformed,
            "this sheet has no [view] table, which every sheet should have. That is a malformed \
         sheet and not something a form should paper over — edit it directly",
        )
    })?;
    for line in text[from..to].lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with('#') {
            return Err(Error::new(
                ErrorKind::Refused,
                format!(
                    "this row's [view] carries a comment — `{t}` — and rewriting the table would \
                 drop it. Every comment in a sheet is somebody's reason, so this is refused \
                 rather than tidied. Edit it directly"
                ),
            ));
        }
        let key = t.split('=').next().unwrap_or("").trim();
        if !matches!(key, "kind" | "over" | "points" | "y" | "over_x" | "over_y") {
            return Err(Error::new(
                ErrorKind::Refused,
                format!(
                    "this row's [view] has a `{key}`, which this form does not know about. It \
                 rewrites the whole table, so it refuses one it does not understand"
                ),
            ));
        }
        if key == "kind" && t.contains("heatmap") {
            return Err(Error::new(
                ErrorKind::Refused,
                "this row draws a heatmap, which has two axes and a grid. This form offers a \
                 number, a line and a bar; turning a heatmap into one of them would throw away \
                 an axis. Edit it directly",
            ));
        }
    }
    let over = over.trim();
    let body = match kind {
        "number" => {
            if !over.is_empty() {
                return Err(Error::new(
                    ErrorKind::Refused,
                    "a number is drawn from the row's own answer and is not drawn against \
                     anything. Nothing was written",
                ));
            }
            "kind = \"number\"\n".to_string()
        }
        // The file calls it `y` for a bar. The face says `over` for both,
        // because "the row it is drawn against" is one question.
        "bar" => format!("kind = \"bar\"\ny = {}\n", toml_quote(over)),
        "line" => {
            let n: u32 = match points.trim().parse() {
                Ok(n) if n >= 2 => n,
                Ok(_) => {
                    return Err(Error::new(
                        ErrorKind::Refused,
                        "a line needs at least two points; one point is a number",
                    ))
                }
                Err(_) => {
                    return Err(Error::new(
                        ErrorKind::Refused,
                        format!("'{}' is not a number of points", points.trim()),
                    ))
                }
            };
            if n > 2000 {
                return Err(Error::new(
                    ErrorKind::Refused,
                    format!(
                    "{n} points is a run of {n} evaluations of this branch for one picture. The \
                     sweeps in this tree use 60 to 80"
                ),
                ));
            }
            format!(
                "kind = \"line\"\nover = {}\npoints = {n}\n",
                toml_quote(over)
            )
        }
        _ => unreachable!("the kind was checked above"),
    };
    let mut out = String::with_capacity(text.len() + 64);
    out.push_str(&text[..from]);
    out.push_str(&body);
    out.push_str(text[to..].trim_start_matches('\n'));
    Ok(out)
}

/// Rewrite how the answer is drawn, and leave the tree consistent or untouched.
///
/// `view_rewrite` decides what the sheet becomes; this is what only the tree can
/// answer. NOTHING ELSE CHECKS IT: the gate validates an input's `var` because
/// an input is an edge, and a view's `over` is not one, so a sweep over a row
/// that was renamed draws nothing and says nothing about why.
pub fn save_view(
    root: &std::path::Path,
    id: &str,
    kind: &str,
    over: &str,
    points: &str,
    base: &str,
) -> Saved {
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
    let over = over.trim();
    if kind != "number" {
        if !tree.sheets.contains_key(over) {
            return Saved::Refused(format!(
                "there is no row '{over}' to draw this against. A sweep over a row that is not \
                 there draws nothing and says nothing about why"
            ));
        }
        if over == id {
            return Saved::Refused(
                "a row cannot be swept against itself — the sweep would hold the answer fixed \
                 and then plot it"
                    .into(),
            );
        }
    }
    let after = match view_rewrite(&before, kind, over, points) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(e.into()),
    };
    commit_edit(root, id, &path, &before, after, false)
}
