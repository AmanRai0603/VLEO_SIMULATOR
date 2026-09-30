//! Saved results: kept, listed, opened, pinned, sent and taken back — viewing
//! one runs nothing.

use super::*;

pub(crate) fn results_dir() -> PathBuf {
    vleo_data::results_path()
}

pub(crate) fn rows_json(j: &mut Json, key: &str, rows: &[vleo_modules::results::Row]) {
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

pub(crate) fn result_head(j: &mut Json, s: &vleo_modules::results::Saved) {
    j.str_field("target", &s.target);
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
pub(crate) fn results_list() -> String {
    let dir = results_dir();
    // THINNED WHEN LISTED, not by a clock of its own: a result past its days
    // keeps its answer, its inputs and its chain and drops every other value.
    // Every machine sharing the folder may do it; each writes the same bytes,
    // whole, so two doing it at once is the same as one.
    let keep = match vleo_data::keep_days() {
        Ok(k) => k,
        Err(e) => return failed(&e),
    };
    let thinned = if keep == 0 {
        Vec::new()
    } else {
        vleo_modules::results::store::thin(
            &dir,
            &vleo_data::clock::days_ago(keep),
            &vleo_data::clock::today(),
        )
    };
    let (good, bad) = vleo_modules::results::store::list(&dir);
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("path", &dir.display().to_string());
    j.num_field("keep_days", keep as f64);
    j.num_field("thinned_now", thinned.len() as f64);
    j.key("results").open_arr();
    for (k, (file, s)) in good.iter().enumerate() {
        if k > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("file", file);
        j.bool_field("pinned", vleo_modules::results::store::pinned(&dir, file));
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

/// One saved result, whole — as it was, and nothing runs.
///
/// A thinned one keeps only its answer and the inputs it changed. With
/// `rerun=1` it is run again on those inputs and today's engine, and every
/// value that run returns is shown BESIDE the record, never in its place: the
/// record's own answer stays, and the two chains say whether the run is the
/// same one. Same chain, same engine, tree and inputs — the values are what
/// it returned then. Another chain, and they are today's, said so.
pub(crate) fn result_json(params: &str, ctx: &Ctx) -> String {
    let dir = results_dir();
    let name = param(params, "name").map(decode).unwrap_or_default();
    let s = match vleo_modules::results::store::open(&dir, &name) {
        Ok(s) => s,
        Err(e) => return failed(&e),
    };
    let again =
        if !s.thinned.is_empty() && param(params, "rerun").map(decode).as_deref() == Some("1") {
            let case = Case {
                base: String::new(),
                supply: s.case_values(),
                target: s.target.clone(),
                mode: RunMode::from_name(&s.mode),
                data: ctx.data.clone(),
                data_versions: ctx.data_versions.clone(),
            };
            match vleo_modules::evaluate(&case, &mut Scratch::new()) {
                Ok(r) => Some(vleo_modules::results::from_run(
                    &r,
                    &case.supply,
                    &s.saved,
                    &s.name,
                )),
                Err(f) => return failed(&format!("it did not run again: {f}")),
            }
        } else {
            None
        };
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("file", &name);
    j.bool_field("pinned", vleo_modules::results::store::pinned(&dir, &name));
    result_head(&mut j, &s);
    let rows = again.as_ref().unwrap_or(&s);
    if let Some(a) = &again {
        j.key("rerun").raw("{");
        j.str_field("chain", &a.chain);
        j.bool_field("same", a.chain == s.chain);
        j.close_obj();
    }
    rows_json(&mut j, "inputs", &rows.inputs);
    rows_json(&mut j, "outputs", &rows.outputs);
    rows_json(&mut j, "blocked_rows", &rows.blocked);
    j.raw("}");
    j.0
}

/// A result as one file to send: its CSV, its report and a manifest, zipped.
pub(crate) fn result_share(params: &str) -> (&'static str, &'static str, Vec<u8>) {
    let name = param(params, "name").map(decode).unwrap_or_default();
    match vleo_modules::results::store::open(&results_dir(), &name) {
        Ok(s) => (
            "200 OK",
            "application/zip",
            vleo_modules::results::share::pack(&s),
        ),
        Err(e) => ("404 Not Found", "text/plain; charset=utf-8", e.into_bytes()),
    }
}

/// Pin a result — kept whole whatever its age — or unpin it: `name`, `on`.
pub(crate) fn result_pin(params: &str) -> String {
    let name = param(params, "name").map(decode).unwrap_or_default();
    let on = param(params, "on").map(decode).as_deref() != Some("0");
    match vleo_modules::results::store::pin(&results_dir(), &name, on) {
        Ok(()) => {
            let mut j = Json::new();
            j.raw("{");
            j.bool_field("ok", true);
            j.bool_field("pinned", on);
            j.raw("}");
            j.0
        }
        Err(e) => failed(&e),
    }
}

pub(crate) fn result_file(params: &str, report: bool) -> (&'static str, &'static str, Vec<u8>) {
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
pub(crate) fn result_save(params: &str, ctx: &Ctx) -> String {
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

/// Keep a result somebody sent — `csv=` its CSV or the report page it rides
/// in, or `vleo=` its share file in base64.
pub(crate) fn result_upload(params: &str) -> String {
    let read = match param(params, "vleo").map(decode) {
        Some(b64) => match base64_decode(&b64) {
            Some(bytes) => vleo_modules::results::share::unpack(&bytes),
            None => Err("the .vleo file did not arrive whole".to_string()),
        },
        None => {
            let text = param(params, "csv").map(decode).unwrap_or_default();
            vleo_modules::results::read(&vleo_modules::results::unwrap_report(&text))
        }
    };
    let s = match read {
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

/// Standard base64, as a browser's `btoa` or `FileReader` writes it. Anything
/// else is `None`: a file that did not arrive whole is not guessed at.
pub(crate) fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.trim().trim_end_matches('=');
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0);
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        };
        acc = ((acc << 6) | u32::from(v)) & 0xFFFF;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}

pub(crate) fn result_delete(params: &str) -> String {
    let name = param(params, "name").map(decode).unwrap_or_default();
    match vleo_modules::results::store::remove(&results_dir(), &name) {
        Ok(()) => "{\"ok\":true}".to_string(),
        Err(e) => failed(&e),
    }
}

/// A result's inputs, made the saved case: checked like any other case, saved
/// only when every value still applies, and carried over first when the result
/// was saved against another set of inputs.
pub(crate) fn result_as_case(params: &str) -> String {
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

#[cfg(test)]
mod share_files {
    use super::base64_decode;

    #[test]
    fn base64_reads_what_a_browser_writes_and_refuses_the_rest() {
        assert_eq!(base64_decode("UEsDBA==").unwrap(), b"PK\x03\x04");
        assert_eq!(base64_decode("aGVsbG8gd29ybGQ").unwrap(), b"hello world");
        let long: Vec<u8> = (0..=255u8).cycle().take(5000).collect();
        let enc: String = long
            .chunks(3)
            .flat_map(|c| {
                const A: &[u8] =
                    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
                let n = c
                    .iter()
                    .enumerate()
                    .fold(0u32, |a, (i, &b)| a | u32::from(b) << (16 - 8 * i));
                (0..c.len() + 1).map(move |i| A[(n >> (18 - 6 * i) & 63) as usize] as char)
            })
            .collect();
        assert_eq!(
            base64_decode(&enc).unwrap(),
            long,
            "a long file did not come back whole"
        );
        assert!(base64_decode("not base64!").is_none());
    }
}
