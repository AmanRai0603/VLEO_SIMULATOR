//! A node's page follows the explanation standard, docs/EXPLAINING.md.
//!
//! The page is generated, so the standard is a property of the generator and
//! is checked here once for every node of the design, read from `design/`:
//! answer first (E1), the station order (E2), the kind of every claim (E3) and
//! of every tab (E8), the depth marks (E9, E11) — and the de-risking record,
//! newest version first, with the register on a risk-register row
//! (docs/DERISKING.md).

use std::path::Path;
use vleo_sheet::load::Tree;
use vleo_sheet::page::fragment;

/// The design, read from `design/`.
fn design() -> Tree {
    vleo_files::convert::open(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .unwrap()
        .0
}

fn page(id: &str) -> String {
    let tree = design();
    let sh = tree.sheets.get(id).unwrap();
    fragment(sh, &tree)
}

/// Where a string first appears, failing by name when it does not.
fn at(html: &str, what: &str) -> usize {
    html.find(what)
        .unwrap_or_else(|| panic!("the page has no «{what}»"))
}

/// The row whose record is the worked example.
const EXAMPLE: &str = "sw_central_expectation";

#[test]
fn the_answer_comes_first_and_the_station_is_in_order() {
    let p = page(EXAMPLE);
    assert!(
        at(&p, "Answer first") < at(&p, "role=\"tablist\""),
        "E1: the answer is not before the tabs"
    );
    let order = [
        "Say it simply",
        "Now the real thing",
        "Where the simple version breaks",
        "Common wrong idea",
        "Try it",
    ];
    let pos: Vec<usize> = order
        .iter()
        .map(|h| at(&p, &format!("</span>{h}</h4>")))
        .collect();
    assert!(
        pos.windows(2).all(|w| w[0] < w[1]),
        "E2: the station is out of order: {pos:?}"
    );
    assert!(
        p.contains("class=\"simply d-lr\""),
        "E9: the plain words are not marked for Learn and Read"
    );
    assert!(
        p.contains("class=\"d-l predict\""),
        "E11: there is no prediction to make at the Learn depth"
    );
}

#[test]
fn every_claim_and_every_tab_says_what_kind_it_is() {
    let p = page(EXAMPLE);
    assert!(
        p.contains("claim claim-sourced"),
        "E3: the relation does not say it is sourced"
    );
    assert!(
        p.contains("claim claim-derived"),
        "E3: the derivation does not say it is derived"
    );
    let tabs = p.matches("role=\"tab\"").count();
    let kinds = p.matches("<p class=\"dx dx-").count();
    assert_eq!(tabs, kinds, "E8: {tabs} tabs and {kinds} kind labels");
    for k in ["explanation", "reference", "tutorial"] {
        assert!(p.contains(&format!("dx dx-{k}")), "E8: no tab is a {k}");
    }
    // A declared row says who declared it, on the value itself.
    let tree = design();
    let declared = tree
        .ordered()
        .into_iter()
        .find(|s| s.is_declared() && !s.is_seeded() && s.value.is_some())
        .expect("no published declared row");
    assert!(
        page(&declared.id).contains("claim claim-declared"),
        "E3: {} is not marked declared",
        declared.id
    );
}

#[test]
fn the_record_reads_newest_first_and_says_what_it_rests_on() {
    let p = page(EXAMPLE);
    let tab = &p[at(&p, "data-panel=\"2\"")..];
    let tab = &tab[..tab.find("</section>").unwrap()];
    assert!(
        tab.contains("Rests on:") && tab.contains("Would break if:"),
        "D1: the current belief is not shown"
    );
    let (v3, v2, v1) = (
        at(tab, "<b>v3</b>"),
        at(tab, "<b>v2</b>"),
        at(tab, "<b>v1</b>"),
    );
    assert!(v3 < v2 && v2 < v1, "D3: versions are not newest first");
    assert!(
        tab.contains("What we now know"),
        "D2: a change does not say what was learned"
    );
    assert!(
        tab.contains("R-01 L3-&gt;L2"),
        "D4: the risk the version moved is not shown"
    );
}

#[test]
fn a_risk_register_row_holds_its_register_and_asks_for_the_conclusion() {
    let p = page("mgt_risk_register_technical_risk");
    assert!(
        p.contains("<b>R-01</b>"),
        "the registered risk is not listed"
    );
    assert!(
        p.contains("class=\"derisk-rollup\" data-row=\"mgt_risk_register_technical_risk\""),
        "the row does not ask for what the tree has done to its risks"
    );
}

#[test]
fn a_seeded_row_says_so_first_rather_than_answering() {
    let tree = design();
    let seeded = tree.ordered().into_iter().find(|s| s.is_seeded()).unwrap();
    let p = page(&seeded.id);
    assert!(at(&p, "Not yet specified") < at(&p, "role=\"tablist\""));
}
