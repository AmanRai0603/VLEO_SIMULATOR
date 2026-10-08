//! Every node's page carries the same tabs, in the same order, each with its
//! panel.
//!
//! A node's page is rendered from its sheet when it is opened; no folder holds
//! a copy. So the page is checked as it is rendered: every sheet in the tree,
//! through `page::fragment`, the function the engine calls. One tab set across
//! the tree means one template; two would mean a page written by a template
//! that has since moved.

use std::collections::BTreeMap;
use std::path::Path;

/// The tab labels, in order, and how many panels the page has.
fn tabs(html: &str) -> (Vec<String>, usize) {
    let mut labels = Vec::new();
    let mut rest = html;
    while let Some(at) = rest.find("<button role=\"tab\" class=\"tab") {
        rest = &rest[at..];
        let Some(open) = rest.find('>') else { break };
        let Some(close) = rest[open..].find("</button>") else {
            break;
        };
        labels.push(rest[open + 1..open + close].to_string());
        rest = &rest[open + close..];
    }
    (labels, html.matches("data-panel=\"").count())
}

#[test]
fn every_rendered_page_has_one_tab_set_and_a_panel_for_each_tab() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tree = vleo_sheet::load_all(&root).expect("the tree loads");
    let mut sets: BTreeMap<Vec<String>, Vec<String>> = BTreeMap::new();
    let mut bad = Vec::new();
    for sh in tree.ordered() {
        let html = vleo_sheet::page::fragment(sh, &tree);
        let (labels, panels) = tabs(&html);
        if labels.is_empty() {
            bad.push(format!("{}: no tabs at all", sh.id));
            continue;
        }
        if panels != labels.len() {
            bad.push(format!(
                "{}: {} tabs but {panels} panels",
                sh.id,
                labels.len()
            ));
        }
        sets.entry(labels).or_default().push(sh.id.clone());
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
    assert_eq!(
        tree.sheets.len(),
        sets.values().map(Vec::len).sum::<usize>()
    );
    assert!(
        sets.len() == 1,
        "{} different tab sets across the tree, not one:\n{}",
        sets.len(),
        sets.iter()
            .map(|(t, who)| format!("  {} nodes: {} (e.g. {})", who.len(), t.join(" · "), who[0]))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn the_tab_reader_sees_a_missing_panel_and_a_moved_label() {
    let one = r#"<button role="tab" class="tab on" data-tab="0">question</button><button role="tab" class="tab" data-tab="1">theory</button><div data-panel="0"></div><div data-panel="1"></div>"#;
    assert_eq!(tabs(one), (vec!["question".into(), "theory".into()], 2));
    let short = one.replace(r#"<div data-panel="1"></div>"#, "");
    assert_eq!(tabs(&short).1, 1);
    let moved = one.replace(">theory<", ">THEORY<");
    assert_ne!(tabs(&moved).0, tabs(one).0);
}
