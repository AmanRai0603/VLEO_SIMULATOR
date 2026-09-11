#!/usr/bin/env python3
"""Guidance, navigation and control.

One subsystem's rows, and nothing else. Split out of the seeder so that
changing what this subsystem declares does not mean opening the file that
declares every other one — the same reason each subsystem has its own crate.

The helpers come from `seed_helpers`, which owns the shared lists these
append to. Import order is authoring order, so `seed_tree` decides it and this
file only says what the rows are.
"""

from seed_helpers import *          # noqa: F401,F403  the row helpers: D, C, layer, S, unnamed

# =============================================================================
# GUIDANCE, NAVIGATION AND CONTROL
# =============================================================================
D("gnc_magnetic_latitude", "Magnetic latitude", "gnc", "gnc", "Angle", "Degree", "lat_m",
  45.0, -90.0, 90.0,
  "magnetic latitude is defined on -90..90",
  "magnetic latitude is defined on -90..90",
  "At what magnetic latitude is the field being evaluated?",
  "igrf2020", "gnc", "A. Rai / 2026-09-01")

D("gnc_residual_dipole", "Residual magnetic dipole", "gnc", "gnc", "DipoleMoment", "AmpereSquareMetre", "m_res",
  0.05, 0.0, 5.0,
  "a residual dipole magnitude cannot be negative",
  "above 5 A.m2 the magnetic disturbance dominates every other torque and the design is uncontrollable",
  "How much uncompensated magnetic moment does the spacecraft carry?",
  "larson_wertz", "gnc", "A. Rai / 2026-09-01")

D("gnc_inertia_max", "Largest principal moment of inertia", "gnc", "gnc", "Ratio", "One", "I_max",
  8.0, 0.01, 1000.0,
  "below 0.01 kg.m2 there is no vehicle",
  "above 1000 kg.m2 the vehicle is not in this mass class",
  "What is the largest principal moment of inertia?",
  "orbitt_case_c1", "gnc", "A. Rai / 2026-09-01")

D("gnc_inertia_min", "Smallest principal moment of inertia", "gnc", "gnc", "Ratio", "One", "I_min",
  1.2, 0.01, 1000.0,
  "below 0.01 kg.m2 there is no vehicle",
  "cannot exceed the largest principal moment",
  "What is the smallest principal moment of inertia?",
  "orbitt_case_c1", "gnc", "A. Rai / 2026-09-01")

D("gnc_sensor_noise", "Attitude sensor noise", "gnc", "gnc", "Angle", "Arcsecond", "sig_sen",
  12.0, 0.5, 3600.0,
  "below 0.5 arcsec no star tracker in this class performs that well",
  "above one degree the sensor does not support any pointing requirement in this design",
  "How noisy is the attitude measurement?",
  "larson_wertz", "gnc", "A. Rai / 2026-09-01")

D("gnc_alignment_error", "Payload to sensor alignment error", "gnc", "gnc", "Angle", "Arcsecond", "sig_ali",
  30.0, 1.0, 3600.0,
  "below 1 arcsec no mechanical alignment survives launch and thermal cycling",
  "above one degree the alignment does not support any pointing requirement in this design",
  "How well is the payload aligned to the attitude sensor after launch and thermal cycling?",
  "larson_wertz", "gnc", "A. Rai / 2026-09-01")

D("gnc_control_error", "Attitude control error", "gnc", "gnc", "Angle", "Arcsecond", "sig_ctl",
  40.0, 1.0, 7200.0,
  "below 1 arcsec no wheel-based controller holds that in the presence of VLEO aerodynamic torque",
  "above two degrees the controller does not support any pointing requirement in this design",
  "How well does the controller hold the commanded attitude?",
  "larson_wertz", "gnc", "A. Rai / 2026-09-01")

D("gnc_thermal_distortion", "Thermal distortion error", "gnc", "gnc", "Angle", "Arcsecond", "sig_thm",
  20.0, 0.0, 3600.0,
  "a distortion cannot be negative",
  "above one degree the structure is not stable enough for any payload in this design",
  "How much does the structure move between the sensor and the payload over an orbit?",
  "larson_wertz", "gnc", "A. Rai / 2026-09-01")

D("gnc_user_range_error", "GNSS user range error", "gnc", "gnc", "Length", "Metre", "URE",
  1.5, 0.05, 50.0,
  "below 5 cm requires carrier-phase processing that this receiver does not perform on board",
  "above 50 m the receiver is not usable for orbit determination",
  "How accurate is one pseudorange measurement?",
  "larson_wertz", "gnc", "A. Rai / 2026-09-01")

D("gnc_gdop", "Geometric dilution of precision", "gnc", "gnc", "Ratio", "One", "GDOP",
  2.5, 1.0, 20.0,
  "a dilution below one is not geometrically possible",
  "above 20 the geometry is degenerate and the solution is not usable",
  "How much does the satellite geometry amplify ranging error into position error?",
  "larson_wertz", "gnc", "A. Rai / 2026-09-01")

D("gnc_dump_time", "Momentum dump interval", "gnc", "gnc", "Time", "Minute", "t_dump",
  45.0, 1.0, 1440.0,
  "below a minute the magnetorquers are being commanded faster than the field geometry changes",
  "above a day the wheels saturate long before the dump",
  "How long may the wheels accumulate momentum before it is dumped?",
  "larson_wertz", "gnc", "A. Rai / 2026-09-01")

C("gnc_gravity_gradient_torque", "Gravity gradient torque", "gnc", "gnc", "Torque", "NewtonMetre", "T_gg",
  "How much torque does the gravity gradient apply?",
  "T = (3*mu/(2*r^3))*|Iz - Iy|*sin(2*theta)",
  "larson_wertz", "gnc",
  [("r", "orbit_radius"), ("imax", "gnc_inertia_max"), ("imin", "gnc_inertia_min")],
  [("evaluate the gravity gradient torque at a 10 degree offset from local vertical",
    "t", "Torque",
    "gnc::gravity_gradient_torque(r, imax.get(), imin.get(), Angle::from_deg(10.0))")],
  0.0, 1.0,
  "a torque magnitude cannot be negative",
  "above 1 N.m no wheel that fits this vehicle can hold attitude",
  tier="A")

C("gnc_solar_torque", "Solar radiation pressure torque", "gnc", "gnc", "Torque", "NewtonMetre", "T_srp",
  "How much torque does solar radiation pressure apply?",
  "T = (S/c)*A*(1+q)*cos(theta)*x_cp",
  "larson_wertz", "gnc",
  [("a", "aero_frontal_area"), ("x", "aero_cp_cm_offset"), ("th", "pwr_array_incidence")],
  [("apply solar radiation pressure over the illuminated area with a reflectivity of 0.6",
    "t", "Torque",
    "gnc::solar_pressure_torque(a, Ratio::new(0.6), x, th)")],
  0.0, 1.0,
  "a torque magnitude cannot be negative",
  "above 1 N.m no wheel that fits this vehicle can hold attitude",
  tier="A")

C("gnc_magnetic_torque", "Residual dipole torque", "gnc", "gnc", "Torque", "NewtonMetre", "T_mag",
  "How much torque does the residual magnetic dipole produce in the geomagnetic field?",
  "T = m_res x B",
  "larson_wertz", "gnc",
  [("m", "gnc_residual_dipole"), ("b", "env_magnetic_field")],
  [("take the worst-case cross product of the residual dipole with the local field",
    "t", "Torque", "gnc::magnetic_torque(m, b)")],
  0.0, 1.0,
  "a torque magnitude cannot be negative",
  "above 1 N.m no wheel that fits this vehicle can hold attitude",
  tier="A")

C("gnc_total_disturbance", "Total disturbance torque", "gnc", "gnc", "Torque", "NewtonMetre", "T_dis",
  "What is the worst-case disturbance torque the control system has to answer?",
  "T = |T_aero| + |T_gg| + |T_srp| + |T_mag|",
  "larson_wertz", "gnc",
  [("ta", "aero_torque"), ("tg", "gnc_gravity_gradient_torque"), ("ts", "gnc_solar_torque"),
   ("tm", "gnc_magnetic_torque")],
  [("sum the magnitudes rather than root-sum-squaring them",
    "t", "Torque", "gnc::total_disturbance_torque(ta, tg, ts, tm)")],
  0.0, 5.0,
  "a torque magnitude cannot be negative",
  "above 5 N.m the vehicle is uncontrollable by any actuator that fits it",
  tier="A",
  note="Summed, not root-sum-squared. These are biases that can and do align, and a wheel sized on their RSS saturates the first time they do.")

C("gnc_momentum_storage", "Momentum storage required", "gnc", "gnc", "AngularMomentum", "NewtonMetreSecond", "h_req",
  "How much angular momentum must the wheels be able to store?",
  "h = T*T_orb/2*(1 + margin)",
  "larson_wertz", "gnc",
  [("t", "gnc_total_disturbance"), ("p", "orbit_period")],
  [("accumulate the secular torque over half a revolution with a 50% margin",
    "h", "AngularMomentum",
    "gnc::momentum_storage_required(t, p, Ratio::new(0.5))")],
  0.0, 100.0,
  "a momentum magnitude cannot be negative",
  "above 100 N.m.s no wheel assembly that fits this vehicle stores it",
  tier="A")

C("gnc_magnetorquer_dipole", "Magnetorquer dipole required", "gnc", "gnc", "DipoleMoment", "AmpereSquareMetre", "m_req",
  "How strong must the torque rods be to dump that momentum?",
  "m = h/(B*t_dump)",
  "larson_wertz", "gnc",
  [("h", "gnc_momentum_storage"), ("b", "env_magnetic_field"), ("td", "gnc_dump_time")],
  [("divide the stored momentum by the field and the available dump interval",
    "m", "DipoleMoment", "gnc::magnetorquer_dipole_required(h, b, td)")],
  0.0, 1000.0,
  "a dipole magnitude cannot be negative",
  "above 1000 A.m2 no torque rod that fits this vehicle produces it",
  tier="A")

C("gnc_pointing_error", "Pointing error, three sigma", "gnc", "gnc", "Angle", "Arcsecond", "sig_pt",
  "How accurately does the payload point?",
  "sigma = sqrt(sum of squares of the independent contributors)",
  "larson_wertz", "gnc",
  [("sen", "gnc_sensor_noise"), ("ali", "gnc_alignment_error"), ("ctl", "gnc_control_error"),
   ("thm", "gnc_thermal_distortion")],
  [("root-sum-square the four independent contributors and scale to three sigma",
    "s", "Angle",
    "Angle::new(3.0 * gnc::pointing_error_rss(&[sen, ali, ctl, thm]).get())")],
  0.0, 10800.0,
  "a pointing error cannot be negative",
  "above three degrees no payload in this design produces a usable product",
  tier="A",
  kpis=["kpi_geolocation"],
  note="Root-sum-squared, unlike the disturbance torques above. These terms are genuinely independent, and summing them linearly would give a budget nothing could ever meet.",
  view=("bar", {"y": "gnc_pointing_error"}))

C("gnc_ground_pointing_error", "Ground pointing error", "gnc", "gnc", "Length", "Metre", "e_gnd",
  "How far off the ground does the pointing error put the aim point?",
  "e = sigma*d",
  "larson_wertz", "gnc",
  [("s", "gnc_pointing_error"), ("d", "orbit_slant_range")],
  [("multiply the pointing error by the slant range",
    "e", "Length", "gnc::pointing_to_ground_error(s, d)")],
  0.0, 100000.0,
  "a ground error cannot be negative",
  "above 100 km the pointing is not usable for any payload here",
  tier="A",
  kpis=["kpi_geolocation"])

C("gnc_nav_position_error", "Navigation position error", "gnc", "gnc", "Length", "Metre", "e_nav",
  "How well does the spacecraft know where it is?",
  "e = URE*GDOP",
  "larson_wertz", "gnc",
  [("u", "gnc_user_range_error"), ("g", "gnc_gdop")],
  [("multiply the user range error by the geometric dilution",
    "e", "Length", "gnc::navigation_position_error(u, g.get())")],
  0.0, 10000.0,
  "a position error cannot be negative",
  "above 10 km the navigation solution is not usable",
  tier="A",
  kpis=["kpi_geolocation"])

C("gnc_along_track_error", "Along-track prediction error after one day", "gnc", "gnc", "Length", "Metre", "e_at",
  "How far out is a one-day orbit prediction, given how wrong the density model is?",
  "e = 1.5*da*t^2",
  "doornbos2011", "gnc",
  [("a", "aero_drag_acceleration"), ("s", "env_density_uncertainty")],
  [("propagate the drag acceleration error, which is the density uncertainty applied to the drag, over one day",
    "e", "Length",
    "gnc::along_track_error_from_drag(Acceleration::new(a.get() * s.get()), Time::from_days(1.0))")],
  0.0, 1.0e7,
  "an error magnitude cannot be negative",
  "above 10000 km the prediction is meaningless and the answer is that the model is unusable there",
  tier="B",
  note="The term that dominates orbit prediction in this regime, and the reason the density uncertainty is a published output rather than a footnote.")
