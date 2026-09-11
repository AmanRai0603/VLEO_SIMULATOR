#!/usr/bin/env python3
"""Power.

One subsystem's rows, and nothing else. Split out of the seeder so that
changing what this subsystem declares does not mean opening the file that
declares every other one — the same reason each subsystem has its own crate.

The helpers come from `seed_helpers`, which owns the shared lists these
append to. Import order is authoring order, so `seed_tree` decides it and this
file only says what the rows are.
"""

from seed_helpers import *          # noqa: F401,F403  the row helpers: D, C, layer, S, unnamed

# =============================================================================
# POWER
# =============================================================================
D("pwr_cell_efficiency", "Solar cell efficiency", "power", "power", "Ratio", "One", "eta_cell",
  0.30, 0.10, 0.40,
  "below 10% no cell technology considered for this programme performs that badly",
  "above 40% is beyond any qualified space cell available at the time of this baseline",
  "How efficiently does one cell convert sunlight?",
  "larson_wertz", "power", "A. Rai / 2026-09-01")

D("pwr_packing", "Array packing factor", "power", "power", "Ratio", "One", "f_pack",
  0.88, 0.5, 0.98,
  "below 50% the panel is mostly structure",
  "above 98% leaves no room for interconnects or the panel edge",
  "What fraction of the panel area is active cell?",
  "larson_wertz", "power", "A. Rai / 2026-09-01")

D("pwr_degradation_rate", "Annual array degradation rate", "power", "power", "Ratio", "One", "d_yr",
  0.025, 0.0, 0.15,
  "a degradation rate cannot be negative",
  "above 15% a year the array is gone within the mission and the design does not close",
  "How much output does the array lose each year?",
  "larson_wertz", "power", "A. Rai / 2026-09-01",
  note="VLEO is below the inner belt, so the dominant mechanism is atomic-oxygen erosion of coverglass adhesive and thermal cycling, not displacement damage. This is the one place a VLEO power design differs from the textbook figure.")

D("pwr_array_incidence", "Mean solar incidence angle on the array", "power", "power", "Angle", "Degree", "theta_s",
  25.0, 0.0, 85.0,
  "zero is normal incidence, the best case and the limit",
  "above 85 degrees the cosine loss leaves almost nothing and the array is not pointed at the Sun in any useful sense",
  "At what mean angle does the sunlight strike the array over an orbit?",
  "larson_wertz", "power", "A. Rai / 2026-09-01")

D("pwr_dod", "Battery depth of discharge", "power", "power", "Ratio", "One", "DoD",
  0.20, 0.05, 0.8,
  "below 5% the battery is absurdly oversized",
  "above 80% no lithium-ion cell survives 29000 cycles, which is what a five-year VLEO mission is",
  "How deeply is the battery discharged each eclipse?",
  "larson_wertz", "power", "A. Rai / 2026-09-01",
  note="A lifetime decision rather than an energy one: 92-minute orbits over five years is roughly 29000 cycles.")

D("pwr_discharge_efficiency", "Battery discharge path efficiency", "power", "power", "Ratio", "One", "eta_dis",
  0.92, 0.6, 0.99,
  "below 60% the discharge path dissipates more than the loads receive",
  "above 99% has not been demonstrated end to end including the regulator",
  "How efficiently does stored energy reach the loads?",
  "larson_wertz", "power", "A. Rai / 2026-09-01")

D("pwr_harness_loss", "Harness and distribution loss", "power", "power", "Ratio", "One", "L_h",
  0.06, 0.0, 0.25,
  "a loss fraction cannot be negative",
  "above 25% the harness is the largest load on the bus",
  "What fraction is lost between the source and the loads?",
  "larson_wertz", "power", "A. Rai / 2026-09-01")

D("pwr_array_areal_density", "Array areal density", "power", "power", "Ratio", "One", "sigma_a",
  2.8, 0.5, 10.0,
  "below 0.5 kg/m2 no deployable array structure of this size is that light",
  "above 10 kg/m2 the array mass alone breaks the mass budget",
  "What does a square metre of deployed array weigh?",
  "larson_wertz", "power", "A. Rai / 2026-09-01")

D("pwr_battery_specific_energy", "Battery specific energy", "power", "power", "Ratio", "One", "e_batt",
  150.0, 50.0, 350.0,
  "below 50 Wh/kg no cell considered is that poor",
  "above 350 Wh/kg is beyond any space-qualified lithium-ion cell at this baseline",
  "How much energy does a kilogram of battery store?",
  "larson_wertz", "power", "A. Rai / 2026-09-01")

D("pwr_avionics_load", "Avionics and housekeeping load", "power", "power", "Power", "Watt", "P_av",
  35.0, 5.0, 500.0,
  "below 5 W no spacecraft avionics set runs",
  "above 500 W the avionics dominate the bus, which no design in this class does",
  "What does the on-board computer, the harness electronics and the housekeeping draw?",
  "orbitt_case_c1", "avionics", "A. Rai / 2026-09-01")

D("pwr_thermal_load", "Thermal control load", "power", "power", "Power", "Watt", "P_th",
  15.0, 0.0, 300.0,
  "a heater load cannot be negative",
  "above 300 W the thermal design is heating rather than rejecting, which does not happen in this orbit",
  "What do the heaters and thermal control electronics draw?",
  "orbitt_case_c1", "thermal", "A. Rai / 2026-09-01")

D("pwr_array_area", "Solar array area", "power", "power", "Area", "SquareMetre", "A_arr",
  6.0, 0.5, 40.0,
  "below 0.5 m2 no useful power is produced at any efficiency",
  "above 40 m2 the array's own drag exceeds the thrust the intake can produce at every altitude in the band",
  "How large is the deployed solar array?",
  "orbitt_case_c1", "power", "A. Rai / 2026-09-01",
  kpis=["kpi_power_margin"],
  note="A design variable rather than a computed one, deliberately: it is coupled to drag through the appendage area, so a solver that sized it from the power demand alone would hide the coupling the tree exists to show.")

C("pwr_degradation", "Array degradation factor", "power", "power", "Ratio", "One", "f_deg",
  "How much of the array's beginning-of-life output survives to the end of the mission?",
  "f_deg = (1 - d_yr)^t",
  "larson_wertz", "power",
  [("d", "pwr_degradation_rate"), ("t", "orbit_mission_duration")],
  [("compound the annual rate over the mission duration in years",
    "f", "Ratio", "power::degradation(d, t.get() / 31_557_600.0)")],
  0.0, 1.0,
  "a degradation factor cannot be negative",
  "an array cannot end life producing more than it began with",
  tier="A")

C("pwr_array_power_bol", "Array power, beginning of life", "power", "power", "Power", "Watt", "P_bol",
  "How much does the array produce when new, in sunlight?",
  "P_bol = S*A*eta_cell*f_pack*cos(theta)",
  "larson_wertz", "power",
  [("a", "pwr_array_area"), ("e", "pwr_cell_efficiency"), ("f", "pwr_packing"), ("th", "pwr_array_incidence")],
  [("apply the solar constant to the illuminated active area at the mean incidence",
    "p", "Power", "power::array_power_bol(a, e, f, th)")],
  0.0, 50000.0,
  "array output cannot be negative",
  "above 50 kW the array is not the one described by these inputs",
  tier="A")

C("pwr_array_power_eol", "Array power, end of life", "power", "power", "Power", "Watt", "P_eol",
  "How much does the array produce at the end of the mission?",
  "P_eol = P_bol*f_deg",
  "larson_wertz", "power",
  [("p", "pwr_array_power_bol"), ("f", "pwr_degradation")],
  [("apply the degradation factor to the beginning-of-life output",
    "pe", "Power", "power::array_power_eol(p, f)")],
  0.0, 50000.0,
  "array output cannot be negative",
  "cannot exceed the beginning-of-life output",
  tier="A",
  note="Every power budget in this tree closes against this number, not against the beginning-of-life one.")

C("pwr_available", "Orbit-average power available", "power", "power", "Power", "Watt", "P_avail",
  "How much power is actually available to the loads, averaged over an orbit?",
  "P_avail = P_eol*(1 - f_ecl)*eta_direct + P_eol*f_ecl*eta_batt",
  "larson_wertz", "power",
  [("pe", "pwr_array_power_eol"), ("fe", "orbit_eclipse_fraction"), ("ed", "pwr_discharge_efficiency")],
  [("average the direct and battery-borne paths over the sunlit and eclipsed parts of the orbit",
    "p", "Power",
    "Power::new(pe.get() * ((1.0 - fe.get()) * 0.97 + fe.get() * ed.get() * 0.95))")],
  0.0, 50000.0,
  "available power cannot be negative",
  "above 50 kW the array is not the one described by these inputs",
  tier="B",
  assumptions=[("The array is sized so that the sunlit period both runs the loads and recharges the battery",
                "if it is not, the battery state of charge walks down over successive orbits and this number is optimistic from the second day onwards")])

C("pwr_demand", "Total bus power demand", "power", "power", "Power", "Watt", "P_dem",
  "How much power does the whole spacecraft need?",
  "P_dem = (P_prop + P_pay + P_av + P_com + P_th)*(1 + L_h)",
  "larson_wertz", "power",
  [("pp", "prop_bus_power"), ("pay", "pay_power"), ("av", "pwr_avionics_load"),
   ("com", "com_tx_power"), ("th", "pwr_thermal_load"), ("lh", "pwr_harness_loss")],
  [("sum the named loads and inflate them by the harness loss",
    "p", "Power", "power::power_demand(pp, pay, av, com, th, lh)")],
  0.0, 50000.0,
  "demand cannot be negative",
  "above 50 kW nothing in this mass class supplies it",
  tier="A",
  note="Written as one function with named arguments rather than a chain of additions, so a forgotten term is a missing argument the compiler names.")

C("pwr_margin", "Power margin", "power", "closure", "Ratio", "One", "M_pwr",
  "Does the power system supply what the spacecraft asks for?",
  "M = (P_avail - P_dem)/P_dem",
  "ecss_e_st_10_02", "power",
  [("av", "pwr_available"), ("dem", "pwr_demand")],
  [("form the signed fractional margin; a negative value is reported, never clamped",
    "m", "Ratio", "power::power_margin(av, dem)")],
  -1.0, 100.0,
  "a margin below -100% would mean the demand is more than twice the supply and the design is not a design",
  "above 10000% the array is absurdly oversized",
  tier="A",
  kpis=["kpi_power_margin"],
  note="A margin silently corrected to zero is a design that drifted without anyone deciding to. This one is signed.")

C("pwr_battery_energy", "Battery energy required", "power", "power", "Energy", "WattHour", "E_batt",
  "How much energy must the battery hold to carry the load through eclipse?",
  "E = P_ecl*t_ecl/(DoD*eta_dis)",
  "larson_wertz", "power",
  [("dem", "pwr_demand"), ("fe", "orbit_eclipse_fraction"), ("t", "orbit_period"),
   ("dod", "pwr_dod"), ("ed", "pwr_discharge_efficiency")],
  [("carry the full demand through the eclipse fraction of one revolution at the declared depth of discharge",
    "e", "Energy",
    "power::battery_energy_required(dem, Time::new(fe.get() * t.get()), dod, ed)")],
  0.0, 27778.0,
  "battery energy cannot be negative",
  "above 27 kWh the battery is heavier than the whole spacecraft",
  tier="A")

C("pwr_battery_cycles", "Battery charge cycles", "power", "power", "Ratio", "One", "N_cyc",
  "How many charge and discharge cycles does the battery see?",
  "N = t_mission/T_orbit",
  "larson_wertz", "power",
  [("tm", "orbit_mission_duration"), ("to", "orbit_period")],
  [("divide the mission duration by the orbital period", "n", "Ratio",
    "Ratio::new(power::battery_cycles(tm, to))")],
  0.0, 1.0e6,
  "a cycle count cannot be negative",
  "above a million cycles no cell chemistry has data",
  tier="A")

C("pwr_battery_mass", "Battery mass", "power", "power", "Mass", "Kilogram", "m_batt",
  "What does the battery weigh?",
  "m = E/e_batt",
  "larson_wertz", "power",
  [("e", "pwr_battery_energy"), ("se", "pwr_battery_specific_energy")],
  [("divide the required energy by the specific energy",
    "m", "Mass", "power::battery_mass(e, se.get())")],
  0.0, 1000.0,
  "a mass cannot be negative",
  "above a tonne the battery is not part of this spacecraft",
  tier="A")

C("pwr_array_mass", "Array mass", "power", "power", "Mass", "Kilogram", "m_arr",
  "What does the deployed array weigh?",
  "m = A*sigma_a",
  "larson_wertz", "power",
  [("a", "pwr_array_area"), ("s", "pwr_array_areal_density")],
  [("multiply the deployed area by the areal density",
    "m", "Mass", "power::array_mass(a, s.get())")],
  0.0, 1000.0,
  "a mass cannot be negative",
  "above a tonne the array is not part of this spacecraft",
  tier="A")

C("pwr_subsystem_mass", "Power subsystem mass", "power", "power", "Mass", "Kilogram", "m_pwr",
  "What does the whole power subsystem weigh?",
  "m = m_arr + m_batt + m_pcdu",
  "larson_wertz", "power",
  [("ma", "pwr_array_mass"), ("mb", "pwr_battery_mass")],
  [("sum the array and battery masses and add 25% for the distribution and control unit",
    "m", "Mass", "Mass::new((ma.get() + mb.get()) * 1.25)")],
  0.0, 2000.0,
  "a mass cannot be negative",
  "above two tonnes the power system is not part of this spacecraft",
  tier="B")
