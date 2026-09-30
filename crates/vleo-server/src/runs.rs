//! Running the engine: one run, a probe, a sweep, the branches of a row and
//! the levers that move it.

use super::*;

pub(crate) fn run_json(params: &str, ctx: &Ctx) -> String {
    if let Some(refusal) = case_refused(params, ctx) {
        return refusal;
    }
    let case = build_case(params, ctx);
    // Refuse before running, not after: a supplied value that cannot survive
    // the run has to be reported as a refusal rather than silently dropped.
    for (id, _) in &case.supply {
        if let Some(why) = unsuppliable(id) {
            return refuse(id, &why);
        }
    }
    let mut scratch = Scratch::new();
    let mut j = Json::new();
    j.raw("{");
    match vleo_modules::evaluate(&case, &mut scratch) {
        Err(f) => {
            // A refusal is a value, not a magic number the caller may forget to
            // check. It names the field, the bound and the reason.
            j.bool_field("ok", false);
            j.str_field("fault", f.kind());
            j.str_field("node", f.node());
            j.str_field("message", &format!("{f}"));
        }
        Ok(r) => {
            j.bool_field("ok", true);
            // Which inputs this answer was for. A number with no case beside it
            // is a number a reader will quote for the wrong inputs.
            let saved = if param(params, "inputs").map(decode).as_deref() == Some("defaults") {
                None
            } else {
                Some(saved_case().reading)
            };
            let note = saved.as_ref().and_then(|s| s.upgrade.as_ref());
            j.key("inputs").raw("{");
            j.bool_field("defaults", saved.is_none());
            j.num_field(
                "changed",
                saved.as_ref().map(|s| s.changed).unwrap_or(0) as f64,
            );
            // What the last update did to the case, until it is next saved:
            // values it could not carry, and inputs it added at their defaults.
            j.bool_field("upgraded", note.is_some());
            j.num_field(
                "set_aside",
                note.map(|u| u.set_aside.len()).unwrap_or(0) as f64,
            );
            j.num_field("new", note.map(|u| u.new.len()).unwrap_or(0) as f64);
            j.close_obj();
            j.key("values").open_arr();
            for (i, v) in r.values.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                let unit = Vleo::find(&v.id)
                    .map(|k| VARS[k as usize].unit)
                    .unwrap_or(vleo_units::Unit::One);
                let (shown, sym) = vleo_bus::present(v.value, unit, 6);
                j.raw("{");
                j.str_field("id", &v.id);
                j.str_field("symbol", &v.symbol);
                j.str_field("label", &v.label);
                j.num_field("si", v.value);
                j.str_field("shown", &shown);
                j.str_field("unit", sym);
                j.num_field("cred", v.cred.governing_score() as f64);
                j.str_field("governing", v.governing);
                j.key("vec").open_arr();
                for (k, c) in v.cred.0.iter().enumerate() {
                    if k > 0 {
                        j.raw(",");
                    }
                    j.raw(&c.to_string());
                }
                j.close_arr();
                j.close_obj();
            }
            j.close_arr();
            j.key("blocked").open_arr();
            for (i, b) in r.blocked.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                j.raw("{");
                j.str_field("id", &b.id);
                j.str_field("kind", b.kind);
                j.str_field("message", &b.message);
                j.close_obj();
            }
            j.close_arr();
            j.key("verdicts").open_arr();
            for (i, v) in r.verdicts.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                j.raw("{");
                j.str_field("node", &v.node);
                j.str_field("label", &v.label);
                j.num_field("expected", v.expected);
                j.num_field("got", v.got);
                j.num_field("error", v.relative_error);
                j.num_field("tolerance", v.tolerance);
                j.bool_field("passed", v.passed);
                j.str_field("provenance", v.provenance);
                j.str_field("source", v.source);
                j.close_obj();
            }
            j.close_arr();
            j.key("manifest").raw("{");
            j.str_field("node", &r.manifest.node);
            j.str_field("mode", r.manifest.mode);
            j.str_field("kernel", &r.manifest.kernel);
            j.str_field("graph", &r.manifest.graph);
            j.str_field("case", &r.manifest.case);
            j.str_field("chain", &r.manifest.chain);
            j.str_field("endpoint", "local-daemon");
            j.num_field("ran", r.manifest.ran as f64);
            j.num_field("blocked", r.manifest.blocked_count as f64);
            j.num_field("iterations", r.manifest.iterations as f64);
            j.key("data").open_arr();
            for (i, d) in r.manifest.data.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                j.push_string(d);
            }
            j.close_arr();
            j.close_obj();
        }
    }
    j.raw("}");
    j.0
}

/// A behaviour sweep. Refused points are recorded with their reason, never
/// dropped — a sweep in which some rows quietly used a substituted value is a
/// sweep whose conclusion is unknown.
/// `/v1/probe?node=<id>&in=<var>:<value>&in=…` — one relation, at given inputs.
///
/// The counterpart to `run` refusing a supplied value on a computed row. That
/// refusal is right: the run would overwrite it. But a relation's behaviour at
/// chosen driver values is a different question with a different answer, and
/// until `env_f107` became computed the two could be asked the same way.
///
/// Inputs are named by the VARIABLE they bind, not given positionally. The
/// dispatch table takes them in the node's declared order, and a caller
/// counting commas to match that order is a caller that silently swaps two
/// drivers of the same type the first time a sheet's input list is reordered —
/// which `env_exospheric_temperature`, whose three drivers are all `Ratio`,
/// would do without a single type error.
pub(crate) fn probe_json(params: &str) -> String {
    let node = param(params, "node").map(decode).unwrap_or_default();
    let Some(k) = Vleo::find(&node) else {
        return refuse(&node, "no such row");
    };
    let def = &NODES[k as usize];

    let mut given: BTreeMap<String, f64> = BTreeMap::new();
    for kv in params.split('&') {
        if let Some(v) = kv.strip_prefix("in=") {
            let d = decode(v);
            let Some((name, val)) = d.rsplit_once(':') else {
                return refuse(&node, &format!("'{d}' is not <var>:<value>"));
            };
            let Ok(x) = val.parse::<f64>() else {
                return refuse(&node, &format!("'{val}' is not a number"));
            };
            given.insert(name.to_string(), x);
        }
    }

    // Every declared input must be given, by name. A missing one defaulting to
    // zero is a probe that answers confidently about a relation nobody asked
    // about, which is the whole failure this endpoint exists to stop repeating.
    let mut inputs = Vec::with_capacity(def.inputs.len());
    for &v in def.inputs {
        let id = VARS[v as usize].id;
        match given.remove(id) {
            Some(x) => inputs.push(x),
            None => {
                return refuse(
                    &node,
                    &format!(
                        "no value given for input '{id}'; this row reads {} of them",
                        def.inputs.len()
                    ),
                )
            }
        }
    }
    if let Some((extra, _)) = given.iter().next() {
        return refuse(&node, &format!("'{extra}' is not an input of {}", def.id));
    }

    let mut j = Json::new();
    j.raw("{");
    match vleo_modules::probe(k, &inputs) {
        Err(f) => {
            j.bool_field("ok", false);
            j.str_field("fault", f.kind());
            j.str_field("node", f.node());
            j.str_field("message", &format!("{f}"));
        }
        Ok(out) => {
            j.bool_field("ok", true);
            j.str_field("node", def.id);
            // The primary answer, then every published member in slot order.
            j.num_field("si", out[0]);
            // UNIT AND FACTOR, the same pair the sweep endpoint sends.
            //
            // Everything crossing this boundary is SI and a face divides by the
            // factor to display. A caller that has to source those from
            // somewhere else is a caller that gets `undefined`, divides by it,
            // and draws nothing — which is exactly what the thermosphere
            // panel's two flux curves did the first time they came through
            // here: legend entries with no lines under them.
            let ov = VARS[def.outputs[0] as usize].unit;
            j.str_field("unit", ov.symbol());
            j.num_field("factor", ov.si_factor());
            j.key("inputs").open_arr();
            for (i, &v) in def.inputs.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                let var = &VARS[v as usize];
                j.raw("{");
                j.str_field("id", var.id);
                j.str_field("unit", var.unit.symbol());
                j.num_field("factor", var.unit.si_factor());
                j.close_obj();
            }
            j.close_arr();
            j.key("outputs").open_arr();
            for (i, &v) in def.outputs.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                j.raw("{");
                j.str_field("id", VARS[v as usize].id);
                j.num_field("si", out[i]);
                j.close_obj();
            }
            j.close_arr();
        }
    }
    j.raw("}");
    j.0
}

pub(crate) fn sweep_json(params: &str, ctx: &Ctx) -> String {
    let node = param(params, "node").map(decode).unwrap_or_default();
    let over = param(params, "over").map(decode).unwrap_or_default();
    let from: f64 = param(params, "from")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    let to: f64 = param(params, "to")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1.0);
    let points: usize = param(params, "points")
        .and_then(|v| v.parse().ok())
        .unwrap_or(48)
        .clamp(2, 400);

    // The axis has to be a row a reader can actually move. Sweeping a computed
    // one drew a flat line and reported no refusals, which is the same silent
    // substitution as `set=` on one and reads as a real result.
    if let Some(why) = unsuppliable(&over) {
        return refuse(&over, &why);
    }
    if let Some(refusal) = case_refused(params, ctx) {
        return refusal;
    }

    let mut j = Json::new();
    j.raw("{");
    let (ni, oi) = match (Vleo::find(&node), Vleo::find(&over)) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            j.bool_field("ok", false);
            j.str_field("message", "the sweep names a node that does not exist");
            j.raw("}");
            return j.0;
        }
    };
    j.bool_field("ok", true);
    j.str_field("x_id", &over);
    j.str_field("y_id", &node);
    j.str_field("x_unit", VARS[oi as usize].unit.symbol());
    j.str_field("y_unit", VARS[ni as usize].unit.symbol());
    j.num_field("x_factor", VARS[oi as usize].unit.si_factor());
    j.num_field("y_factor", VARS[ni as usize].unit.si_factor());

    let mut scratch = Scratch::new();
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut refused: Vec<(f64, String)> = Vec::new();
    for i in 0..points {
        let t = i as f64 / (points - 1) as f64;
        let x = from + t * (to - from);
        let mut case = build_case(params, ctx);
        case.target = node.clone();
        case.supply.push((over.clone(), x));
        match vleo_modules::evaluate(&case, &mut scratch) {
            Ok(r) => match r.values.iter().find(|v| v.id == node) {
                Some(v) => {
                    xs.push(x);
                    ys.push(v.value);
                }
                None => refused.push((x, "blocked".to_string())),
            },
            Err(f) => refused.push((x, format!("{f}"))),
        }
    }
    j.key("x").open_arr();
    for (i, v) in xs.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw(&json::num(*v));
    }
    j.close_arr();
    j.key("y").open_arr();
    for (i, v) in ys.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw(&json::num(*v));
    }
    j.close_arr();
    j.key("refused").open_arr();
    for (i, (x, why)) in refused.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.num_field("x", *x);
        j.str_field("why", why);
        j.close_obj();
    }
    j.close_arr();
    j.raw("}");
    j.0
}

/// Which declared decisions actually move this node's answer.
///
/// The sweep control has to offer the reader a decision, and every declared
/// row anywhere upstream is a candidate. That list is useless on its own: a
/// term can be in the relation, be correct, and still be inert — the
/// persistence weight in sw_central_expectation is exp(-L/27) and contributes
/// 0.0001 per cent at any mission lead, so a reader handed it as the default
/// sweeps it, sees a flat line, and concludes the tool is broken. It is not
/// broken; it is answering a question nobody would have asked had they known
/// the answer.
///
/// So this measures rather than guesses. Each candidate is evaluated at both
/// ends of its own declared range with everything else held at the case, and
/// the answer's span is reported. A face can then lead with the decision that
/// moves the answer most, and say plainly of the others that they do not.
///
/// A candidate whose ends both refuse is reported with its reason, not
/// dropped: a decision that cannot be swept is a different fact from one that
/// changes nothing, and collapsing the two would hide a broken bound.
/// The maximal active branches a row is in.
///
/// A branch is the dependency closure of one row. A row is in as many branches
/// as there are rows that read it, and most of those nest inside each other, so
/// what is useful is the maximal ones: a candidate that no other candidate
/// reads, directly or at any distance. Running that set computes every row in
/// every active branch containing the input and computes none of them twice.
///
/// ACTIVE means every row in the closure is `published` — the tree is filled in
/// that far. It does not mean the branch will succeed: a published row can
/// still refuse when its own value lands outside its own declared domain, and
/// that refusal is a real answer rather than a gap.
///
/// THIS LIVES HERE RATHER THAN IN THE FACE because two things need it — the
/// node page and the audit in tools/ — and a rule with two implementations is
/// a rule that drifts. The face used to walk the graph in JavaScript off the
/// index; it now asks for this.
pub(crate) fn branches_json(params: &str) -> String {
    let node = param(params, "node").map(decode).unwrap_or_default();
    let mut j = Json::new();
    j.raw("{");
    let ni = match Vleo::find(&node) {
        Some(a) => VARS[a as usize].producer,
        None => {
            j.bool_field("ok", false);
            j.str_field("message", "the branches name a node that does not exist");
            j.raw("}");
            return j.0;
        }
    };
    j.bool_field("ok", true);
    j.str_field("node", &node);

    // node -> the nodes that read it. Built from `inputs`, which holds VARIABLE
    // indices, so each is mapped to its producing node first; conflating the
    // two indices is a defect this file has already shipped once.
    let mut cons: Vec<Vec<u16>> = vec![Vec::new(); NODES.len()];
    for (n, d) in NODES.iter().enumerate() {
        for x in d.inputs {
            cons[VARS[*x as usize].producer as usize].push(n as u16);
        }
    }
    let walk = |from: u16, edges: &Vec<Vec<u16>>| -> Vec<bool> {
        let mut seen = vec![false; NODES.len()];
        let mut st = vec![from];
        while let Some(n) = st.pop() {
            for m in &edges[n as usize] {
                if !seen[*m as usize] {
                    seen[*m as usize] = true;
                    st.push(*m);
                }
            }
        }
        seen
    };
    let mut prod: Vec<Vec<u16>> = vec![Vec::new(); NODES.len()];
    for (n, d) in NODES.iter().enumerate() {
        for x in d.inputs {
            prod[n].push(VARS[*x as usize].producer);
        }
    }
    let live = |n: u16| NODES[n as usize].state == vleo_core::graph::State::Published;

    let down = walk(ni, &cons);
    let mut cand: Vec<(u16, usize)> = Vec::new();
    for n in 0..NODES.len() as u16 {
        if !down[n as usize] || !live(n) {
            continue;
        }
        let cl = walk(n, &prod);
        let mut ok = true;
        let mut size = 1usize;
        for (k, inside) in cl.iter().enumerate() {
            if *inside {
                size += 1;
                if !live(k as u16) {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            cand.push((n, size));
        }
    }
    let is_cand: Vec<bool> = {
        let mut v = vec![false; NODES.len()];
        for (n, _) in &cand {
            v[*n as usize] = true;
        }
        v
    };
    let mut out: Vec<(u16, usize)> = cand
        .iter()
        .filter(|(n, _)| {
            let up = walk(*n, &cons);
            !(0..NODES.len()).any(|k| up[k] && is_cand[k])
        })
        .cloned()
        .collect();
    // Biggest first: the branch that covers most of the design is the one a
    // reader wants at the top.
    out.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| NODES[a.0 as usize].id.cmp(NODES[b.0 as usize].id))
    });

    // How many rows read this one at all, so a face can say whether an empty
    // list means "nothing reads it" or "everything that does is unfinished".
    let read_by = (0..NODES.len()).filter(|k| down[*k]).count();
    j.num_field("read_by", read_by as f64);

    j.key("branches").open_arr();
    for (i, (n, size)) in out.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        let d = &NODES[*n as usize];
        j.raw("{");
        j.str_field("id", d.id);
        j.str_field("label", d.label);
        j.str_field("sub", d.subsystem);
        j.num_field("rows", *size as f64);
        j.close_obj();
    }
    j.close_arr();
    j.raw("}");
    j.0
}

pub(crate) fn levers_json(params: &str, ctx: &Ctx) -> String {
    let node = param(params, "node").map(decode).unwrap_or_default();
    if let Some(refusal) = case_refused(params, ctx) {
        return refusal;
    }
    let mut j = Json::new();
    j.raw("{");
    let ni = match Vleo::find(&node) {
        Some(a) => a,
        None => {
            j.bool_field("ok", false);
            j.str_field("message", "the levers name a node that does not exist");
            j.raw("}");
            return j.0;
        }
    };
    j.bool_field("ok", true);
    j.str_field("node", &node);

    // Every declared row upstream, however far: a decision three rows away is
    // still a decision, and the reason a reader opens this row may be a choice
    // taken well before it.
    // The walk is over NODES, because `inputs` is a node's declaration of what
    // it reads. A candidate is a VARIABLE, because that is what a sweep
    // supplies: the two indices are different spaces and conflating them is
    // the defect this file has already shipped once.
    let start = VARS[ni as usize].producer;
    let mut seen = vec![false; NODES.len()];
    let mut stack = vec![start];
    seen[start as usize] = true;
    let mut cands: Vec<u16> = Vec::new();
    while let Some(n) = stack.pop() {
        for x in NODES[n as usize].inputs {
            let pv = &VARS[*x as usize];
            let pn = pv.producer;
            if !seen[pn as usize] {
                seen[pn as usize] = true;
                stack.push(pn);
            }
            if *x != ni
                && NODES[pn as usize].kind == vleo_core::graph::Kind::Declared
                && pv.limit.upper > pv.limit.lower
                && !cands.contains(x)
            {
                cands.push(*x);
            }
        }
    }

    let mut scratch = Scratch::new();
    let base = {
        let mut case = build_case(params, ctx);
        case.target = node.clone();
        vleo_modules::evaluate(&case, &mut scratch)
            .ok()
            .and_then(|r| r.values.iter().find(|v| v.id == node).map(|v| v.value))
    };
    match base {
        Some(b) => j.num_field("base", b),
        None => j.key("base").raw("null"),
    };

    /// One decision and what moving it across its own declared range does to
    /// the answer. `span` is negative where an end could not be evaluated at
    /// all, which is a different fact from a span of zero.
    struct Lever {
        span: f64,
        var: u16,
        at_lower: Option<f64>,
        at_upper: Option<f64>,
        why: String,
    }
    let mut out: Vec<Lever> = Vec::new();
    for v in cands {
        let d = &VARS[v as usize];
        let mut ends: [Option<f64>; 2] = [None, None];
        let mut why = String::new();
        for (k, x) in [d.limit.lower, d.limit.upper].iter().enumerate() {
            let mut case = build_case(params, ctx);
            case.target = node.clone();
            case.supply.push((d.id.to_string(), *x));
            match vleo_modules::evaluate(&case, &mut scratch) {
                Ok(r) => {
                    ends[k] = r.values.iter().find(|q| q.id == node).map(|q| q.value);
                    if ends[k].is_none() && why.is_empty() {
                        why = "blocked".to_string();
                    }
                }
                Err(f) => {
                    if why.is_empty() {
                        why = format!("{f}");
                    }
                }
            }
        }
        // The span is relative to the base where there is one, so decisions on
        // rows of different size can be ranked against each other at all.
        let span = match (ends[0], ends[1]) {
            (Some(a), Some(b)) => {
                let d = (b - a).abs();
                match base {
                    Some(z) if z != 0.0 => d / z.abs(),
                    _ => d,
                }
            }
            _ => -1.0,
        };
        out.push(Lever {
            span,
            var: v,
            at_lower: ends[0],
            at_upper: ends[1],
            why,
        });
    }
    // Most movement first, so the face's default is the decision worth asking
    // about. Ties keep a stable order by id for a page that does not reshuffle.
    out.sort_by(|a, b| {
        b.span
            .partial_cmp(&a.span)
            .unwrap_or(core::cmp::Ordering::Equal)
            .then_with(|| VARS[a.var as usize].id.cmp(VARS[b.var as usize].id))
    });

    j.key("levers").open_arr();
    for (i, lev) in out.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        let d = &VARS[lev.var as usize];
        j.raw("{");
        j.str_field("id", d.id);
        j.str_field("symbol", d.symbol);
        j.str_field("label", d.label);
        j.str_field("unit", d.unit.symbol());
        j.num_field("factor", d.unit.si_factor());
        j.num_field("lower", d.limit.lower);
        j.num_field("upper", d.limit.upper);
        match lev.at_lower {
            Some(x) => j.num_field("at_lower", x),
            None => j.key("at_lower").raw("null"),
        };
        match lev.at_upper {
            Some(x) => j.num_field("at_upper", x),
            None => j.key("at_upper").raw("null"),
        };
        if lev.span < 0.0 {
            j.key("span").raw("null");
        } else {
            j.num_field("span", lev.span);
        }
        j.str_field("why", &lev.why);
        j.close_obj();
    }
    j.close_arr();
    j.raw("}");
    j.0
}
