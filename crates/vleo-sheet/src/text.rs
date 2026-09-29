//! Small text rules every generator shares, written once.
//!
//! The page, the guides and the node form each had their own HTML escaper, and
//! three modules had their own way to name the row behind `node.member`. Two
//! copies of a rule drift; these are the one copy.

/// Text as it may appear inside HTML, in an element or a quoted attribute.
pub fn html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The row that produces a variable: `node` for `node` and for `node.member`.
pub fn producer_of(var: &str) -> &str {
    var.split_once('.').map_or(var, |(node, _)| node)
}

#[cfg(test)]
mod tests {
    #[test]
    fn html_escapes_what_can_close_or_open_markup() {
        assert_eq!(
            super::html("<a href=\"x\">&</a>"),
            "&lt;a href=&quot;x&quot;&gt;&amp;&lt;/a&gt;"
        );
    }

    #[test]
    fn a_member_names_its_producer() {
        assert_eq!(super::producer_of("sw_driver_set.kp"), "sw_driver_set");
        assert_eq!(super::producer_of("orbit_velocity"), "orbit_velocity");
    }
}
