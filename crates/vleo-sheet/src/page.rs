//! The document fragment, and the index the shell loads.
//!
//! One fragment per node, assembled exactly like the back end. An earlier draft
//! had per-node Rust files assembled into a crate and a *single* document, and
//! that asymmetry is wrong for a measurable reason: five engineers adding a node
//! on the same afternoon produce four merge conflicts in one document — in
//! generated content nobody is allowed to hand-edit — and zero with a fragment
//! each. It is the same rule already stated for the module list and the index:
//! never commit an aggregate. The document was an aggregate hiding in plain
//! sight.
//!
//! Every word and tag of the page is in `web/pages/row.html`, as named parts.
//! What is here is the page's logic — which part, how many times, filled with
//! what — so the frontend owns how a row reads and this owns what it says.

use crate::load::Tree;
use crate::model::*;
use crate::shell::Parts;
use crate::short_hex;
use vleo_units::Unit;

use crate::text::html as h;

/// The row page's parts (`web/pages/row.html`), read once.
fn parts() -> &'static Parts {
    static P: std::sync::OnceLock<Parts> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        Parts::parse(
            "web/pages/row.html",
            include_str!("../../../web/pages/row.html"),
        )
        .unwrap_or_else(|e| panic!("{e}"))
    })
}

/// A part as written.
fn t(name: &str) -> &'static str {
    parts().text(name)
}

/// A part with its slots filled.
fn f(name: &str, slots: &[(&str, &str)]) -> String {
    parts().fill(name, slots)
}

/// A part that is a list, one item a line.
fn lines(name: &str) -> Vec<&'static str> {
    t(name).lines().filter(|l| !l.trim().is_empty()).collect()
}

fn unit_symbol(name: &str) -> String {
    Unit::from_name(name)
        .unwrap_or(Unit::One)
        .symbol()
        .to_string()
}

/// A unit to put after a number in a sentence: nothing for a dimensionless
/// one, whose symbol `-` reads as punctuation in prose.
fn unit_after(name: &str) -> String {
    match unit_symbol(name).as_str() {
        "-" | "" => String::new(),
        u => format!(" {u}"),
    }
}

/// The tabs, in the order a node is read.
///
/// Not a template — each one answers a question that would otherwise be
/// answered in a corridor, and it is the same set for every node in every
/// layer. Counting them here in prose is how the count goes stale, so it is
/// the `tab-names` part that says how many there are. The empty states carry
/// as much weight as the filled ones: most of these are read by somebody about
/// to fill their first node, and "no data" teaches nothing.
pub fn fragment(sh: &Sheet, tree: &Tree) -> String {
    let mut o = String::new();
    let gaps = crate::emit::gap_pass(sh);

    // --- breadcrumb and identity -------------------------------------------
    o.push_str(&f(
        "article",
        &[
            ("id", &h(&sh.id)),
            ("sh_hash", &short_hex(sh.sheet_hash)),
            ("im_hash", &short_hex(sh.impl_hash)),
            ("crumb", &breadcrumb(sh, tree)),
            ("label", &h(&sh.label)),
            ("kind", &h(&sh.kind)),
            ("owner", &h(&sh.owner)),
            ("tier", &h(&sh.tier)),
            ("state", &h(&sh.state)),
        ],
    ));
    o.push_str(&answer_first(sh));

    o.push_str(t("tabs-open"));
    for (i, name) in lines("tab-names").iter().enumerate() {
        o.push_str(&f(
            "tab",
            &[
                ("sel", if i == 0 { " sel" } else { "" }),
                ("i", &i.to_string()),
                ("name", name),
            ],
        ));
    }
    o.push_str(t("tabs-close"));

    // --- 1 said simply, then the real thing --------------------------------
    //
    // The station order of docs/EXPLAINING.md: say it simply, then the real
    // thing with its source, then where the simple version breaks, then the
    // wrong idea most readers bring, then where to try it. Every claim carries
    // its kind — sourced, derived, declared — so a reader can tell a citation
    // from a working from a choice without asking.
    tab(&mut o, 0, true, |o| {
        if sh.question.trim().is_empty() {
            empty(o, t("simply-unspecified"));
            return;
        }
        o.push_str(t("station-open"));
        if sh.explain.simply.trim().is_empty() {
            o.push_str(t("simply-none"));
        } else {
            for para in paragraphs(&sh.explain.simply) {
                o.push_str(&f("simply-para", &[("text", &h(para))]));
            }
            if !sh.explain.by.is_empty() {
                o.push_str(&f("simply-by", &[("by", &h(&sh.explain.by))]));
            }
        }
        o.push_str(&f(
            "real",
            &[
                ("question", &h(&sh.question)),
                ("expression", &h(&sh.expression)),
                ("claim", &claim("sourced", &sh.source)),
            ],
        ));
        match tree.sources.get(&sh.source) {
            Some(s) => o.push_str(&f(
                "source",
                &[
                    ("id", &h(&s.id)),
                    ("title", &h(&s.title)),
                    ("where", &h(&s.where_)),
                ],
            )),
            None => o.push_str(&f("source-missing", &[("id", &h(&sh.source))])),
        }
        if sh.is_declared() {
            if let Some(v) = sh.value {
                o.push_str(&f(
                    "declared",
                    &[
                        ("value", &v.to_string()),
                        ("unit", &h(&unit_after(&sh.unit))),
                        ("claim", &claim("declared", &sh.confirmed_by)),
                    ],
                ));
            }
        }
        if !sh.note.is_empty() {
            o.push_str(&f("note", &[("text", &h(&sh.note))]));
        }
        o.push_str(t("breaks-h"));
        for para in paragraphs(&sh.explain.breaks) {
            o.push_str(&f("breaks-para", &[("text", &h(para))]));
        }
        if sh.assumptions.is_empty() {
            o.push_str(t("assumptions-none"));
        } else {
            o.push_str(t("assumptions-open"));
            for a in &sh.assumptions {
                o.push_str(&f(
                    "assumption",
                    &[("text", &h(&a.text)), ("fails_when", &h(&a.fails_when))],
                ));
            }
            o.push_str(t("assumptions-close"));
        }
        o.push_str(&f(
            "refuses-outside",
            &[
                ("lo", &sh.lower.to_string()),
                ("hi", &sh.upper.to_string()),
                ("unit", &h(&unit_after(&sh.unit))),
                ("reason_lower", &h(&sh.reason_lower)),
                ("reason_upper", &h(&sh.reason_upper)),
            ],
        ));
        if !sh.explain.wrong.trim().is_empty() {
            o.push_str(t("wrong-h"));
            for para in paragraphs(&sh.explain.wrong) {
                o.push_str(&f("wrong-para", &[("text", &h(para))]));
            }
        }
        o.push_str(t("try-it"));
    });

    // --- 2 theory -----------------------------------------------------------
    //
    // Where the relation came from, which is the one thing the tab before this
    // cannot say. `expression` is the relation as the generators need it: one
    // line, no reason. A reviewer who cannot reconstruct why that line is that
    // line has to take it on trust, and taking mathematics on trust is the
    // failure the two reviews exist to prevent.
    //
    // The derivation is rendered COMPLETE and visible. The face then offers to
    // walk it one line at a time, which is an addition to a readable page rather
    // than the only way to read it — a page.html opened straight off the disk,
    // with no engine and no script, still carries the whole argument.
    tab(&mut o, 1, false, |o| {
        if sh.theory.is_empty() {
            empty(o, t("theory-none"));
            return;
        }
        for para in paragraphs(&sh.theory.why) {
            o.push_str(&f("theory-why", &[("text", &h(para))]));
        }
        if !sh.theory.steps.is_empty() {
            o.push_str(&f(
                "theory-steps-open",
                &[
                    ("derived", &claim("derived", "")),
                    ("sourced", &claim("sourced", &sh.source)),
                    ("n", &sh.theory.steps.len().to_string()),
                ],
            ));
            for st in &sh.theory.steps {
                o.push_str(&f("theory-step", &[("text", &h(&st.text))]));
                if !st.math.trim().is_empty() {
                    o.push_str(&f("theory-step-math", &[("math", &h(&st.math))]));
                }
                o.push_str(t("theory-step-end"));
            }
            o.push_str(t("theory-steps-close"));
        }
        let read = paragraphs(&sh.theory.reading);
        if !read.is_empty() {
            o.push_str(t("theory-reading-h"));
            for para in read {
                o.push_str(&f("theory-reading", &[("text", &h(para))]));
            }
        }
        // The derivation ends at the relation, so the relation is repeated here
        // rather than left a tab away: a derivation whose conclusion is not in
        // front of the reader is an argument they have to hold in their head.
        if !sh.expression.trim().is_empty() {
            o.push_str(&f("theory-concl", &[("expression", &h(&sh.expression))]));
        }
    });

    // --- 3 de-risking --------------------------------------------------------
    //
    // Why the row is what it is. Each version says what it rests on and what
    // would break it; every version after the first says which belief broke,
    // what was tested, what we now know, and what changed. On a risk-register
    // row the register itself is here, and the face adds, from the whole tree,
    // every version that has moved each risk.
    tab(&mut o, 2, false, |o| {
        derisk_tab(o, sh, tree);
    });

    // --- 4 interface --------------------------------------------------------
    tab(&mut o, 3, false, |o| {
        o.push_str(t("iface-open"));
        if sh.inputs.is_empty() {
            o.push_str(t("iface-none"));
        }
        for i in &sh.inputs {
            // A member of a published set is not a node, so its unit lives on
            // the producing node's `[[publishes]]` entry and the page it opens
            // is that node's. The variable keeps its full dotted name: that is
            // what the reader has to match against the producer's table.
            o.push_str(&f(
                "iface-in",
                &[
                    ("binding", &h(&i.binding)),
                    ("goto", &h(producer_of(&i.var))),
                    ("var", &h(&i.var)),
                    ("ty", &h(&i.ty)),
                    ("unit", &h(&var_unit(tree, &i.var))),
                ],
            ));
        }
        o.push_str(&f(
            "iface-out",
            &[
                ("symbol", &h(&sh.symbol)),
                ("id", &h(&sh.id)),
                ("ty", &h(&sh.ty)),
                ("unit", &h(&unit_symbol(&sh.unit))),
            ],
        ));
        // A set row answers with more than one variable, and every one of them
        // is an interface. Leaving them off this table would say the node
        // publishes one thing when it publishes N — and the rule this table
        // states is that what is not here is not an interface.
        for pb in &sh.publishes {
            o.push_str(&f(
                "iface-member",
                &[
                    ("symbol", &h(&pb.symbol)),
                    ("id", &h(&sh.id)),
                    ("member", &h(&pb.id)),
                    ("ty", &h(&pb.ty)),
                    ("unit", &h(&unit_symbol(&pb.unit))),
                ],
            ));
        }
        o.push_str(t("table-close"));
        if !sh.publishes.is_empty() {
            o.push_str(&f(
                "iface-set",
                &[("n", &(1 + sh.publishes.len()).to_string()), ("s", "s")],
            ));
        }
        let consumers: Vec<&str> = tree
            .sheets
            .values()
            .filter(|c| c.inputs.iter().any(|i| producer_of(&i.var) == sh.id))
            .map(|c| c.id.as_str())
            .collect();
        let list = if consumers.is_empty() {
            t("consumers-none").to_string()
        } else {
            consumers
                .iter()
                .map(|c| f("xref", &[("id", c), ("text", c)]))
                .collect::<Vec<_>>()
                .join(t("list-sep"))
        };
        o.push_str(&f(
            "consumers",
            &[
                ("n", &consumers.len().to_string()),
                ("s", if consumers.len() == 1 { "" } else { "s" }),
                ("list", &list),
            ],
        ));
    });

    // --- 4 algorithm --------------------------------------------------------
    tab(&mut o, 4, false, |o| {
        if sh.steps.is_empty() {
            empty(o, t("algo-none"));
            if sh.is_declared() {
                o.push_str(&f(
                    "algo-declared",
                    &[
                        ("value", &sh.value.unwrap_or(0.0).to_string()),
                        ("unit", &h(&unit_symbol(&sh.unit))),
                        ("by", &h(&sh.confirmed_by)),
                    ],
                ));
            }
        } else {
            o.push_str(t("algo-open"));
            for st in &sh.steps {
                o.push_str(&f(
                    "algo-step",
                    &[
                        ("text", &h(&st.text)),
                        ("binds", &h(&st.binds)),
                        ("ty", &h(&st.ty)),
                    ],
                ));
            }
            o.push_str(t("algo-close"));
        }
        if !sh.is_seeded() {
            o.push_str(&pseudocode(sh));
        }
        o.push_str(&method_section(sh));
        o.push_str(&f(
            "guards",
            &[
                ("symbol", &h(&sh.symbol)),
                ("lo", &sh.lower.to_string()),
                ("hi", &sh.upper.to_string()),
                ("reason_lower", &h(&sh.reason_lower)),
                ("reason_upper", &h(&sh.reason_upper)),
            ],
        ));
    });

    // --- 5 the relation, moving ---------------------------------------------
    //
    // The same relation the tabs either side of this one state in symbols and
    // in code, walked. A reader who cannot yet read the expression can watch
    // the answer move as the input crosses its declared domain, and see where
    // the guards cut it off.
    //
    // The face fills this: the curve comes from the engine's own sweep, so what
    // animates here is what the node computes and not a second drawing of the
    // same idea. An empty host is the honest state when the node has nothing
    // upstream to sweep over.
    tab(&mut o, 5, false, |o| {
        if sh.is_seeded() {
            empty(o, t("relation-seeded"));
            return;
        }
        o.push_str(&f(
            "relation",
            &[
                ("id", &h(&sh.id)),
                ("lo", &sh.lower.to_string()),
                ("hi", &sh.upper.to_string()),
                ("symbol", &h(&sh.symbol)),
            ],
        ));
    });

    // --- 6 evidence ---------------------------------------------------------
    tab(&mut o, 6, false, |o| {
        if sh.fixtures.is_empty() {
            empty(o, t("fx-none"));
        } else {
            o.push_str(t("fx-open"));
            for x in &sh.fixtures {
                o.push_str(&f(
                    "fx-row",
                    &[
                        ("label", &h(&x.label)),
                        ("expect", &x.expect.to_string()),
                        ("tolerance", &x.tolerance.to_string()),
                        ("provenance", &h(&x.provenance)),
                        ("source", &h(&x.source)),
                    ],
                ));
            }
            o.push_str(t("fx-close"));
        }
        o.push_str(&cases_section(sh));
    });

    // --- 7 flags ------------------------------------------------------------
    tab(&mut o, 7, false, |o| {
        o.push_str(&f(
            "flags",
            &[
                ("lo", &sh.lower.to_string()),
                ("hi", &sh.upper.to_string()),
                ("reason_lower", &h(&sh.reason_lower)),
                ("reason_upper", &h(&sh.reason_upper)),
            ],
        ));
        if !sh.inputs.is_empty() {
            o.push_str(t("flag-blocked"));
        }
        for b in &sh.bundles {
            o.push_str(&f("flag-bundle", &[("bundle", &h(b))]));
        }
        o.push_str(t("flags-close"));
    });

    // --- 8 credibility ------------------------------------------------------
    tab(&mut o, 8, false, |o| {
        if sh.tier.is_empty() || sh.tier == "unset" {
            empty(o, t("tier-unset"));
        }
        let blind = match sh.tier.as_str() {
            "A+" | "A" | "B" | "C" => format!("tier-blind-{}", sh.tier),
            _ => "tier-blind-other".to_string(),
        };
        o.push_str(&f(
            "tier",
            &[
                ("tier", &h(&sh.tier)),
                ("blind", t(&blind)),
                ("id", &h(&sh.id)),
            ],
        ));
    });

    // --- 9 design space ----------------------------------------------------
    tab(&mut o, 9, false, |o| {
        o.push_str(&f(
            "space",
            &[
                ("lo", &sh.lower.to_string()),
                ("hi", &sh.upper.to_string()),
                ("unit", &h(&unit_symbol(&sh.unit))),
                ("reason_lower", &h(&sh.reason_lower)),
                ("reason_upper", &h(&sh.reason_upper)),
            ],
        ));
        match &sh.view {
            View::Number => o.push_str(t("view-number")),
            View::Line { over, points } => o.push_str(&f(
                "view-line",
                &[
                    ("over", &h(over)),
                    ("points", &points.to_string()),
                    ("y", &h(&sh.id)),
                ],
            )),
            View::Heatmap {
                over_x,
                over_y,
                points,
            } => o.push_str(&f(
                "view-heatmap",
                &[
                    ("x", &h(over_x)),
                    ("y", &h(over_y)),
                    ("points", &points.to_string()),
                ],
            )),
            View::Bar { y } => o.push_str(&f("view-bar", &[("y", &h(y))])),
        }
        o.push_str(t("space-close"));
    });

    // --- the gap list, always visible ---------------------------------------
    if !gaps.is_empty() {
        o.push_str(t("gaps-open"));
        for g in &gaps {
            o.push_str(&f("gap", &[("text", &h(g))]));
        }
        o.push_str(t("gaps-close"));
    }

    o.push_str(t("article-end"));
    o
}

/// The node in four lines, before any tab: what it asks, what it answers,
/// and the three things a reader needs to trust it — what it rests on, where
/// it holds, and where it comes from. Overview first; the tabs are the zoom
/// (docs/EXPLAINING.md E1, E6).
fn answer_first(sh: &Sheet) -> String {
    let mut o = String::from(t("af-open"));
    if sh.is_seeded() {
        o.push_str(t("af-seeded"));
        return o;
    }
    o.push_str(&f("af-question", &[("question", &h(&sh.question))]));
    let first = sh
        .explain
        .simply
        .split(". ")
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    o.push_str(&match (sh.is_declared(), sh.value) {
        (true, Some(v)) => f(
            "af-declared",
            &[
                ("value", &v.to_string()),
                ("unit", &h(&unit_after(&sh.unit))),
                ("claim", &claim("declared", &sh.confirmed_by)),
            ],
        ),
        _ => f("af-computed", &[("symbol", &h(&sh.symbol))]),
    });
    o.push_str(t("af-points"));
    if let Some(s) = first {
        o.push_str(&f(
            "af-simply",
            &[
                ("text", &h(s)),
                ("stop", if s.ends_with('.') { "" } else { "." }),
            ],
        ));
    }
    o.push_str(&match sh.versions.last() {
        Some(v) => f(
            "af-rests",
            &[
                ("rests_on", &h(&v.rests_on)),
                ("n", &v.n.to_string()),
                ("breaks_if", &h(&v.breaks_if)),
            ],
        ),
        None => t("af-no-belief").to_string(),
    });
    o.push_str(&f(
        "af-holds",
        &[
            ("lo", &sh.lower.to_string()),
            ("hi", &sh.upper.to_string()),
            ("unit", &h(&unit_after(&sh.unit))),
        ],
    ));
    o.push_str(&f("af-from", &[("claim", &claim("sourced", &sh.source))]));
    o
}

/// A claim's kind, on the claim: `sourced` names its source, `derived` was
/// worked here from sourced relations, `declared` names who chose it,
/// `illustrative` is an example and nothing more (E3).
fn claim(kind: &str, whose: &str) -> String {
    let title = match kind {
        "sourced" | "derived" | "declared" => format!("claim-title-{kind}"),
        _ => "claim-title-illustrative".to_string(),
    };
    let whose = if whose.trim().is_empty() {
        String::new()
    } else {
        f("claim-whose", &[("whose", &h(whose))])
    };
    f(
        "claim",
        &[("kind", kind), ("title", t(&title)), ("whose", &whose)],
    )
}

/// The de-risking tab: the current belief, then every version, newest first.
fn derisk_tab(o: &mut String, sh: &Sheet, tree: &Tree) {
    // A risk a version moved links to the register row that holds it, so a
    // reader can follow a belief to the risk it bought down.
    let holder = |id: &str| {
        tree.sheets
            .values()
            .find(|s| s.risks.iter().any(|r| r.id == id))
            .map(|s| s.id.clone())
    };
    let moves = |ms: &[String]| {
        ms.iter()
            .map(|m| {
                let id = m.split_whitespace().next().unwrap_or("");
                match holder(id) {
                    Some(row) => f("xref", &[("id", &h(&row)), ("text", &h(m))]),
                    None => h(m),
                }
            })
            .collect::<Vec<_>>()
            .join(t("version-risks-sep"))
    };
    match sh.versions.last() {
        None => empty(o, t("derisk-none")),
        Some(v) => {
            let release = if v.release == crate::derisk::NEXT {
                t("release-next").to_string()
            } else {
                f("release-since", &[("release", &h(&v.release))])
            };
            o.push_str(&f(
                "belief",
                &[
                    ("n", &v.n.to_string()),
                    ("release", &release),
                    ("date", &h(&v.date)),
                    ("rests_on", &h(&v.rests_on)),
                    ("breaks_if", &h(&v.breaks_if)),
                ],
            ));
            o.push_str(t("versions-open"));
            let names = lines("version-fields");
            for v in sh.versions.iter().rev() {
                o.push_str(&f(
                    "version",
                    &[
                        ("n", &v.n.to_string()),
                        ("release", &h(&v.release)),
                        ("date", &h(&v.date)),
                        ("by", &h(&v.by)),
                        ("about", &h(&v.about.join(", "))),
                    ],
                ));
                let fields = [
                    &v.believed,
                    &v.tested,
                    &v.learned,
                    &v.cost,
                    &v.changed,
                    &v.rests_on,
                    &v.breaks_if,
                ];
                for (k, x) in names.iter().zip(fields) {
                    if !x.trim().is_empty() {
                        o.push_str(&f("version-field", &[("name", k), ("text", &h(x))]));
                    }
                }
                if !v.risks.is_empty() {
                    o.push_str(&f("version-risks", &[("risks", &moves(&v.risks))]));
                }
                o.push_str(&f(
                    "version-end",
                    &[
                        ("relation", &h(&v.relation)),
                        ("claim", &claim("sourced", &v.source)),
                    ],
                ));
            }
            o.push_str(t("versions-close"));
        }
    }
    if sh.parent == crate::derisk::REGISTER {
        o.push_str(t("register-h"));
        if sh.risks.is_empty() {
            empty(o, t("register-none"));
        } else {
            o.push_str(t("register-open"));
            for r in &sh.risks {
                o.push_str(&f(
                    "register-risk",
                    &[
                        ("id", &h(&r.id)),
                        ("title", &h(&r.title)),
                        ("level", &h(&r.level)),
                        ("since", &h(&r.since)),
                        ("owner", &h(&r.owner)),
                        ("why", &h(&r.why)),
                    ],
                ));
            }
            o.push_str(t("table-close"));
        }
        o.push_str(&f("register-rollup", &[("id", &h(&sh.id))]));
    }
}

/// Prose split into its paragraphs. The loader has already reflowed each one, so
/// a blank line is the only break left and it is the one the author meant.
fn paragraphs(s: &str) -> Vec<&str> {
    s.split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect()
}

/// One tab's panel, with what kind of documentation it is (Diátaxis,
/// docs/EXPLAINING.md E8) on it, so a reader knows whether to read, look up
/// or try.
fn tab<F: FnOnce(&mut String)>(o: &mut String, i: usize, sel: bool, body: F) {
    let kinds = lines("tab-kinds");
    let kind = kinds[i];
    o.push_str(&f(
        "panel",
        &[
            ("sel", if sel { " sel" } else { "" }),
            ("i", &i.to_string()),
            ("kind", kind),
            ("title", t(&format!("kind-title-{kind}"))),
        ],
    ));
    body(o);
    o.push_str(t("panel-end"));
}

/// The node's method, whose it is, and — folded — the author's own code and
/// the flight software kept with the node. Part of the algorithm tab: the
/// method IS the algorithm, stated so the tool can check it.
fn method_section(sh: &Sheet) -> String {
    let mut o = String::new();
    if sh.method.text.trim().is_empty() {
        if !sh.is_seeded() && !sh.is_declared() {
            o.push_str(t("method-none"));
        }
    } else {
        o.push_str(&f(
            "method",
            &[("version", &crate::method::LANGUAGE_VERSION.to_string())],
        ));
        if !sh.method.by.trim().is_empty() {
            o.push_str(&f("method-by", &[("by", &h(&sh.method.by))]));
        }
        o.push_str(&f("code", &[("code", &h(&sh.method.text))]));
    }
    if !sh.author.is_empty() {
        let name = if sh.author.name.trim().is_empty() {
            t("author-name")
        } else {
            sh.author.name.trim()
        };
        let entry = if sh.author.entry.trim().is_empty() {
            String::new()
        } else {
            f("author-entry", &[("entry", &h(&sh.author.entry))])
        };
        o.push_str(&f(
            "author-open",
            &[
                ("name", &h(name)),
                ("language", &h(&sh.author.language)),
                ("entry", &entry),
                ("code", &h(&sh.author.code)),
            ],
        ));
        if !sh.author.test_code.trim().is_empty() {
            o.push_str(&f("author-test", &[("code", &h(&sh.author.test_code))]));
        }
        if !sh.author.how_run.trim().is_empty() {
            o.push_str(&f("author-run", &[("how", &h(&sh.author.how_run))]));
        }
        o.push_str(t("details-close"));
    }
    if !sh.flight.is_empty() {
        o.push_str(&f("flight-h", &[("n", &sh.flight.len().to_string())]));
        for x in &sh.flight {
            o.push_str(&f(
                "flight",
                &[
                    ("name", &h(&x.name)),
                    ("language", &h(&x.language)),
                    ("purpose", &h(&x.purpose)),
                    ("code", &h(&x.code)),
                ],
            ));
            if !x.test_code.trim().is_empty() {
                o.push_str(&f("flight-test", &[("code", &h(&x.test_code))]));
            }
            if !x.test_result.trim().is_empty() {
                o.push_str(&f("flight-result", &[("result", &h(&x.test_result))]));
            }
            o.push_str(t("details-close"));
        }
    }
    o
}

/// The author's test cases, each with what the method gives for it — the
/// check the form ran before the node was sent, run again here.
fn cases_section(sh: &Sheet) -> String {
    use crate::method;
    if sh.cases.is_empty() {
        return String::new();
    }
    let mut o = String::from(t("cases-open"));
    let sig = method::node_signature(sh).unwrap_or(method::Signature {
        inputs: Vec::new(),
        output: vleo_units::unit::Dim::NONE,
        publishes: Vec::new(),
    });
    let r = (!sh.method.text.trim().is_empty())
        .then(|| method::report(&sh.method.text, &sig, &sh.cases));
    for (i, c) in sh.cases.iter().enumerate() {
        let inputs = c
            .inputs
            .iter()
            .map(|(k, v)| format!("{k} = {}", method::show(*v)))
            .collect::<Vec<_>>()
            .join(", ");
        let want = match c.expect {
            Some(v) => method::show(v),
            None => t("case-refuses").into(),
        };
        let verdict = match &r {
            None => t("case-no-method").to_string(),
            Some(r) => match r.cases.get(i) {
                None => t("case-not-run").to_string(),
                Some((_, v)) => {
                    let got = r.got.get(i).copied().flatten();
                    let got = match (got, v.agrees(), c.refuses()) {
                        (Some(g), true, false) => f("case-got", &[("got", &method::show(g))]),
                        _ => String::new(),
                    };
                    f(
                        "case-verdict",
                        &[
                            ("class", if v.agrees() { "ok" } else { "bad" }),
                            ("text", &h(&v.text(c))),
                            ("got", &got),
                        ],
                    )
                }
            },
        };
        o.push_str(&f(
            "case",
            &[
                ("label", &h(&c.label)),
                ("inputs", &h(&inputs)),
                ("want", &h(&want)),
                (
                    "tolerance",
                    &if c.refuses() {
                        String::new()
                    } else {
                        format!("{:e}", c.tolerance)
                    },
                ),
                ("verdict", &verdict),
            ],
        ));
    }
    o.push_str(t("table-close"));
    o
}

fn empty(o: &mut String, msg: &str) {
    o.push_str(&f("empty", &[("text", &h(msg))]));
}

fn breadcrumb(sh: &Sheet, tree: &Tree) -> String {
    let mut chain = Vec::new();
    let mut cur = sh.parent.clone();
    let mut guard = 0;
    while let Some(g) = tree.groups.get(&cur) {
        chain.push(f(
            "crumb-link",
            &[("id", &h(&g.id)), ("label", &h(&g.label))],
        ));
        cur = g.parent.clone();
        guard += 1;
        if guard > 12 {
            break;
        }
    }
    chain.reverse();
    chain.join(t("crumb-sep"))
}
/// The index the shell loads: every row, its state, and all three graphs.
///
/// Small enough to ship inside the page, so first paint needs no server and
/// does not get slower as the tree fills. Opening a node costs one fragment.
pub fn index_json(tree: &Tree) -> String {
    let sheets = tree.ordered();
    let mut o = String::new();
    o.push_str("{\n  \"generator\": \"vleo xtask assemble\",\n");
    o.push_str(&format!("  \"nodes\": {},\n", sheets.len()));
    o.push_str("  \"rows\": [\n");
    for (i, sh) in sheets.iter().enumerate() {
        let gaps = crate::emit::gap_pass(sh).len();
        o.push_str(&format!(
            "    {{\"i\":{i},\"id\":\"{id}\",\"label\":\"{label}\",\"parent\":\"{par}\",\"sub\":\"{sub}\",\"kind\":\"{kind}\",\"state\":\"{state}\",\"owner\":\"{owner}\",\"tier\":\"{tier}\",\"unit\":\"{unit}\",\"symbol\":\"{sym}\",\"lo\":{lo},\"hi\":{hi},\"value\":{val},\"gaps\":{gaps},\"sheet\":\"{sh_hash}\",\"in\":[{ins}],\"kpi\":[{kpis}]}}{comma}\n",
            i = i,
            id = h(&sh.id),
            label = h(&sh.label),
            par = h(&sh.parent),
            sub = h(&sh.subsystem),
            kind = h(&sh.kind),
            state = h(&sh.state),
            owner = h(&sh.owner),
            tier = h(&sh.tier),
            unit = h(&unit_symbol(&sh.unit)),
            sym = h(&sh.symbol),
            lo = json_num(sh.lower * Unit::from_name(&sh.unit).unwrap_or(Unit::One).si_factor()),
            hi = json_num(sh.upper * Unit::from_name(&sh.unit).unwrap_or(Unit::One).si_factor()),
            val = sh
                .value
                .map(|v| json_num(v * Unit::from_name(&sh.unit).unwrap_or(Unit::One).si_factor()))
                .unwrap_or_else(|| "null".into()),
            gaps = gaps,
            sh_hash = short_hex(sh.sheet_hash),
            // `in` is a list of ROW indices, and a set row's members are not
            // rows: an input on `<node>.<member>` is an edge to `<node>`. A
            // node reading twenty members of one interface is one edge, so the
            // list is de-duplicated — twenty copies would draw twenty times.
            ins = {
                let mut seen: Vec<u16> = Vec::new();
                for x in &sh.inputs {
                    if let Some(r) = tree.index_of(producer_of(&x.var)) {
                        if !seen.contains(&r) {
                            seen.push(r);
                        }
                    }
                }
                seen.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            },
            kpis = sh
                .kpis
                .iter()
                .map(|k| format!("\"{}\"", h(k)))
                .collect::<Vec<_>>()
                .join(","),
            comma = if i + 1 == sheets.len() { "" } else { "," }
        ));
    }
    o.push_str("  ],\n  \"groups\": [\n");
    let groups: Vec<&Group> = tree.groups.values().collect();
    for (i, g) in groups.iter().enumerate() {
        o.push_str(&format!(
            "    {{\"id\":\"{}\",\"label\":\"{}\",\"parent\":\"{}\",\"owner\":\"{}\"}}{}\n",
            h(&g.id),
            h(&g.label),
            h(&g.parent),
            h(&g.owner),
            if i + 1 == groups.len() { "" } else { "," }
        ));
    }
    o.push_str("  ],\n  \"relations\": [\n");
    for (i, r) in tree.relations.iter().enumerate() {
        o.push_str(&format!(
            "    {{\"from\":\"{}\",\"to\":\"{}\",\"why\":\"{}\"}}{}\n",
            h(&r.from),
            h(&r.to),
            h(&r.why),
            if i + 1 == tree.relations.len() {
                ""
            } else {
                ","
            }
        ));
    }
    o.push_str("  ],\n  \"cases\": [\n");
    let cases: Vec<&Case> = tree.cases.values().collect();
    for (i, c) in cases.iter().enumerate() {
        o.push_str(&format!(
            "    {{\"id\":\"{}\",\"label\":\"{}\",\"note\":\"{}\",\"supply\":{{{}}}}}{}\n",
            h(&c.id),
            h(&c.label),
            h(&c.note),
            c.supply
                .iter()
                .map(|(k, v)| format!("\"{}\":{}", h(k), json_num(*v)))
                .collect::<Vec<_>>()
                .join(","),
            if i + 1 == cases.len() { "" } else { "," }
        ));
    }
    o.push_str("  ],\n  \"sources\": [\n");
    let srcs: Vec<&Source> = tree.sources.values().collect();
    for (i, s) in srcs.iter().enumerate() {
        o.push_str(&format!(
            "    {{\"id\":\"{}\",\"title\":\"{}\",\"where\":\"{}\",\"status\":\"{}\",\"used_for\":\"{}\"}}{}\n",
            h(&s.id),
            h(&s.title),
            h(&s.where_),
            h(&s.status),
            h(&s.used_for),
            if i + 1 == srcs.len() { "" } else { "," }
        ));
    }
    o.push_str("  ]\n}\n");
    o
}

fn json_num(v: f64) -> String {
    if v.is_finite() {
        format!("{v:?}")
    } else if v > 0.0 {
        "1e308".into()
    } else {
        "-1e308".into()
    }
}

/// Pseudocode, derived from the sheet rather than from the generated Rust.
///
/// The code tab already shows exactly what runs. This says the same thing in a
/// form somebody can read who does not read Rust, and — because it is built
/// from the sheet — it cannot drift from what the sheet declares. What it adds
/// over the numbered steps is the shape around them: the signature, the order,
/// and the guards as postconditions rather than as a list underneath.
fn pseudocode(sh: &Sheet) -> String {
    let mut o = String::from(t("pseudo-open"));
    let args: Vec<String> = sh
        .inputs
        .iter()
        .map(|i| {
            f(
                "pseudo-arg",
                &[("binding", &h(&i.binding)), ("ty", &h(&i.ty))],
            )
        })
        .collect();
    o.push_str(&f(
        "pseudo-sig",
        &[
            ("id", &h(&sh.id)),
            ("args", &args.join(t("list-sep"))),
            ("ty", &h(&sh.ty)),
        ],
    ));
    for i in &sh.inputs {
        o.push_str(&f(
            "pseudo-given",
            &[("binding", &h(&i.binding)), ("var", &h(&i.var))],
        ));
    }
    if sh.steps.is_empty() {
        if sh.is_declared() {
            // A dimensionless unit prints as "-", which beside a number reads
            // as a minus sign rather than as an absence of unit.
            let u = unit_symbol(&sh.unit);
            let u = if u == "-" {
                String::new()
            } else {
                format!(" {u}")
            };
            o.push_str(&f(
                "pseudo-declared",
                &[
                    ("symbol", &h(&sh.symbol)),
                    ("value", &sh.value.unwrap_or(0.0).to_string()),
                    ("unit", &h(&u)),
                    ("by", &h(&sh.confirmed_by)),
                ],
            ));
        }
    } else {
        for (n, st) in sh.steps.iter().enumerate() {
            // A step that binds `out` IS the answer — that is what the generated
            // model does with it — so it is named by the symbol here rather than
            // by the placeholder, or the pseudocode returns something it never
            // assigned.
            let bind = if st.binds == "out" {
                sh.symbol.as_str()
            } else {
                st.binds.as_str()
            };
            o.push_str(&f(
                "pseudo-step",
                &[
                    ("text", &h(&st.text)),
                    ("bind", &h(bind)),
                    ("n", &(n + 1).to_string()),
                ],
            ));
        }
    }
    o.push_str(&f(
        "pseudo-close",
        &[
            ("symbol", &h(&sh.symbol)),
            ("lo", &sh.lower.to_string()),
            ("hi", &sh.upper.to_string()),
        ],
    ));
    o
}

/// The node that answers a variable. For a member of a published set the
/// variable is `<node id>.<publish id>` and the node is the part before the
/// dot; a node id never contains one.
use crate::text::producer_of;

/// The unit a variable is measured in, whether it is a node's primary answer
/// or one member of a set that node publishes.
fn var_unit(tree: &Tree, var: &str) -> String {
    if let Some((node, member)) = var.split_once('.') {
        return tree
            .sheets
            .get(node)
            .and_then(|p| p.publishes.iter().find(|pb| pb.id == member))
            .map(|pb| unit_symbol(&pb.unit))
            .unwrap_or_else(|| "?".into());
    }
    tree.sheets
        .get(var)
        .map(|p| unit_symbol(&p.unit))
        .unwrap_or_else(|| "?".into())
}
