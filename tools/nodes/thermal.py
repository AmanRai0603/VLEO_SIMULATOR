#!/usr/bin/env python3
"""Thermal.

One subsystem's rows, and nothing else. Split out of the seeder so that
changing what this subsystem declares does not mean opening the file that
declares every other one — the same reason each subsystem has its own crate.

The helpers come from `seed_helpers`, which owns the shared lists these
append to. Import order is authoring order, so `seed_tree` decides it and this
file only says what the rows are.
"""

from seed_helpers import *          # noqa: F401,F403  the row helpers: D, C, layer, S, unnamed

# =============================================================================
# THERMAL
# =============================================================================
D("thm_absorptivity", "Solar absorptivity", "thm", "thermal", "Ratio", "One", "alpha_s",
  0.30, 0.05, 1.0,
  "below 0.05 no space-qualified coating is that reflective after atomic-oxygen exposure",
  "unity is a perfect black body, the limit rather than a coating",
  "What fraction of incident sunlight does the external surface absorb?",
  "larson_wertz", "thermal", "A. Rai / 2026-09-01")

D("thm_emissivity", "Infrared emissivity", "thm", "thermal", "Ratio", "One", "eps",
  0.85, 0.05, 1.0,
  "below 0.05 the surface cannot reject heat at all and no design closes",
  "unity is a perfect emitter, the limit rather than a coating",
  "How well does the external surface radiate in the infrared?",
  "larson_wertz", "thermal", "A. Rai / 2026-09-01")

D("thm_radiator_area", "Radiator area", "thm", "thermal", "Area", "SquareMetre", "A_rad",
  2.5, 0.1, 30.0,
  "below 0.1 m2 no useful heat is rejected",
  "above 30 m2 the radiator is the spacecraft",
  "How much area is available to reject heat?",
  "orbitt_case_c1", "thermal", "A. Rai / 2026-09-01")

D("thm_temperature_limit", "Upper temperature limit", "thm", "thermal", "Temperature", "Kelvin", "T_max",
  323.0, 273.0, 400.0,
  "below 273 K no electronics box in this design is qualified to operate",
  "above 400 K no electronics box in this design survives",
  "What is the highest temperature the electronics may reach?",
  "ecss_e_st_10_02", "thermal", "A. Rai / 2026-09-01")

D("thm_internal_dissipation", "Internal dissipation", "thm", "thermal", "Power", "Watt", "Q_int",
  0.0, 0.0, 20000.0,
  "dissipation cannot be negative",
  "above 20 kW nothing in this mass class dissipates that much",
  "How much of the electrical power ends up as heat inside the spacecraft?",
  "larson_wertz", "thermal", "A. Rai / 2026-09-01",
  note="Left at zero and overridden by the computed dissipation node. It exists so a case can pin a measured value in place of the computed one.")

C("thm_dissipation", "Heat to reject", "thm", "thermal", "Power", "Watt", "Q",
  "How much heat does the spacecraft have to get rid of?",
  "Q = P_dem - P_rf - P_beam",
  "larson_wertz", "thermal",
  [("dem", "pwr_demand"), ("pj", "prop_jet_power"), ("tx", "com_tx_power")],
  [("everything drawn from the bus becomes heat except the beam kinetic power and the radiated radio-frequency power",
    "q", "Power",
    "Power::new(pmath::max(0.0, dem.get() - pj.get() - tx.get()))")],
  0.0, 50000.0,
  "heat to reject cannot be negative",
  "above 50 kW nothing in this mass class dissipates that much",
  tier="B",
  assumptions=[("All bus power that is not beam kinetic energy or radiated radio power becomes heat",
                "true to within the accuracy of this budget; it ignores the small fraction stored in the battery over a cycle, which averages to zero over an orbit")])

C("thm_view_factor", "Earth view factor", "thm", "thermal", "Ratio", "One", "F_E",
  "How much of the sky, from a nadir-facing surface, is filled by the Earth?",
  "F = (Re/r)^2",
  "larson_wertz", "thermal",
  [("r", "orbit_radius")],
  [("apply the flat-plate to sphere view factor at the orbital radius",
    "f", "Ratio", "thermal::earth_view_factor(r)")],
  0.0, 1.0,
  "a view factor cannot be negative",
  "a view factor cannot exceed one",
  tier="A")

C("thm_absorbed_solar", "Absorbed direct solar", "thm", "thermal", "Power", "Watt", "Q_sol",
  "How much heat does direct sunlight put into the spacecraft?",
  "Q = S*A*alpha_s*cos(theta)",
  "larson_wertz", "thermal",
  [("a", "aero_frontal_area"), ("al", "thm_absorptivity"), ("th", "pwr_array_incidence")],
  [("apply the solar constant to the illuminated area at the mean incidence",
    "q", "Power", "thermal::absorbed_solar(a, al, th)")],
  0.0, 100000.0,
  "absorbed power cannot be negative",
  "above 100 kW the illuminated area is not this spacecraft",
  tier="A")

C("thm_absorbed_albedo", "Absorbed albedo", "thm", "thermal", "Power", "Watt", "Q_alb",
  "How much reflected sunlight off the Earth does the spacecraft absorb?",
  "Q = S*A*alpha_s*a_E*F",
  "larson_wertz", "thermal",
  [("a", "aero_frontal_area"), ("al", "thm_absorptivity"), ("f", "thm_view_factor")],
  [("apply the Earth's mean Bond albedo through the view factor",
    "q", "Power", "thermal::absorbed_albedo(a, al, f)")],
  0.0, 100000.0,
  "absorbed power cannot be negative",
  "above 100 kW the illuminated area is not this spacecraft",
  tier="B")

C("thm_absorbed_ir", "Absorbed Earth infrared", "thm", "thermal", "Power", "Watt", "Q_ir",
  "How much of the Earth's own thermal emission does the spacecraft absorb?",
  "Q = q_IR*A*eps*F",
  "larson_wertz", "thermal",
  [("a", "aero_frontal_area"), ("e", "thm_emissivity"), ("f", "thm_view_factor")],
  [("apply the mean outgoing longwave radiation through the view factor",
    "q", "Power", "thermal::absorbed_earth_ir(a, e, f)")],
  0.0, 100000.0,
  "absorbed power cannot be negative",
  "above 100 kW the surface is not this spacecraft",
  tier="B",
  note="Unlike albedo this does not switch off in eclipse, which is what stops a low-orbiting spacecraft getting as cold as an intuition built on higher orbits expects.")

C("thm_aero_heating", "Free-molecular aerodynamic heating", "thm", "thermal", "Power", "Watt", "Q_aero",
  "How much does the air itself heat the spacecraft?",
  "Q = 0.5*rho*V^3*A*C_h",
  "larson_wertz", "thermal",
  [("rho", "env_mass_density"), ("v", "orbit_velocity"), ("a", "aero_frontal_area")],
  [("form the incident kinetic energy flux and apply a heat transfer coefficient of 0.9",
    "q", "Power", "thermal::free_molecular_heating(rho, v, a, Ratio::new(0.9))")],
  0.0, 100000.0,
  "heating cannot be negative",
  "above 100 kW the vehicle is re-entering",
  tier="C",
  note="Usually neglected, and at 200 km it should not be. It is a real term in the balance of a small, fast, low-flying vehicle.")

C("thm_equilibrium_temperature", "Equilibrium temperature", "thm", "thermal", "Temperature", "Kelvin", "T_eq",
  "What temperature does the spacecraft settle at?",
  "T = (Q_total/(eps*sigma*A_rad))^(1/4)",
  "larson_wertz", "thermal",
  [("qs", "thm_absorbed_solar"), ("qa", "thm_absorbed_albedo"), ("qi", "thm_absorbed_ir"),
   ("qh", "thm_aero_heating"), ("qd", "thm_dissipation"), ("ar", "thm_radiator_area"),
   ("e", "thm_emissivity")],
  [("balance every absorbed and dissipated term against grey-body radiation from the radiator",
    "t", "Temperature",
    "thermal::equilibrium_temperature(Power::new(qs.get() + qa.get() + qi.get() + qh.get()), qd, ar, e)")],
  100.0, 1000.0,
  "below 100 K the spacecraft would be colder than deep space plus the Earth's infrared, which is not possible in this orbit",
  "above 1000 K nothing survives and the balance is being fed a nonsense load",
  tier="B",
  assumptions=[("One node — the whole spacecraft is at one temperature",
                "a real design has a gradient across every panel; this sizes a radiator and bounds a temperature, and it does not predict a gradient")],
  view=("line", {"over": "orbit_altitude", "points": 50}))

C("thm_margin", "Thermal margin", "thm", "closure", "Temperature", "Kelvin", "M_thm",
  "Is the spacecraft inside its temperature limit?",
  "M = T_max - T_eq",
  "ecss_e_st_10_02", "thermal",
  [("t", "thm_equilibrium_temperature"), ("lim", "thm_temperature_limit")],
  [("subtract the predicted temperature from the declared limit",
    "m", "Temperature", "thermal::thermal_margin(t, lim)")],
  -500.0, 500.0,
  "a margin below -500 K means the balance is being fed a nonsense load",
  "a margin above 500 K means the limit is not the one that binds",
  tier="A",
  kpis=["kpi_thermal_margin"])

C("thm_required_radiator_area", "Radiator area required", "thm", "thermal", "Area", "SquareMetre", "A_req",
  "How much radiator would be needed to hold the temperature limit?",
  "A = Q/(eps*sigma*(T^4 - T_sink^4))",
  "larson_wertz", "thermal",
  [("q", "thm_dissipation"), ("e", "thm_emissivity"), ("lim", "thm_temperature_limit")],
  [("solve the grey-body balance for area against a 250 K effective sink",
    "a", "Area",
    "thermal::required_radiator_area(q, e, lim, Temperature::new(250.0))")],
  0.0, 1000.0,
  "a required area cannot be negative",
  "above 1000 m2 the design does not close and the answer is that the dissipation is wrong",
  tier="A",
  note="The inverse question, and the one a thermal design actually asks.")
