//! The generators.
//!
//! Six per-node generators that never read another node, and three assembly
//! generators that never decide anything. Output is byte-stable: no timestamps,
//! no absolute paths in committed files, and sorted iteration everywhere —
//! otherwise the regenerate-and-compare check fails at random and within a
//! fortnight nobody reads it.

use crate::model::*;
use crate::{load::Tree, short_hex};
use std::collections::BTreeMap;
use vleo_units::Unit;

const BANNER: &str = "// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a\n\
                      // numbered HOLE block: a hand edit anywhere else is discarded by the next\n\
                      // regeneration and fails the regeneration diff in the gate.\n";

fn unit_of(name: &str) -> Unit {
    Unit::from_name(name).unwrap_or(Unit::One)
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ---------------------------------------------------------------------------
// 1. the implementation scaffold
// ---------------------------------------------------------------------------

/// The whole file, with one hole per numbered step.
///
/// The signature, the unit types, the domain guard with its reason, the fault
/// construction and the ordering are all functions of the sheet. Only the body
/// of each numbered step is left for a person, and a hole is usually two or
/// three typed lines. Structural verification is inherited by every node at
/// once; the arithmetic is not, and that is why the holes are verified
/// conventionally.
pub fn model_rs(sh: &Sheet, holes: &BTreeMap<u32, String>) -> String {
    let mut o = String::new();
    o.push_str(BANNER);
    o.push_str(
        "#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]\n\n",
    );
    o.push_str("use vleo_core::fault::{Edge, Fault};\n");
    o.push_str("use vleo_core::physics::*;\n");
    o.push_str("use vleo_core::units::pmath;\n");
    o.push_str("use vleo_core::units::*;\n\n");

    o.push_str(&format!("/// {}\n///\n", sh.question));
    o.push_str(&format!("/// `{}`\n///\n", sh.expression));
    o.push_str(&format!("/// Source: `{}`\n", sh.source));
    if !sh.note.is_empty() {
        o.push_str("///\n");
        for line in wrap(&sh.note, 74) {
            o.push_str(&format!("/// {line}\n"));
        }
    }
    if !sh.assumptions.is_empty() {
        o.push_str("///\n/// # Assumptions\n///\n");
        for a in &sh.assumptions {
            o.push_str(&format!("/// * {} — fails when {}\n", a.text, a.fails_when));
        }
    }
    o.push_str("pub const NODE_ID: &str = \"");
    o.push_str(&sh.id);
    o.push_str("\";\n");
    o.push_str(&format!(
        "/// Hash of the sheet this file was generated from. A face carrying a\n\
         /// different one refuses to run rather than showing a stale page.\n\
         pub const SHEET_HASH: u64 = 0x{:016x};\n\n",
        sh.sheet_hash
    ));

    let args = sh
        .inputs
        .iter()
        .map(|i| format!("{}: {}", i.binding, i.ty))
        .collect::<Vec<_>>()
        .join(", ");
    o.push_str(&format!(
        "pub fn evaluate({args}) -> Result<{ty}, Fault> {{\n",
        ty = sh.ty
    ));

    let last = if sh.steps.is_empty() {
        // A declared value publishes itself. Nothing is computed, and the
        // conversion from the unit it was written in is explicit rather than a
        // constant somebody folded by hand.
        o.push_str(
            "    // generated · a declared value, converted from the unit it was written in\n",
        );
        o.push_str(&format!(
            "    let declared: {ty} = match {ty}::from_unit({v:?}, Unit::{u}) {{\n\
             \x20       Some(q) => q,\n\
             \x20       None => return Err(Fault::Degenerate {{ node: NODE_ID, field: \"{sym}\", reason: \"the declared unit does not match the declared type\" }}),\n\
             \x20   }};\n",
            ty = sh.ty,
            v = sh.value.unwrap_or(0.0),
            u = unit_of(&sh.unit).name(),
            sym = esc(&sh.symbol)
        ));
        "declared".to_string()
    } else {
        for st in &sh.steps {
            o.push_str(&format!(
                "    // ---- HOLE {n} : {text} -> {ty}\n",
                n = st.number,
                text = st.text,
                ty = st.ty
            ));
            match holes.get(&st.number) {
                Some(body) if !body.trim().is_empty() => {
                    for line in dedent(body).lines() {
                        if line.trim().is_empty() {
                            o.push('\n');
                        } else {
                            o.push_str("    ");
                            o.push_str(line.trim_end());
                            o.push('\n');
                        }
                    }
                }
                _ => {
                    o.push_str(&format!(
                        "    let {b}: {t} = todo!(\"hole {n} is empty — the gap pass blocks this node\");\n",
                        b = st.binds,
                        t = st.ty,
                        n = st.number
                    ));
                }
            }
            o.push_str(&format!("    // ---- end HOLE {}\n", st.number));
        }
        sh.steps.last().unwrap().binds.clone()
    };

    o.push_str("\n    // generated · the declared domain of this node's own answer. The\n");
    o.push_str("    // reason travels with the guard, because a guard whose reason is not\n");
    o.push_str("    // written down gets deleted by the next person who finds it awkward.\n");
    o.push_str(&format!("    let answer: {ty} = {last};\n", ty = sh.ty));
    o.push_str("    if !answer.is_finite() {\n");
    o.push_str(&format!(
        "        return Err(Fault::Degenerate {{ node: NODE_ID, field: \"{}\", reason: \"the computation produced a value that is not a number\" }});\n",
        esc(&sh.symbol)
    ));
    o.push_str("    }\n");
    let lo_si = to_si(sh.lower, &sh.unit);
    let hi_si = to_si(sh.upper, &sh.unit);
    if lo_si.is_finite() {
        o.push_str(&format!(
            "    if answer.get() < {lo:?} {{\n        return Err(Fault::OutOfDomain {{ node: NODE_ID, field: \"{sym}\", value: answer.get(), bound: {lo:?}, edge: Edge::Lower, unit: {ty}::UNIT, reason: \"{r}\" }});\n    }}\n",
            lo = lo_si,
            sym = esc(&sh.symbol),
            ty = sh.ty,
            r = esc(&sh.reason_lower)
        ));
    }
    if hi_si.is_finite() {
        o.push_str(&format!(
            "    if answer.get() > {hi:?} {{\n        return Err(Fault::OutOfDomain {{ node: NODE_ID, field: \"{sym}\", value: answer.get(), bound: {hi:?}, edge: Edge::Upper, unit: {ty}::UNIT, reason: \"{r}\" }});\n    }}\n",
            hi = hi_si,
            sym = esc(&sh.symbol),
            ty = sh.ty,
            r = esc(&sh.reason_upper)
        ));
    }
    o.push_str("    Ok(answer)\n}\n");
    o
}

fn to_si(v: f64, unit: &str) -> f64 {
    v * unit_of(unit).si_factor()
}

/// Strip the common leading indentation, keeping relative indentation inside a
/// multi-line body. Without this, splicing an already-indented body adds four
/// spaces on every regeneration and the generator is not a function of its
/// input.
fn dedent(body: &str) -> String {
    let min = body
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    body.lines()
        .map(|l| {
            if l.len() >= min {
                &l[min..]
            } else {
                l.trim_start()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn wrap(s: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for w in s.split_whitespace() {
        if line.len() + w.len() + 1 > width && !line.is_empty() {
            out.push(line.clone());
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(w);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

// ---------------------------------------------------------------------------
// 2. the contract — the untyped adapter the bus calls
// ---------------------------------------------------------------------------

/// A node publishes a contract: what it reads, what it promises, in what units.
/// The adapter is generated so no face can call a node with the arguments in
/// the wrong order, and so the bus never has to know a quantity type.
pub fn contract_rs(sh: &Sheet) -> String {
    let mut o = String::new();
    o.push_str(BANNER);
    o.push_str("#![allow(unused_variables)]\n\n");
    o.push_str("use vleo_core::fault::Fault;\n");
    o.push_str("use vleo_core::units::*;\n\n");
    o.push_str(&format!(
        "/// What this node publishes: `{}` ({}), in `{}`.\n",
        sh.symbol,
        sh.label,
        unit_of(&sh.unit).symbol()
    ));
    o.push_str("pub const NODE_ID: &str = \"");
    o.push_str(&sh.id);
    o.push_str("\";\n");
    o.push_str(&format!(
        "pub const SHEET_HASH: u64 = 0x{:016x};\n",
        sh.sheet_hash
    ));
    o.push_str("/// The variables this node reads, in the order `call` expects them.\n");
    o.push_str("pub const INPUT_VARS: &[&str] = &[\n");
    for i in &sh.inputs {
        o.push_str(&format!("    \"{}\",\n", i.var));
    }
    o.push_str("];\n");
    o.push_str("/// The variables this node publishes.\n");
    o.push_str(&format!(
        "pub const OUTPUT_VARS: &[&str] = &[\"{}\"];\n",
        sh.id
    ));
    o.push_str(&format!(
        "/// The SI unit every value crossing this boundary is expressed in.\n\
         pub const OUTPUT_UNIT: Unit = {}::UNIT;\n\n",
        sh.ty
    ));
    o.push_str(
        "/// The untyped adapter. Values cross as SI `f64` and are re-typed here,\n\
         /// so the bus carries no quantity types and a face cannot pass arguments\n\
         /// in the wrong order.\n",
    );
    o.push_str("pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {\n");
    match sh.inputs.len() {
        0 => o.push_str("    if outputs.is_empty() {\n"),
        1 => o.push_str("    if inputs.is_empty() || outputs.is_empty() {\n"),
        n => o.push_str(&format!(
            "    if inputs.len() < {n} || outputs.is_empty() {{\n"
        )),
    }
    o.push_str("        return Err(Fault::Blocked { node: NODE_ID, missing: \"an input the contract declares\" });\n");
    o.push_str("    }\n");
    for (n, i) in sh.inputs.iter().enumerate() {
        o.push_str(&format!(
            "    let {b}: {t} = {t}::new(inputs[{n}]);\n",
            b = i.binding,
            t = i.ty,
            n = n
        ));
    }
    let args = sh
        .inputs
        .iter()
        .map(|i| i.binding.clone())
        .collect::<Vec<_>>()
        .join(", ");
    o.push_str(&format!(
        "    let answer = super::model::evaluate({args})?;\n"
    ));
    o.push_str("    outputs[0] = answer.get();\n    Ok(())\n}\n");
    o
}

// ---------------------------------------------------------------------------
// 3. the module wiring
// ---------------------------------------------------------------------------

pub fn mod_rs(sh: &Sheet) -> String {
    let mut o = String::new();
    o.push_str(BANNER);
    o.push_str(&format!("//! `{}` — {}\n//!\n", sh.id, sh.label));
    o.push_str(&format!("//! {}\n", sh.question));
    o.push_str("\n#[path = \"model.rs\"]\npub mod model;\n");
    o.push_str("#[path = \"contract.rs\"]\npub mod contract;\n");
    o.push_str("#[cfg(test)]\n#[path = \"evidence.rs\"]\nmod evidence;\n\n");
    o.push_str(
        "pub use contract::{call, INPUT_VARS, NODE_ID, OUTPUT_UNIT, OUTPUT_VARS, SHEET_HASH};\n",
    );
    o
}

// ---------------------------------------------------------------------------
// 4. the evidence harness
// ---------------------------------------------------------------------------

/// The fixtures, as tests. The same function the run calls, called from the
/// test — one implementation, checked one way.
pub fn evidence_rs(sh: &Sheet) -> String {
    let mut o = String::new();
    o.push_str(BANNER);
    o.push_str(&format!(
        "//! Evidence for `{}`.\n//!\n\
         //! Every expected value below names a source outside this code. A number\n\
         //! produced by the thing being tested proves nothing, so the schema\n\
         //! refuses a fixture whose provenance is the implementation.\n\n",
        sh.id
    ));
    if sh.fixtures.is_empty() {
        o.push_str(
            "// No fixtures yet. The gap pass reports this node as unevidenced and\n\
             // its validation credibility factor is zero, which governs the whole\n\
             // vector — an unvalidated node cannot be quietly relied on.\n",
        );
        return o;
    }
    // A fixture input that happens to equal a named constant is still a fixture
    // input: substituting the constant would make the test compare the
    // implementation against itself.
    o.push_str("#![allow(clippy::approx_constant, clippy::excessive_precision)]\n\n");
    o.push_str("use super::model;\nuse vleo_core::units::*;\n\n");
    o.push_str("fn relative_error(got: f64, expected: f64) -> f64 {\n");
    o.push_str("    if expected == 0.0 { pmath::abs(got) } else { pmath::abs((got - expected) / expected) }\n}\n\n");
    for (n, fx) in sh.fixtures.iter().enumerate() {
        o.push_str(&format!("/// {}\n", fx.label));
        o.push_str(&format!(
            "///\n/// Provenance: `{}`, source `{}`.\n",
            fx.provenance, fx.source
        ));
        o.push_str(&format!("#[test]\nfn fixture_{n}() {{\n"));
        let mut args = Vec::new();
        for i in &sh.inputs {
            let v = fx
                .inputs
                .iter()
                .find(|(k, _)| *k == i.binding)
                .map(|(_, v)| *v)
                .unwrap_or(0.0);
            args.push(format!("{}::new({:?})", i.ty, v));
        }
        o.push_str(&format!(
            "    let got = model::evaluate({}).expect(\"the fixture case must not be refused\");\n",
            args.join(", ")
        ));
        o.push_str(&format!(
            "    let err = relative_error(got.get(), {:?});\n",
            fx.expect
        ));
        o.push_str(&format!(
            "    assert!(err <= {tol:?}, \"{label}: got {{}} want {expect:?}, relative error {{}} exceeds the declared tolerance {tol:?}. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.\", got.get(), err);\n",
            tol = fx.tolerance,
            label = esc(&fx.label),
            expect = fx.expect
        ));
        o.push_str("}\n\n");
    }
    o
}

// ---------------------------------------------------------------------------
// 5. the metadata the gate writes
// ---------------------------------------------------------------------------

pub fn meta_json(sh: &Sheet, gaps: &[String]) -> String {
    let mut o = String::new();
    o.push_str("{\n");
    o.push_str(&format!("  \"id\": \"{}\",\n", esc(&sh.id)));
    o.push_str(&format!("  \"state\": \"{}\",\n", esc(&sh.state)));
    o.push_str(&format!(
        "  \"sheet_hash\": \"{}\",\n",
        short_hex(sh.sheet_hash)
    ));
    o.push_str(&format!(
        "  \"impl_hash\": \"{}\",\n",
        short_hex(sh.impl_hash)
    ));
    o.push_str(&format!("  \"owner\": \"{}\",\n", esc(&sh.owner)));
    o.push_str(&format!("  \"tier\": \"{}\",\n", esc(&sh.tier)));
    o.push_str(&format!("  \"fixtures\": {},\n", sh.fixtures.len()));
    o.push_str(&format!("  \"holes\": {},\n", sh.steps.len()));
    o.push_str("  \"inputs\": [");
    o.push_str(
        &sh.inputs
            .iter()
            .map(|i| format!("\"{}\"", esc(&i.var)))
            .collect::<Vec<_>>()
            .join(", "),
    );
    o.push_str("],\n");
    o.push_str("  \"contributes\": [");
    o.push_str(
        &sh.kpis
            .iter()
            .map(|k| format!("\"{}\"", esc(k)))
            .collect::<Vec<_>>()
            .join(", "),
    );
    o.push_str("],\n");
    o.push_str("  \"gaps\": [");
    o.push_str(
        &gaps
            .iter()
            .map(|g| format!("\"{}\"", esc(g)))
            .collect::<Vec<_>>()
            .join(", "),
    );
    o.push_str("]\n}\n");
    o
}

// ---------------------------------------------------------------------------
// 6. the gap pass
// ---------------------------------------------------------------------------

/// What the sheet promised and nothing yet covers.
///
/// The gate asks whether what exists passes. It does not ask whether anything
/// the sheet promised is *absent*, and the dominant way this class of work fails
/// is not that it cannot make progress — it is that something plausible gets
/// built, the checks it chose for itself pass, a confident summary is written,
/// and it stops while real requirements are still unmet.
///
/// Here the requirement is a schema rather than prose, so the difference
/// between the sheet and the artefacts is a diff and not a judgement. It costs
/// nothing and it runs on every node, every night. Its output is a list, not a
/// verdict: a node with an open gap cannot enter the second human review.
pub fn gap_pass(sh: &Sheet, holes: &BTreeMap<u32, String>) -> Vec<String> {
    let mut g = Vec::new();
    if sh.is_seeded() {
        g.push(
            "seeded, not yet specified — it needs a question, a relation cited to a page, \
             an interface with units, a domain with a reason per bound, a numbered algorithm \
             and a known-good value from outside this code"
                .into(),
        );
        return g;
    }
    if sh.question.trim().is_empty() {
        g.push(
            "no question stated — an equation with no question gets reused for the wrong thing"
                .into(),
        );
    }
    if sh.expression.trim().is_empty() {
        g.push("no relation stated".into());
    }
    if sh.source.trim().is_empty() {
        g.push("no source cited — this is the claim everything else rests on".into());
    }
    if sh.owner.trim().is_empty() {
        g.push("no owner — reviews on this node route nowhere".into());
    }
    if sh.symbol.trim().is_empty() || sh.ty.trim().is_empty() || sh.unit.trim().is_empty() {
        g.push("the output is not fully declared: symbol, type and unit are all required".into());
    }
    if sh.reason_lower.trim().is_empty() || sh.reason_upper.trim().is_empty() {
        g.push("a declared limit has no reason — a guard whose reason is not written gets deleted by the next person".into());
    }
    if !sh.is_declared() && sh.inputs.is_empty() {
        g.push(
            "a computed node with no declared input claims to compute something from nothing"
                .into(),
        );
    }
    if sh.is_declared() && sh.value.is_none() {
        g.push("a declared value with no number".into());
    }
    if sh.is_declared() && sh.confirmed_by.trim().is_empty() {
        g.push("a declared value with nobody's confirmation against it".into());
    }
    for st in &sh.steps {
        match holes.get(&st.number) {
            Some(b) if !b.trim().is_empty() && !b.contains("todo!") => {}
            _ => g.push(format!("hole {} is empty: {}", st.number, st.text)),
        }
        if st.binds.trim().is_empty() || st.ty.trim().is_empty() {
            g.push(format!(
                "step {} does not say what it binds or at what type",
                st.number
            ));
        }
    }
    if !sh.is_declared() && sh.fixtures.is_empty() {
        g.push(
            "no fixture: nothing outside this code has ever agreed with what it computes".into(),
        );
    }
    for fx in &sh.fixtures {
        if fx.provenance == "self-snapshot" || fx.provenance == "agent-generated" {
            g.push(format!(
                "fixture '{}' has provenance '{}' — an expected value may not come from the code under test",
                fx.label, fx.provenance
            ));
        }
        if fx.source.trim().is_empty() {
            g.push(format!("fixture '{}' cites no source", fx.label));
        }
        for i in &sh.inputs {
            if !fx.inputs.iter().any(|(k, _)| *k == i.binding) {
                g.push(format!(
                    "fixture '{}' does not supply the input '{}'",
                    fx.label, i.binding
                ));
            }
        }
    }
    if sh.assumptions.is_empty() && !sh.is_declared() && sh.steps.len() > 1 {
        g.push("no assumption stated — a multi-step relation always has at least one".into());
    }
    g
}

/// Does any fixture exercise each declared edge of the domain?
///
/// Reported separately from the gap pass because it is the check most likely to
/// be argued with: an edge case that has never been run is the most common
/// place a guard turns out to be wrong.
pub fn unexercised_domain_edges(sh: &Sheet) -> bool {
    !sh.is_declared() && sh.fixtures.len() < 2 && !sh.steps.is_empty()
}

/// The whole tree's gaps, for the nightly pass.
pub fn gap_report(tree: &Tree) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    for sh in tree.ordered() {
        let holes = crate::load::read_holes(&sh.dir);
        let g = gap_pass(sh, &holes);
        if !g.is_empty() {
            out.push((sh.id.clone(), g));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// 7. assembly — the three generators that combine and never decide
// ---------------------------------------------------------------------------

fn kind_variant(k: &str) -> &'static str {
    match k {
        "declared" => "Kind::Declared",
        "required" => "Kind::Required",
        "achieved" => "Kind::Achieved",
        "kpi" => "Kind::Kpi",
        _ => "Kind::Computed",
    }
}

fn state_variant(s: &str) -> &'static str {
    match s {
        "" | "empty" => "State::Empty",
        "specified" => "State::Specified",
        "implemented" => "State::Implemented",
        "verified" => "State::Verified",
        "deprecated" => "State::Deprecated",
        _ => "State::Published",
    }
}

fn tier_variant(t: &str) -> &'static str {
    match t {
        "A+" => "Tier::APlus",
        "A" => "Tier::A",
        "B" => "Tier::B",
        "C" => "Tier::C",
        _ => "Tier::Unset",
    }
}

fn prov_variant(p: &str) -> &'static str {
    match p {
        "independent-derivation" => "Provenance::IndependentDerivation",
        "published-source" => "Provenance::PublishedSource",
        "independent-tool" => "Provenance::IndependentTool",
        "physical-bound" => "Provenance::PhysicalBound",
        "self-snapshot" => "Provenance::SelfSnapshot",
        _ => "Provenance::AgentGenerated",
    }
}

fn view_expr(v: &View) -> String {
    match v {
        View::Number => "View::Number".into(),
        View::Line { over, points } => {
            format!(
                "View::Line {{ over: \"{}\", y: \"\", points: {} }}",
                esc(over),
                points
            )
        }
        View::Heatmap {
            over_x,
            over_y,
            points,
        } => format!(
            "View::Heatmap {{ over_x: \"{}\", over_y: \"{}\", z: \"\", points: {} }}",
            esc(over_x),
            esc(over_y),
            points
        ),
        View::Bar { y } => format!("View::Bar {{ y: \"{}\" }}", esc(y)),
    }
}

fn crate_ident(c: &str) -> String {
    c.replace('-', "_")
}

/// The whole graph, as compiled-in tables.
///
/// The resolver needs all three graphs. If it read them from a file at run time
/// the engine and the graph could disagree — an engine built on Tuesday walking
/// Wednesday's graph — so they are generated as tables and compiled in. Adding
/// an edge is therefore a rebuild, which is correct: it changes what the engine
/// computes, so it should go through the gate.
pub fn tables_rs(tree: &Tree) -> String {
    let sheets = tree.ordered();
    let n = sheets.len();
    let idx: BTreeMap<&str, usize> = sheets
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.as_str(), i))
        .collect();

    let mut o = String::new();
    o.push_str("// GENERATED at build time from the sheets. Never committed: it is an\n");
    o.push_str("// aggregate, and every node would touch it, so every merge would conflict\n");
    o.push_str("// in generated content nobody is allowed to edit.\n\n");
    o.push_str("use vleo_core::credibility::Tier;\n");
    o.push_str("use vleo_core::evidence::{Fixture, Provenance};\n");
    o.push_str("use vleo_core::fault::Fault;\n");
    o.push_str("use vleo_core::graph::{Kind, Limit, NodeDef, Retirement, State, VarDef, View};\n");
    o.push_str("use vleo_core::units::Unit;\n\n");
    o.push_str(&format!("pub const NODE_COUNT: usize = {n};\n\n"));

    o.push_str("pub static NODES: [NodeDef; NODE_COUNT] = [\n");
    for sh in &sheets {
        let inputs = sh
            .inputs
            .iter()
            .map(|i| {
                idx.get(i.var.as_str())
                    .map(|v| v.to_string())
                    .unwrap_or("0".into())
            })
            .collect::<Vec<_>>()
            .join(", ");
        let outputs = idx[sh.id.as_str()].to_string();
        let assumptions = sh
            .assumptions
            .iter()
            .map(|a| format!("(\"{}\", \"{}\")", esc(&a.text), esc(&a.fails_when)))
            .collect::<Vec<_>>()
            .join(", ");
        let steps = sh
            .steps
            .iter()
            .map(|s| format!("\"{}\"", esc(&s.text)))
            .collect::<Vec<_>>()
            .join(", ");
        let kpis = sh
            .kpis
            .iter()
            .map(|k| format!("\"{}\"", esc(k)))
            .collect::<Vec<_>>()
            .join(", ");
        let bundles = sh
            .bundles
            .iter()
            .map(|b| format!("\"{}\"", esc(b)))
            .collect::<Vec<_>>()
            .join(", ");
        let fixtures = sh
            .fixtures
            .iter()
            .map(|f| {
                let ins = sh
                    .inputs
                    .iter()
                    .map(|i| {
                        f.inputs
                            .iter()
                            .find(|(k, _)| *k == i.binding)
                            .map(|(_, v)| format!("{v:?}"))
                            .unwrap_or_else(|| "0.0".into())
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "Fixture {{ label: \"{}\", expected: {:?}, tolerance: {:?}, provenance: {}, source: \"{}\", inputs: &[{ins}] }}",
                    esc(&f.label),
                    f.expect,
                    f.tolerance,
                    prov_variant(&f.provenance),
                    esc(&f.source)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        o.push_str(&format!(
            "    NodeDef {{ id: \"{id}\", label: \"{label}\", subsystem: \"{sub}\", folder: \"{folder}\", \
             parent: \"{par}\", layer: {layer}, crosses_to: \"{crosses}\", \
             kind: {kind}, state: {state}, retirement: Retirement::Live, owner: \"{owner}\", tier: {tier}, \
             question: \"{q}\", expression: \"{e}\", source: \"{src}\", assumptions: &[{asm}], steps: &[{steps}], \
             inputs: &[{inputs}], outputs: &[{outputs}], contributes: &[{kpis}], bundles: &[{bundles}], \
             fixtures: &[{fixtures}], sheet_hash: 0x{sh_hash:016x}, impl_hash: 0x{im_hash:016x}, view: {view} }},\n",
            id = esc(&sh.id),
            label = esc(&sh.label),
            sub = esc(&sh.subsystem),
            folder = esc(&format!("crates/{}/nodes/{}", sh.crate_name, sh.folder)),
            par = esc(&sh.parent),
            layer = sh.layer,
            crosses = esc(&sh.crosses_to),
            kind = kind_variant(&sh.kind),
            state = state_variant(&sh.state),
            owner = esc(&sh.owner),
            tier = tier_variant(&sh.tier),
            q = esc(&sh.question),
            e = esc(&sh.expression),
            src = esc(&sh.source),
            asm = assumptions,
            steps = steps,
            inputs = inputs,
            outputs = outputs,
            kpis = kpis,
            bundles = bundles,
            fixtures = fixtures,
            sh_hash = sh.sheet_hash,
            im_hash = sh.impl_hash,
            view = view_expr(&sh.view),
        ));
    }
    o.push_str("];\n\n");

    o.push_str("pub static VARS: [VarDef; NODE_COUNT] = [\n");
    for (i, sh) in sheets.iter().enumerate() {
        o.push_str(&format!(
            "    VarDef {{ id: \"{id}\", symbol: \"{sym}\", label: \"{label}\", unit: Unit::{unit}, producer: {p}, \
             limit: Limit {{ lower: {lo:?}, upper: {hi:?}, reason_lower: \"{rl}\", reason_upper: \"{ru}\" }} }},\n",
            id = esc(&sh.id),
            sym = esc(&sh.symbol),
            label = esc(&sh.label),
            unit = unit_of(&sh.unit).name(),
            p = i,
            lo = to_si(sh.lower, &sh.unit),
            hi = to_si(sh.upper, &sh.unit),
            rl = esc(&sh.reason_lower),
            ru = esc(&sh.reason_upper),
        ));
    }
    o.push_str("];\n\n");

    o.push_str("type NodeFn = fn(&[f64], &mut [f64]) -> Result<(), Fault>;\n\n");
    o.push_str(
        "/// A row the tree knows about that nobody has specified yet.\n\
         ///\n\
         /// It refuses by name rather than being absent: a node hidden to make a\n\
         /// run look complete is the one thing the run control may never do. The\n\
         /// answer is always `n ran, m blocked`, and the blocked ones are named.\n\
         fn unspecified(_: &[f64], _: &mut [f64]) -> Result<(), Fault> {\n\
         \x20   Err(Fault::NotRun { node: \"seeded, not yet specified\" })\n\
         }\n\n",
    );
    o.push_str("pub static DISPATCH: [NodeFn; NODE_COUNT] = [\n");
    for sh in &sheets {
        if sh.is_seeded() {
            o.push_str("    unspecified,\n");
        } else {
            o.push_str(&format!(
                "    {}::nodes::{}::call,\n",
                crate_ident(&sh.crate_name),
                sh.rust_ident()
            ));
        }
    }
    o.push_str("];\n\n");

    // Cases, including the declared cycles.
    o.push_str("/// A loop the design actually has, declared where design decisions live.\n");
    o.push_str("pub struct CycleDef {\n    pub nodes: &'static [u16],\n    pub converge_on: u16,\n    pub tolerance: f64,\n    pub max_iter: u32,\n    pub seeds: &'static [(u16, f64)],\n}\n\n");
    o.push_str("pub struct CaseDef {\n    pub id: &'static str,\n    pub label: &'static str,\n    pub note: &'static str,\n    pub supply: &'static [(u16, f64)],\n    pub cycles: &'static [CycleDef],\n}\n\n");
    o.push_str(&format!(
        "pub static CASES: [CaseDef; {}] = [\n",
        tree.cases.len()
    ));
    for c in tree.cases.values() {
        let supply = c
            .supply
            .iter()
            .filter_map(|(k, v)| idx.get(k.as_str()).map(|i| format!("({}, {:?})", i, v)))
            .collect::<Vec<_>>()
            .join(", ");
        let cycles = c
            .cycles
            .iter()
            .map(|cy| {
                let nodes = cy
                    .nodes
                    .iter()
                    .filter_map(|n| idx.get(n.as_str()).map(|i| i.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ");
                let seeds = cy
                    .seeds
                    .iter()
                    .filter_map(|(k, v)| idx.get(k.as_str()).map(|i| format!("({}, {:?})", i, v)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "CycleDef {{ nodes: &[{nodes}], converge_on: {c}, tolerance: {t:?}, max_iter: {m}, seeds: &[{seeds}] }}",
                    c = idx.get(cy.converge_on.as_str()).copied().unwrap_or(0),
                    t = cy.tolerance,
                    m = cy.max_iter
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        o.push_str(&format!(
            "    CaseDef {{ id: \"{}\", label: \"{}\", note: \"{}\", supply: &[{supply}], cycles: &[{cycles}] }},\n",
            esc(&c.id),
            esc(&c.label),
            esc(&c.note)
        ));
    }
    o.push_str("];\n\n");

    // The navigation graph: groups and the relation edges between them.
    o.push_str(
        "/// One heading in the tree.\n\
         pub struct GroupDef {\n\
         \x20   pub id: &'static str,\n\
         \x20   pub label: &'static str,\n\
         \x20   pub parent: &'static str,\n\
         \x20   pub owner: &'static str,\n\
         \x20   /// 1 management · 2 the system · 3 subsystem · 4 the run.\n\
         \x20   pub layer: u8,\n\
         \x20   /// Drawn as a nested box on the diagonal. A mark inside a box is\n\
         \x20   /// coupling that subtree owns; a mark outside it crosses a boundary.\n\
         \x20   pub is_box: bool,\n\
         \x20   /// The colour family the branch is drawn in — what makes a branch\n\
         \x20   /// findable on a tree of thirteen hundred rows.\n\
         \x20   pub tone: &'static str,\n\
         \x20   /// The cases this branch is in play for. Empty means every case.\n\
         \x20   pub cases: &'static [&'static str],\n\
         }\n\n",
    );
    o.push_str(&format!(
        "pub static GROUPS: [GroupDef; {}] = [\n",
        tree.groups.len()
    ));
    for g in tree.groups.values() {
        o.push_str(&format!(
            "    GroupDef {{ id: \"{}\", label: \"{}\", parent: \"{}\", owner: \"{}\", layer: {}, is_box: {}, tone: \"{}\", cases: &[{}] }},\n",
            esc(&g.id),
            esc(&g.label),
            esc(&g.parent),
            esc(&g.owner),
            g.layer,
            g.is_box,
            esc(&g.tone),
            g.cases
                .iter()
                .map(|c| format!("\"{}\"", esc(c)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    o.push_str("];\n\n");
    o.push_str(&format!(
        "pub static RELATIONS: [(&str, &str, &str); {}] = [\n",
        tree.relations.len()
    ));
    for r in &tree.relations {
        o.push_str(&format!(
            "    (\"{}\", \"{}\", \"{}\"),\n",
            esc(&r.from),
            esc(&r.to),
            esc(&r.why)
        ));
    }
    o.push_str("];\n");
    o
}

/// The module list for one subsystem crate. An aggregate: built, never
/// committed.
pub fn crate_registry_rs(node_dirs: &[(String, String)]) -> String {
    let mut o = String::new();
    o.push_str("// GENERATED at build time. An aggregate — built, never committed, because\n");
    o.push_str("// every node added would otherwise touch this file and every merge would\n");
    o.push_str("// conflict in generated content nobody is allowed to edit.\n\n");
    for (ident, path) in node_dirs {
        o.push_str(&format!("#[path = \"{path}\"]\npub mod {ident};\n"));
    }
    o
}
