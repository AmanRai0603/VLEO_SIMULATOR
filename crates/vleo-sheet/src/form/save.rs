//! The declaration form: checking an answer, and writing it as the file holds it.

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
pub fn normalise(field: &str, value: &str) -> Result<String, Error> {
    let Some(f) = self::field(field) else {
        return Err(Error::new(
            ErrorKind::Refused,
            format!("'{field}' is not a field this form writes"),
        ));
    };
    let v = value.trim();
    match f.shape {
        Shape::Number => {
            let n: f64 = v.parse().map_err(|_| {
                Error::new(
                    ErrorKind::Refused,
                    format!(
                    "'{v}' is not a number. It is written into the sheet unquoted and into the \
                     generated guard as an f64; anything else would be read back as zero"
                ),
                )
            })?;
            if !n.is_finite() {
                return Err(Error::new(
                    ErrorKind::Refused,
                    format!(
                        "'{v}' is not finite. A bound that is not a number cannot guard anything"
                    ),
                ));
            }
            Ok(format!("{n:?}"))
        }
        Shape::Count => v.parse::<u32>().map(|n| n.to_string()).map_err(|_| {
            Error::new(
                ErrorKind::Refused,
                format!("'{v}' is not a whole number of at least zero"),
            )
        }),
        Shape::Choice(options) => {
            if options.contains(&v) {
                Ok(v.to_string())
            } else {
                Err(Error::new(
                    ErrorKind::Refused,
                    format!(
                        "'{v}' is not one of: {}. This is a closed set, not a label",
                        options.join(", ")
                    ),
                ))
            }
        }
        Shape::Quantity => {
            if crate::is_quantity_name(v) {
                Ok(v.to_string())
            } else {
                Err(Error::new(
                    ErrorKind::Missing,
                    format!(
                    "'{v}' is not a quantity this system has. The type is written straight into \
                     the generated signature, so one that does not exist stops the tree \
                     compiling. One of: {}",
                    vleo_units::QUANTITIES.join(", ")
                ),
                ))
            }
        }
        Shape::UnitName => {
            if crate::unit_exists(v) {
                Ok(v.to_string())
            } else {
                Err(Error::new(
                    ErrorKind::Refused,
                    format!(
                        "'{v}' is not a unit this system knows, so nothing could convert it at a \
                     face boundary. See vleo_units::Unit for the ones that exist."
                    ),
                ))
            }
        }
        // A row id is checked against the tree by the gate, which is the only
        // place the whole graph is visible. Refusing the obviously impossible
        // here saves a write and a rollback for a typo with a space in it.
        Shape::RowId => {
            if v.is_empty() || v.contains(char::is_whitespace) {
                Err(Error::new(
                    ErrorKind::Refused,
                    format!(
                    "'{v}' is not a row id — an id is one word, and whether it resolves is the \
                     gate's question"
                ),
                ))
            } else {
                Ok(v.to_string())
            }
        }
        Shape::Line => {
            if v.contains('\n') {
                Err(Error::new(
                    ErrorKind::Refused,
                    format!(
                    "'{field}' is one line. What was sent has {} of them; if the answer needs a \
                     paragraph it belongs in a field that holds one",
                    v.lines().count()
                ),
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
pub(super) fn inputs_text(value: &str) -> Result<String, Error> {
    inputs_text_of(value, false)
}

/// The same, for a case the node must refuse, where an input may be what no
/// node takes: not a number, or infinite. Every generated node refuses one at
/// its door, and a case saying so is evidence of that refusal, so it goes in
/// as written: `nan`, `inf` and `-inf`, TOML's own spellings.
pub(super) fn refused_inputs_text(value: &str) -> Result<String, Error> {
    inputs_text_of(value, true)
}

fn inputs_text_of(value: &str, refused: bool) -> Result<String, Error> {
    let v = value.trim();
    let doc: toml::Value = format!("x = {v}").parse().map_err(|_| {
        Error::new(
            ErrorKind::Refused,
            format!("'{v}' is not a set of inputs — write them as {{ name = number, … }}"),
        )
    })?;
    let t = doc
        .get("x")
        .and_then(|x| x.as_table())
        .ok_or_else(|| Error::new(ErrorKind::Refused, format!("'{v}' is not a set of inputs")))?;
    let mut parts = Vec::new();
    for (k, x) in t {
        let n = x
            .as_float()
            .or_else(|| x.as_integer().map(|i| i as f64))
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::Refused,
                    format!("the input «{k}» is not a number"),
                )
            })?;
        if !n.is_finite() && !refused {
            return Err(Error::new(
                ErrorKind::Refused,
                format!(
                    "the input «{k}» is not finite. Only a case the node must refuse may give \
                     one, to show the node refuses it"
                ),
            ));
        }
        parts.push(format!("{k} = {}", toml_number(n)));
    }
    Ok(format!("{{ {} }}", parts.join(", ")))
}

/// A number as TOML writes it. Rust's `{:?}` says `NaN` and `inf`, which TOML
/// does not read; these are the spellings it does.
pub(crate) fn toml_number(n: f64) -> String {
    if n.is_nan() {
        "nan".into()
    } else if n == f64::INFINITY {
        "inf".into()
    } else if n == f64::NEG_INFINITY {
        "-inf".into()
    } else {
        format!("{n:?}")
    }
}

/// Whether a value is one this field may hold. `normalise` without the value.
pub fn value_allowed(field: &str, value: &str) -> Result<(), Error> {
    normalise(field, value).map(|_| ())
}
