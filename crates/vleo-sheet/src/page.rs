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

use crate::load::Tree;
use crate::model::*;
use crate::short_hex;
use vleo_units::Unit;

fn h(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn unit_symbol(name: &str) -> String {
    Unit::from_name(name)
        .unwrap_or(Unit::One)
        .symbol()
        .to_string()
}

/// The tabs, in the order a node is read.
///
/// Not a template — each one answers a question that would otherwise be
/// answered in a corridor, and it is the same set for every node in every
/// layer. Counting them here in prose is how the count goes stale, so it is
/// [`TABS`] that says how many there are. The empty states carry as much weight as the filled ones: most of
/// these are read by somebody about to fill their first node, and "no data"
/// teaches nothing.
pub fn fragment(
    sh: &Sheet,
    holes: &std::collections::BTreeMap<u32, String>,
    tree: &Tree,
) -> String {
    let mut o = String::new();
    let gaps = crate::emit::gap_pass(sh, holes);
    o.push_str(&format!(
        "<article class=\"node\" data-node=\"{id}\" data-sheet-hash=\"{sh_hash}\" data-impl-hash=\"{im_hash}\">\n",
        id = h(&sh.id),
        sh_hash = short_hex(sh.sheet_hash),
        im_hash = short_hex(sh.impl_hash)
    ));

    // --- breadcrumb and identity -------------------------------------------
    o.push_str("<header class=\"node-head\">\n");
    o.push_str(&format!(
        "  <nav class=\"crumb\">{}</nav>\n",
        breadcrumb(sh, tree)
    ));
    o.push_str(&format!("  <h2>{}</h2>\n", h(&sh.label)));
    o.push_str(&format!(
        "  <p class=\"ident\"><code>{id}</code> · <span class=\"kind kind-{k}\">{k}</span> · owner <b>{o}</b> · tier <b>{t}</b> · state <b>{st}</b></p>\n",
        id = h(&sh.id),
        k = h(&sh.kind),
        o = h(&sh.owner),
        t = h(&sh.tier),
        st = h(&sh.state)
    ));
    o.push_str("</header>\n");

    o.push_str("<div class=\"tabs\" role=\"tablist\">\n");
    for (i, name) in TABS.iter().enumerate() {
        o.push_str(&format!(
            "  <button role=\"tab\" class=\"tab{sel}\" data-tab=\"{i}\">{n}</button>\n",
            sel = if i == 0 { " sel" } else { "" },
            i = i,
            n = name
        ));
    }
    o.push_str("</div>\n");

    // --- 1 question and mathematics ----------------------------------------
    tab(&mut o, 0, true, |o| {
        if sh.question.trim().is_empty() {
            empty(
                o,
                "Not yet specified. Needs a question, an expression and a source.",
            );
        } else {
            o.push_str(&format!("<p class=\"question\">{}</p>\n", h(&sh.question)));
            o.push_str(&format!(
                "<p class=\"maths\"><code>{}</code></p>\n",
                h(&sh.expression)
            ));
            let src = tree.sources.get(&sh.source);
            match src {
                Some(s) => o.push_str(&format!(
                    "<p class=\"src\">Source <b>{}</b> — {} <span class=\"where\">{}</span></p>\n",
                    h(&s.id),
                    h(&s.title),
                    h(&s.where_)
                )),
                None => o.push_str(&format!(
                    "<p class=\"src warn\">Source <code>{}</code> does not resolve to an entry in <code>sources/</code>.</p>\n",
                    h(&sh.source)
                )),
            }
        }
        if !sh.note.is_empty() {
            o.push_str(&format!("<p class=\"note\">{}</p>\n", h(&sh.note)));
        }
        if sh.assumptions.is_empty() {
            o.push_str("<p class=\"muted\">No assumption stated. A multi-step relation always has at least one.</p>\n");
        } else {
            o.push_str("<h4>Assumptions</h4>\n<dl class=\"asm\">\n");
            for a in &sh.assumptions {
                o.push_str(&format!(
                    "<dt>{}</dt><dd>fails when {}</dd>\n",
                    h(&a.text),
                    h(&a.fails_when)
                ));
            }
            o.push_str("</dl>\n");
        }
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
            empty(
                o,
                "Nobody has written the theory for this row yet. The relation is stated in the \
                 tab before this one and generated in the tab after it; what is missing is why \
                 it is that relation, which is the part a reader cannot recover from either.",
            );
            return;
        }
        for para in paragraphs(&sh.theory.why) {
            o.push_str(&format!("<p class=\"why\">{}</p>\n", h(para)));
        }
        if !sh.theory.steps.is_empty() {
            o.push_str(&format!(
                "<div class=\"theory-walk\" data-steps=\"{}\">\n",
                sh.theory.steps.len()
            ));
            o.push_str("<ol class=\"derive\">\n");
            for st in &sh.theory.steps {
                o.push_str(&format!("<li><span class=\"dt\">{}</span>", h(&st.text)));
                if !st.math.trim().is_empty() {
                    o.push_str(&format!("<code class=\"dm\">{}</code>", h(&st.math)));
                }
                o.push_str("</li>\n");
            }
            o.push_str("</ol>\n</div>\n");
        }
        let read = paragraphs(&sh.theory.reading);
        if !read.is_empty() {
            o.push_str("<h4>Reading the answer</h4>\n");
            for para in read {
                o.push_str(&format!("<p class=\"reading\">{}</p>\n", h(para)));
            }
        }
        // The derivation ends at the relation, so the relation is repeated here
        // rather than left a tab away: a derivation whose conclusion is not in
        // front of the reader is an argument they have to hold in their head.
        if !sh.expression.trim().is_empty() {
            o.push_str(&format!(
                "<p class=\"maths concl\">which is the relation this node states: <code>{}</code></p>\n",
                h(&sh.expression)
            ));
        }
    });

    // --- 3 interface --------------------------------------------------------
    tab(&mut o, 2, false, |o| {
        o.push_str("<p class=\"muted\">Every input and output, typed, with its unit. If it is not here it is not an interface.</p>\n");
        o.push_str("<table class=\"iface\"><thead><tr><th>direction</th><th>symbol</th><th>variable</th><th>type</th><th>unit</th></tr></thead><tbody>\n");
        if sh.inputs.is_empty() {
            o.push_str("<tr class=\"muted\"><td colspan=\"5\">Inputs and outputs are declared. Units are not — a unit is a decision.</td></tr>\n");
        }
        for i in &sh.inputs {
            // A member of a published set is not a node, so its unit lives on
            // the producing node's `[[publishes]]` entry and the page it opens
            // is that node's. The variable keeps its full dotted name: that is
            // what the reader has to match against the producer's table.
            let unit = var_unit(tree, &i.var);
            o.push_str(&format!(
                "<tr><td>in</td><td><code>{}</code></td><td><a class=\"xref\" data-goto=\"{g}\">{v}</a></td><td>{t}</td><td>{u}</td></tr>\n",
                h(&i.binding),
                g = h(producer_of(&i.var)),
                v = h(&i.var),
                t = h(&i.ty),
                u = h(&unit)
            ));
        }
        o.push_str(&format!(
            "<tr class=\"out\"><td>out</td><td><code>{s}</code></td><td>{id}</td><td>{t}</td><td>{u}</td></tr>\n",
            s = h(&sh.symbol),
            id = h(&sh.id),
            t = h(&sh.ty),
            u = h(&unit_symbol(&sh.unit))
        ));
        // A set row answers with more than one variable, and every one of them
        // is an interface. Leaving them off this table would say the node
        // publishes one thing when it publishes N — and the rule this table
        // states is that what is not here is not an interface.
        for pb in &sh.publishes {
            o.push_str(&format!(
                "<tr class=\"out member\"><td>out</td><td><code>{s}</code></td><td>{id}.{m}</td><td>{t}</td><td>{u}</td></tr>\n",
                s = h(&pb.symbol),
                id = h(&sh.id),
                m = h(&pb.id),
                t = h(&pb.ty),
                u = h(&unit_symbol(&pb.unit))
            ));
        }
        o.push_str("</tbody></table>\n");
        if !sh.publishes.is_empty() {
            o.push_str(&format!(
                "<p class=\"muted\">This node answers with a set: <b>{}</b> variable{} in all. The first is the answer the node is named for; the rest are read by name.</p>\n",
                1 + sh.publishes.len(),
                if sh.publishes.is_empty() { "" } else { "s" }
            ));
        }
        let consumers: Vec<&str> = tree
            .sheets
            .values()
            .filter(|c| c.inputs.iter().any(|i| producer_of(&i.var) == sh.id))
            .map(|c| c.id.as_str())
            .collect();
        o.push_str(&format!(
            "<p class=\"consumers\"><b>{}</b> node{} read this: {}</p>\n",
            consumers.len(),
            if consumers.len() == 1 { "" } else { "s" },
            if consumers.is_empty() {
                "nothing yet.".to_string()
            } else {
                consumers
                    .iter()
                    .map(|c| format!("<a class=\"xref\" data-goto=\"{c}\">{c}</a>"))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ));
    });

    // --- 4 algorithm --------------------------------------------------------
    tab(&mut o, 3, false, |o| {
        if sh.steps.is_empty() {
            empty(
                o,
                "No steps yet. Each step becomes one hole in the generated code.",
            );
            if sh.is_declared() {
                o.push_str(&format!(
                    "<p>This is a declared value: <b>{v}</b> {u}, confirmed by <b>{c}</b>.</p>\n",
                    v = sh.value.unwrap_or(0.0),
                    u = h(&unit_symbol(&sh.unit)),
                    c = h(&sh.confirmed_by)
                ));
            }
        } else {
            o.push_str("<ol class=\"algo\">\n");
            for st in &sh.steps {
                o.push_str(&format!(
                    "<li><span class=\"step\">{}</span> <span class=\"binds\">→ <code>{}: {}</code></span></li>\n",
                    h(&st.text),
                    h(&st.binds),
                    h(&st.ty)
                ));
            }
            o.push_str("</ol>\n");
        }
        if !sh.is_seeded() {
            o.push_str(&pseudocode(sh));
        }
        o.push_str("<h4>Guards</h4>\n<ul class=\"guards\">\n");
        o.push_str(&format!(
            "<li><code>{s} &ge; {lo}</code> — {r}</li>\n",
            s = h(&sh.symbol),
            lo = sh.lower,
            r = h(&sh.reason_lower)
        ));
        o.push_str(&format!(
            "<li><code>{s} &le; {hi}</code> — {r}</li>\n",
            s = h(&sh.symbol),
            hi = sh.upper,
            r = h(&sh.reason_upper)
        ));
        o.push_str("</ul>\n<p class=\"muted\">The reason travels with the guard. A guard whose reason is not written down gets deleted by the next person who finds it awkward.</p>\n");
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
    tab(&mut o, 4, false, |o| {
        if sh.is_seeded() {
            empty(
                o,
                "Nothing to walk: the sheet is seeded and the node returns NotRun.",
            );
            return;
        }
        o.push_str(&format!(
            "<div class=\"relation-host\" data-node=\"{}\" data-lo=\"{}\" data-hi=\"{}\" data-symbol=\"{}\">\n",
            h(&sh.id),
            sh.lower,
            sh.upper,
            h(&sh.symbol)
        ));
        o.push_str("<p class=\"muted\">asking the engine…</p>\n</div>\n");
    });

    // --- 6 generated code ---------------------------------------------------
    tab(&mut o, 5, false, |o| {
        if sh.steps.is_empty() && sh.value.is_none() {
            empty(o, "Nothing generated — the sheet is incomplete.");
        } else {
            o.push_str("<p class=\"muted\">This is the file compiled into the engine that returns the number above. There is no version of this where the page and the code disagree, because disagreement is a build failure.</p>\n");
            o.push_str("<pre class=\"code\"><code>");
            o.push_str(&h(&crate::emit::model_rs(sh, holes)));
            o.push_str("</code></pre>\n");
        }
    });

    // --- 7 evidence ---------------------------------------------------------
    tab(&mut o, 6, false, |o| {
        if sh.fixtures.is_empty() {
            empty(
                o,
                "No known-good numbers yet. A number from our own code does not count.",
            );
        } else {
            o.push_str("<table class=\"fx\"><thead><tr><th>case</th><th>expected</th><th>tolerance</th><th>provenance</th><th>source</th></tr></thead><tbody>\n");
            for f in &sh.fixtures {
                o.push_str(&format!(
                    "<tr data-fixture=\"{l}\"><td>{l}</td><td class=\"num\">{e}</td><td class=\"num\">{t}</td><td>{p}</td><td>{s}</td></tr>\n",
                    l = h(&f.label),
                    e = f.expect,
                    t = f.tolerance,
                    p = h(&f.provenance),
                    s = h(&f.source)
                ));
            }
            o.push_str("</tbody></table>\n");
            o.push_str("<p class=\"muted\">An expected value may never be produced by the code under test. The schema refuses a row whose provenance is the implementation.</p>\n");
        }
    });

    // --- 8 flags ------------------------------------------------------------
    tab(&mut o, 7, false, |o| {
        o.push_str("<p class=\"muted\">What this node refuses, and what it returns when it refuses.</p>\n<ul class=\"flags\">\n");
        o.push_str(&format!(
            "<li><code>OutOfDomain</code> below {lo} — {rl}</li>\n<li><code>OutOfDomain</code> above {hi} — {ru}</li>\n",
            lo = sh.lower,
            hi = sh.upper,
            rl = h(&sh.reason_lower),
            ru = h(&sh.reason_upper)
        ));
        o.push_str("<li><code>Degenerate</code> when the computation produces a value that is not a number</li>\n");
        if !sh.inputs.is_empty() {
            o.push_str("<li><code>Blocked</code> when any declared input has never run — and it names which</li>\n");
        }
        for b in &sh.bundles {
            o.push_str(&format!("<li><code>DataMissing</code> / <code>DataUnverified</code> for bundle <b>{}</b></li>\n", h(b)));
        }
        o.push_str("</ul>\n<p class=\"muted\">Out of domain is an error naming the field and the bound, at every face. A value silently corrected is a design that drifted without anyone deciding to.</p>\n");
    });

    // --- 9 credibility ------------------------------------------------------
    tab(&mut o, 8, false, |o| {
        if sh.tier.is_empty() || sh.tier == "unset" {
            empty(
                o,
                "Tier not set — this decides how much evidence the gate demands.",
            );
        }
        o.push_str(&format!(
            "<p>Evidence tier <b>{t}</b>. What this tier cannot detect: {c}.</p>\n",
            t = h(&sh.tier),
            c = match sh.tier.as_str() {
                "A+" => "an error common to both independent routes",
                "A" => "an error in the published source itself",
                "B" => "an error the independent tool shares with this one",
                "C" => "a wrong constant that still conserves the quantity",
                _ => "anything",
            }
        ));
        o.push_str("<div class=\"cred\" data-cred-for=\"");
        o.push_str(&h(&sh.id));
        o.push_str("\"><p class=\"muted\">Eight factors, and the lowest governs. Computed on the run, never stored — a stored badge is a claim about last March. Run this node to fill it in.</p></div>\n");
    });

    // --- 10 design space ----------------------------------------------------
    tab(&mut o, 9, false, |o| {
        o.push_str(&format!(
            "<p>Valid over <code>{lo} &hellip; {hi}</code> {u}.</p>\n",
            lo = sh.lower,
            hi = sh.upper,
            u = h(&unit_symbol(&sh.unit))
        ));
        o.push_str(&format!(
            "<ul class=\"guards\"><li>lower — {}</li><li>upper — {}</li></ul>\n",
            h(&sh.reason_lower),
            h(&sh.reason_upper)
        ));
        match &sh.view {
            View::Number => o.push_str("<p class=\"muted\">This node draws as a number with its unit, its verdict and its provenance — which is what all but a handful of nodes want.</p>\n"),
            View::Line { over, points } => o.push_str(&format!(
                "<div class=\"sweep\" data-over=\"{o}\" data-points=\"{p}\" data-y=\"{y}\"><p class=\"muted\">Sweeps against <a class=\"xref\" data-goto=\"{o}\">{o}</a> over {p} points. Run the branch to draw it.</p></div>\n",
                o = h(over), p = points, y = h(&sh.id))),
            View::Heatmap { over_x, over_y, points } => o.push_str(&format!(
                "<div class=\"field\" data-x=\"{x}\" data-y=\"{yy}\" data-points=\"{p}\"><p class=\"muted\">A field over {x} and {yy}.</p></div>\n",
                x = h(over_x), yy = h(over_y), p = points)),
            View::Bar { y } => o.push_str(&format!(
                "<div class=\"bars\" data-y=\"{y}\"><p class=\"muted\">Drawn as a contribution bar per term.</p></div>\n", y = h(y))),
        }
        o.push_str("<p class=\"muted\">Extrapolation outside the declared range is the most common silent error in this class of model. A node that means to extrapolate declares a wider range and says why.</p>\n");
    });

    // --- the gap list, always visible ---------------------------------------
    if !gaps.is_empty() {
        o.push_str("<section class=\"gaps\"><h4>Open gaps</h4><ul>\n");
        for g in &gaps {
            o.push_str(&format!("<li>{}</li>\n", h(g)));
        }
        o.push_str("</ul><p class=\"muted\">The gap pass lists what the sheet promised and nothing yet covers. It is a list, not a verdict — but a node with an open gap cannot enter the second review.</p></section>\n");
    }

    o.push_str("</article>\n");
    o
}

const TABS: [&str; 10] = [
    "question &amp; mathematics",
    "theory",
    "interface",
    "algorithm",
    "the relation, moving",
    "generated code",
    "evidence",
    "flags",
    "credibility",
    "design space",
];

/// Prose split into its paragraphs. The loader has already reflowed each one, so
/// a blank line is the only break left and it is the one the author meant.
fn paragraphs(s: &str) -> Vec<&str> {
    s.split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect()
}

fn tab<F: FnOnce(&mut String)>(o: &mut String, i: usize, sel: bool, body: F) {
    o.push_str(&format!(
        "<section class=\"panel{}\" data-panel=\"{}\">\n",
        if sel { " sel" } else { "" },
        i
    ));
    body(o);
    o.push_str("</section>\n");
}

fn empty(o: &mut String, msg: &str) {
    o.push_str(&format!("<p class=\"empty\">{}</p>\n", h(msg)));
}

fn breadcrumb(sh: &Sheet, tree: &Tree) -> String {
    let mut chain = Vec::new();
    let mut cur = sh.parent.clone();
    let mut guard = 0;
    while let Some(g) = tree.groups.get(&cur) {
        chain.push(format!(
            "<a class=\"xref\" data-group=\"{}\">{}</a>",
            h(&g.id),
            h(&g.label)
        ));
        cur = g.parent.clone();
        guard += 1;
        if guard > 12 {
            break;
        }
    }
    chain.reverse();
    chain.join(" <span class=\"sep\">/</span> ")
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
        let gaps = crate::emit::gap_pass(sh, &crate::load::read_holes(&sh.dir)).len();
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
    let mut o = String::from("<h4>Pseudocode</h4>\n<pre class=\"pseudo\"><code>");
    let args: Vec<String> = sh
        .inputs
        .iter()
        .map(|i| format!("{}: {}", h(&i.binding), h(&i.ty)))
        .collect();
    o.push_str(&format!(
        "function {}({}) -> {}\n",
        h(&sh.id),
        args.join(", "),
        h(&sh.ty)
    ));
    for i in &sh.inputs {
        o.push_str(&format!("    given {} from {}\n", h(&i.binding), h(&i.var)));
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
            o.push_str(&format!(
                "    {} \u{2190} {}{}          // declared, confirmed by {}\n",
                h(&sh.symbol),
                sh.value.unwrap_or(0.0),
                h(&u),
                h(&sh.confirmed_by)
            ));
        }
    } else {
        for (n, st) in sh.steps.iter().enumerate() {
            o.push_str(&format!("    // {}\n", h(&st.text)));
            // A step that binds `out` IS the answer — that is what the generated
            // model does with it — so it is named by the symbol here rather than
            // by the placeholder, or the pseudocode returns something it never
            // assigned.
            let bind = if st.binds == "out" {
                sh.symbol.as_str()
            } else {
                st.binds.as_str()
            };
            o.push_str(&format!("    {} \u{2190} step {}\n", h(bind), n + 1));
        }
    }
    o.push_str(&format!(
        "\n    // a guard whose reason is not written down gets deleted\n    require {s} \u{2265} {lo}\n    require {s} \u{2264} {hi}\n    require {s} is finite\n\n    return {s}\n",
        s = h(&sh.symbol),
        lo = sh.lower,
        hi = sh.upper
    ));
    o.push_str("</code></pre>\n<p class=\"muted\">Derived from the sheet, not from the generated Rust, so it says what the sheet declares rather than what one compiler made of it. The guards are postconditions: this node refuses rather than returning a number outside them.</p>\n");
    o
}

/// The node that answers a variable. For a member of a published set the
/// variable is `<node id>.<publish id>` and the node is the part before the
/// dot; a node id never contains one.
fn producer_of(var: &str) -> &str {
    match var.split_once('.') {
        Some((node, _)) => node,
        None => var,
    }
}

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
