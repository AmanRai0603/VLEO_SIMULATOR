//! A figure description holds together, or it is refused with why.
//!
//! What must hold: one sample of every kind passes the check; a sweep's figure
//! keeps every refused point as a gap where it fell and calls out the case; and
//! each way a description can be malformed — lengths that disagree, a value
//! that is not a number, an axis with no factor to SI, a kind missing what it
//! needs — is refused, naming the figure.

#![cfg(feature = "std")]

use vleo_modules::figure::{check, from_sweep, json, samples, Kind};
use vleo_modules::results::Sweep;

#[test]
fn every_kind_has_a_sample_that_holds_together() {
    let all = samples();
    assert_eq!(all.len(), Kind::ALL.len());
    for k in Kind::ALL {
        let f = all
            .iter()
            .find(|f| f.kind == k)
            .expect("a kind with no sample");
        assert_eq!(check(f), Ok(()), "the {} sample", k.name());
        assert!(json(f).contains(&format!("\"kind\":\"{}\"", k.name())));
    }
}

#[test]
fn a_sweep_draws_its_refused_points_as_gaps_where_they_fell() {
    let w = Sweep {
        over: "alt".into(),
        over_name: "Altitude".into(),
        x_unit: "km".into(),
        x_factor: 1000.0,
        y_unit: "m/s".into(),
        y_factor: 1.0,
        from: 200e3,
        to: 400e3,
        points: 3,
        x: vec![200e3, 400e3],
        y: vec![5.0, 7.0],
        refused: vec![(300e3, "no density here".into())],
    };
    let f = from_sweep("dv", &w, Some((250e3, 6.0)));
    assert_eq!(check(&f), Ok(()));
    let s = &f.series[0];
    assert_eq!(
        s.x,
        vec![200.0, 300.0, 400.0],
        "not in display units, or not in order"
    );
    assert_eq!(
        s.y,
        vec![Some(5.0), None, Some(7.0)],
        "a refused point was not a gap where it fell"
    );
    assert_eq!(f.gaps.len(), 1);
    assert_eq!(f.gaps[0].why, "no density here");
    assert_eq!((f.notes[0].x, f.notes[0].y), (250.0, 6.0));
    assert_eq!(f.x.unit, "km");
    assert_eq!(f.x.label, "Altitude");
    assert!(json(&f).contains("\"y\":[5.0,null,7.0]"));
}

#[test]
fn a_description_that_does_not_hold_together_is_refused() {
    let base = || {
        samples()
            .into_iter()
            .find(|f| f.kind == Kind::Line)
            .unwrap()
    };
    let heat = || {
        samples()
            .into_iter()
            .find(|f| f.kind == Kind::Heatmap)
            .unwrap()
    };
    let anim = || {
        samples()
            .into_iter()
            .find(|f| f.kind == Kind::Animation)
            .unwrap()
    };
    let scene = || {
        samples()
            .into_iter()
            .find(|f| f.kind == Kind::Scene3d)
            .unwrap()
    };
    let cases: Vec<(vleo_modules::figure::Figure, &str)> = vec![
        (
            {
                let mut f = base();
                f.series[0].y.pop();
                f
            },
            "has 5 x and 4 y",
        ),
        (
            {
                let mut f = base();
                f.series[0].y[0] = Some(f64::NAN);
                f
            },
            "not a number",
        ),
        (
            {
                let mut f = base();
                f.x.factor = 0.0;
                f
            },
            "no factor to SI",
        ),
        (
            {
                let mut f = base();
                f.series.clear();
                f
            },
            "needs a series",
        ),
        (
            {
                let mut f = heat();
                f.grid.as_mut().unwrap().z.pop();
                f
            },
            "one value per grid point",
        ),
        (
            {
                let mut f = heat();
                f.z = None;
                f
            },
            "needs a value axis",
        ),
        (
            {
                let mut f = anim();
                f.frames.swap(0, 1);
                f
            },
            "frames in time order",
        ),
        (
            {
                let mut f = anim();
                f.frames.clear();
                f
            },
            "needs frames",
        ),
        (
            {
                let mut f = scene();
                f.scene[1].points.truncate(1);
                f
            },
            "a point or a path",
        ),
        (
            {
                let mut f = scene();
                f.scene[0].points[0][2] = f64::INFINITY;
                f
            },
            "numbers for every coordinate",
        ),
    ];
    for (f, says) in cases {
        let e = check(&f).expect_err(says);
        assert_eq!(e.kind(), vleo_modules::ErrorKind::Malformed, "{e}");
        assert!(
            e.message().contains(says) && e.message().starts_with(&f.id),
            "refused as «{e}», expected «{says}»"
        );
    }
}
