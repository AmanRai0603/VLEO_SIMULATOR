//! Property tests for `prop_capture_efficiency`.
//!
//! The sheet (`nodes/prop_capture_efficiency/node.toml`, `[question].note`)
//! states: "the collection efficiency does not depend on the flight speed at
//! all, only on geometry. Speed buys compression, not capture." That is a
//! claim about the *shape* of `vleo_core::physics::prop::intake_balance`, not
//! about any one number, so it is tested here as a property over the
//! declared domain of `orbit_velocity`
//! (`crates/vleo-mod-envorbit/nodes/orbit_velocity/node.toml`: 7000-8200 m/s,
//! `reason_lower`/`reason_upper`), rather than by pinning a single fixture.
//!
//! No expected value is invented anywhere in this file: every assertion is
//! either a comparison between two calls to the same relation (independence,
//! monotonicity) or a check against the node's own declared bounds and fault
//! names, which come from `node.toml`, not from this test.
//!
//! The one concrete number used as a starting point — the "IRS-class
//! baseline intake" geometry — is the fixture already accepted in
//! `nodes/prop_capture_efficiency/fixtures.toml` with
//! `provenance = "independent-derivation"`; it is reused here only to fix a
//! plausible geometry, not as an expected output.

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::prop;
use vleo_core::units::*;
use vleo_mod_prop::nodes::prop_capture_efficiency::model;

/// Geometry and chamber conditions held fixed while velocity moves — the
/// "IRS-class baseline intake" fixture, minus the velocity term.
struct Geometry {
    a_in: Area,
    a_out: Area,
    eta_geo: Ratio,
    beta: Ratio,
    t_c: Temperature,
    m: MolarMass,
    n: NumberDensity,
}

fn baseline_geometry() -> Geometry {
    Geometry {
        a_in: Area::new(0.2),
        a_out: Area::new(0.01),
        eta_geo: Ratio::new(0.9),
        beta: Ratio::new(0.06),
        t_c: Temperature::new(600.0),
        m: MolarMass::new(0.018_72),
        n: NumberDensity::new(2.133e15),
    }
}

/// The property the sheet states: collection efficiency does not move with
/// flight speed, over the whole declared domain of `orbit_velocity` and
/// beyond it, while the *other* half of the same balance — compression
/// ratio — does move, strictly increasing with speed. Testing both halves
/// together is what makes this a test of the sentence on the sheet rather
/// than of a relation that happens to ignore all of its arguments.
#[test]
fn collection_efficiency_is_independent_of_velocity_while_compression_ratio_is_not() {
    let g = baseline_geometry();

    // orbit_velocity's declared domain is 7000-8200 m/s (node.toml). Walk it,
    // both bounds included, and step outside it in both directions, because
    // the sheet's claim is about the relation, not about a range it happens
    // to be evaluated in.
    let velocities = [
        1.0, 100.0, 7000.0, 7250.0, 7500.0, 7754.6, 7800.0, 8200.0, 20_000.0, 1.0e6,
    ];

    let first = prop::intake_balance(
        g.n,
        Velocity::new(velocities[0]),
        g.a_in,
        g.a_out,
        g.eta_geo,
        g.beta,
        g.t_c,
        g.m,
    );
    let eta_at_first_v = first.collection_efficiency.get();
    let mut previous_cr = first.compression_ratio.get();

    for &v in &velocities[1..] {
        let r = prop::intake_balance(
            g.n,
            Velocity::new(v),
            g.a_in,
            g.a_out,
            g.eta_geo,
            g.beta,
            g.t_c,
            g.m,
        );

        // The property under test: not merely "close", but the identical
        // bit pattern, because the formula the sheet gives for eta_c has no
        // v term in it at all.
        assert_eq!(
            r.collection_efficiency.get().to_bits(),
            eta_at_first_v.to_bits(),
            "collection efficiency moved with velocity: v={} gave {}, v={} gave {}",
            v,
            r.collection_efficiency.get(),
            velocities[0],
            eta_at_first_v,
        );

        // Sanity check that the test is not vacuous: the other output of the
        // very same balance call does move with velocity, strictly
        // increasing, which is what "speed buys compression, not capture"
        // asserts on its other half.
        assert!(
            r.compression_ratio.get() > previous_cr,
            "compression ratio did not increase with velocity: v={} gave CR={}, previous CR={}",
            v,
            r.compression_ratio.get(),
            previous_cr,
        );
        previous_cr = r.compression_ratio.get();
    }
}

/// Geometry with the back-flow term zeroed. With `beta = 0` the throat area
/// cancels algebraically — `eta_c = a_out*eta_geo/(a_out + 0*a_in) =
/// eta_geo` — which lets the edges of `eta_geo` probe the edges of the
/// node's declared output domain directly, without relying on any
/// intermediate value nobody derived.
fn zero_backflow_geometry(eta_geo: f64) -> Geometry {
    Geometry {
        a_in: Area::new(0.2),
        a_out: Area::new(0.01),
        eta_geo: Ratio::new(eta_geo),
        beta: Ratio::new(0.0),
        t_c: Temperature::new(600.0),
        m: MolarMass::new(0.018_72),
        n: NumberDensity::new(2.133e15),
    }
}

fn evaluate(g: Geometry) -> Result<Ratio, Fault> {
    model::evaluate(
        g.a_in,
        g.a_out,
        g.eta_geo,
        g.beta,
        g.t_c,
        g.m,
        g.n,
        Velocity::new(7754.6),
    )
}

#[test]
fn lower_bound_of_the_declared_output_domain_refuses_by_name() {
    // Just outside: the guard names the field, the edge and the reason —
    // all declared in node.toml, none invented here.
    match evaluate(zero_backflow_geometry(-1.0e-9)) {
        Err(Fault::OutOfDomain {
            node,
            field,
            edge,
            bound,
            reason,
            ..
        }) => {
            assert_eq!(node, "prop_capture_efficiency");
            assert_eq!(field, "eta_c");
            assert_eq!(edge, Edge::Lower);
            assert_eq!(bound, 0.0);
            assert_eq!(reason, "a collection efficiency cannot be negative");
        }
        other => panic!("expected a named Fault::OutOfDomain at the lower edge, got {other:?}"),
    }

    // The bound itself is closed and admitted.
    let at_bound = evaluate(zero_backflow_geometry(0.0))
        .expect("eta_c = 0.0 is inside the closed lower bound");
    assert_eq!(at_bound.get(), 0.0);

    // Just inside is admitted too.
    let just_inside = evaluate(zero_backflow_geometry(1.0e-9))
        .expect("eta_c just above 0 is inside the declared domain");
    assert!(just_inside.get() > 0.0);
}

#[test]
fn upper_bound_of_the_declared_output_domain_refuses_by_name() {
    // Just outside: the guard names the field, the edge and the reason —
    // all declared in node.toml, none invented here.
    match evaluate(zero_backflow_geometry(1.0 + 1.0e-9)) {
        Err(Fault::OutOfDomain {
            node,
            field,
            edge,
            bound,
            reason,
            ..
        }) => {
            assert_eq!(node, "prop_capture_efficiency");
            assert_eq!(field, "eta_c");
            assert_eq!(edge, Edge::Upper);
            assert_eq!(bound, 1.0);
            assert_eq!(
                reason,
                "an intake cannot deliver more than enters its mouth"
            );
        }
        other => panic!("expected a named Fault::OutOfDomain at the upper edge, got {other:?}"),
    }

    // The bound itself is closed and admitted.
    let at_bound = evaluate(zero_backflow_geometry(1.0))
        .expect("eta_c = 1.0 is inside the closed upper bound");
    assert_eq!(at_bound.get(), 1.0);

    // Just inside is admitted too.
    let just_inside = evaluate(zero_backflow_geometry(1.0 - 1.0e-9))
        .expect("eta_c just below 1.0 is inside the declared domain");
    assert!(just_inside.get() < 1.0);
}
