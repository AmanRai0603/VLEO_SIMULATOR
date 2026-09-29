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

pub(super) fn result_head(j: &mut Json, s: &vleo_modules::results::Saved) {
    j.str_field("target", &s.target);
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
        result_head(&mut j, s);
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
    result_head(&mut j, &s);
    rows_json(&mut j, "inputs", &s.inputs);
    rows_json(&mut j, "outputs", &s.outputs);
    rows_json(&mut j, "blocked_rows", &s.blocked);
    j.raw("}");
    j.0
}

pub(super) fn result_file(params: &str, report: bool) -> (&'static str, &'static str, Vec<u8>) {
    let name = param(params, "name").map(decode).unwrap_or_default();
    match vleo_modules::results::store::open(&results_dir(), &name) {
        Ok(s) if report => (
            "200 OK",
            "text/html; charset=utf-8",
            vleo_modules::results::html(&s).into_bytes(),
        ),
        Ok(s) => (
            "200 OK",
            "text/csv; charset=utf-8",
            vleo_modules::results::csv(&s).into_bytes(),
        ),
        Err(e) => ("404 Not Found", "text/plain; charset=utf-8", e.into_bytes()),
    }
}

/// Run a row on the saved case — with whatever the request sets on top — and
/// keep what it returned.
pub(super) fn result_save(params: &str, ctx: &Ctx) -> String {
    if let Some(why) = case_refused(params, ctx) {
        return failed(&why);
    }
    let case = build_case(params, ctx);
    let mut scratch = Scratch::new();
    let r = match vleo_modules::evaluate(&case, &mut scratch) {
        Ok(r) => r,
        Err(f) => return failed(&format!("{f}")),
    };
    let label = param(params, "label").map(decode).unwrap_or_default();
    let s = vleo_modules::results::from_run(&r, &case.supply, &now_utc(), label.trim());
    match vleo_modules::results::store::save(&results_dir(), &s) {
        Ok(file) => {
            let mut j = Json::new();
            j.raw("{");
            j.bool_field("ok", true);
            j.str_field("file", &file);
            result_head(&mut j, &s);
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

/// Keep a result somebody sent — its CSV, or the report page it rides in.
pub(super) fn result_upload(params: &str) -> String {
    let text = param(params, "csv").map(decode).unwrap_or_default();
    let s = match vleo_modules::results::read(&vleo_modules::results::unwrap_report(&text)) {
        Ok(s) => s,
        Err(e) => return failed(&e),
    };
    match vleo_modules::results::store::save(&results_dir(), &s) {
        Ok(file) => {
            let mut j = Json::new();
            j.raw("{");
            j.bool_field("ok", true);
            j.str_field("file", &file);
            result_head(&mut j, &s);
            j.raw("}");
            j.0
        }
        Err(e) => failed(&e),
    }
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
