//! CSV, read and written the one way the group folder uses it.
//!
//! The same rules as the page's `web/js/csv.js`, so a table this library
//! writes is byte for byte the one the page writes, and a fingerprint taken
//! over it is the same: RFC 4180, a header row, a field in double quotes only
//! when it holds a comma, a quote or a line break, a doubled quote inside one,
//! and every line ended by `\n`.

/// The header and rows of one CSV text: a leading byte-order mark is skipped,
/// a carriage return is ignored, a blank line is not a row, and every cell is
/// trimmed. A row shorter than the header is filled with empty cells.
pub fn parse(text: &str) -> (Vec<String>, Vec<Vec<String>>) {
    let src = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut out: Vec<Vec<String>> = Vec::new();
    let (mut row, mut field, mut quoted) = (Vec::new(), String::new(), false);
    let end_row = |row: &mut Vec<String>, field: &mut String, out: &mut Vec<Vec<String>>| {
        row.push(std::mem::take(field));
        let r = std::mem::take(row);
        if !(r.len() == 1 && r[0].trim().is_empty()) {
            out.push(r);
        }
    };
    let mut chars = src.chars().peekable();
    while let Some(ch) = chars.next() {
        if quoted {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    quoted = false;
                }
            } else {
                field.push(ch);
            }
            continue;
        }
        match ch {
            '"' if field.trim().is_empty() => {
                field.clear();
                quoted = true;
            }
            ',' => row.push(std::mem::take(&mut field)),
            '\r' => {}
            '\n' => end_row(&mut row, &mut field, &mut out),
            c => field.push(c),
        }
    }
    if !field.is_empty() || !row.is_empty() {
        end_row(&mut row, &mut field, &mut out);
    }
    let mut lines = out.into_iter();
    let head: Vec<String> = lines
        .next()
        .unwrap_or_default()
        .into_iter()
        .map(|h| h.trim().to_string())
        .collect();
    let rows = lines
        .map(|r| {
            (0..head.len())
                .map(|k| r.get(k).map_or(String::new(), |c| c.trim().to_string()))
                .collect()
        })
        .collect();
    (head, rows)
}

/// The name a header gives, its unit in square brackets taken off:
/// `altitude [km]` is `altitude`.
pub fn name_of(header: &str) -> &str {
    let h = header.trim();
    match (h.rfind('['), h.ends_with(']')) {
        (Some(i), true) => h[..i].trim_end(),
        _ => h,
    }
}

/// CSV text from a header and rows, quoting only where it must.
pub fn write<S: AsRef<str>>(head: &[S], rows: &[Vec<String>]) -> String {
    fn q(v: &str) -> String {
        if v.contains(['"', ',', '\n', '\r']) {
            format!("\"{}\"", v.replace('"', "\"\""))
        } else {
            v.to_string()
        }
    }
    let mut out = head
        .iter()
        .map(|h| q(h.as_ref()))
        .collect::<Vec<_>>()
        .join(",");
    for r in rows {
        out.push('\n');
        out.push_str(&r.iter().map(|c| q(c)).collect::<Vec<_>>().join(","));
    }
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_field_is_quoted_only_where_it_must_be() {
        let rows = vec![vec!["a,b".to_string(), "say \"x\"".into(), "plain".into()]];
        let text = write(&["one", "two", "three"], &rows);
        assert_eq!(text, "one,two,three\n\"a,b\",\"say \"\"x\"\"\",plain\n");
        assert_eq!(
            parse(&text),
            (vec!["one".into(), "two".into(), "three".into()], rows)
        );
    }

    #[test]
    fn a_unit_rides_in_the_header() {
        assert_eq!(name_of("altitude [km]"), "altitude");
        assert_eq!(name_of("version"), "version");
    }
}
