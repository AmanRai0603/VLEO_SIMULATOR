//! Mass budgets and multi-payload allocation.
//!
//! Mass is where a multi-payload VLEO design either works or quietly does not.
//! Three payloads that each fit are not three payloads that fit together, and
//! the place that shows up is not mass alone — it is mass, power, thermal and
//! data at once, each with its own margin, each individually defensible.

use vleo_units::pmath;
use vleo_units::*;

/// Mass growth allowance by maturity, following the ECSS convention.
///
/// Applied per item, not to the total. Applying one blanket margin at the end
/// is the standard way a mass budget becomes fiction: it gives the same
/// allowance to a flight-qualified unit and to a sketch.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Maturity {
    /// Flight hardware, measured. 0%.
    Measured,
    /// Existing design, minor modification. 5%.
    Existing,
    /// Detailed design, not yet built. 10%.
    Detailed,
    /// Preliminary design. 20%.
    Preliminary,
    /// Estimate from a similar system. 50%.
    Estimated,
}

impl Maturity {
    pub const fn allowance(self) -> f64 {
        match self {
            Maturity::Measured => 0.00,
            Maturity::Existing => 0.05,
            Maturity::Detailed => 0.10,
            Maturity::Preliminary => 0.20,
            Maturity::Estimated => 0.50,
        }
    }
    pub const fn name(self) -> &'static str {
        match self {
            Maturity::Measured => "measured",
            Maturity::Existing => "existing",
            Maturity::Detailed => "detailed",
            Maturity::Preliminary => "preliminary",
            Maturity::Estimated => "estimated",
        }
    }
}

/// Mass of one item with its maturity allowance applied.
pub fn with_allowance(basic: Mass, maturity: Maturity) -> Mass {
    basic * (1.0 + maturity.allowance())
}

/// Dry mass from the subsystem breakdown, with a system-level margin on top.
///
/// The system margin is separate from and additional to the per-item
/// allowances: it covers what is not on the list yet, which on any real
/// programme is the term that actually bites.
#[allow(clippy::too_many_arguments)]
pub fn dry_mass(
    structure: Mass,
    propulsion: Mass,
    power: Mass,
    thermal: Mass,
    avionics: Mass,
    comms: Mass,
    gnc: Mass,
    harness: Mass,
    payloads: Mass,
    system_margin: Ratio,
) -> Mass {
    let sum =
        structure + propulsion + power + thermal + avionics + comms + gnc + harness + payloads;
    sum * (1.0 + system_margin.get())
}

/// Structure mass as a fraction of dry mass — the closure relation used before
/// a structural design exists.
///
/// Circular by construction, and honest about it: dry mass depends on structure
/// mass, which is a fraction of dry mass. Solved directly rather than iterated,
/// because the closed form exists:
///
/// ```text
/// m_dry = m_other/(1 - f_struct)
/// ```
///
/// Returns `None` when the fraction is at or above one, which is not a
/// convergence failure but a statement that the assumed fraction is not
/// physical.
pub fn dry_mass_from_structure_fraction(
    everything_else: Mass,
    structure_fraction: Ratio,
) -> Option<Mass> {
    let f = structure_fraction.get();
    if !(0.0..1.0).contains(&f) {
        return None;
    }
    Some(Mass::new(everything_else.get() / (1.0 - f)))
}

/// Wet mass — dry plus whatever propellant is still carried.
///
/// An air-breathing design does not reach zero here: it still carries
/// propellant for attitude control, for orbit raising after launch, and for the
/// disposal manoeuvre ISO 24113 requires.
pub fn wet_mass(dry: Mass, propellant: Mass) -> Mass {
    dry + propellant
}

/// Mass margin against a launch or bus limit, as a fraction of the limit.
pub fn mass_margin(limit: Mass, actual: Mass) -> Ratio {
    Ratio::new((limit.get() - actual.get()) / limit.get())
}

/// What one payload actually costs the platform, across every resource at once.
///
/// A multi-payload allocation is not a mass problem with three other problems
/// attached. It is one problem, and this is the shape of it.
#[derive(Clone, Copy, Debug)]
pub struct PayloadAllocation {
    pub mass: Mass,
    pub orbit_average_power: Power,
    pub peak_power: Power,
    pub dissipation: Power,
    pub data_rate: DataRate,
    pub frontal_area: Area,
}

impl PayloadAllocation {
    pub const ZERO: PayloadAllocation = PayloadAllocation {
        mass: Mass::new(0.0),
        orbit_average_power: Power::new(0.0),
        peak_power: Power::new(0.0),
        dissipation: Power::new(0.0),
        data_rate: DataRate::new(0.0),
        frontal_area: Area::new(0.0),
    };
    /// Sum two allocations. Peak power is the *maximum*, not the sum, when the
    /// payloads are declared non-concurrent — but that declaration is a design
    /// decision made on a sheet, so this function takes the pessimistic sum and
    /// a node that knows better subtracts.
    pub fn plus(self, o: PayloadAllocation) -> PayloadAllocation {
        PayloadAllocation {
            mass: self.mass + o.mass,
            orbit_average_power: self.orbit_average_power + o.orbit_average_power,
            peak_power: self.peak_power + o.peak_power,
            dissipation: self.dissipation + o.dissipation,
            data_rate: self.data_rate + o.data_rate,
            frontal_area: self.frontal_area + o.frontal_area,
        }
    }
    /// The tightest of the resource margins, and which one it is. A design is
    /// as feasible as its worst resource, and naming that resource is what
    /// turns a budget into a work item.
    pub fn governing_margin(
        &self,
        mass_limit: Mass,
        power_limit: Power,
        thermal_limit: Power,
        data_limit: DataRate,
    ) -> (&'static str, f64) {
        let candidates = [
            (
                "mass",
                (mass_limit.get() - self.mass.get()) / mass_limit.get(),
            ),
            (
                "power",
                (power_limit.get() - self.orbit_average_power.get()) / power_limit.get(),
            ),
            (
                "thermal",
                (thermal_limit.get() - self.dissipation.get()) / thermal_limit.get(),
            ),
            (
                "data",
                (data_limit.get() - self.data_rate.get()) / data_limit.get(),
            ),
        ];
        let mut best = candidates[0];
        for c in candidates.iter().skip(1) {
            if c.1 < best.1 {
                best = *c;
            }
        }
        best
    }
}

/// Moment of inertia of a uniform cylinder about its transverse axis — the
/// first-order figure a gravity-gradient and slew estimate needs before a mass
/// model exists.
pub fn cylinder_inertia_transverse(mass: Mass, radius: Length, length: Length) -> f64 {
    let m = mass.get();
    let r = radius.get();
    let l = length.get();
    m * (3.0 * r * r + l * l) / 12.0
}

/// Moment of inertia of a uniform cylinder about its axis.
pub fn cylinder_inertia_axial(mass: Mass, radius: Length) -> f64 {
    0.5 * mass.get() * radius.get() * radius.get()
}

/// Centre-of-pressure to centre-of-mass offset for a body whose area is
/// distributed along its length. Positive means the centre of pressure is aft,
/// which is the stable configuration — and arranging that is most of what
/// passive aerostability in VLEO means.
pub fn cp_cm_offset(area_centroid_from_nose: Length, mass_centroid_from_nose: Length) -> Length {
    Length::new(pmath::abs(
        area_centroid_from_nose.get() - mass_centroid_from_nose.get(),
    ))
}
