//! The saved case: its inputs read, checked, saved and reset, and the values a request sets.

use super::*;

pub(super) fn case_path() -> PathBuf {
    vleo_data::case_path()
}

/// The saved case, read and checked against the tree as it is now.
///
/// Read on every request rather than held, because it is a small file and a
/// person may replace it from another tab. CARRIED OVER on the first read after
/// the tool changes under it: a case written for another set of inputs is
/// copied aside and written again in this tree's template, with every value
/// that could not be carried set aside by name — see `inputs::saved::load`. So
/// a stored case always applies whole, and what an update did to it is shown
/// on the Inputs page and counted on every run until the case is next saved.
pub(super) fn saved_case() -> vleo_modules::inputs::saved::Saved {
    vleo_modules::inputs::saved::load(&case_path())
}

pub(super) fn inputs_csv(params: &str) -> (&'static str, &'static str, Vec<u8>) {
    const CSV: &str = "text/csv; charset=utf-8";
    if param(params, "inputs") == Some("defaults") {
        return ("200 OK", CSV, vleo_modules::inputs::csv(&[]).into_bytes());
    }
    let saved = saved_case();
    if param(params, "backup") == Some("1") {
        // Only the copy the upgrade itself recorded, and only beside the case:
        // a path read out of a file is never a path this route will open
        // anywhere else.
        let dir = case_path().parent().map(|d| d.to_path_buf());
        let kept = saved
            .reading
            .upgrade
            .as_ref()
            .and_then(|u| u.backup.as_ref())
            .map(PathBuf::from)
            .filter(|b| b.parent().map(|d| d.to_path_buf()) == dir);
        return match kept.and_then(|b| std::fs::read(b).ok()) {
            Some(bytes) => ("200 OK", CSV, bytes),
            None => (
                "404 Not Found",
                "text/plain; charset=utf-8",
                b"no copy of an earlier case is recorded".to_vec(),
            ),
        };
    }
    (
        "200 OK",
        CSV,
        vleo_modules::inputs::csv_with(&saved.reading.set, saved.reading.upgrade.as_ref())
            .into_bytes(),
    )
}

pub(super) fn reading_json(j: &mut Json, r: &vleo_modules::inputs::Reading) {
    j.num_field("changed", r.changed as f64);
    j.num_field("defaulted", r.defaulted as f64);
    j.key("set").open_arr();
    for (k, (id, si)) in r.set.iter().enumerate() {
        if k > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("id", id);
        j.num_field("value", *si);
        j.close_obj();
    }
    j.close_arr();
    j.key("refused").open_arr();
    for (k, (line, id, why)) in r.refused.iter().enumerate() {
        if k > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.num_field("line", *line as f64);
        j.str_field("id", id);
        j.str_field("why", why);
        j.close_obj();
    }
    j.close_arr();
    match &r.template {
        Some(t) => j.str_field("template", t),
        None => j.key("template").raw("null"),
    };
    j.bool_field("outdated", r.outdated);
    j.key("upgrade");
    match &r.upgrade {
        None => {
            j.raw("null");
        }
        Some(u) => {
            j.raw("{");
            j.str_field("from", &u.from);
            match &u.backup {
                Some(b) => j.str_field("backup", b),
                None => j.key("backup").raw("null"),
            };
            j.key("new").open_arr();
            for (k, n) in u.new.iter().enumerate() {
                if k > 0 {
                    j.raw(",");
                }
                j.push_string(n);
            }
            j.close_arr();
            j.key("set_aside").open_arr();
            for (k, a) in u.set_aside.iter().enumerate() {
                if k > 0 {
                    j.raw(",");
                }
                j.raw("{");
                j.str_field("id", &a.id);
                j.str_field("value", &a.value);
                j.str_field("unit", &a.unit);
                j.str_field("why", &a.why);
                j.close_obj();
            }
            j.close_arr();
            j.close_obj();
        }
    }
}

/// Every input of the case, its group, default, range and saved value.
pub(super) fn inputs_json() -> String {
    let loaded = saved_case();
    let saved = &loaded.reading;
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    if let Some(c) = Vleo::default_case() {
        j.str_field("case", c.id);
        j.str_field("label", c.label);
        j.str_field("note", c.note);
    }
    j.str_field("path", &case_path().display().to_string());
    j.bool_field("stored", loaded.stored);
    j.str_field("template_now", &vleo_modules::inputs::template());
    if let Some(e) = &loaded.error {
        j.str_field("error", e);
    }
    reading_json(&mut j, saved);
    j.key("inputs").open_arr();
    for (k, i) in vleo_modules::inputs::case_inputs().iter().enumerate() {
        if k > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("id", i.id);
        j.str_field("label", i.label);
        j.str_field("symbol", i.symbol);
        j.str_field("group", i.group.name());
        j.str_field("unit", i.unit);
        j.num_field("factor", i.factor);
        j.num_field("lo", i.lo);
        j.num_field("hi", i.hi);
        j.num_field("default", i.default);
        match saved.set.iter().find(|(id, _)| id == i.id) {
            Some((_, v)) => j.num_field("value", *v),
            None => j.key("value").raw("null"),
        };
        j.close_obj();
    }
    j.close_arr();
    j.raw("}");
    j.0
}

/// What a CSV or a set of values would do, without saving it.
pub(super) fn inputs_reading(params: &str) -> vleo_modules::inputs::Reading {
    match param(params, "csv") {
        Some(csv) => vleo_modules::inputs::read_csv(&decode(csv)),
        None => {
            let mut r = vleo_modules::inputs::check_values(&sets(params));
            if let Some((id, why)) = set_refusal(params) {
                r.refused.insert(0, (0, id, why));
            }
            r
        }
    }
}

pub(super) fn inputs_check(params: &str) -> String {
    let r = inputs_reading(params);
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", r.ok());
    reading_json(&mut j, &r);
    j.raw("}");
    j.0
}

/// Save a case — a whole one, replacing what was there.
///
/// ALL OR NOTHING. A file with one refused row is not saved at all, and the
/// reply names every refusal: keeping the good rows of a bad file would leave
/// the tool running on a case nobody wrote.
pub(super) fn inputs_save(params: &str) -> String {
    let r = inputs_reading(params);
    let mut j = Json::new();
    j.raw("{");
    if !r.ok() {
        j.bool_field("ok", false);
        j.str_field(
            "message",
            &format!(
                "nothing was saved: {} row(s) cannot be applied. Correct them and upload again.",
                r.refused.len()
            ),
        );
        reading_json(&mut j, &r);
        j.raw("}");
        return j.0;
    }
    let path = case_path();
    let written = vleo_modules::inputs::saved::store(&path, &vleo_modules::inputs::csv(&r.set));
    match written {
        Ok(()) => {
            j.bool_field("ok", true);
            j.str_field("path", &path.display().to_string());
            reading_json(&mut j, &r);
        }
        Err(e) => {
            j.bool_field("ok", false);
            j.str_field(
                "message",
                &format!("the case could not be written to {}: {e}", path.display()),
            );
        }
    }
    j.raw("}");
    j.0
}

/// Back to every default: the stored case is removed.
pub(super) fn inputs_reset() -> String {
    let path = case_path();
    let mut j = Json::new();
    j.raw("{");
    match vleo_modules::inputs::saved::clear(&path) {
        Ok(()) => {
            j.bool_field("ok", true);
        }
        Err(e) => {
            j.bool_field("ok", false);
            j.str_field(
                "message",
                &format!("could not remove {}: {e}", path.display()),
            );
        }
    }
    j.raw("}");
    j.0
}

/// `set=id:value` repeated. Values are SI, always: a face converts for display
/// and never for transport.
pub(super) fn sets(params: &str) -> Vec<(String, f64)> {
    params
        .split('&')
        .filter_map(|kv| {
            let (k, v) = kv.split_once('=')?;
            if k != "set" {
                return None;
            }
            let d = decode(v);
            let (id, val) = d.split_once(':')?;
            Some((id.to_string(), val.parse().ok()?))
        })
        .collect()
}

/// The first `set=` that cannot be applied, named, with why.
///
/// `sets` keeps only what it can read, so a value typed wrong was dropped and
/// the run went ahead on the design's own number — a refusal turned into a
/// substitution. Every endpoint that takes values asks this first, as the
/// command line always has.
pub(super) fn set_refusal(params: &str) -> Option<(String, String)> {
    for kv in params.split('&') {
        let Some(("set", v)) = kv.split_once('=') else {
            continue;
        };
        let d = decode(v);
        let Some((id, val)) = d.split_once(':') else {
            return Some((d.clone(), format!("'{d}' is not id:value")));
        };
        if Vleo::find(id).is_none() {
            return Some((id.to_string(), format!("there is no row called '{id}'")));
        }
        match val.trim().parse::<f64>() {
            Ok(x) if x.is_finite() => {}
            _ => return Some((id.to_string(), format!("'{val}' is not a number"))),
        }
    }
    None
}

/// A supplied value only survives on a row that declares its own number.
///
/// Every other kind works its answer out during the run and overwrites what was
/// supplied, so a `set=` on one was accepted, ignored, and reported as a
/// successful run against a number nobody asked for. A sweep over one drew a
/// flat line and said "0 refused", which reads as a real result — a reader
/// turns the knob and nothing moves, and nothing anywhere says why.
///
/// The CLI has refused this since it was written (`suppliable`); this path
/// never checked. It stayed invisible while every driver was declared, and
/// became load-bearing the moment `env_f107` started reading the solar
/// subsystem: the two faces then disagreed about the same request.
pub(super) fn unsuppliable(id: &str) -> Option<String> {
    vleo_modules::why_not_suppliable(id)
}
