//! Text made safe for where it is written: HTML text and attributes, and a
//! JSON string. One of each, for every page and payload this crate writes.

/// Text for HTML — an element's text or a quoted attribute.
pub fn html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// A JSON string, quotes included.
pub fn json(v: &str) -> String {
    let mut o = String::with_capacity(v.len() + 2);
    o.push('"');
    for c in v.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

#[cfg(test)]
mod tests {
    #[test]
    fn each_escape_keeps_its_text_inert() {
        assert_eq!(
            super::html("<a href=\"x\">&</a>"),
            "&lt;a href=&quot;x&quot;&gt;&amp;&lt;/a&gt;"
        );
        assert_eq!(super::json("a\"b\\c\nd\u{1}"), "\"a\\\"b\\\\c\\nd\\u0001\"");
    }
}
