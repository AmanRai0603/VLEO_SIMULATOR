//! Cost.
//!
//! Cost estimating relationships are fitted to historical programmes, and every
//! one of them is a power law in a mass or a power with a large residual. They
//! are here because a design tool that cannot say what a decision costs is a
//! tool that will be overruled by a spreadsheet — and the spreadsheet will not
//! have the design in it.
//!
//! The honest statement about accuracy: these are order-of-magnitude at
//! concept stage. They belong at the management layer, downstream of a selected
//! design, and they carry their own credibility like every other node.

use vleo_units::pmath;
use vleo_units::*;

/// A cost estimating relationship of the form `cost = a·x^b`, in a stated
/// currency year.
#[derive(Clone, Copy, Debug)]
pub struct Cer {
    pub a: f64,
    pub b: f64,
    /// The currency year the fit is in. A CER quoted without one is unusable,
    /// and quoting it without one is how a 2005 fit becomes a 2026 estimate.
    pub base_year: u16,
    /// One-sigma residual of the fit, as a fraction. Reported with every
    /// estimate, because a point estimate from a power law with a 40% residual
    /// is not a number, it is a centre.
    pub sigma: f64,
}

impl Cer {
    /// Evaluate, in the CER's own base year currency.
    pub fn evaluate(&self, driver: f64) -> Money {
        Money::new(self.a * pmath::powf(driver, self.b) * 1.0e6)
    }
    /// Evaluate and inflate to a target year at a stated annual rate.
    pub fn evaluate_inflated(&self, driver: f64, target_year: u16, annual_inflation: f64) -> Money {
        let years = (target_year as f64) - (self.base_year as f64);
        Money::new(self.evaluate(driver).get() * pmath::powf(1.0 + annual_inflation, years))
    }
}

/// Wright's learning curve: the `n`-th unit costs `T1·n^(log2(b))`.
///
/// `b` is the learning-curve slope — 0.95 means each doubling of cumulative
/// production costs 95% of the previous doubling. For a constellation this is
/// the term that decides whether forty satellites cost forty times one, and it
/// is the term most concept estimates leave out.
pub fn unit_cost(first_unit: Money, unit_number: f64, slope: f64) -> Money {
    Money::new(first_unit.get() * pmath::powf(unit_number, pmath::log2(slope)))
}

/// Total cost of a production run under a learning curve, summed rather than
/// integrated — the run is tens of units, not thousands, and the integral
/// approximation is wrong by a few per cent at that size.
pub fn production_run_cost(first_unit: Money, units: u32, slope: f64) -> Money {
    let mut total = 0.0;
    for n in 1..=units {
        total += unit_cost(first_unit, n as f64, slope).get();
    }
    Money::new(total)
}

/// Programme cost: non-recurring, plus the production run, plus operations for
/// the mission duration.
pub fn programme_cost(
    non_recurring: Money,
    production: Money,
    annual_operations: Money,
    years: f64,
) -> Money {
    Money::new(non_recurring.get() + production.get() + annual_operations.get() * years)
}

/// Cost per unit of delivered service — the number the management layer
/// actually compares options on.
pub fn cost_per_service_unit(programme_cost: Money, service_units: f64) -> Money {
    Money::new(programme_cost.get() / service_units)
}

/// The saving an air-breathing design buys, against a stored-propellant design
/// that has to be replaced when its propellant runs out.
///
/// This is the commercial argument for the whole programme, so it is a node
/// with a source and evidence like any other rather than a slide.
pub fn replacement_avoided(
    satellite_unit_cost: Money,
    launch_cost: Money,
    replacements_avoided: f64,
) -> Money {
    Money::new((satellite_unit_cost.get() + launch_cost.get()) * replacements_avoided)
}

/// A published spacecraft bus CER, dry mass driven.
///
/// Small-satellite bus recurring cost against dry mass, fitted over the small
/// end of the historical record. Stated with its residual because the residual
/// is larger than most of the design decisions it would be used to compare.
pub const BUS_RECURRING_CER: Cer = Cer { a: 0.98, b: 0.72, base_year: 2020, sigma: 0.40 };

/// Payload recurring cost against payload mass.
pub const PAYLOAD_RECURRING_CER: Cer = Cer { a: 1.85, b: 0.65, base_year: 2020, sigma: 0.45 };

/// Non-recurring engineering against total dry mass.
pub const NON_RECURRING_CER: Cer = Cer { a: 14.2, b: 0.55, base_year: 2020, sigma: 0.55 };
