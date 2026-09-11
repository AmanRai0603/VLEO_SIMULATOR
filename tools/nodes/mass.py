#!/usr/bin/env python3
"""Mass.

One subsystem's rows, and nothing else. Split out of the seeder so that
changing what this subsystem declares does not mean opening the file that
declares every other one — the same reason each subsystem has its own crate.

The helpers come from `seed_helpers`, which owns the shared lists these
append to. Import order is authoring order, so `seed_tree` decides it and this
file only says what the rows are.
"""

from seed_helpers import *          # noqa: F401,F403  the row helpers: D, C, layer, S, unnamed

# =============================================================================
# MASS
# =============================================================================
D("mass_structure", "Structure and mechanisms mass", "mass", "mass_aero", "Mass", "Kilogram", "m_str",
  38.0, 1.0, 2000.0,
  "below a kilogram there is no structure",
  "above two tonnes the vehicle is not in this class",
  "What do the primary structure, panels and mechanisms weigh?",
  "orbitt_case_c1", "mass", "A. Rai / 2026-09-01")

D("mass_propulsion_hw", "Propulsion hardware mass", "mass", "mass_aero", "Mass", "Kilogram", "m_prop",
  26.0, 0.5, 500.0,
  "below half a kilogram no intake, thruster and power processing unit exist",
  "above 500 kg the propulsion system is the spacecraft",
  "What do the intake, thruster and power processing unit weigh?",
  "orbitt_case_c1", "propulsion", "A. Rai / 2026-09-01")

D("mass_avionics", "Avionics mass", "mass", "mass_aero", "Mass", "Kilogram", "m_av",
  9.0, 0.5, 200.0,
  "below half a kilogram no avionics set exists",
  "above 200 kg the avionics are not in this class",
  "What do the on-board computer, harness electronics and data handling weigh?",
  "orbitt_case_c1", "avionics", "A. Rai / 2026-09-01")

D("mass_comms_hw", "Communications hardware mass", "mass", "mass_aero", "Mass", "Kilogram", "m_com",
  7.5, 0.2, 200.0,
  "below 200 grams no transceiver and antenna exist",
  "above 200 kg the communications payload is not in this class",
  "What do the transceivers and antennas weigh?",
  "orbitt_case_c1", "comms", "A. Rai / 2026-09-01")

D("mass_gnc_hw", "GNC hardware mass", "mass", "mass_aero", "Mass", "Kilogram", "m_gnc",
  11.0, 0.5, 200.0,
  "below half a kilogram no sensor and actuator set exists",
  "above 200 kg the control hardware is not in this class",
  "What do the sensors, wheels and torque rods weigh?",
  "orbitt_case_c1", "gnc", "A. Rai / 2026-09-01")

D("mass_harness", "Harness and thermal hardware mass", "mass", "mass_aero", "Mass", "Kilogram", "m_har",
  12.0, 0.2, 300.0,
  "below 200 grams no harness exists",
  "above 300 kg the harness is not in this class",
  "What do the harness, multi-layer insulation and radiators weigh?",
  "orbitt_case_c1", "thermal", "A. Rai / 2026-09-01")

D("mass_payload", "Payload mass", "mass", "mass_aero", "Mass", "Kilogram", "m_pay",
  55.0, 0.5, 1000.0,
  "below half a kilogram no service payload exists",
  "above a tonne the payload set is not in this class",
  "What does the whole service payload set weigh?",
  "orbitt_case_c1", "payload", "A. Rai / 2026-09-01")

D("mass_system_margin", "System-level mass margin", "mass", "mass_aero", "Ratio", "One", "M_sys",
  0.20, 0.0, 0.5,
  "a margin cannot be negative",
  "above 50% the design is a sketch and the number is not a budget",
  "What system-level margin is held above the sum of the subsystem masses?",
  "ecss_e_st_10_02", "mass", "A. Rai / 2026-09-01",
  note="Separate from and additional to the per-item maturity allowances. It covers what is not on the list yet, which on any real programme is the term that bites.")

D("mass_limit", "Mass limit", "mass", "mass_aero", "Mass", "Kilogram", "m_lim",
  250.0, 5.0, 5000.0,
  "below 5 kg no multipayload mission fits",
  "above 5 tonnes the launch and the class are not the ones costed here",
  "What is the mass the launch slot or the bus allows?",
  "orbitt_case_c1", "systems", "A. Rai / 2026-09-01")

C("mass_dry", "Dry mass", "mass", "mass_aero", "Mass", "Kilogram", "m_dry",
  "What does the spacecraft weigh with no propellant on board?",
  "m_dry = (sum of subsystem masses)*(1 + M_sys)",
  "ecss_e_st_10_02", "mass",
  [("st", "mass_structure"), ("pr", "mass_propulsion_hw"), ("pw", "pwr_subsystem_mass"),
   ("th", "mass_harness"), ("av", "mass_avionics"), ("cm", "mass_comms_hw"),
   ("gn", "mass_gnc_hw"), ("pa", "mass_payload"), ("mg", "mass_system_margin")],
  [("sum the subsystem masses and apply the system margin",
    "m", "Mass",
    "mass::dry_mass(st, pr, pw, Mass::ZERO, av, cm, gn, th, pa, mg)")],
  1.0, 5000.0,
  "below a kilogram there is no spacecraft",
  "above 5 tonnes the vehicle is not in this class",
  tier="A",
  kpis=["kpi_mass_margin"])

C("mass_disposal_propellant", "Disposal propellant mass", "mass", "mass_aero", "Mass", "Kilogram", "m_dis",
  "How much stored propellant must be carried for the disposal manoeuvre?",
  "m = m_dry*(exp(dv/v_e) - 1)",
  "iso24113", "propulsion",
  [("m", "mass_dry"), ("dv", "orbit_deorbit_delta_v")],
  [("apply the rocket equation at a 220 s stored-propellant specific impulse",
    "mp", "Mass",
    "orbit::propellant_mass(m, dv, orbit::exhaust_velocity(Time::new(220.0)))")],
  0.0, 1000.0,
  "a propellant mass cannot be negative",
  "above a tonne the disposal budget is larger than the spacecraft",
  tier="A",
  note="An air-breathing design does not reach zero here. Debris mitigation is a requirement, not a courtesy.")

C("mass_wet", "Wet mass", "mass", "mass_aero", "Mass", "Kilogram", "m_wet",
  "What does the spacecraft weigh at separation?",
  "m_wet = m_dry + m_prop",
  "ecss_e_st_10_02", "mass",
  [("d", "mass_dry"), ("p", "mass_disposal_propellant")],
  [("add the disposal propellant to the dry mass", "m", "Mass", "mass::wet_mass(d, p)")],
  1.0, 6000.0,
  "below a kilogram there is no spacecraft",
  "above 6 tonnes the vehicle is not in this class",
  tier="A")

C("mass_margin", "Mass margin", "mass", "closure", "Ratio", "One", "M_mass",
  "Does the spacecraft fit the mass it is allowed?",
  "M = (m_lim - m_wet)/m_lim",
  "ecss_e_st_10_02", "mass",
  [("l", "mass_limit"), ("w", "mass_wet")],
  [("form the signed fractional margin against the declared limit",
    "m", "Ratio", "mass::mass_margin(l, w)")],
  -5.0, 1.0,
  "a margin below -500% means the design is five times over and the inputs are wrong",
  "a margin of one means the spacecraft weighs nothing",
  tier="A",
  kpis=["kpi_mass_margin"])
