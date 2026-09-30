//! The one page template, `web/page.html`, and the one way to fill it.
//!
//! Every page the tool writes as a file — a node form, a lesson form, a role
//! guide, each page of the readers' folder, and (from `vleo-modules`, which
//! carries its own copy of [`fill`] because it cannot depend on this crate) a
//! saved result — is that template with its four slots filled. The markup the
//! pages share lives with the frontend, in `web/`, and a generator supplies
//! only what is its own: a title, what it adds to the head, the body.

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
pub fn fill_from(template: &str, p: &Page) -> Result<String, String> {
    let at = template
        .find("<!doctype html>")
        .ok_or("the template has no doctype")?;
    let mut rest = &template[at..];
    let mut o = String::with_capacity(rest.len() + p.head.len() + p.body.len() + 256);
    while let Some(i) = rest.find("{{") {
        o.push_str(&rest[..i]);
        let j = rest[i..]
            .find("}}")
            .ok_or("a slot is opened and never closed")?
            + i;
        match &rest[i + 2..j] {
            "title" => o.push_str(&crate::text::html(p.title)),
            "head" => o.push_str(p.head),
            "body_attrs" => o.push_str(p.body_attrs),
            "body" => o.push_str(p.body),
            other => return Err(format!("a slot `{other}` that no page fills")),
        }
        rest = &rest[j + 2..];
    }
    o.push_str(rest);
    Ok(o)
}

/// Whether a page is the template, filled: the template's own text, every
/// part of it between the slots, in order, with nothing before it or after.
/// The test every generator is held to.
pub fn is_filled(template: &str, page: &str) -> Result<(), String> {
    let at = template
        .find("<!doctype html>")
        .ok_or("the template has no doctype")?;
    let mut parts: Vec<&str> = Vec::new();
    let mut rest = &template[at..];
    while let Some(i) = rest.find("{{") {
        parts.push(&rest[..i]);
        let j = rest[i..]
            .find("}}")
            .ok_or("a slot is opened and never closed")?
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
                return Err(format!(
                    "the page does not carry the template's {:?}",
                    part.trim()
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
        assert!(fill_from(t, &Page::default())
            .unwrap_err()
            .contains("footer"));
        assert!(fill_from("<!doctype html>{{title", &Page::default()).is_err());
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
