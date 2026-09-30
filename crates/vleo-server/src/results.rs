//! Saved results: listed, read, saved, uploaded, removed, and turned back into a case.

use super::*;

/// Now, as a result records it: UTC, to the second. Through `date` rather than a
/// crate, as xtask stamps its dates.
pub(super) fn now_utc() -> String {
    vleo_units::calendar::Civil::from_unix(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
    )
    .to_string()
}

pub(super) fn results_dir() -> PathBuf {
    vleo_data::results_path()
}

/// A yes/no parameter: present as `1`, `true` or `yes`.
pub(super) fn flag(params: &str, name: &str) -> bool {
    matches!(
        param(params, name).map(decode).as_deref(),
        Some("1" | "true" | "yes")
    )
}

/// The question a request asks, as a saved result's key.
pub(super) fn question_of(case: &Case, sweep: Option<(&str, f64, f64, usize)>) -> String {
    vleo_modules::results::question_for(case, sweep)
}

/// Which saved result an answer was read from — said on every answer that was
/// not run just now, so it is never mistaken for one that was.
pub(super) fn from_saved_json(j: &mut Json, file: &str, s: &vleo_modules::results::Saved, q: &str) {
    j.key("from_saved").raw("{");
    j.str_field("file", file);
    j.str_field("saved", &s.saved);
    j.str_field("name", &s.name);
    j.str_field("question", q);
    j.close_obj();
}

/// A saved result, answered as a run: the same fields a run returns, read from
/// the record, with `from_saved` naming it. The checks against known-good
/// values are not repeated — they ran when the result was made — so there are
/// none here, and the face says so.
pub(super) fn saved_run_json(
    params: &str,
    file: &str,
    s: &vleo_modules::results::Saved,
    q: &str,
) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    from_saved_json(&mut j, file, s, q);
    inputs_note(&mut j, params);
    j.key("values").open_arr();
    let mut first = true;
    for o in &s.outputs {
        let Some(si) = o.si else { continue };
        if !first {
            j.raw(",");
        }
        first = false;
        let (symbol, unit) = match Vleo::find(&o.id) {
            Some(k) => (VARS[k as usize].symbol, VARS[k as usize].unit),
            None => ("", vleo_units::Unit::One),
        };
        let (shown, sym) = vleo_bus::present(si, unit, 6);
        j.raw("{");
        j.str_field("id", &o.id);
        j.str_field("symbol", symbol);
        j.str_field("label", &o.name);
        j.num_field("si", si);
        j.str_field("shown", &shown);
        j.str_field("unit", sym);
        j.num_field("cred", o.credibility.parse::<f64>().unwrap_or(0.0));
        j.str_field("governing", &o.governing);
        j.key("vec").open_arr();
        for (k, c) in o.cred.chars().filter_map(|c| c.to_digit(10)).enumerate() {
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
    for (i, b) in s.blocked.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("id", &b.id);
        j.str_field("kind", "blocked");
        j.str_field("message", &b.note);
        j.close_obj();
    }
    j.close_arr();
    j.key("verdicts").raw("[]");
    j.key("manifest").raw("{");
    j.str_field("node", &s.target);
    j.str_field("mode", &s.mode);
    j.str_field("kernel", &s.kernel);
    j.str_field("graph", &s.graph);
    j.str_field("case", &s.case);
    j.str_field("chain", &s.chain);
    j.str_field("endpoint", "saved result");
    j.num_field("ran", s.ran as f64);
    j.num_field("blocked", s.blocked_count as f64);
    j.num_field("iterations", 0.0);
    j.key("data").open_arr();
    for (i, d) in s.data.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.push_string(d);
    }
    j.close_arr();
    j.close_obj();
    j.raw("}");
    j.0
}

pub(super) fn rows_json(j: &mut Json, key: &str, rows: &[vleo_modules::results::Row]) {
    j.key(key).open_arr();
    for (k, r) in rows.iter().enumerate() {
        if k > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("id", &r.id);
        j.str_field("name", &r.name);
        j.str_field("value", &r.value);
        j.str_field("unit", &r.unit);
        match r.si {
            Some(v) => j.num_field("si", v),
            None => j.key("si").raw("null"),
        };
        j.str_field("credibility", &r.credibility);
        j.str_field("governing", &r.governing);
        j.str_field("note", &r.note);
        j.close_obj();
    }
    j.close_arr();
}

pub(super) fn result_head(j: &mut Json, file: &str, s: &vleo_modules::results::Saved) {
    j.str_field("target", &s.target);
    // Kept whole for good, or thinned to its summary on the date given.
    j.bool_field(
        "pinned",
        vleo_modules::results::store::is_pinned(&results_dir(), file),
    );
    j.str_field("thinned", &s.thinned);
    j.str_field("name", &s.name);
    j.str_field("saved", &s.saved);
    j.str_field("mode", &s.mode);
    j.str_field("chain", &s.chain);
    j.str_field("kernel", &s.kernel);
    j.str_field("graph", &s.graph);
    j.str_field("template", &s.template);
    j.bool_field(
        "template_current",
        s.template == vleo_modules::inputs::template(),
    );
    j.str_field("data", &s.data.join(" "));
    j.num_field("ran", s.ran as f64);
    j.num_field("blocked", s.blocked_count as f64);
    j.num_field("changed", s.changed() as f64);
    j.str_field("question", &s.question());
    match &s.sweep {
        Some(w) => {
            j.key("sweep").raw("{");
            j.str_field("over", &w.over);
            j.num_field("points", w.points as f64);
            j.num_field("refused", w.refused.len() as f64);
            j.close_obj();
        }
        None => {
            j.key("sweep").raw("null");
        }
    }
    // The node versions it rests on, and those whose record has moved on
    // since: each a belief the result rested on that has broken.
    j.key("versions").open_arr();
    for (i, (id, n, rel)) in s.versions.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("node", id);
        j.num_field("n", *n as f64);
        j.str_field("release", rel);
        j.close_obj();
    }
    j.close_arr();
    j.key("moved").open_arr();
    for (i, (id, then, now)) in vleo_modules::results::moved_since(s).iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("node", id);
        j.num_field("then", *then as f64);
        j.num_field("now", *now as f64);
        j.close_obj();
    }
    j.close_arr();
    match s.answer() {
        Some(a) => {
            j.key("answer").raw("{");
            j.str_field("value", &a.value);
            j.str_field("unit", &a.unit);
            j.close_obj();
        }
        None => {
            j.key("answer").raw("null");
        }
    }
}

/// Every saved result, newest first — and any file that no longer reads,
/// named with why rather than left out of the list.
pub(super) fn results_list() -> String {
    let (good, bad) = vleo_modules::results::store::list(&results_dir());
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("path", &results_dir().display().to_string());
    j.key("results").open_arr();
    for (k, (file, s)) in good.iter().enumerate() {
        if k > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("file", file);
        result_head(&mut j, file, s);
        j.close_obj();
    }
    j.close_arr();
    j.key("unreadable").open_arr();
    for (k, (file, why)) in bad.iter().enumerate() {
        if k > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("file", file);
        j.str_field("why", why);
        j.close_obj();
    }
    j.close_arr();
    j.raw("}");
    j.0
}

/// One figure of every kind, from [`vleo_modules::figure::samples`].
/// A figure of the record's numbers, by the panel's id: what the page draws
/// beside the record's own days, worked out here so a test holds them and
/// Python or a script can ask for them too. Read from the same verified
/// bundle `/v1/bundle` serves the page, so the picture and the numbers are one
/// claim. A panel with no numbers here yet is refused by name.
pub(super) fn record_figure(ctx: &Ctx, id: &str) -> String {
    const BUNDLE: &str = "solar-weather";
    if id != "density" {
        return failed(&format!(
            "no figure of the record called '{id}'. The engine works out: density"
        ));
    }
    let Some((dir, _)) = ctx.bundles.get(BUNDLE) else {
        return failed(&format!(
            "{BUNDLE} is not installed here, so there is no record to work the figure out from"
        ));
    };
    let days = match vleo_data::load_bundle(dir).and_then(|b| vleo_data::read_solar_days(&b)) {
        Ok(d) => d,
        Err(e) => return failed(&e),
    };
    let d = vleo_modules::record::density(&days);
    let opt = |j: &mut Json, k: &str, v: Option<f64>| {
        match v {
            Some(v) => j.num_field(k, v),
            None => j.key(k).raw("null"),
        };
    };
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("figure", id);
    j.str_field(
        "bundle",
        &format!(
            "{BUNDLE}@{}",
            dir.file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default()
        ),
    );
    j.num_field("days", d.days as f64);
    opt(&mut j, "r", d.r);
    opt(&mut j, "median_f107", d.median_f107);
    opt(&mut j, "median_ap", d.median_ap);
    j.num_field("below_both_pct", d.below_both_pct);
    j.num_field("storm_ap", vleo_modules::record::STORM_AP);
    j.num_field("storm_deciles", d.storm_deciles as f64);
    j.raw("}");
    j.0
}

pub(super) fn figure_samples() -> String {
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.key("figures").open_arr();
    for (i, f) in vleo_modules::figure::samples().iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw(&vleo_modules::figure::json(f));
    }
    j.close_arr();
    j.raw("}");
    j.0
}

/// One saved result, whole.
pub(super) fn result_json(params: &str) -> String {
    let name = param(params, "name").map(decode).unwrap_or_default();
    let s = match vleo_modules::results::store::open(&results_dir(), &name) {
        Ok(s) => s,
        Err(e) => return failed(&e),
    };
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("file", &name);
    result_head(&mut j, &name, &s);
    rows_json(&mut j, "inputs", &s.inputs);
    rows_json(&mut j, "outputs", &s.outputs);
    rows_json(&mut j, "blocked_rows", &s.blocked);
    if let Some(w) = &s.sweep {
        // The same wire form a sweep run now has, so one drawing serves both.
        j.key("sweep_data").raw(&sweep_wire(&s.target, w, None));
    }
    // What the result draws, described by the engine: any face draws this
    // rather than inventing its own picture of the numbers.
    j.key("figures").open_arr();
    for (i, f) in vleo_modules::results::figures(&s).iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw(&vleo_modules::figure::json(f));
    }
    j.close_arr();
    j.raw("}");
    j.0
}

/// One file of a saved result, to download: its `csv`, its `sweep`, or its
/// `report` page.
pub(super) fn result_file(params: &str, kind: &str) -> (&'static str, &'static str, Vec<u8>) {
    let name = param(params, "name").map(decode).unwrap_or_default();
    match vleo_modules::results::store::open(&results_dir(), &name) {
        Ok(s) => match kind {
            "report" => (
                "200 OK",
                "text/html; charset=utf-8",
                vleo_modules::results::html(&s).into_bytes(),
            ),
            "sweep" => match &s.sweep {
                Some(w) => (
                    "200 OK",
                    "text/csv; charset=utf-8",
                    vleo_modules::results::sweep_csv(w).into_bytes(),
                ),
                None => (
                    "404 Not Found",
                    "text/plain; charset=utf-8",
                    format!("{name}: this result is not a sweep").into_bytes(),
                ),
            },
            _ => (
                "200 OK",
                "text/csv; charset=utf-8",
                vleo_modules::results::csv(&s).into_bytes(),
            ),
        },
        Err(e) => ("404 Not Found", "text/plain; charset=utf-8", e.into_bytes()),
    }
}

/// Run a row on the saved case — with whatever the request sets on top — and
/// keep what it returned. With `over`, `from`, `to` and `points` it is a sweep:
/// the run at the case, and the answer across the range, kept together.
///
/// Kept once: a question already saved is not saved again, and the answer
/// names the result that already holds it (`already`).
pub(super) fn result_save(params: &str, ctx: &Ctx) -> String {
    if let Some(why) = case_refused(params, ctx) {
        return failed(&why);
    }
    let case = build_case(params, ctx);
    for (id, _) in &case.supply {
        if let Some(why) = unsuppliable(id) {
            return refuse(id, &why);
        }
    }
    let sweep = if param(params, "over").is_some() {
        match run_sweep(params, ctx) {
            Ok(w) => Some(w),
            Err(refusal) => return refusal,
        }
    } else {
        None
    };
    let mut scratch = Scratch::new();
    let r = match vleo_modules::evaluate(&case, &mut scratch) {
        Ok(r) => r,
        Err(f) => return failed(&format!("{f}")),
    };
    let label = param(params, "label").map(decode).unwrap_or_default();
    let mut s = vleo_modules::results::from_run(&r, &case.supply, &now_utc(), label.trim());
    s.sweep = sweep;
    kept_json(&s)
}

/// Keep a result and answer with where — or with the result that already
/// answers the same question.
fn kept_json(s: &vleo_modules::results::Saved) -> String {
    match vleo_modules::results::store::save(&results_dir(), s) {
        Ok((file, already)) => {
            let kept = if already {
                vleo_modules::results::store::open(&results_dir(), &file)
                    .unwrap_or_else(|_| s.clone())
            } else {
                s.clone()
            };
            let mut j = Json::new();
            j.raw("{");
            j.bool_field("ok", true);
            j.str_field("file", &file);
            j.bool_field("already", already);
            result_head(&mut j, &file, &kept);
            j.raw("}");
            j.0
        }
        Err(e) => failed(&e),
    }
}

/// Why the design is what it is: every registered risk, where it stands and
/// every node version that moved it — the conclusion the risk-register rows
/// draw — and the version each node is at.
///
/// Read from the sheets on each request rather than held: the daemon never
/// writes a sheet, and a developer applying forms beside a running copy
/// should see the register move without a restart.
pub(super) fn derisk_json(ctx: &Ctx) -> String {
    let tree = match vleo_sheet::load::load_all(&ctx.root) {
        Ok(t) => t,
        Err(e) => return failed(&format!("the tree does not load: {e}")),
    };
    let reg = vleo_sheet::derisk::register(&tree);
    let (changes, _) = vleo_sheet::derisk::narrative(&tree);
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.num_field("changes", changes.len() as f64);
    j.num_field(
        "published",
        tree.sheets.values().filter(|s| !s.is_seeded()).count() as f64,
    );
    j.num_field(
        "versioned",
        tree.sheets
            .values()
            .filter(|s| !s.versions.is_empty())
            .count() as f64,
    );
    j.key("register").open_arr();
    for (i, r) in reg.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        for (k, v) in [
            ("id", &r.id),
            ("title", &r.title),
            ("owner", &r.owner),
            ("why", &r.why),
            ("row", &r.row),
            ("row_label", &r.row_label),
            ("registered", &r.registered),
            ("now", &r.now),
        ] {
            j.str_field(k, v);
        }
        j.key("moves").open_arr();
        for (m_i, m) in r.moves.iter().enumerate() {
            if m_i > 0 {
                j.raw(",");
            }
            j.raw("{");
            j.str_field("node", &m.node);
            j.str_field("label", &m.label);
            j.num_field("n", m.n as f64);
            j.str_field("date", &m.date);
            j.str_field("release", &m.release);
            j.str_field("what", &m.what);
            j.str_field("learned", &m.learned);
            j.close_obj();
        }
        j.close_arr();
        j.close_obj();
    }
    j.close_arr();
    j.key("nodes").raw("{");
    let mut first = true;
    for sh in tree.ordered() {
        let Some(v) = sh.versions.last() else {
            continue;
        };
        if !first {
            j.raw(",");
        }
        first = false;
        j.push_string(&sh.id);
        j.raw(":{");
        j.num_field("n", v.n as f64);
        j.str_field("release", &v.release);
        j.str_field("rests_on", &v.rests_on);
        j.str_field("learned", &v.learned);
        j.close_obj();
    }
    j.raw("}");
    j.raw("}");
    j.0
}

/// Keep a result somebody sent — its report page, which carries everything, or
/// its `result.csv` with, for a sweep, its `sweep.csv` beside it.
pub(super) fn result_upload(params: &str) -> String {
    let text = param(params, "csv").map(decode).unwrap_or_default();
    let mut s = match vleo_modules::results::read(&vleo_modules::results::unwrap_report(&text)) {
        Ok(s) => s,
        Err(e) => return failed(&e),
    };
    let sweep_text = vleo_modules::results::unwrap_sweep(&text).or_else(|| {
        param(params, "sweep")
            .map(decode)
            .filter(|t| !t.trim().is_empty())
    });
    if let Some(t) = sweep_text {
        match vleo_modules::results::read_sweep(&t) {
            Ok(w) => s.sweep = Some(w),
            Err(e) => return failed(&format!("the sweep that came with it: {e}")),
        }
    }
    kept_json(&s)
}

/// Pin a result (`on=1`) so it is kept whole for good, or unpin it (`on=0`)
/// so it is thinned to its summary once it is older than the keep period.
pub(super) fn result_pin(params: &str) -> String {
    let name = param(params, "name").map(decode).unwrap_or_default();
    let on = flag(params, "on");
    match vleo_modules::results::store::pin(&results_dir(), &name, on) {
        Ok(()) => {
            let mut j = Json::new();
            j.raw("{");
            j.bool_field("ok", true);
            j.str_field("file", &name);
            j.bool_field("pinned", on);
            j.raw("}");
            j.0
        }
        Err(e) => failed(&e),
    }
}

/// Thin every unpinned result older than the keep period, once, at start: in
/// the background, so a large shared folder never holds the page back.
pub(super) fn thin_at_start() {
    let days = match vleo_data::keep_days() {
        Ok(Some(d)) => d,
        Ok(None) => {
            println!("  results kept whole for good (VLEO_KEEP_DAYS=0, or a VLEO_RESULTS folder)");
            return;
        }
        Err(why) => {
            println!("  \x1b[33m{why} — nothing was thinned\x1b[0m");
            return;
        }
    };
    println!("  results older than {days} days, unpinned, are thinned to their summary");
    std::thread::spawn(move || {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        match vleo_modules::results::store::thin_old(&results_dir(), now, days) {
            Ok(done) if !done.is_empty() => {
                println!("  thinned {} result(s) to their summary", done.len())
            }
            Ok(_) => {}
            Err(e) => println!("  \x1b[33mthinning stopped: {e}\x1b[0m"),
        }
    });
}

pub(super) fn result_delete(params: &str) -> String {
    let name = param(params, "name").map(decode).unwrap_or_default();
    match vleo_modules::results::store::remove(&results_dir(), &name) {
        Ok(()) => "{\"ok\":true}".to_string(),
        Err(e) => failed(&e),
    }
}

/// A result's inputs, made the saved case: checked like any other case, saved
/// only when every value still applies, and carried over first when the result
/// was saved against another set of inputs.
pub(super) fn result_as_case(params: &str) -> String {
    let name = param(params, "name").map(decode).unwrap_or_default();
    let s = match vleo_modules::results::store::open(&results_dir(), &name) {
        Ok(s) => s,
        Err(e) => return failed(&e),
    };
    let r = vleo_modules::inputs::check_values(&s.case_values());
    let mut j = Json::new();
    j.raw("{");
    if !r.ok() {
        j.bool_field("ok", false);
        j.str_field(
            "message",
            &format!(
                "the case was not changed: {} of the result's inputs no longer apply to this tree",
                r.refused.len()
            ),
        );
        reading_json(&mut j, &r);
        j.raw("}");
        return j.0;
    }
    let path = case_path();
    let written = if r.set.is_empty() {
        vleo_modules::inputs::saved::clear(&path)
    } else {
        vleo_modules::inputs::saved::store(&path, &vleo_modules::inputs::csv(&r.set))
    };
    match written {
        Ok(()) => {
            j.bool_field("ok", true);
            reading_json(&mut j, &r);
        }
        Err(e) => {
            j.bool_field("ok", false);
            j.str_field("message", &format!("{}: {e}", path.display()));
        }
    }
    j.raw("}");
    j.0
}
