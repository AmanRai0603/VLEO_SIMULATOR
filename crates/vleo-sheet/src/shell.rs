//! The one page template, `web/page.html`, and the one way to fill it.
//!
//! Every page the tool writes as a file — a node form, a lesson form, a role
//! guide, a row's page, a saved result, each page of the readers' folder — is
//! that template with its four slots filled. What each page puts in it is its
//! own parts file in `web/pages` ([`Parts`]): every word and tag, by name, so
//! a generator decides only which part, how often and with what. The markup
//! lives with the frontend, and a test refuses a generator that writes any.

use crate::{Error, ErrorKind};

/// `web/page.html`, as it is on disk when the tool is built.
pub const TEMPLATE: &str = include_str!("../../../web/page.html");

/// The slots a filler knows, in the order the template holds them.
pub const SLOTS: [&str; 4] = ["title", "head", "body_attrs", "body"];

/// What one page puts in the template.
#[derive(Default)]
pub struct Page<'a> {
    /// The page's name, as text: it is escaped here.
    pub title: &'a str,
    /// Styles, a stylesheet link, a generator note — markup, placed as given.
    pub head: &'a str,
    /// Attributes on `<body>`, with their leading space, or nothing.
    pub body_attrs: &'a str,
    /// The page — markup, placed as given.
    pub body: &'a str,
}

/// The template from its doctype on, with each slot filled once.
///
/// One pass, and what is put in is never read again, so a page whose content
/// happens to spell a slot cannot be filled twice. A slot this does not know
/// is a template this build was not written for, and the tests refuse it
/// before anything ships (`xtask/tests/the_pages_share_one_template.rs`).
pub fn fill(p: &Page) -> String {
    fill_from(TEMPLATE, p).unwrap_or_else(|e| panic!("web/page.html: {e}"))
}

/// [`fill`] over any template text — the tests hand it a broken one.
pub fn fill_from(template: &str, p: &Page) -> Result<String, Error> {
    let at = template
        .find("<!doctype html>")
        .ok_or_else(|| Error::new(ErrorKind::Malformed, "the template has no doctype"))?;
    let mut rest = &template[at..];
    let mut o = String::with_capacity(rest.len() + p.head.len() + p.body.len() + 256);
    while let Some(i) = rest.find("{{") {
        o.push_str(&rest[..i]);
        let j = rest[i..]
            .find("}}")
            .ok_or_else(|| Error::new(ErrorKind::Malformed, "a slot is opened and never closed"))?
            + i;
        match &rest[i + 2..j] {
            "title" => o.push_str(&crate::text::html(p.title)),
            "head" => o.push_str(p.head),
            "body_attrs" => o.push_str(p.body_attrs),
            "body" => o.push_str(p.body),
            other => {
                return Err(Error::new(
                    ErrorKind::Malformed,
                    format!("a slot `{other}` that no page fills"),
                ))
            }
        }
        rest = &rest[j + 2..];
    }
    o.push_str(rest);
    Ok(o)
}

/// Every `{{slot}}` in `text` filled once from `slots`, in one pass; a slot
/// with nothing to fill it, or a filling with no slot, refused.
fn fill_slots(text: &str, slots: &[(&str, &str)]) -> Result<String, Error> {
    let mut rest = text;
    let mut used = vec![false; slots.len()];
    let mut o = String::with_capacity(rest.len() + slots.iter().map(|s| s.1.len()).sum::<usize>());
    while let Some(i) = rest.find("{{") {
        o.push_str(&rest[..i]);
        let j = rest[i..]
            .find("}}")
            .ok_or_else(|| Error::new(ErrorKind::Malformed, "a slot is opened and never closed"))?
            + i;
        let name = &rest[i + 2..j];
        let k = slots.iter().position(|(n, _)| *n == name).ok_or_else(|| {
            Error::new(
                ErrorKind::Malformed,
                format!("the template has a slot `{name}` that nothing fills"),
            )
        })?;
        o.push_str(slots[k].1);
        used[k] = true;
        rest = &rest[j + 2..];
    }
    o.push_str(rest);
    if let Some(k) = used.iter().position(|u| !u) {
        return Err(Error::new(
            ErrorKind::Malformed,
            format!(
                "`{}` is filled and the template has no slot for it",
                slots[k].0
            ),
        ));
    }
    Ok(o)
}

/// A template file of named parts, for a page whose shape the data decides —
/// a row's page, where a tab may be empty, a table has a row per input and a
/// section appears only when the sheet says something.
///
/// `<!-- part: name -->` opens a part, which runs to the next marker; the file
/// ends with `<!-- end -->`, and anything before the first part is the file's
/// note. The newline straight after a marker is the marker's, so a part
/// written on its own lines ends with a newline and one closed by the next
/// marker on the same line does not.
pub struct Parts {
    file: &'static str,
    parts: std::collections::BTreeMap<&'static str, &'static str>,
}

impl Parts {
    /// Read a parts file; `file` is its path, for the messages.
    pub fn parse(file: &'static str, text: &'static str) -> Result<Parts, Error> {
        const OPEN: &str = "<!-- part: ";
        let end = text.rfind("<!-- end -->").ok_or_else(|| {
            Error::new(
                ErrorKind::Malformed,
                format!("{file} does not end with <!-- end -->"),
            )
        })?;
        let body = &text[..end];
        // The file's own note comes first, and may well quote a marker.
        let from = match body.strip_prefix("<!--") {
            Some(_) if !body.starts_with(OPEN) => {
                body.find("-->").map(|k| k + 3).ok_or_else(|| {
                    Error::new(
                        ErrorKind::Malformed,
                        format!("{file}: the opening note is never closed"),
                    )
                })?
            }
            _ => 0,
        };
        let mut parts = std::collections::BTreeMap::new();
        let mut at = body[from..].find(OPEN).map(|k| k + from);
        while let Some(i) = at {
            let close = body[i..].find("-->").ok_or_else(|| {
                Error::new(
                    ErrorKind::Malformed,
                    format!("{file}: a part marker is never closed"),
                )
            })? + i;
            let name = body[i + OPEN.len()..close].trim();
            let start = close + 3;
            let start = if body[start..].starts_with('\n') {
                start + 1
            } else {
                start
            };
            let next = body[start..].find(OPEN).map(|k| k + start);
            let stop = next.unwrap_or(body.len());
            if parts.insert(name, &body[start..stop]).is_some() {
                return Err(Error::new(
                    ErrorKind::Malformed,
                    format!("{file}: the part `{name}` is written twice"),
                ));
            }
            at = next;
        }
        Ok(Parts { file, parts })
    }

    /// A part as it is written.
    pub fn text(&self, name: &str) -> &'static str {
        self.parts
            .get(name)
            .unwrap_or_else(|| panic!("{} has no part `{name}`", self.file))
    }

    /// A part with its slots filled.
    pub fn fill(&self, name: &str, slots: &[(&str, &str)]) -> String {
        fill_slots(self.text(name), slots)
            .unwrap_or_else(|e| panic!("{}, part `{name}`: {e}", self.file))
    }

    /// Every part's name.
    pub fn names(&self) -> impl Iterator<Item = &&'static str> {
        self.parts.keys()
    }
}

/// Whether a page is the template, filled: the template's own text, every
/// part of it between the slots, in order, with nothing before it or after.
/// The test every generator is held to.
pub fn is_filled(template: &str, page: &str) -> Result<(), Error> {
    let at = template
        .find("<!doctype html>")
        .ok_or_else(|| Error::new(ErrorKind::Malformed, "the template has no doctype"))?;
    let mut parts: Vec<&str> = Vec::new();
    let mut rest = &template[at..];
    while let Some(i) = rest.find("{{") {
        parts.push(&rest[..i]);
        let j = rest[i..]
            .find("}}")
            .ok_or_else(|| Error::new(ErrorKind::Malformed, "a slot is opened and never closed"))?
            + i;
        rest = &rest[j + 2..];
    }
    parts.push(rest);
    let mut from = 0;
    for (k, part) in parts.iter().enumerate() {
        let found = if k == 0 {
            page.starts_with(part).then_some(0)
        } else if k == parts.len() - 1 {
            (page.len() >= from + part.len() && page.ends_with(part))
                .then(|| page.len() - part.len())
        } else {
            page[from..].find(part).map(|i| i + from)
        };
        match found {
            Some(i) => from = i + part.len(),
            None => {
                return Err(Error::new(
                    ErrorKind::Malformed,
                    format!("the page does not carry the template's {:?}", part.trim()),
                ))
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_filled_page_is_the_template_and_nothing_is_filled_twice() {
        let p = Page {
            title: "a <b> & c",
            head: "<style>x{}</style>",
            body_attrs: " class=\"k\"",
            body: "<p>{{title}} stays as it was written</p>",
        };
        let o = fill(&p);
        assert!(o.starts_with("<!doctype html>\n"), "{o}");
        assert!(
            !o.contains("THE PAGE TEMPLATE"),
            "the template's note reached the page"
        );
        assert!(o.contains("<title>a &lt;b&gt; &amp; c</title>"), "{o}");
        assert!(o.contains("<body class=\"k\">"), "{o}");
        assert!(
            o.contains("<p>{{title}} stays as it was written</p>"),
            "{o}"
        );
        assert!(o.ends_with("</html>\n"), "{o}");
        is_filled(TEMPLATE, &o).unwrap();
    }

    #[test]
    fn a_slot_nobody_fills_is_refused() {
        let t = "<!doctype html><title>{{title}}</title>{{footer}}";
        let e = fill_from(t, &Page::default()).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::Malformed, "{e}");
        assert!(e.message().contains("footer"), "{e}");
        assert!(fill_from("<!doctype html>{{title", &Page::default()).is_err());
    }

    #[test]
    fn parts_are_read_by_name_and_end_where_the_next_begins() {
        let t = "<!-- note quoting <!-- part: q --> -->\n<!-- part: a -->\n<p>{{x}}</p>\n<!-- part: b --><i>{{y}}</i><!-- part: c -->z<!-- end -->\n";
        let p = Parts::parse("t.html", t).unwrap();
        assert_eq!(p.text("a"), "<p>{{x}}</p>\n");
        assert_eq!(p.fill("b", &[("y", "1")]), "<i>1</i>");
        assert_eq!(p.text("c"), "z");
        assert_eq!(p.names().count(), 3);
        assert!(Parts::parse("t.html", "<!-- part: a -->x").is_err());
        assert!(Parts::parse("t.html", "<!-- part: a -->x<!-- part: a -->y<!-- end -->").is_err());
    }

    #[test]
    fn a_page_that_is_not_the_template_is_told_apart() {
        let own = "<!doctype html>\n<html><head><title>x</title></head><body></body></html>\n";
        assert!(is_filled(TEMPLATE, own).is_err());
        let mut o = fill(&Page::default());
        assert!(is_filled(TEMPLATE, &o).is_ok());
        o.push_str("<script>after the end</script>");
        assert!(is_filled(TEMPLATE, &o).is_err());
        let every: Vec<&str> = SLOTS.to_vec();
        for s in every {
            assert!(
                TEMPLATE.contains(&format!("{{{{{s}}}}}")),
                "web/page.html lost {s}"
            );
        }
    }
}
