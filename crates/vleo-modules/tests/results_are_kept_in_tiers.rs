//! Results are kept in tiers: all values while recent or pinned, the summary for
//! good — and found through an index that is never the only copy of anything.
//!
//! What must hold: the index answers `find` and is rebuilt the moment the
//! folder holds something it does not (another laptop's result, in a shared
//! folder); a pin keeps a result whole; thinning keeps every input, the answer
//! and what could not run, drops the rest, says so, and never touches a
//! pinned or a recent result; a thinned result is not an answer to reuse.

#![cfg(feature = "std")]

mod design;

use vleo_bus::{Case, RunMode};
use vleo_modules::inputs::case_inputs;
use vleo_modules::results::{from_run, question_for, read, store, thin, Saved};
use vleo_modules::{evaluate, Scratch};

fn saved_at(when: &str, v: Option<f64>) -> (Saved, Case) {
    let i = case_inputs()
        .into_iter()
        .find(|i| i.default > i.lo && i.default < i.hi)
        .expect("no input with room to move");
    let case = Case {
        target: "kpi_mass_margin".into(),
        mode: RunMode::Branch,
        supply: v.map(|v| vec![(i.id.to_string(), v)]).unwrap_or_default(),
        ..Default::default()
    };
    let r = evaluate(&case, &mut Scratch::new()).expect("the run did not run");
    (from_run(&r, &case.supply, when, ""), case)
}

fn dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("vleo-tiers-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

const DAY: i64 = 86_400;
/// 2026-09-29T00:00:00Z.
const NOW: i64 = 1_790_640_000;

#[test]
fn thinning_keeps_the_inputs_the_answer_and_what_could_not_run() {
    crate::design::open();
    let (s, _) = saved_at("2026-08-01T10:00:00Z", None);
    assert!(
        s.outputs.len() > 5,
        "a run with too few values to test thinning"
    );
    let t = thin(&s, "2026-09-29");
    assert_eq!(t.inputs, s.inputs, "thinning lost an input");
    assert_eq!(t.blocked, s.blocked, "thinning lost what could not run");
    assert!(t.outputs.iter().all(|o| o.id == s.target));
    assert_eq!(
        t.outputs.len(),
        s.outputs.iter().filter(|o| o.id == s.target).count()
    );
    assert_eq!(
        t.thinned,
        format!("2026-09-29 {}", s.outputs.len() - t.outputs.len())
    );
    assert_eq!(
        t.question(),
        s.question(),
        "a thinned result answers another question"
    );
    let back = read(&vleo_modules::results::csv(&t)).unwrap();
    assert_eq!(
        back, t,
        "a thinned result does not read back as it was written"
    );
    assert_eq!(thin(&t, "2026-12-01"), t, "thinning twice changed it again");
}

#[test]
fn old_unpinned_results_are_thinned_and_the_rest_left_whole() {
    crate::design::open();
    let d = dir("thin");
    let (old, _) = saved_at("2026-08-01T10:00:00Z", None);
    let (pinned, _) = saved_at("2026-08-01T10:00:01Z", Some(case_inputs()[0].hi));
    let (recent, _) = saved_at("2026-09-20T10:00:00Z", Some(case_inputs()[0].lo));
    let (a, _) = store::save(&d, &old).unwrap();
    let (b, _) = store::save(&d, &pinned).unwrap();
    let (c, _) = store::save(&d, &recent).unwrap();
    store::pin(&d, &b, true).unwrap();
    assert!(store::is_pinned(&d, &b));

    let done = store::thin_old(&d, NOW, 30).unwrap();
    assert_eq!(done, vec![a.clone()], "the wrong results were thinned");
    assert!(!store::open(&d, &a).unwrap().thinned.is_empty());
    assert_eq!(
        store::open(&d, &b).unwrap(),
        pinned,
        "a pinned result was thinned"
    );
    assert_eq!(
        store::open(&d, &c).unwrap(),
        recent,
        "a recent result was thinned"
    );
    let report = std::fs::read_to_string(d.join(&a).join(store::REPORT)).unwrap();
    assert!(
        report.contains("#! thinned"),
        "the report still carries every value"
    );
    assert!(
        report.contains("Thinned to its summary on 2026-09-29"),
        "the report does not say it was thinned"
    );
    assert!(
        store::thin_old(&d, NOW, 30).unwrap().is_empty(),
        "a result was thinned twice"
    );

    // Unpinned, the pinned one is old enough to thin too.
    store::pin(&d, &b, false).unwrap();
    assert_eq!(store::thin_old(&d, NOW, 30).unwrap(), vec![b.clone()]);
    // A keep period long enough holds everything whole.
    let d2 = dir("keep");
    store::save(&d2, &old).unwrap();
    assert!(store::thin_old(&d2, NOW, 365).unwrap().is_empty());
    let _ = std::fs::remove_dir_all(&d);
    let _ = std::fs::remove_dir_all(&d2);
}

#[test]
fn a_thinned_result_is_not_an_answer_to_reuse() {
    crate::design::open();
    let d = dir("reuse");
    let (old, case) = saved_at("2026-08-01T10:00:00Z", None);
    let q = question_for(&case, None);
    let (a, _) = store::save(&d, &old).unwrap();
    assert_eq!(store::find(&d, &q).map(|(n, _)| n), Some(a.clone()));
    store::thin_old(&d, NOW, 30).unwrap();
    assert!(
        store::find(&d, &q).is_none(),
        "a thinned result was offered as the answer"
    );
    // Asked again, the question is kept whole beside the summary.
    let (again, _) = saved_at("2026-09-29T09:00:00Z", None);
    let (b, already) = store::save(&d, &again).unwrap();
    assert!(!already && b != a);
    assert_eq!(store::find(&d, &q).map(|(n, _)| n), Some(b));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn the_index_answers_and_is_rebuilt_when_the_folder_moves_on() {
    crate::design::open();
    let d = dir("index");
    let (one, c1) = saved_at("2026-09-01T10:00:00Z", None);
    let (a, _) = store::save(&d, &one).unwrap();
    let q1 = question_for(&c1, None);
    assert_eq!(store::find(&d, &q1).map(|(n, _)| n), Some(a.clone()));
    let idx = std::fs::read_to_string(d.join(store::INDEX)).expect("no index was written");
    assert!(idx.contains(&a) && idx.contains(&q1));

    // Another laptop's result lands in the shared folder, written by hand.
    let (two, c2) = saved_at("2026-09-02T10:00:00Z", Some(case_inputs()[0].hi));
    let other = d.join("2026-09-02T10-00-00Z_from_elsewhere");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(other.join(store::RESULT), vleo_modules::results::csv(&two)).unwrap();
    let q2 = question_for(&c2, None);
    assert_eq!(
        store::find(&d, &q2).map(|(n, _)| n).as_deref(),
        Some("2026-09-02T10-00-00Z_from_elsewhere"),
        "a result another laptop added was not found"
    );
    // An index that says something false is not believed: one naming a result
    // that is not there is rebuilt from the folders.
    std::fs::write(d.join(store::INDEX), format!("gone\t{q1}\tx\tx\t0\t0\n")).unwrap();
    assert_eq!(store::find(&d, &q1).map(|(n, _)| n), Some(a.clone()));
    // Deleted, it is written again.
    std::fs::remove_file(d.join(store::INDEX)).unwrap();
    assert_eq!(store::index(&d).len(), 2);
    assert!(d.join(store::INDEX).is_file());
    // The index is not a result, and a pin marks the entry.
    assert_eq!(
        store::list(&d).0.len(),
        2,
        "the index was listed as a result"
    );
    store::pin(&d, &a, true).unwrap();
    assert!(
        store::index(&d).iter().any(|e| e.name == a && e.pinned),
        "a pin did not reach the index"
    );
    assert!(store::pin(&d, "../x", true).is_err());
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_result_exactly_the_keep_period_old_is_kept_whole() {
    crate::design::open();
    let d = dir("edge");
    let at = |t: i64| vleo_units::calendar::Civil::from_unix(t).to_string();
    let (edge, _) = saved_at(&at(NOW - 30 * DAY), None);
    let (past, _) = saved_at(&at(NOW - 30 * DAY - 1), Some(case_inputs()[0].hi));
    store::save(&d, &edge).unwrap();
    let (b, _) = store::save(&d, &past).unwrap();
    assert_eq!(store::thin_old(&d, NOW, 30).unwrap(), vec![b]);
    let _ = std::fs::remove_dir_all(&d);
}
