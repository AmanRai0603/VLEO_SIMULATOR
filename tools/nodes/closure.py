#!/usr/bin/env python3
"""The closure loop.

One subsystem's rows, and nothing else. Split out of the seeder so that
changing what this subsystem declares does not mean opening the file that
declares every other one — the same reason each subsystem has its own crate.

The helpers come from `seed_helpers`, which owns the shared lists these
append to. Import order is authoring order, so `seed_tree` decides it and this
file only says what the rows are.
"""

from seed_helpers import *          # noqa: F401,F403  the row helpers: D, C, layer, S, unnamed

# =============================================================================
# THE CLOSURE LOOP
#
# Real closure is cyclic. Power becomes heat, heat sets the array temperature,
# array temperature sets cell efficiency, cell efficiency sets available power,
# available power throttles the thruster, and the thruster is the largest load
# on the bus. If node implementations called each other the crate graph would
# have to mirror that dataflow, crate cycles are forbidden, and this loop would
# force propulsion, power and thermal to be merged into one crate.
#
# They are not merged. A node calls nothing, the loop is data, it is declared
# in the case, and the resolver relaxes it to a stated convergence criterion.
# An undeclared cycle is a named error rather than a hung resolver.
# =============================================================================
D("pwr_cell_temperature_coefficient", "Cell power temperature coefficient", "power", "power",
  "Ratio", "One", "k_T",
  -0.0025, -0.010, 0.0,
  "below -1% per kelvin no cell technology degrades that fast",
  "a positive coefficient would mean a cell that improves when heated, which does not happen",
  "How much output does the array lose per kelvin above its reference temperature?",
  "larson_wertz", "power", "A. Rai / 2026-09-01")

C("pwr_cell_derating", "Array temperature derating factor", "power", "power", "Ratio", "One", "f_T",
  "How much output does the array lose because it is hotter than its rating?",
  "f_T = 1 + k_T*(T_eq - 298.15)",
  "larson_wertz", "power",
  [("k", "pwr_cell_temperature_coefficient"), ("t", "thm_equilibrium_temperature")],
  [("apply the linear temperature coefficient about the 298.15 K reference, floored at 30% so the relaxation cannot chase a negative array",
    "f", "Ratio",
    "Ratio::new(pmath::max(0.3, 1.0 + k.get() * (t.get() - 298.15)))")],
  0.0, 1.5,
  "a derating factor cannot be negative",
  "above 1.5 the array would be producing half again its rating, which the coefficient does not permit",
  tier="A",
  note="This node closes the loop. Without it power, thermal and propulsion are three independent chains; with it they are one system, and the resolver has to relax them together.")

C("prop_throttle", "Propulsion throttle", "prop", "prop_thruster", "Ratio", "One", "k_thr",
  "How much of what the thruster asks for can the bus actually give it?",
  "k = clamp((P_avail - P_other)/P_prop, 0, 1)",
  "orbitt_case_c1", "propulsion",
  [("av", "pwr_available"), ("pp", "prop_bus_power"), ("pay", "pay_power"),
   ("ax", "pwr_avionics_load"), ("cm", "com_tx_power"), ("th", "pwr_thermal_load"),
   ("lh", "pwr_harness_loss")],
  [("subtract every other load from what is available and give the remainder to propulsion, capped at what it asked for",
    "k", "Ratio",
    "{ let other = (pay.get() + ax.get() + cm.get() + th.get()) * (1.0 + lh.get()); "
    "let spare = pmath::max(0.0, av.get() - other); "
    "Ratio::new(if pp.get() <= 0.0 { 0.0 } else { pmath::min(1.0, spare / pp.get()) }) }")],
  0.0, 1.0,
  "a throttle cannot be negative",
  "the thruster cannot be given more than it asked for",
  tier="B",
  kpis=["kpi_thrust_margin", "kpi_power_margin"],
  note="The node that makes the power budget bite on the physics rather than being reported beside it.")

C("prop_delivered_bus_power", "Propulsion power actually drawn", "prop", "prop_thruster", "Power", "Watt", "P_del",
  "How much power does the thruster actually take, after throttling?",
  "P = k*P_prop",
  "orbitt_case_c1", "propulsion",
  [("pp", "prop_bus_power"), ("k", "prop_throttle")],
  [("scale the requested bus power by the throttle", "p", "Power", "pp * k.get()")],
  0.0, 1.0e5,
  "power drawn cannot be negative",
  "above 100 kW no bus in this mass class supplies it",
  tier="A")

C("prop_delivered_thrust", "Thrust actually delivered", "prop", "prop_thruster", "Force", "Millinewton", "T_del",
  "How much thrust is produced once the power available is taken into account?",
  "T = k*T",
  "orbitt_case_c1", "propulsion",
  [("t", "prop_thrust"), ("k", "prop_throttle")],
  [("scale thrust with the throttle at fixed specific impulse", "d", "Force", "t * k.get()")],
  0.0, 1000.0,
  "thrust cannot be negative",
  "above 1 N no air-breathing thruster in this band produces thrust",
  tier="B",
  assumptions=[("Thrust scales linearly with delivered power at fixed specific impulse",
                "a real thruster's efficiency falls away from its design point, so a deeply throttled system delivers less than this; the relation is optimistic below about half throttle")],
  kpis=["kpi_thrust_margin"])
