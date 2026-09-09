//! Mission performance — coverage, revisit, latency, availability.
//!
//! These are the quantities a customer is actually promised, and they are the
//! ones furthest from the physics. Everything below is a geometric estimate
//! with its assumptions written down; a real coverage number comes from
//! propagating the constellation over a grid, which is a campaign rather than a
//! node.

use vleo_units::constants::*;
use vleo_units::pmath;
use vleo_units::*;

/// Instantaneous access area on the ground, from the Earth-central half-angle.
pub fn access_area(earth_central_angle: Angle) -> Area {
    let re = R_EARTH_MEAN.get();
    Area::new(2.0 * pmath::PI * re * re * (1.0 - earth_central_angle.cos()))
}

/// Fraction of the Earth's surface one satellite sees at any instant.
pub fn instantaneous_coverage_fraction(earth_central_angle: Angle) -> Ratio {
    Ratio::new(0.5 * (1.0 - earth_central_angle.cos()))
}

/// Mean revisit time for a Walker-like constellation, by an area-rate argument.
///
/// ```text
/// t_revisit = A_earth / (N·A_swath_rate)
/// ```
///
/// where the swath sweep rate is the swath width times the ground track speed.
/// This is a mean over the globe and it hides the thing a customer usually
/// cares about, which is the worst case at their latitude. It is stated as a
/// mean, and a node that needs the worst case declares a different question.
pub fn mean_revisit_time(
    swath_width: Length,
    ground_track_speed: Velocity,
    satellites: f64,
) -> Time {
    let re = R_EARTH_MEAN.get();
    let a_earth = 4.0 * pmath::PI * re * re;
    let sweep_rate = swath_width.get() * ground_track_speed.get();
    Time::new(a_earth / (satellites * sweep_rate))
}

/// Number of satellites needed to hit a revisit target — the inverse, which is
/// the form a concept trade asks in.
pub fn satellites_for_revisit(
    target_revisit: Time,
    swath_width: Length,
    ground_track_speed: Velocity,
) -> f64 {
    let re = R_EARTH_MEAN.get();
    let a_earth = 4.0 * pmath::PI * re * re;
    a_earth / (target_revisit.get() * swath_width.get() * ground_track_speed.get())
}

/// End-to-end latency: collect, hold until a downlink is available, downlink,
/// process, deliver.
///
/// The hold term dominates, and it is the term a constellation design can
/// actually move — by adding ground stations, by adding cross-links, or by
/// adding satellites.
pub fn end_to_end_latency(
    time_to_downlink: Time,
    downlink_duration: Time,
    ground_processing: Time,
    delivery: Time,
) -> Time {
    time_to_downlink + downlink_duration + ground_processing + delivery
}

/// Mean time to the next ground contact, for a satellite with `n` stations
/// giving `passes_per_day` opportunities in total.
pub fn mean_time_to_downlink(passes_per_day: f64) -> Time {
    if passes_per_day <= 0.0 {
        return Time::new(f64::INFINITY);
    }
    Time::new(86_400.0 / (2.0 * passes_per_day))
}

/// Availability of a series chain of independent elements.
pub fn series_availability(elements: &[Ratio]) -> Ratio {
    let mut p = 1.0;
    for e in elements {
        p *= e.get();
    }
    Ratio::new(p)
}

/// Availability of `k`-of-`n` redundancy, binomial.
pub fn k_of_n_availability(unit_availability: Ratio, n: u32, k: u32) -> Ratio {
    let p = unit_availability.get();
    let mut total = 0.0;
    for i in k..=n {
        total += binomial(n, i) * pmath::powi(p, i as i32) * pmath::powi(1.0 - p, (n - i) as i32);
    }
    Ratio::new(total)
}

fn binomial(n: u32, k: u32) -> f64 {
    let mut r = 1.0f64;
    for i in 0..k {
        r = r * ((n - i) as f64) / ((i + 1) as f64);
    }
    r
}

/// Whether an achieved value meets a required one, in the direction the
/// requirement is stated.
///
/// The closure rule, in one function. It is identical at every layer boundary —
/// customer to system, system to subsystem — which is the property that makes
/// the four-layer split mean anything, and it is one of the five ideas here
/// that is ours and unproven.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sense {
    /// Achieved must be at least required — coverage, margin, availability.
    AtLeast,
    /// Achieved must be at most required — revisit time, latency, mass, cost.
    AtMost,
}

/// The result of comparing achieved against required.
#[derive(Clone, Copy, Debug)]
pub struct Closure {
    pub closed: bool,
    /// Signed margin as a fraction of the requirement. Positive is compliant in
    /// both senses, so a reader does not have to remember which way round this
    /// particular requirement was written.
    pub margin: f64,
}

/// Compare achieved against required.
pub fn closure(required: f64, achieved: f64, sense: Sense) -> Closure {
    if required == 0.0 {
        return Closure {
            closed: false,
            margin: 0.0,
        };
    }
    let margin = match sense {
        Sense::AtLeast => (achieved - required) / required,
        Sense::AtMost => (required - achieved) / required,
    };
    Closure {
        closed: margin >= 0.0,
        margin,
    }
}
