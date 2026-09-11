#!/usr/bin/env python3
"""Mission performance.

One subsystem's rows, and nothing else. Split out of the seeder so that
changing what this subsystem declares does not mean opening the file that
declares every other one — the same reason each subsystem has its own crate.

The helpers come from `seed_helpers`, which owns the shared lists these
append to. Import order is authoring order, so `seed_tree` decides it and this
file only says what the rows are.
"""

from seed_helpers import *          # noqa: F401,F403  the row helpers: D, C, layer, S, unnamed

# =============================================================================
# MISSION PERFORMANCE
# =============================================================================
D("mis_satellites", "Number of satellites", "mis", "mission", "Ratio", "One", "N_sat",
  24.0, 1.0, 500.0,
  "one satellite is the single-satellite concept, the limit of the trade",
  "above 500 the constellation is not the one costed here",
  "How many satellites are in the constellation?",
  "orbitt_case_c1", "systems", "A. Rai / 2026-09-01",
  kpis=["kpi_revisit", "kpi_cost_per_year"])

D("mis_processing_time", "Ground processing time", "mis", "mission", "Time", "Minute", "t_proc",
  8.0, 0.1, 1440.0,
  "below six seconds no ground processing chain in this design runs",
  "above a day the product is not the near-real-time one being sold",
  "How long does ground processing take after the data lands?",
  "orbitt_case_c1", "operations", "A. Rai / 2026-09-01",
  kpis=["kpi_latency"])

D("mis_delivery_time", "Delivery time to the customer", "mis", "mission", "Time", "Minute", "t_del",
  3.0, 0.1, 1440.0,
  "below six seconds no delivery path in this design runs",
  "above a day the product is not the near-real-time one being sold",
  "How long does the product take to reach the customer once processed?",
  "orbitt_case_c1", "operations", "A. Rai / 2026-09-01",
  kpis=["kpi_latency"])

D("mis_unit_availability", "Single satellite availability", "mis", "mission", "Ratio", "One", "A_unit",
  0.97, 0.5, 0.9999,
  "below 50% the satellite is unavailable more often than not",
  "above 0.9999 no small satellite has demonstrated that availability",
  "What fraction of the time is one satellite fully operational?",
  "orbitt_case_c1", "operations", "A. Rai / 2026-09-01",
  kpis=["kpi_availability"])

C("mis_access_area", "Instantaneous access area", "mis", "mission", "Area", "SquareMetre", "A_acc",
  "How much ground does one satellite see at any instant?",
  "A = 2*pi*Re^2*(1 - cos(lambda))",
  "larson_wertz", "systems",
  [("lam", "orbit_earth_central_angle")],
  [("integrate the spherical cap subtended by the Earth-central half-angle",
    "a", "Area", "mission::access_area(lam)")],
  0.0, 5.2e14,
  "an area cannot be negative",
  "cannot exceed the surface area of the Earth",
  tier="A")

C("mis_coverage_fraction", "Instantaneous coverage fraction", "mis", "mission", "Ratio", "One", "f_cov",
  "What fraction of the Earth does one satellite see at any instant?",
  "f = (1 - cos(lambda))/2",
  "larson_wertz", "systems",
  [("lam", "orbit_earth_central_angle")],
  [("take the spherical cap as a fraction of the whole sphere",
    "f", "Ratio", "mission::instantaneous_coverage_fraction(lam)")],
  0.0, 1.0,
  "a fraction cannot be negative",
  "a satellite cannot see more than the whole Earth",
  tier="A")

C("mis_revisit", "Mean revisit time", "mis", "mission", "Time", "Hour", "t_rev",
  "How often does the constellation come back to a given place?",
  "t = A_earth/(N*W*V_g)",
  "larson_wertz", "systems",
  [("w", "pay_swath"), ("v", "orbit_ground_track_speed"), ("n", "mis_satellites")],
  [("divide the Earth's surface area by the total swath sweep rate of the constellation",
    "t", "Time", "mission::mean_revisit_time(w, v, n.get())")],
  0.0, 2400.0,
  "a revisit time cannot be negative",
  "above 100 days the constellation does not provide a service",
  tier="C",
  kpis=["kpi_revisit"],
  assumptions=[("Uniform coverage of the whole globe, no latitude weighting",
                "a Sun-synchronous constellation revisits the poles far more often than the equator, so this mean is optimistic at low latitudes and pessimistic at high ones")],
  view=("line", {"over": "mis_satellites", "points": 60}))

C("mis_satellites_required", "Satellites required for the revisit target", "mis", "mission", "Ratio", "One", "N_req",
  "How many satellites would the revisit requirement need?",
  "N = A_earth/(t_target*W*V_g)",
  "larson_wertz", "systems",
  [("t", "kpi_revisit_required"), ("w", "pay_swath"), ("v", "orbit_ground_track_speed")],
  [("invert the revisit relation for the satellite count",
    "n", "Ratio",
    "Ratio::new(mission::satellites_for_revisit(t, w, v))")],
  0.0, 100000.0,
  "a satellite count cannot be negative",
  "above 100000 the concept is not the one being designed",
  tier="C",
  note="The inverse question, and the one a concept trade actually asks.")

C("mis_time_to_downlink", "Mean time to next downlink", "mis", "mission", "Time", "Minute", "t_dl",
  "How long does data wait on board before it can come down?",
  "t = 86400/(2*N_pass)",
  "larson_wertz", "operations",
  [("n", "com_passes_per_day")],
  [("take half the mean interval between passes",
    "t", "Time", "mission::mean_time_to_downlink(n.get())")],
  0.0, 1440.0,
  "a wait cannot be negative",
  "above a day the mission is not operable",
  tier="A",
  kpis=["kpi_latency"])

C("mis_latency", "End to end latency", "mis", "mission", "Time", "Minute", "t_lat",
  "How long from collection to the product reaching the customer?",
  "t = t_wait + t_downlink + t_process + t_deliver",
  "orbitt_case_c1", "operations",
  [("tw", "mis_time_to_downlink"), ("td", "com_contact_time"), ("tp", "mis_processing_time"),
   ("tdl", "mis_delivery_time")],
  [("sum the wait, the downlink, the processing and the delivery",
    "t", "Time", "mission::end_to_end_latency(tw, td, tp, tdl)")],
  0.0, 2880.0,
  "a latency cannot be negative",
  "above two days the product is not the near-real-time one being sold",
  tier="B",
  kpis=["kpi_latency"],
  note="The wait term dominates, and it is the one a constellation design can actually move.")

C("mis_availability", "Constellation service availability", "mis", "mission", "Ratio", "One", "A_svc",
  "What fraction of the time is the service available?",
  "A = P(at least ceil(0.8*N) of N satellites up)",
  "larson_wertz", "operations",
  [("a", "mis_unit_availability"), ("n", "mis_satellites")],
  [("evaluate the binomial probability that at least 80% of the constellation is operational",
    "av", "Ratio",
    "mission::k_of_n_availability(a, n.get() as u32, ((n.get() * 0.8).ceil()) as u32)")],
  0.0, 1.0,
  "an availability cannot be negative",
  "an availability cannot exceed one",
  tier="B",
  kpis=["kpi_availability"])
