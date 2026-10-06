//! CSV, read and written the one way the group folder uses it.
//!
//! The same rules as the page's `web/js/csv.js`, so a table this library
//! writes is byte for byte the one the page writes, and a fingerprint taken
//! over it is the same: RFC 4180, a header row, a field in double quotes only
//! when it holds a comma, a quote or a line break, a doubled quote inside one,
//! and every line ended by `\n`.

/// One CSV text, read: its header, its rows (each as long as the header),
/// the line each row starts on, and what did not parse, by line. As
/// `web/js/csv.js` reads it: a leading byte-order mark is skipped, a carriage
/// return is ignored, a blank line is not a row, every cell is trimmed, and
/// nothing is dropped silently.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Parsed {
    pub head: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub lines: Vec<usize>,
    pub problems: Vec<(usize, String)>,
}

/// Read one CSV text whole.
pub fn read(text: &str) -> Parsed {
    let src = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut out: Vec<(Vec<String>, usize)> = Vec::new();
    let mut problems = Vec::new();
    let (mut row, mut field, mut quoted) = (Vec::new(), String::new(), false);
    let (mut line, mut row_line) = (1usize, 1usize);
    let end_row = |row: &mut Vec<String>,
                   field: &mut String,
                   out: &mut Vec<(Vec<String>, usize)>,
                   at: usize| {
        row.push(std::mem::take(field));
        let r = std::mem::take(row);
        if !(r.len() == 1 && r[0].trim().is_empty()) {
            out.push((r, at));
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
                if ch == '\n' {
                    line += 1;
                }
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
            '\n' => {
                line += 1;
                end_row(&mut row, &mut field, &mut out, row_line);
                row_line = line;
            }
            c => field.push(c),
        }
    }
    if quoted {
        problems.push((row_line, "a quoted field is never closed".to_string()));
    }
    if !field.is_empty() || !row.is_empty() {
        end_row(&mut row, &mut field, &mut out, row_line);
    }
    if out.is_empty() {
        problems.push((1, "the file is empty".to_string()));
        return Parsed {
            problems,
            ..Parsed::default()
        };
    }
    let mut lines_iter = out.into_iter();
    let (first, _) = lines_iter.next().unwrap_or_default();
    let head: Vec<String> = first.into_iter().map(|h| h.trim().to_string()).collect();
    let (mut rows, mut lines) = (Vec::new(), Vec::new());
    for (r, at) in lines_iter {
        if r.len() != head.len() {
            problems.push((
                at,
                format!("has {} fields where the header has {}", r.len(), head.len()),
            ));
        }
        rows.push(
            (0..head.len())
                .map(|k| r.get(k).map_or(String::new(), |c| c.trim().to_string()))
                .collect(),
        );
        lines.push(at);
    }
    Parsed {
        head,
        rows,
        lines,
        problems,
    }
}

/// The header and rows of one CSV text ([`read`], without its lines and
/// problems).
pub fn parse(text: &str) -> (Vec<String>, Vec<Vec<String>>) {
    let p = read(text);
    (p.head, p.rows)
}

/// The name a header gives, its unit in square brackets taken off:
/// `altitude [km]` is `altitude`.
pub fn name_of(header: &str) -> &str {
    split_unit(header).0
}

/// A header's name and the unit it carries in square brackets, as
/// `web/js/csv.js` splits them: `altitude [km]` is `altitude` and `km`.
pub fn split_unit(header: &str) -> (&str, &str) {
    let h = header.trim();
    match (h.rfind('['), h.ends_with(']')) {
        (Some(i), true) if !h[i + 1..h.len() - 1].contains(']') => {
            (h[..i].trim_end(), h[i + 1..h.len() - 1].trim())
        }
        _ => (h, ""),
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
    fn every_row_says_its_line_and_a_short_one_is_said() {
        let p = read("a,b\n1,\"two\nlines\"\n\n3\n");
        assert_eq!(p.lines, vec![2, 5]);
        assert_eq!(p.rows[1], vec!["3".to_string(), String::new()]);
        assert_eq!(
            p.problems,
            vec![(5, "has 1 fields where the header has 2".to_string())]
        );
        assert_eq!(
            read("").problems,
            vec![(1, "the file is empty".to_string())]
        );
        assert_eq!(
            read("a\n\"open").problems[0].1,
            "a quoted field is never closed"
        );
    }

    #[test]
    fn a_unit_rides_in_the_header() {
        assert_eq!(name_of("altitude [km]"), "altitude");
        assert_eq!(name_of("version"), "version");
    }
}
