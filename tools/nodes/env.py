#!/usr/bin/env python3
"""Space environment.

One subsystem's rows, and nothing else. Split out of the seeder so that
changing what this subsystem declares does not mean opening the file that
declares every other one — the same reason each subsystem has its own crate.

The helpers come from `seed_helpers`, which owns the shared lists these
append to. Import order is authoring order, so `seed_tree` decides it and this
file only says what the rows are.
"""

from seed_helpers import *          # noqa: F401,F403  the row helpers: D, C, layer, S, unnamed

# =============================================================================
# SPACE ENVIRONMENT
# The regime nobody models well, and the one the programme lives in.
# =============================================================================
D("env_f107", "Solar radio flux F10.7, daily", "env", "space_env", "Ratio", "One", "F107",
  150.0, 60.0, 400.0,
  "below 60 sfu has never been observed; the fit has no support there",
  "above 400 sfu is beyond the largest recorded daily value, so the temperature relation is extrapolated",
  "How active is the Sun today, on the 10.7 cm radio flux index?",
  "noaa_swpc", "environment", "A. Rai / 2026-09-01",
  note="Shipped climatology stands in when no solar-drivers bundle is synced, and the run is marked amber.")

D("env_f107a", "Solar radio flux F10.7, 81-day mean", "env", "space_env", "Ratio", "One", "F107A",
  150.0, 60.0, 400.0,
  "same support limit as the daily value",
  "same support limit as the daily value",
  "Where has the Sun been sitting over the last three solar rotations?",
  "noaa_swpc", "environment", "A. Rai / 2026-09-01",
  note="Separate from the daily value on purpose: the mean sets where the atmosphere sits, the daily value sets how far it is from there today.")

D("env_kp", "Planetary geomagnetic index Kp", "env", "space_env", "Ratio", "One", "Kp",
  3.0, 0.0, 9.0,
  "Kp is defined on 0..9; a negative index is not a quiet day, it is a unit error",
  "Kp is defined on 0..9. This is the guard that catches an Ap value passed in by mistake, which would otherwise return an exospheric temperature of 1e18 K without complaint",
  "How disturbed is the geomagnetic field?",
  "noaa_swpc", "environment", "A. Rai / 2026-09-01")

D("env_wall_temperature", "Spacecraft surface temperature", "env", "space_env", "Temperature", "Kelvin", "T_w",
  300.0, 150.0, 500.0,
  "below 150 K no external surface of this design is predicted to fall, and the accommodation fit has no data there",
  "above 500 K the surface materials are outside their qualification range, so a drag figure computed there is meaningless anyway",
  "What temperature are the surfaces the atmosphere strikes?",
  "sentman1961", "thermal", "A. Rai / 2026-09-01")

D("env_chamber_temperature", "Intake chamber gas temperature", "env", "space_env", "Temperature", "Kelvin", "T_c",
  600.0, 300.0, 1500.0,
  "the chamber cannot be colder than the walls that bound it",
  "above 1500 K the assumption that the chamber gas is thermalised and un-ionised stops holding",
  "What temperature does the collected gas reach in the collection chamber?",
  "romano2021", "propulsion", "A. Rai / 2026-09-01",
  note="Drives the mean thermal speed, and therefore back-flow: a hotter chamber leaks harder.")

C("env_exospheric_temperature", "Exospheric temperature", "env", "space_env", "Temperature", "Kelvin", "T_inf",
  "How hot is the upper thermosphere today, given the Sun and the geomagnetic field?",
  "T_inf = 379 + 3.24*F10.7A + 1.3*(F10.7 - F10.7A) + 28*Kp + 0.03*exp(Kp)",
  "jacchia1971", "environment",
  [("f107", "env_f107"), ("f107a", "env_f107a"), ("kp", "env_kp")],
  [("apply the Jacchia 1971 night-time minimum relation with its geomagnetic correction",
    "t_inf", "Temperature", "env::exospheric_temperature(f107.get(), f107a.get(), kp.get())")],
  400.0, 2500.0,
  "no observed thermosphere is colder than 400 K; below that the Bates profile inverts",
  "above 2500 K is beyond any recorded storm and beyond the fit",
  tier="A",
  assumptions=[("Night-time minimum, with no diurnal or seasonal term",
                "the diurnal bulge adds up to 30% at 14:00 local solar time; a design sized on this value alone is sized on the quiet side")],
  fixtures=[("solar minimum, quiet", {"f107": 70.0, "f107a": 70.0, "kp": 1.0}, 633.9, 2e-3, "published-source", "jacchia1971"),
            ("moderate activity", {"f107": 150.0, "f107a": 150.0, "kp": 3.0}, 949.6, 2e-3, "published-source", "jacchia1971"),
            ("solar maximum, storm", {"f107": 250.0, "f107a": 250.0, "kp": 7.0}, 1417.9, 2e-3, "published-source", "jacchia1971")],
  view=("line", {"over": "env_f107", "points": 60}))

C("env_local_temperature", "Local kinetic temperature", "env", "space_env", "Temperature", "Kelvin", "T",
  "What is the gas temperature at the flight altitude?",
  "T(z) = T_inf - (T_inf - T_120)*exp(-s*(z - z_120))",
  "jacchia1971", "environment",
  [("h", "orbit_altitude"), ("t_inf", "env_exospheric_temperature")],
  [("evaluate the Bates profile between the 120 km base and the exospheric limit",
    "t", "Temperature", "env::temperature(h, t_inf)")],
  200.0, 2500.0,
  "below the 120 km base temperature the profile is not defined",
  "cannot exceed the exospheric temperature it approaches",
  tier="A")

C("env_mass_density", "Atmospheric mass density", "env", "space_env", "MassDensity", "KgPerCubicMetre", "rho",
  "How much air is there at the flight altitude, in kilograms per cubic metre?",
  "rho = SUM_i n_i(z)*m_i,  n_i(z) = n_i(z0)*(T0/T)^(1+alpha_i)*exp(-INT M_i*g/(R*T) dz)",
  "jacchia1971", "environment",
  [("h", "orbit_altitude"), ("t_inf", "env_exospheric_temperature")],
  [("integrate diffusive equilibrium from the 120 km base for every species and sum the masses",
    "rho", "MassDensity", "env::mass_density(h, t_inf)")],
  1e-15, 1e-6,
  "below 1e-15 kg/m3 the free-molecular relations downstream stop producing a meaningful drag",
  "above 1e-6 kg/m3 the flow is no longer free-molecular and every aerodynamic node in this tree is outside its envelope",
  tier="B",
  assumptions=[("Diffusive equilibrium above 120 km, five species",
                "below about 100 km turbulent mixing dominates and species do not separate; the model is not valid there"),
               ("No diurnal, seasonal or longitudinal variation",
                "the real density at a fixed altitude varies by a factor of two around the orbit; this returns the daily mean")],
  fixtures=[("250 km, moderate activity", {"h": 250000.0, "t_inf": 949.6}, 6.631e-11, 5e-3, "independent-tool", "matlab_legacy"),
            ("400 km, moderate activity", {"h": 400000.0, "t_inf": 949.6}, 2.907e-12, 5e-3, "independent-tool", "matlab_legacy"),
            ("200 km, solar minimum", {"h": 200000.0, "t_inf": 633.9}, 1.679e-10, 5e-3, "independent-tool", "matlab_legacy")],
  kpis=["kpi_service_lifetime"],
  view=("line", {"over": "orbit_altitude", "points": 80}),
  bundles=["solar-drivers"])

C("env_number_density", "Total number density", "env", "space_env", "NumberDensity", "PerCubicMetre", "n",
  "How many particles per cubic metre are there at the flight altitude?",
  "n = SUM_i n_i(z)",
  "jacchia1971", "environment",
  [("h", "orbit_altitude"), ("t_inf", "env_exospheric_temperature")],
  [("sum the per-species diffusive-equilibrium densities",
    "n", "NumberDensity", "env::composition(h, t_inf).total()")],
  1e8, 1e20,
  "below 1e8 per cubic metre there is nothing for an intake to collect",
  "above 1e20 the flow is continuum and the intake model does not apply",
  tier="B")

C("env_mean_molar_mass", "Mean molar mass of the local gas", "env", "space_env", "MolarMass", "KgPerMol", "M",
  "What is the mean molar mass of the air the vehicle is flying through?",
  "M = rho*N_A/n",
  "jacchia1971", "environment",
  [("h", "orbit_altitude"), ("t_inf", "env_exospheric_temperature")],
  [("take the mass-weighted mean over the five species",
    "m", "MolarMass", "env::composition(h, t_inf).mean_molar_mass()")],
  0.002, 0.030,
  "below 2 g/mol the mixture would be pure hydrogen, which does not occur in this band",
  "above 30 g/mol the mixture would be heavier than molecular nitrogen, which does not occur above the base",
  tier="B",
  note="Falls from 26 g/mol at 120 km to near 16 — pure atomic oxygen — by 350 km. Every free-molecular relation downstream reads it.")

C("env_atomic_oxygen_density", "Atomic oxygen number density", "env", "space_env", "NumberDensity", "PerCubicMetre", "n_O",
  "How much atomic oxygen is there — the species that erodes materials and the one an intake actually collects?",
  "n_O(z) = n_O(z0)*(T0/T)*exp(-INT M_O*g/(R*T) dz)",
  "jacchia1971", "environment",
  [("h", "orbit_altitude"), ("t_inf", "env_exospheric_temperature")],
  [("integrate diffusive equilibrium for atomic oxygen alone",
    "n_o", "NumberDensity", "env::composition(h, t_inf).o")],
  1e8, 1e19,
  "below 1e8 there is effectively no atomic oxygen and the accommodation model returns a specular surface",
  "above 1e19 is denser than the total number density at the model base",
  tier="B",
  kpis=["kpi_service_lifetime"])

C("env_scale_height", "Density scale height", "env", "space_env", "Length", "Kilometre", "H",
  "Over what altitude change does the density fall by a factor of e?",
  "H = R*T/(M*g)",
  "vallado2013", "environment",
  [("h", "orbit_altitude"), ("t_inf", "env_exospheric_temperature")],
  [("form the barometric scale height from the local temperature, mean molar mass and gravity",
    "hs", "Length", "env::scale_height(h, t_inf)")],
  5.0, 200.0,
  "a scale height below 5 km does not occur above the model base",
  "above 200 km the atmosphere is effectively isothermal and the concept stops being useful",
  tier="A")

C("env_mean_free_path", "Mean free path", "env", "space_env", "Length", "Metre", "lambda",
  "How far does a molecule travel between collisions?",
  "lambda = 1/(sqrt(2)*n*sigma)",
  "us_std_1976", "environment",
  [("n", "env_number_density")],
  [("apply the hard-sphere mean free path with an effective cross-section of 1e-19 m2",
    "l", "Length", "env::mean_free_path(n)")],
  0.001, 1e9,
  "a mean free path below a millimetre would put a spacecraft-scale body in continuum flow",
  "beyond 1e9 m the concept of a mean free path has no operational meaning here",
  tier="A")

C("env_knudsen", "Knudsen number", "env", "space_env", "Ratio", "One", "Kn",
  "Is the flow free-molecular — which is what licenses every aerodynamic relation in this tree?",
  "Kn = lambda/L",
  "us_std_1976", "environment",
  [("lam", "env_mean_free_path"), ("l_body", "aero_body_length")],
  [("divide the mean free path by the vehicle's characteristic length",
    "kn", "Ratio", "env::knudsen(lam, l_body)")],
  10.0, 1e12,
  "below Kn = 10 the flow is transitional and Sentman's free-molecular relations are outside their envelope. This bound is the Ariane 501 lesson written as a guard: a component used outside the envelope it was specified for, with nothing re-derived",
  "above 1e12 the number is meaningless but harmless",
  tier="A",
  kpis=[])

C("env_density_uncertainty", "Density model uncertainty", "env", "space_env", "Ratio", "One", "sigma_rho",
  "How wrong might the density be, and therefore how wrong is every margin built on it?",
  "sigma_rho = 0.15 + 0.03*Kp",
  "doornbos2011", "environment",
  [("kp", "env_kp")],
  [("apply the quiet-time residual and grow it with geomagnetic activity",
    "s", "Ratio", "env::density_uncertainty(kp.get())")],
  0.05, 1.0,
  "no published comparison of an empirical thermosphere model against measured drag does better than 5%",
  "an uncertainty above 100% of the value is not an uncertainty, it is an absence of a model",
  tier="A",
  note="A node that consumes density and does not carry this forward is a node whose margin is fictional.")

C("env_magnetic_field", "Geomagnetic flux density", "env", "space_env", "MagneticFluxDensity", "Tesla", "B",
  "How strong is the magnetic field at the flight altitude?",
  "B = B0*(Re/r)^3*sqrt(1 + 3*sin^2(lat_m))",
  "igrf2020", "environment",
  [("r", "orbit_radius"), ("lat_m", "gnc_magnetic_latitude")],
  [("evaluate the tilted dipole approximation at the orbital radius",
    "b", "MagneticFluxDensity", "env::magnetic_field(r, lat_m)")],
  1e-6, 1e-4,
  "the field never falls below a microtesla at these altitudes",
  "the surface equatorial field is 3.1e-5 T; 1e-4 would be four times the polar surface value",
  tier="A",
  assumptions=[("Centred dipole, no higher-order terms",
                "the South Atlantic Anomaly departs from a dipole by up to 30%; a magnetorquer sized on this alone is undersized there")])
