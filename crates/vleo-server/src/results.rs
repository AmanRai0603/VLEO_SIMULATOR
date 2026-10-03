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
pub(super) fn record_figure(ctx: &Ctx, id: &str, params: &str) -> String {
    const BUNDLE: &str = "solar-weather";
    const KNOWN: [&str; 17] = [
        "density",
        "storm-scale",
        "kp-ap",
        "recurrence-f107",
        "recurrence-ap",
        "spikes",
        "mean-cycle",
        "segments",
        "regime-phase",
        "climate",
        "smoother",
        "growth",
        "forecast",
        "drivers",
        "thermosphere",
        "design",
        "closure",
    ];
    if !KNOWN.contains(&id) {
        return failed(&format!(
            "no figure of the record called '{id}'. The engine works out: {}",
            KNOWN.join(", ")
        ));
    }
    let Some((dir, _)) = ctx.bundles.get(BUNDLE) else {
        return failed(&format!(
            "{BUNDLE} is not installed here, so there is no record to work the figure out from"
        ));
    };
    let bundle = match vleo_data::load_bundle(dir) {
        Ok(b) => b,
        Err(e) => return failed(e.message()),
    };
    let days = match vleo_data::read_solar_days(&bundle) {
        Ok(d) => d,
        Err(e) => return failed(e.message()),
    };
    // Only the figures that fold by cycle read the cycle table; one that is
    // missing refuses them and nothing else.
    use vleo_modules::record::Driver;
    // The figures drawn over any of the three drivers say which; one they do
    // not know is refused by name.
    let any_driver = || match param(params, "v").unwrap_or("f107") {
        "f107" => Ok(Driver::F107),
        "ap" => Ok(Driver::Ap),
        "ssn" => Ok(Driver::Ssn),
        other => Err(failed(&format!(
            "no driver '{other}' in the record's figures: f107, ap or ssn"
        ))),
    };
    let driver_name = |d: Driver| match d {
        Driver::F107 => "f107",
        Driver::Ap => "ap",
        Driver::Ssn => "ssn",
    };
    let cycles = if matches!(id, "storm-scale" | "spikes" | "mean-cycle" | "regime-phase")
        || (id == "growth" && param(params, "by") == Some("cycle"))
    {
        match vleo_data::read_solar_cycles(&bundle) {
            Ok(c) => c,
            Err(e) => return failed(e.message()),
        }
    } else {
        Vec::new()
    };
    let opt = |j: &mut Json, k: &str, v: Option<f64>| {
        match v {
            Some(v) => j.num_field(k, v),
            None => j.key(k).raw("null"),
        };
    };
    // An array of numbers, a missing one written as null: the page draws a
    // gap there, and the number it would have been is never invented.
    let arr = |j: &mut Json, v: &[Option<f64>]| {
        j.open_arr();
        for (i, x) in v.iter().enumerate() {
            if i > 0 {
                j.raw(",");
            }
            j.raw(&x.map_or_else(|| "null".to_string(), crate::json::num));
        }
        j.close_arr();
    };
    let some = |v: &[f64]| v.iter().map(|&x| Some(x)).collect::<Vec<_>>();
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
    match id {
        "density" => {
            let d = vleo_modules::record::density(&days);
            j.num_field("days", d.days as f64);
            opt(&mut j, "r", d.r);
            opt(&mut j, "median_f107", d.median_f107);
            opt(&mut j, "median_ap", d.median_ap);
            j.num_field("below_both_pct", d.below_both_pct);
            j.num_field("storm_ap", vleo_modules::record::STORM_AP);
            j.num_field("storm_deciles", d.storm_deciles as f64);
        }
        "recurrence-f107" | "recurrence-ap" => {
            use vleo_modules::record::{recurrence, RECURRENCE_MAX_LAG, RECURRENCE_WINDOWS};
            let driver = if id == "recurrence-ap" {
                Driver::Ap
            } else {
                Driver::F107
            };
            let r = recurrence(&days, driver);
            j.num_field("max_lag", RECURRENCE_MAX_LAG as f64);
            j.key("windows");
            arr(
                &mut j,
                &RECURRENCE_WINDOWS
                    .iter()
                    .map(|&w| Some(w as f64))
                    .collect::<Vec<_>>(),
            );
            j.key("r").open_arr();
            for (k, c) in r.r.iter().enumerate() {
                if k > 0 {
                    j.raw(",");
                }
                arr(&mut j, c);
            }
            j.close_arr();
            j.num_field("n_lag1", r.n_lag1 as f64);
            opt(&mut j, "naive_band", r.naive_band);
            j.key("band");
            arr(&mut j, &r.band);
            j.num_field("outside", r.outside as f64);
            j.key("peak_lag");
            arr(
                &mut j,
                &r.peaks
                    .iter()
                    .map(|p| p.map(|(lag, _)| lag as f64))
                    .collect::<Vec<_>>(),
            );
            j.key("peak_r");
            arr(
                &mut j,
                &r.peaks
                    .iter()
                    .map(|p| p.map(|(_, r)| r))
                    .collect::<Vec<_>>(),
            );
            j.num_field("above_long", r.above_long as f64);
            j.num_field("compared", r.compared as f64);
        }
        "mean-cycle" => {
            use vleo_modules::record::{mean_cycle, MEAN_CYCLE_MIN_DAYS};
            // The mean cycle is asked for over one driver in some number of phase
            // bins, and says which it was given; anything else is refused by name.
            let driver = match any_driver() {
                Ok(d) => d,
                Err(e) => return e,
            };
            let bins = match param(params, "bins").unwrap_or("20").parse::<usize>() {
                Ok(n) if (2..=100).contains(&n) => n,
                _ => return failed("bins is a whole number of phase bins from 2 to 100"),
            };
            let m = mean_cycle(&days, &cycles, driver, bins, MEAN_CYCLE_MIN_DAYS);
            j.str_field("variable", driver_name(driver));
            j.num_field("bins", bins as f64);
            j.num_field("min_days", MEAN_CYCLE_MIN_DAYS as f64);
            j.key("phase");
            arr(&mut j, &some(&m.phase));
            j.key("cycles");
            arr(
                &mut j,
                &m.cycles.iter().map(|&n| Some(n as f64)).collect::<Vec<_>>(),
            );
            j.key("complete").open_arr();
            for (k, done) in m.complete.iter().enumerate() {
                j.raw(if k > 0 { "," } else { "" });
                j.raw(if *done { "true" } else { "false" });
            }
            j.close_arr();
            j.key("curves").open_arr();
            for (k, c) in m.curves.iter().enumerate() {
                if k > 0 {
                    j.raw(",");
                }
                arr(&mut j, c);
            }
            j.close_arr();
            j.key("mean_cycle");
            arr(&mut j, &m.mean_cycle);
            j.key("pair");
            match m.pair {
                Some((a, b)) => arr(&mut j, &[Some(a as f64), Some(b as f64)]),
                None => {
                    j.raw("null");
                }
            }
            opt(&mut j, "r", m.r);
            j.num_field("usable", m.usable as f64);
            j.num_field("above", m.above as f64);
            opt(&mut j, "gap_rise", m.gap_rise);
            opt(&mut j, "gap_fall", m.gap_fall);
            j.key("peak");
            match m.peak {
                Some((a, b)) => arr(&mut j, &[Some(a), Some(b)]),
                None => {
                    j.raw("null");
                }
            }
        }
        "segments" => {
            // Banded by the rows that define the bands, so only the drivers
            // a row bands: Ap by sw_regime, F10.7 by sw_activity_band.
            let driver = match param(params, "v").unwrap_or("ap") {
                "ap" => Driver::Ap,
                "f107" => Driver::F107,
                other => {
                    return failed(&format!(
                        "no row bands '{other}': ap (sw_regime) or f107 (sw_activity_band)"
                    ))
                }
            };
            let s = vleo_modules::record::segments(&days, driver);
            j.str_field("variable", if driver == Driver::Ap { "ap" } else { "f107" });
            j.num_field("days", s.days as f64);
            j.num_field("refused", s.refused as f64);
            j.num_field("bin_width", s.bin_width);
            j.key("counts");
            arr(
                &mut j,
                &s.counts.iter().map(|&c| Some(c as f64)).collect::<Vec<_>>(),
            );
            opt(&mut j, "max", s.max);
            j.key("band_days");
            arr(
                &mut j,
                &s.band_days
                    .iter()
                    .map(|&c| Some(c as f64))
                    .collect::<Vec<_>>(),
            );
            j.key("band_pct");
            arr(&mut j, &some(&s.band_pct));
            j.key("below_cut");
            arr(&mut j, &s.below_cut);
            j.key("above_cut");
            arr(&mut j, &s.above_cut);
        }
        "regime-phase" => {
            let r = vleo_modules::record::regime_phase(&days, &cycles);
            j.key("phase");
            arr(&mut j, &some(&r.phase));
            j.key("storm_pct");
            arr(&mut j, &r.storm_pct);
            j.key("quiet_pct");
            arr(&mut j, &r.quiet_pct);
            j.key("storm_peak");
            match r.storm_peak {
                Some((v, x)) => arr(&mut j, &[Some(v), Some(x)]),
                None => {
                    j.raw("null");
                }
            }
            j.key("quiet_low");
            match r.quiet_low {
                Some((v, x)) => arr(&mut j, &[Some(v), Some(x)]),
                None => {
                    j.raw("null");
                }
            }
            opt(&mut j, "mirror", r.mirror);
        }
        "climate" => {
            use vleo_modules::record::{climate, Grouping};
            let driver = match any_driver() {
                Ok(d) => d,
                Err(e) => return e,
            };
            let by = match param(params, "by").unwrap_or("year") {
                "year" => Grouping::Year,
                "doy" => Grouping::DayOfYear,
                "month" => Grouping::Month,
                other => {
                    return failed(&format!(
                        "no grouping '{other}': year, doy (five-day bins of the year) or month"
                    ))
                }
            };
            let c = climate(&days, driver, by);
            j.str_field("variable", driver_name(driver));
            j.str_field(
                "by",
                match by {
                    Grouping::Year => "year",
                    Grouping::DayOfYear => "doy",
                    Grouping::Month => "month",
                },
            );
            j.key("key");
            arr(
                &mut j,
                &c.key.iter().map(|&k| Some(k as f64)).collect::<Vec<_>>(),
            );
            j.key("x");
            arr(&mut j, &some(&c.x));
            j.key("mean");
            arr(&mut j, &some(&c.mean));
            j.key("days_in");
            arr(
                &mut j,
                &c.days_in
                    .iter()
                    .map(|&n| Some(n as f64))
                    .collect::<Vec<_>>(),
            );
            opt(&mut j, "overall", c.overall);
            j.num_field("days", c.days as f64);
            for (k, v) in [("high", c.high), ("low", c.low)] {
                j.key(k);
                match v {
                    Some((v, x)) => arr(&mut j, &[Some(v), Some(x)]),
                    None => {
                        j.raw("null");
                    }
                }
            }
            j.num_field("below", c.below as f64);
        }
        "growth" => {
            use vleo_modules::record::{growth, growth_by_cycle, GROWTH_PERCENTILES};
            let driver = match any_driver() {
                Ok(d) => d,
                Err(e) => return e,
            };
            let by = param(params, "by").unwrap_or("all");
            if by != "all" && by != "cycle" {
                return failed(&format!(
                    "no split '{by}': all (the whole record) or cycle (pairs within each cycle)"
                ));
            }
            j.str_field("variable", driver_name(driver));
            j.str_field("by", by);
            let rows = |j: &mut Json, k: &str, v: &[Vec<Option<f64>>]| {
                j.key(k).open_arr();
                for (i, c) in v.iter().enumerate() {
                    if i > 0 {
                        j.raw(",");
                    }
                    arr(j, c);
                }
                j.close_arr();
            };
            if by == "cycle" {
                let g = growth_by_cycle(&days, &cycles, driver);
                j.key("lead_years");
                arr(&mut j, &some(&g.lead_years));
                j.key("cycles");
                arr(
                    &mut j,
                    &g.cycles.iter().map(|&n| Some(n as f64)).collect::<Vec<_>>(),
                );
                rows(&mut j, "change", &g.change);
                opt(&mut j, "shared", g.shared);
                j.key("spread");
                match g.spread {
                    Some((a, b)) => arr(&mut j, &[Some(a), Some(b)]),
                    None => {
                        j.raw("null");
                    }
                }
            } else {
                let g = growth(&days, driver);
                j.key("percentiles");
                arr(&mut j, &some(&GROWTH_PERCENTILES));
                j.key("lead_years");
                arr(&mut j, &some(&g.lead_years));
                j.key("pairs");
                arr(
                    &mut j,
                    &g.pairs.iter().map(|&n| Some(n as f64)).collect::<Vec<_>>(),
                );
                rows(&mut j, "change", &g.change);
                j.num_field("at_year", g.at_year as f64);
                // Where the hump and the dip are, as the lead in years and the
                // 95th's change there — the two things drawn — rather than as
                // indices into arrays the reader has to hold together.
                for (k, at) in [("hump", g.hump), ("dip", g.dip)] {
                    j.key(k);
                    match at.and_then(|i| Some((g.lead_years[i], g.change[2][i]?))) {
                        Some((x, y)) => arr(&mut j, &[Some(x), Some(y)]),
                        None => {
                            j.raw("null");
                        }
                    }
                }
                j.bool_field("humped", g.humped);
            }
        }
        "forecast" => {
            use vleo_modules::record::{
                forecast_by_lead, forecast_by_year, issue_age, FORECAST_YEAR_LEADS,
                FORECAST_YEAR_MIN_PAIRS, ISSUE_GAP_CAP,
            };
            let view = param(params, "view").unwrap_or("lead");
            if !matches!(view, "lead" | "year" | "age") {
                return failed(&format!(
                    "no view '{view}': lead (scored at each lead), year (by the year of issue) \
                     or age (the gap between issues)"
                ));
            }
            j.str_field("view", view);
            let counts = |v: &[u32]| v.iter().map(|&n| Some(n as f64)).collect::<Vec<_>>();
            let holes = |v: &[Option<u32>]| v.iter().map(|n| n.map(f64::from)).collect::<Vec<_>>();
            // A point the page names — where a line peaks, the year the bias is
            // lowest — as the two numbers drawn there, not an index into
            // arrays the reader has to hold together.
            let point = |j: &mut Json, k: &str, at: Option<(f64, f64)>| {
                j.key(k);
                match at {
                    Some((x, y)) => arr(j, &[Some(x), Some(y)]),
                    None => {
                        j.raw("null");
                    }
                }
            };
            if view == "age" {
                let issues = match vleo_data::read_forecast_issues(&bundle) {
                    Ok(i) => i,
                    Err(e) => return failed(e.message()),
                };
                let a = issue_age(&issues);
                j.num_field("issues", a.issues as f64);
                j.num_field("cap", ISSUE_GAP_CAP as f64);
                j.key("gap");
                arr(&mut j, &counts(&a.gap));
                j.key("count");
                arr(&mut j, &counts(&a.count));
                opt(&mut j, "median", a.median);
                opt(&mut j, "mean", a.mean);
                opt(&mut j, "commonest", a.commonest.map(f64::from));
                opt(&mut j, "over_a_day_pct", a.over_a_day_pct);
            } else {
                let issued = match vleo_data::read_forecast_issued(&bundle) {
                    Ok(f) => f,
                    Err(e) => return failed(e.message()),
                };
                if view == "lead" {
                    let f = forecast_by_lead(&days, &issued);
                    j.key("lead");
                    arr(&mut j, &counts(&f.lead));
                    j.key("pairs");
                    arr(&mut j, &counts(&f.pairs));
                    j.key("pairs_strict");
                    arr(&mut j, &counts(&f.pairs_strict));
                    j.key("pairs_leaky");
                    arr(&mut j, &counts(&f.pairs_leaky));
                    j.key("skill_strict");
                    arr(&mut j, &f.skill_strict);
                    j.key("skill_leaky");
                    arr(&mut j, &f.skill_leaky);
                    j.key("bias");
                    arr(&mut j, &some(&f.bias));
                    j.key("rmse");
                    arr(&mut j, &some(&f.rmse));
                    point(
                        &mut j,
                        "peak",
                        f.peak
                            .and_then(|i| Some((f.lead[i] as f64, f.skill_strict[i]?))),
                    );
                    opt(&mut j, "widest", f.widest.map(f64::from));
                } else {
                    let f = forecast_by_year(&days, &issued);
                    j.key("leads");
                    arr(
                        &mut j,
                        &[Some(FORECAST_YEAR_LEADS.0), Some(FORECAST_YEAR_LEADS.1)],
                    );
                    j.num_field("min_pairs", FORECAST_YEAR_MIN_PAIRS as f64);
                    j.key("year");
                    arr(
                        &mut j,
                        &f.year.iter().map(|&y| Some(y as f64)).collect::<Vec<_>>(),
                    );
                    j.key("pairs");
                    arr(&mut j, &holes(&f.pairs));
                    j.key("pairs_strict");
                    arr(&mut j, &holes(&f.pairs_strict));
                    j.key("pairs_leaky");
                    arr(&mut j, &holes(&f.pairs_leaky));
                    j.key("skill_strict");
                    arr(&mut j, &f.skill_strict);
                    j.key("skill_leaky");
                    arr(&mut j, &f.skill_leaky);
                    j.key("bias");
                    arr(&mut j, &f.bias);
                    j.key("rmse");
                    arr(&mut j, &f.rmse);
                    j.key("thin");
                    arr(
                        &mut j,
                        &f.thin.iter().map(|&y| Some(y as f64)).collect::<Vec<_>>(),
                    );
                    opt(&mut j, "worst", f.worst.map(f64::from));
                    opt(&mut j, "best", f.best.map(f64::from));
                    let at = |y: i32| {
                        let i = f.year.iter().position(|&v| v == y)?;
                        Some((y as f64, f.bias[i]?))
                    };
                    point(&mut j, "lowest_bias", f.lowest_bias.and_then(at));
                }
            }
        }
        "drivers" => {
            use vleo_modules::record::{drivers_parity, DRIVER_QUANTITIES, DRIVER_SCENARIOS};
            // OURS: the solar crossing run on the saved case, as every panel's
            // engine values are — its members are `<node>.<quantity>_<scenario>`,
            // and its own answer is the one cell that is not a member.
            const NODE: &str = "l3_solar_interface";
            let case = build_case(&format!("node={NODE}"), ctx);
            let mut scratch = Scratch::new();
            let run = match vleo_modules::evaluate(&case, &mut scratch) {
                Ok(r) => r,
                Err(f) => return failed(&format!("{NODE} did not run: {f}")),
            };
            let value = |id: &str| {
                run.values
                    .iter()
                    .find(|v| v.id == id)
                    .map(|v| v.value)
                    .filter(|v| v.is_finite())
            };
            let mut ours = [[None; 5]; 5];
            for (q, qn) in DRIVER_QUANTITIES.iter().enumerate() {
                for (sc, sn) in DRIVER_SCENARIOS.iter().enumerate() {
                    ours[q][sc] = if *qn == "f107" && *sn == "hotmean" {
                        value(NODE)
                    } else {
                        value(&format!("{NODE}.{qn}_{sn}"))
                    };
                }
            }
            // THEIRS: the legacy tool's own answers for the same cells, read by
            // the column names in the file's header. Read from matlab/reference
            // and never through the bundle readers, on purpose: a bundle is what
            // was OBSERVED, with a provenance and a licence, and this is what a
            // DIFFERENT PROGRAM computed. One reader for both is the first step
            // toward presenting a second implementation's output as evidence
            // about the sky.
            const LEGACY: &str = "mission_drivers.csv";
            let path = ctx.root.join("matlab").join("reference").join(LEGACY);
            let text = match std::fs::read_to_string(&path) {
                Ok(t) => t,
                Err(_) => return failed(&format!("matlab/reference/{LEGACY} is not on disk")),
            };
            let mut theirs = [[None; 5]; 5];
            let mut head: Option<Vec<&str>> = None;
            for line in text.lines().map(str::trim) {
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let f: Vec<&str> = line.split(',').map(str::trim).collect();
                let Some(h) = &head else {
                    head = Some(f);
                    continue;
                };
                let at = |name: &str| h.iter().position(|c| *c == name).and_then(|i| f.get(i));
                let Some(sc) =
                    at("scenario").and_then(|s| DRIVER_SCENARIOS.iter().position(|x| x == s))
                else {
                    continue;
                };
                for (q, qn) in DRIVER_QUANTITIES.iter().enumerate() {
                    theirs[q][sc] = at(qn)
                        .and_then(|v| v.parse::<f64>().ok())
                        .filter(|v| v.is_finite());
                }
            }
            let p = drivers_parity(&ours, &theirs);
            let names = |j: &mut Json, k: &str, v: &[&str]| {
                j.key(k).open_arr();
                for (i, n) in v.iter().enumerate() {
                    if i > 0 {
                        j.raw(",");
                    }
                    j.raw(&crate::json::string(n));
                }
                j.close_arr();
            };
            let rows = |j: &mut Json, k: &str, v: &[[Option<f64>; 5]; 5]| {
                j.key(k).open_arr();
                for (i, c) in v.iter().enumerate() {
                    if i > 0 {
                        j.raw(",");
                    }
                    arr(j, c);
                }
                j.close_arr();
            };
            j.str_field("node", NODE);
            j.str_field("legacy", &format!("matlab/reference/{LEGACY}"));
            names(&mut j, "scenarios", &DRIVER_SCENARIOS);
            names(&mut j, "quantities", &DRIVER_QUANTITIES);
            rows(&mut j, "ours", &ours);
            rows(&mut j, "theirs", &theirs);
            rows(&mut j, "ratio", &p.ratio);
            rows(&mut j, "gap", &p.gap);
            // Where each quantity's gap is widest, as the scenario's name — a
            // name the page looks up, not an index it has to trust.
            j.key("widest").open_arr();
            for (i, w) in p.widest.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                j.raw(&w.map_or_else(
                    || "null".to_string(),
                    |w| crate::json::string(DRIVER_SCENARIOS[w]),
                ));
            }
            j.close_arr();
            j.num_field("agree", p.agree as f64);
            j.num_field("below", p.below as f64);
            j.num_field("above", p.above as f64);
            opt(&mut j, "factor", p.factor);
        }
        "thermosphere" => {
            if let Err(e) = thermosphere(ctx, param(params, "view").unwrap_or("solar"), &mut j) {
                return failed(&e);
            }
        }
        // Both run on the case the request carries, overrides and all, as the
        // page's own sweeps did; a bad override is refused as a run's is.
        "design" | "closure" => {
            if let Some(refusal) = case_refused(params, ctx) {
                return refusal;
            }
            let made = if id == "design" {
                design(ctx, params, &days, &mut j)
            } else {
                closure(ctx, params, &mut j)
            };
            if let Err(e) = made {
                return failed(&e);
            }
        }
        "smoother" => {
            let driver = match any_driver() {
                Ok(d) => d,
                Err(e) => return e,
            };
            let months = match vleo_data::read_monthly_means(&bundle) {
                Ok(m) => m,
                Err(e) => return failed(e.message()),
            };
            let s = vleo_modules::record::smoother(&months, driver);
            j.str_field("variable", driver_name(driver));
            j.key("x");
            arr(&mut j, &some(&s.x));
            j.key("raw");
            arr(&mut j, &s.raw);
            j.key("smooth");
            arr(&mut j, &s.smooth);
            j.num_field("missing", s.missing as f64);
            j.num_field("months", s.months as f64);
            j.num_field("raw_range", s.raw_range);
            j.num_field("smooth_range", s.smooth_range);
            j.num_field("rms", s.rms);
            j.num_field("crossings", s.crossings as f64);
            j.num_field("max_departure", s.max_departure);
        }
        "spikes" => {
            let s = vleo_modules::record::spikes(&days, &cycles);
            opt(&mut j, "threshold", s.threshold);
            j.num_field("days", s.days as f64);
            j.num_field("bursts", s.bursts as f64);
            j.key("phase");
            arr(&mut j, &some(&s.phase));
            j.key("rate");
            arr(&mut j, &s.rate);
        }
        "storm-scale" => {
            let s = vleo_modules::record::storm_scale(&days, &cycles);
            j.key("levels").open_arr();
            for (k, (name, _)) in vleo_modules::record::STORM_LEVELS.iter().enumerate() {
                j.open_obj();
                j.str_field("name", name);
                j.num_field("ap", s.level_ap[k]);
                j.close_obj();
            }
            j.close_arr();
            let n = |v: &[u32]| v.iter().map(|&x| Some(x as f64)).collect::<Vec<_>>();
            j.key("cycles");
            arr(&mut j, &n(&s.cycles));
            j.key("days");
            arr(
                &mut j,
                &s.days.iter().map(|&x| Some(x as f64)).collect::<Vec<_>>(),
            );
            j.key("max_ap");
            arr(&mut j, &s.max_ap);
            j.key("per_year").open_arr();
            for (k, row) in s.per_year.iter().enumerate() {
                if k > 0 {
                    j.raw(",");
                }
                arr(&mut j, row);
            }
            j.close_arr();
            opt(&mut j, "evenness", s.evenness);
            j.key("busiest");
            arr(
                &mut j,
                &s.busiest
                    .iter()
                    .map(|b| b.map(|n| n as f64))
                    .collect::<Vec<_>>(),
            );
        }
        _ => {
            let k = vleo_modules::record::kp_ap(&days);
            j.key("kp");
            arr(&mut j, &some(&k.kp));
            j.key("median");
            arr(&mut j, &k.median);
            j.key("p10");
            arr(&mut j, &k.p10);
            j.key("p90");
            arr(&mut j, &k.p90);
            j.key("table");
            arr(&mut j, &k.table);
            j.num_field("above_median", k.above_median as f64);
            j.num_field("above_p90", k.above_p90 as f64);
            opt(&mut j, "kp7_table", k.kp7_table);
            opt(&mut j, "kp7_median", k.kp7_median);
        }
    }
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
        Err(e) => return failed(e.message()),
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
        Err(e) => (
            "404 Not Found",
            "text/plain; charset=utf-8",
            String::from(e).into_bytes(),
        ),
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
        Err(e) => failed(e.message()),
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
    let tree = match ctx.load() {
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
        Err(e) => return failed(e.message()),
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

/// Keep every result a `results.vleor` holds — the file somebody sent, as the
/// browser sends a file that is not text: base64, as `vleor=`. Each is kept once,
/// as a save is; the first is the one the page opens.
pub(super) fn result_upload_file(params: &str) -> String {
    let text = param(params, "vleor").map(decode).unwrap_or_default();
    let Some(bytes) = crate::results_file::unbase64(&text) else {
        return failed("the upload did not arrive whole — send the results file again");
    };
    match crate::results_file::import_bytes(&results_dir(), &bytes) {
        Ok(done) => {
            let mut j = Json::new();
            j.raw("{");
            j.bool_field("ok", true);
            j.str_field(
                "file",
                done.results.first().map(|(n, _)| n.as_str()).unwrap_or(""),
            );
            j.num_field("added", done.added() as f64);
            j.num_field("already", done.already() as f64);
            j.key("files").raw("[");
            for (i, (n, _)) in done.results.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                j.push_string(n);
            }
            j.raw("]");
            j.raw("}");
            j.0
        }
        Err(e) => failed(&e),
    }
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
        Err(e) => failed(e.message()),
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
        Err(e) => failed(e.message()),
    }
}

/// A result's inputs, made the saved case: checked like any other case, saved
/// only when every value still applies, and carried over first when the result
/// was saved against another set of inputs.
pub(super) fn result_as_case(params: &str) -> String {
    let name = param(params, "name").map(decode).unwrap_or_default();
    let s = match vleo_modules::results::store::open(&results_dir(), &name) {
        Ok(s) => s,
        Err(e) => return failed(e.message()),
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

/// One run of `node` on the saved case — what the page's engine values are —
/// and every value on its path, by id.
fn run_on_saved_case(ctx: &Ctx, node: &str) -> Result<BTreeMap<String, f64>, String> {
    let case = build_case(&format!("node={node}"), ctx);
    let mut scratch = Scratch::new();
    let run = vleo_modules::evaluate(&case, &mut scratch).map_err(|f| format!("{node}: {f}"))?;
    Ok(run
        .values
        .iter()
        .filter(|v| v.value.is_finite())
        .map(|v| (v.id.to_string(), v.value))
        .collect())
}

/// One relation at chosen inputs, named by the variables they bind — the
/// question `/v1/probe` answers. None where the relation refuses or an input
/// is missing: a point that did not answer is a point not drawn.
fn probe_at(node: &str, given: &[(&str, Option<f64>)]) -> Option<f64> {
    let k = Vleo::find(node)?;
    let def = &NODES[k as usize];
    let mut inputs = Vec::with_capacity(def.inputs.len());
    for &v in def.inputs {
        let id = VARS[v as usize].id;
        inputs.push(given.iter().find(|(g, _)| *g == id)?.1?);
    }
    vleo_modules::probe(k, &inputs)
        .ok()
        .map(|out| out[0])
        .filter(|y| y.is_finite())
}

/// A sweep of one relation over one input, the others held, point by point:
/// `points` evenly from `from` to `to`, a refused point recorded and not
/// drawn — the shape the page's own probe sweep had.
struct ProbeSweep {
    over: &'static str,
    node: &'static str,
    x: Vec<f64>,
    y: Vec<f64>,
    refused: Vec<f64>,
}

fn probe_sweep(
    node: &'static str,
    over: &'static str,
    (from, to, points): (f64, f64, usize),
    held: &[(&str, Option<f64>)],
) -> ProbeSweep {
    let mut s = ProbeSweep {
        over,
        node,
        x: Vec::new(),
        y: Vec::new(),
        refused: Vec::new(),
    };
    for i in 0..points {
        let f = if points == 1 {
            0.0
        } else {
            i as f64 / (points - 1) as f64
        };
        let x = from + (to - from) * f;
        let mut given: Vec<(&str, Option<f64>)> = held.to_vec();
        given.push((over, Some(x)));
        match probe_at(node, &given) {
            Some(y) => {
                s.x.push(x);
                s.y.push(y);
            }
            None => s.refused.push(x),
        }
    }
    s
}

fn write_sweep(j: &mut Json, k: &str, s: &ProbeSweep) {
    let var = |id: &str| Vleo::find(id).map(|i| VARS[i as usize].unit);
    j.key(k).raw("{");
    j.str_field("x_id", s.over);
    j.str_field("y_id", s.node);
    for (key, id) in [("x", s.over), ("y", s.node)] {
        let u = var(id);
        j.str_field(&format!("{key}_unit"), u.map(|u| u.symbol()).unwrap_or("-"));
        j.num_field(
            &format!("{key}_factor"),
            u.map(|u| u.si_factor()).unwrap_or(1.0),
        );
    }
    j.key("x");
    nums(j, &s.x);
    j.key("y");
    nums(j, &s.y);
    j.key("refused");
    nums(j, &s.refused);
    j.close_obj();
}

fn nums(j: &mut Json, v: &[f64]) {
    j.open_arr();
    for (i, x) in v.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw(&crate::json::num(*x));
    }
    j.close_arr();
}

fn opt_field(j: &mut Json, k: &str, v: Option<f64>) {
    match v {
        Some(v) => j.num_field(k, v),
        None => j.key(k).raw("null"),
    };
}

/// The thermosphere figure: the temperatures `env_exospheric_temperature`
/// gives under each driver, run and probed as the panel's four views ask, and
/// what the panel says about them (vleo_modules::thermo).
fn thermosphere(ctx: &Ctx, view: &str, j: &mut Json) -> Result<(), String> {
    use vleo_modules::record::DRIVER_SCENARIOS;
    use vleo_modules::thermo::{departure, end_slope, gap_extent, widest_apart};
    const T: &str = "env_exospheric_temperature";
    const CROSSING: &str = "l3_solar_interface";
    if !matches!(view, "solar" | "kp" | "slot" | "shape") {
        return Err(format!(
            "no view '{view}': solar (against the flux), kp (one curve per scenario), \
             slot (the two Kp readings) or shape (where the geomagnetic term bends)"
        ));
    }
    // What the design currently says, each row run on its own as the page's
    // engine values are.
    let own = |node: &str| run_on_saved_case(ctx, node).map(|v| v.get(node).copied());
    let (f0, fa0, kp0) = (own("env_f107")?, own("env_f107a")?, own("env_kp")?);
    let now = own(T)?;
    let crossing = run_on_saved_case(ctx, CROSSING)?;
    let si = crossing.get(CROSSING).copied();
    j.str_field("view", view);
    opt_field(j, "env_f107", f0);
    opt_field(j, "env_f107a", fa0);
    opt_field(j, "env_kp", kp0);
    opt_field(j, "now", now);
    opt_field(j, "crossing", si);
    // One scenario's four drivers out of the crossing's run.
    let drivers = |sc: &str| {
        let at = |q: &str| crossing.get(&format!("{CROSSING}.{q}_{sc}")).copied();
        (
            if sc == "hotmean" { si } else { at("f107") },
            at("f107bar"),
            at("kp_mean"),
            at("kp_peak"),
        )
    };
    let ids = ("env_f107", "env_f107a", "env_kp");
    match view {
        "solar" => {
            if f0.is_none() || fa0.is_none() || kp0.is_none() {
                return Err("env_f107, env_f107a or env_kp did not answer".into());
            }
            // A sustained rise: the day and its 81-day mean moved together.
            let (mut sx, mut sy) = (Vec::new(), Vec::new());
            for i in 0..24 {
                let x = 60.0 + (400.0 - 60.0) * i as f64 / 23.0;
                if let Some(y) = probe_at(T, &[(ids.0, Some(x)), (ids.1, Some(x)), (ids.2, kp0)]) {
                    sx.push(x);
                    sy.push(y);
                }
            }
            let fast = probe_sweep(
                T,
                "env_f107",
                (60.0, 400.0, 60),
                &[(ids.1, fa0), (ids.2, kp0)],
            );
            let slow = probe_sweep(
                T,
                "env_f107a",
                (60.0, 400.0, 60),
                &[(ids.0, f0), (ids.2, kp0)],
            );
            j.key("sustained").raw("{");
            j.key("x");
            nums(j, &sx);
            j.key("y");
            nums(j, &sy);
            j.close_obj();
            write_sweep(j, "day", &fast);
            write_sweep(j, "mean", &slow);
            let slope = |s: &ProbeSweep| end_slope(&s.x, &s.y);
            opt_field(j, "slope_sustained", end_slope(&sx, &sy));
            opt_field(j, "slope_day", slope(&fast));
            opt_field(j, "slope_mean", slope(&slow));
            // Whether the declared flux is what the crossing publishes, to the
            // twentieth of an sfu the page shows it to.
            j.key("agrees").raw(&match (si, f0) {
                (Some(a), Some(b)) => (vleo_units::pmath::abs(a - b) < 0.05).to_string(),
                _ => "null".into(),
            });
        }
        "kp" => {
            j.key("curves").open_arr();
            let mut lines: Vec<Vec<Option<f64>>> = Vec::new();
            let mut worst: Option<(usize, f64, f64, f64)> = None;
            let mut kps = Vec::new();
            for (n, sc) in DRIVER_SCENARIOS.iter().enumerate() {
                let (f, fa, km, kpk) = drivers(sc);
                kps.push((km, kpk));
                if n > 0 {
                    j.raw(",");
                }
                j.raw("{");
                j.str_field("scenario", sc);
                opt_field(j, "kp_mean", km);
                opt_field(j, "kp_peak", kpk);
                if f.is_none() || fa.is_none() {
                    j.key("sweep").raw("null");
                    j.key("t_mean").raw("null");
                    j.key("t_peak").raw("null");
                    j.close_obj();
                    continue;
                }
                let s = probe_sweep(T, "env_kp", (0.0, 9.0, 46), &[(ids.0, f), (ids.1, fa)]);
                let tm = probe_at(T, &[(ids.0, f), (ids.1, fa), (ids.2, km)]);
                let tp = probe_at(T, &[(ids.0, f), (ids.1, fa), (ids.2, kpk)]);
                write_sweep(j, "sweep", &s);
                opt_field(j, "t_mean", tm);
                opt_field(j, "t_peak", tp);
                j.close_obj();
                lines.push(s.y.iter().map(|y| Some(*y)).collect());
                if let (Some(tm), Some(tp)) = (tm, tp) {
                    if worst.is_none_or(|w| tp - tm > w.1) {
                        worst = Some((n, tp - tm, tm, tp));
                    }
                }
            }
            j.close_arr();
            match worst {
                Some((n, gap, tm, tp)) => {
                    j.key("worst").raw("{");
                    j.str_field("scenario", DRIVER_SCENARIOS[n]);
                    j.num_field("gap", gap);
                    j.num_field("share_pct", 100.0 * gap / tm);
                    j.num_field("t_mean", tm);
                    j.num_field("t_peak", tp);
                    opt_field(j, "kp_mean", kps[n].0);
                    opt_field(j, "kp_peak", kps[n].1);
                    j.close_obj();
                }
                None => {
                    j.key("worst").raw("null");
                }
            }
            // How close the two lightest curves ever come apart.
            opt_field(
                j,
                "close",
                (lines.len() > 1).then(|| widest_apart(&lines[0], &lines[1])),
            );
        }
        "slot" => {
            let mut gaps = Vec::new();
            j.key("points").open_arr();
            for (n, sc) in DRIVER_SCENARIOS.iter().enumerate() {
                let (f, fa, km, kpk) = drivers(sc);
                let (tm, tp) = if f.is_none() || km.is_none() || kpk.is_none() {
                    (None, None)
                } else {
                    (
                        probe_at(T, &[(ids.0, f), (ids.1, fa), (ids.2, km)]),
                        probe_at(T, &[(ids.0, f), (ids.1, fa), (ids.2, kpk)]),
                    )
                };
                let gap = tm.zip(tp).map(|(m, p)| p - m);
                gaps.push(gap);
                if n > 0 {
                    j.raw(",");
                }
                j.raw("{");
                j.str_field("scenario", sc);
                opt_field(j, "t_mean", tm);
                opt_field(j, "t_peak", tp);
                opt_field(j, "gap", gap);
                j.close_obj();
            }
            j.close_arr();
            let (at, range) = gap_extent(&gaps);
            j.key("widest").raw(&at.map_or_else(
                || "null".to_string(),
                |i| crate::json::string(DRIVER_SCENARIOS[i]),
            ));
            opt_field(j, "narrowest_gap", range.map(|r| r.0));
            opt_field(j, "widest_gap", range.map(|r| r.1));
        }
        _ => {
            if f0.is_none() || fa0.is_none() {
                return Err("env_f107 or env_f107a did not answer".into());
            }
            let s = probe_sweep(T, "env_kp", (0.0, 9.0, 46), &[(ids.0, f0), (ids.1, fa0)]);
            let dep = departure(&s.x, &s.y, 2.0).ok_or("the Kp sweep returned nothing to draw")?;
            write_sweep(j, "sweep", &s);
            j.key("d");
            nums(j, &dep.d);
            j.key("line");
            nums(j, &dep.line);
            j.num_field("excess", dep.excess);
            opt_field(j, "split_at", dep.split_at);
        }
    }
    Ok(())
}

/// One row swept across the whole declared range of another, on the case the
/// request carries — the saved case and the reader's overrides, as the page's
/// own sweeps are.
fn case_sweep(
    ctx: &Ctx,
    params: &str,
    node: &str,
    over: &str,
    points: usize,
) -> Result<vleo_modules::results::Sweep, String> {
    let d = Vleo::find(over)
        .map(|i| &VARS[i as usize])
        .ok_or_else(|| format!("no row called {over}"))?;
    vleo_modules::results::sweep(
        &build_case(params, ctx),
        node,
        over,
        d.limit.lower,
        d.limit.upper,
        points,
    )
    .map_err(String::from)
}

/// What a row answers on its own run of the saved case, or why it did not —
/// without the request's overrides, as the page's engine values are: what the
/// tree says as it stands, where the sweeps show what moving a decision does.
fn own_answer(ctx: &Ctx, node: &str) -> Result<f64, String> {
    let why = |e: String| format!("{node} did not answer: {e}");
    run_on_saved_case(ctx, node)
        .map_err(why)?
        .get(node)
        .copied()
        .ok_or_else(|| why("the run did not return this row".into()))
}

/// The design figure: the return curve against the G-level bound and an Ap
/// requirement, with the record's own days above that bound; or the F10.7
/// window's four edges and the analogue's peak against an F10.7 requirement
/// (vleo_modules::design).
fn design(
    ctx: &Ctx,
    params: &str,
    days: &[vleo_data::SolarDay],
    j: &mut Json,
) -> Result<(), String> {
    use vleo_modules::design::{band_extent, exceedance, first_above, level_crossing};
    const DURATION: &str = "sys_mission_requirements_mission_duration";
    // A Julian year, the axis the panel is read in.
    const YEAR: f64 = 31557600.0;
    let pick = |k: &str, dflt: &str| param(params, k).map(decode).unwrap_or(dflt.into());
    let years = |w: &vleo_modules::results::Sweep| w.x.iter().map(|v| v / YEAR).collect::<Vec<_>>();
    let view = pick("v", "ap");
    j.str_field("view", &view);
    match view.as_str() {
        "ap" => {
            let g = pick("g", "3");
            let req_id = pick("req", "l3_solar_req_03");
            if !matches!(g.as_str(), "1" | "2" | "3") {
                return Err(format!("no G level '{g}': 1, 2 or 3"));
            }
            if !matches!(
                req_id.as_str(),
                "l3_solar_req_03" | "l3_solar_req_04" | "l3_solar_req_05"
            ) {
                return Err(format!(
                    "no Ap requirement '{req_id}': l3_solar_req_03, _04 or _05"
                ));
            }
            let ret = case_sweep(ctx, params, "sw_storm_return_level", DURATION, 120)?;
            let gmap = case_sweep(ctx, params, "sw_ap_design", "sw_storm_design_level", 3)?;
            let level: f64 = g.parse().unwrap_or(f64::NAN);
            // sw_ap_design's own G-to-Ap conversion, read off its sweep over
            // the G level rather than from a table written out here.
            let bound = gmap
                .x
                .iter()
                .position(|x| *x == level)
                .and_then(|i| gmap.y.get(i).copied())
                .ok_or_else(|| format!("sw_ap_design did not answer at G{g}"))?;
            let req = own_answer(ctx, &req_id)?;
            let xs = years(&ret);
            j.str_field("g", &g);
            j.str_field("req_id", &req_id);
            j.key("years");
            nums(j, &xs);
            j.key("level");
            nums(j, &ret.y);
            j.num_field("points_refused", ret.refused.len() as f64);
            j.num_field("bound", bound);
            j.num_field("req", req);
            opt_field(j, "hits_at", level_crossing(&xs, &ret.y, bound));
            opt_field(j, "req_at", level_crossing(&xs, &ret.y, req));
            // What the record did above the bound, counted from the record
            // itself, so it answers for a level the rows are not set to.
            let ap: Vec<(i32, Option<f64>)> = days.iter().map(|d| (d.day, d.ap)).collect();
            let e = exceedance(&ap, bound);
            j.key("record").raw("{");
            j.num_field("days", e.days as f64);
            j.num_field("years", e.years);
            j.num_field("above", e.above as f64);
            j.num_field("runs", e.runs as f64);
            j.num_field("rate", e.rate);
            j.close_obj();
        }
        "f107" => {
            let req_id = pick("reqf", "l3_solar_req_01");
            if !matches!(req_id.as_str(), "l3_solar_req_01" | "l3_solar_req_02") {
                return Err(format!(
                    "no F10.7 requirement '{req_id}': l3_solar_req_01 or _02"
                ));
            }
            let req = own_answer(ctx, &req_id)?;
            let sw = |node: &str| case_sweep(ctx, params, node, DURATION, 60);
            let (fl, fs) = (sw("sw_f107_design_long")?, sw("sw_f107_design_short")?);
            let (cl, cs) = (sw("sw_f107_cold_long")?, sw("sw_f107_cold_short")?);
            let pk = sw("sw_window_peak_level")?;
            let xs = years(&fl);
            let refused: usize = [&fl, &fs, &cl, &cs, &pk]
                .iter()
                .map(|w| w.refused.len())
                .sum();
            j.num_field("points_refused", refused as f64);
            j.str_field("reqf", &req_id);
            j.num_field("req", req);
            j.key("years");
            nums(j, &xs);
            for (k, w) in [
                ("hot", &fs),
                ("hot_long", &fl),
                ("cold_long", &cl),
                ("cold", &cs),
                ("analogue", &pk),
            ] {
                j.key(k);
                nums(j, &w.y);
            }
            // The worst single day any declared window reaches, and what is
            // left of the requirement above it.
            let peak =
                (!fs.y.is_empty()).then(|| fs.y.iter().copied().fold(f64::NEG_INFINITY, f64::max));
            opt_field(j, "peak", peak);
            opt_field(j, "margin", peak.map(|p| req - p));
            // How wide the filled window is along its length — its own two
            // edges, so the sentence and the area are one measurement.
            let band = band_extent(&fs.y, &cs.y, xs.len());
            opt_field(j, "narrowest", band.map(|b| b.0));
            opt_field(j, "widest", band.map(|b| b.1));
            // Where the analogue's own maximum first climbs above the hot
            // single-day design value.
            opt_field(j, "over", first_above(&xs, &pk.y, &fs.y));
        }
        other => {
            return Err(format!(
                "no driver '{other}' in the design figure: ap or f107"
            ))
        }
    }
    Ok(())
}

/// The closure figure: one requirement and its achieved quantity swept over
/// the decision that spends the margin fastest, the margin itself over the
/// same decision, and where each runs out (vleo_modules::design).
fn closure(ctx: &Ctx, params: &str, j: &mut Json) -> Result<(), String> {
    use vleo_modules::design::{same_crossing, zero_crossing, CLOSURE_PAIRS};
    let pair = param(params, "pair").map(decode).unwrap_or("01".into());
    let Some(&(_, quantity)) = CLOSURE_PAIRS.iter().find(|(n, _)| *n == pair) else {
        return Err(format!("no closure '{pair}': 01 to 05"));
    };
    let (ach, req) = (
        format!("l3_solar_ach_{pair}"),
        format!("l3_solar_req_{pair}"),
    );
    j.str_field("pair", &pair);
    j.str_field("ach", &ach);
    j.str_field("req_id", &req);
    j.str_field("quantity", quantity);
    // THE AXIS IS THE ENGINE'S ANSWER, NOT THE AUTHOR'S. Levers come ordered
    // by how much they move this row, and the requirement is always near the
    // top because a margin is a fraction OF it — moving the bound moves the
    // margin by construction and says nothing about the sky. So the bound is
    // skipped and the next decision that moves it at all taken.
    let ni = Vleo::find(&ach).ok_or_else(|| format!("no row called {ach}"))?;
    let (_, levers) = crate::levers_of(params, ctx, &ach, ni);
    let Some(lv) = levers
        .iter()
        .map(|l| (l, &VARS[l.var as usize]))
        .find(|(l, d)| d.id != req && l.span > 0.0)
    else {
        j.key("lever").raw("null");
        return Ok(());
    };
    let (lev, d) = lv;
    j.key("lever").raw("{");
    j.str_field("id", d.id);
    j.str_field("label", d.label);
    j.str_field("unit", d.unit.symbol());
    j.num_field("lower", d.limit.lower);
    j.num_field("upper", d.limit.upper);
    j.num_field("span", lev.span);
    j.close_obj();
    let mar = case_sweep(ctx, params, &ach, d.id, 90)?;
    let qty = case_sweep(ctx, params, quantity, d.id, 90)?;
    // Divided back by the factors the sweeps report, the boundary rule every
    // panel here obeys.
    let x: Vec<f64> = mar.x.iter().map(|v| v / mar.x_factor).collect();
    let margin: Vec<f64> = mar.y.iter().map(|v| v / mar.y_factor).collect();
    let value: Vec<f64> = qty.y.iter().map(|v| v / qty.y_factor).collect();
    let req_value = own_answer(ctx, &req).ok().map(|r| {
        r / if qty.y_factor == 0.0 {
            1.0
        } else {
            qty.y_factor
        }
    });
    let now = own_answer(ctx, &ach).ok();
    // Where the margin goes through zero, and — from a different array — where
    // the achieved curve meets the requirement line. panels/closure.toml says
    // they are the same x; the figure measures it rather than assuming it.
    let cross = zero_crossing(&x, &margin);
    let cross_q = req_value.and_then(|r| {
        let over: Vec<f64> = value.iter().map(|v| v - r).collect();
        zero_crossing(&x, &over)
    });
    j.key("x");
    nums(j, &x);
    j.key("margin");
    nums(j, &margin);
    j.key("value");
    nums(j, &value);
    j.num_field(
        "points_refused",
        (mar.refused.len() + qty.refused.len()) as f64,
    );
    opt_field(j, "req", req_value);
    opt_field(j, "now", now);
    opt_field(j, "cross", cross);
    opt_field(j, "cross_q", cross_q);
    j.bool_field("agree", same_crossing(cross, cross_q, &x));
    Ok(())
}
