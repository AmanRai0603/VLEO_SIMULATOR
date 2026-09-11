#!/usr/bin/env python3
"""Cost.

One subsystem's rows, and nothing else. Split out of the seeder so that
changing what this subsystem declares does not mean opening the file that
declares every other one — the same reason each subsystem has its own crate.

The helpers come from `seed_helpers`, which owns the shared lists these
append to. Import order is authoring order, so `seed_tree` decides it and this
file only says what the rows are.
"""

from seed_helpers import *          # noqa: F401,F403  the row helpers: D, C, layer, S, unnamed

# =============================================================================
# COST
# =============================================================================
D("cost_learning_slope", "Production learning curve slope", "cost", "mgt_cost", "Ratio", "One", "b_lc",
  0.92, 0.7, 1.0,
  "below 0.70 no space production programme has learned that fast",
  "unity is no learning at all, the pessimistic limit",
  "By what factor does unit cost fall with each doubling of cumulative production?",
  "nasa_cer", "programme", "A. Rai / 2026-09-01",
  note="The term that decides whether forty satellites cost forty times one, and the term most concept estimates leave out.")

D("cost_launch_per_satellite", "Launch cost per satellite", "cost", "mgt_cost", "Money", "MillionUsDollar", "C_lch",
  1.8, 0.05, 100.0,
  "below 50 thousand dollars no launch service exists",
  "above 100 million per satellite the constellation is not the one costed here",
  "What does it cost to put one satellite in orbit?",
  "orbitt_case_c1", "programme", "A. Rai / 2026-09-01")

D("cost_annual_operations", "Annual operations cost", "cost", "mgt_cost", "Money", "MillionUsDollar", "C_ops",
  4.5, 0.1, 200.0,
  "below 100 thousand a year no ground segment and team exist",
  "above 200 million a year the programme is not the one costed here",
  "What does running the constellation cost each year?",
  "orbitt_case_c1", "programme", "A. Rai / 2026-09-01")

D("cost_inflation", "Annual inflation rate", "cost", "mgt_cost", "Ratio", "One", "i_r",
  0.030, 0.0, 0.15,
  "a negative rate is deflation, which the cost model has no basis for",
  "above 15% a year the estimate is dominated by the escalation rather than the design",
  "At what annual rate are the fitted cost relations escalated to the target year?",
  "nasa_cer", "programme", "A. Rai / 2026-09-01")

C("cost_bus_recurring", "Bus recurring cost, first unit", "cost", "mgt_cost", "Money", "MillionUsDollar", "C_bus",
  "What does the first spacecraft bus cost to build?",
  "C = a*m_dry^b, escalated",
  "nasa_cer", "programme",
  [("m", "mass_dry"), ("i", "cost_inflation")],
  [("evaluate the bus cost estimating relation on dry mass and escalate to 2026",
    "c", "Money", "cost::BUS_RECURRING_CER.evaluate_inflated(m.get(), 2026, i.get())")],
  0.0, 10000.0,
  "a cost cannot be negative",
  "above 10 billion for one bus the relation is being fed a mass it was not fitted on",
  tier="C",
  assumptions=[("The historical fit applies to this class of spacecraft",
                "the fit's one-sigma residual is 40%, which is larger than most of the design decisions it would be used to compare")])

C("cost_payload_recurring", "Payload recurring cost, first unit", "cost", "mgt_cost", "Money", "MillionUsDollar", "C_pay",
  "What does the first payload set cost to build?",
  "C = a*m_pay^b, escalated",
  "nasa_cer", "programme",
  [("m", "mass_payload"), ("i", "cost_inflation")],
  [("evaluate the payload cost estimating relation on payload mass and escalate to 2026",
    "c", "Money", "cost::PAYLOAD_RECURRING_CER.evaluate_inflated(m.get(), 2026, i.get())")],
  0.0, 10000.0,
  "a cost cannot be negative",
  "above 10 billion for one payload set the relation is being fed a mass it was not fitted on",
  tier="C")

C("cost_non_recurring", "Non-recurring engineering cost", "cost", "mgt_cost", "Money", "MillionUsDollar", "C_nre",
  "What does developing the design cost, once?",
  "C = a*m_dry^b, escalated",
  "nasa_cer", "programme",
  [("m", "mass_dry"), ("i", "cost_inflation")],
  [("evaluate the non-recurring cost estimating relation on dry mass and escalate to 2026",
    "c", "Money", "cost::NON_RECURRING_CER.evaluate_inflated(m.get(), 2026, i.get())")],
  0.0, 100000.0,
  "a cost cannot be negative",
  "above 100 billion the relation is being fed a mass it was not fitted on",
  tier="C")

C("cost_production", "Production run cost", "cost", "mgt_cost", "Money", "MillionUsDollar", "C_prod",
  "What does building the whole constellation cost?",
  "C = sum over n of T1*n^log2(b)",
  "nasa_cer", "programme",
  [("cb", "cost_bus_recurring"), ("cp", "cost_payload_recurring"), ("n", "mis_satellites"),
   ("b", "cost_learning_slope")],
  [("sum the learning curve over the production run of buses and payloads together",
    "c", "Money",
    "cost::production_run_cost(Money::new(cb.get() + cp.get()), n.get() as u32, b.get())")],
  0.0, 1000000.0,
  "a cost cannot be negative",
  "above a trillion dollars the programme is not the one being designed",
  tier="C")

C("cost_programme", "Total programme cost", "cost", "mgt_cost", "Money", "MillionUsDollar", "C_tot",
  "What does the whole programme cost over the mission?",
  "C = NRE + production + launch + operations*years",
  "nasa_cer", "programme",
  [("nre", "cost_non_recurring"), ("pr", "cost_production"), ("lc", "cost_launch_per_satellite"),
   ("n", "mis_satellites"), ("op", "cost_annual_operations"), ("y", "orbit_mission_duration")],
  [("add the non-recurring cost, the production run, the launch of every satellite and the operations over the mission",
    "c", "Money",
    "cost::programme_cost(nre, Money::new(pr.get() + lc.get() * n.get()), op, y.get() / 31_557_600.0)")],
  0.0, 1000000.0,
  "a cost cannot be negative",
  "above a trillion dollars the programme is not the one being designed",
  tier="C",
  kpis=["kpi_cost_per_year"])

C("cost_per_year", "Cost per operational year", "cost", "mgt_cost", "Money", "MillionUsDollar", "C_yr",
  "What does a year of service cost?",
  "C = C_total/years",
  "nasa_cer", "programme",
  [("c", "cost_programme"), ("y", "orbit_mission_duration")],
  [("divide the programme cost by the mission duration in years",
    "cy", "Money",
    "cost::cost_per_service_unit(c, y.get() / 31_557_600.0)")],
  0.0, 1000000.0,
  "a cost cannot be negative",
  "above a trillion dollars a year the programme is not the one being designed",
  tier="C",
  kpis=["kpi_cost_per_year"])

C("cost_replacement_avoided", "Replacement cost avoided", "cost", "mgt_cost", "Money", "MillionUsDollar", "C_avd",
  "What does air-breathing propulsion save by removing the need to replace satellites when propellant runs out?",
  "C = (C_unit + C_launch)*N_replacements",
  "orbitt_case_c1", "programme",
  [("cb", "cost_bus_recurring"), ("cp", "cost_payload_recurring"), ("lc", "cost_launch_per_satellite"),
   ("n", "mis_satellites"), ("td", "prop_thrust_to_drag")],
  [("count one avoided replacement of the whole constellation when thrust exceeds drag, and none when it does not",
    "c", "Money",
    "cost::replacement_avoided(Money::new(cb.get() + cp.get()), lc, if td.get() >= 1.0 { n.get() } else { 0.0 })")],
  0.0, 1000000.0,
  "a saving cannot be negative",
  "above a trillion dollars the programme is not the one being designed",
  tier="C",
  note="The commercial argument for the whole programme, as a node with a source and evidence rather than a slide.")
