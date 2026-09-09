#!/usr/bin/env python3
"""
Seed the node tree.

`xtask seed` in the delivery plan: run once, ever. It writes one folder per
node — the sheet, the initial hole bodies and the fixture table — and after
that the sheets are the source and this script is history. It is kept in the
repository because the alternative is that nobody can tell, in two years,
whether a field was authored or inherited from a seed.

Folder names are frozen here and recorded in the sheet. Editing a label never
moves a directory: renaming a heading would otherwise show up in version
control as hundreds of deletes and adds and would conflict with every open
branch.

Run:  python3 tools/seed_tree.py
"""
import os
import textwrap

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
NODES = []
LAYERS = {}


def esc(s):
    return s.replace("\\", "\\\\").replace('"', '\\"')


def toml_str(s):
    return '"%s"' % esc(s)


class Node(dict):
    pass


def D(nid, label, subsystem, parent, ty, unit, symbol, value, lo, hi, rlo, rhi,
      q, src, owner, conf, tier="A", kpis=(), note=""):
    """A declared value — a number a person picked.

    Two thirds of the tree is these. They are cheaper than a computed node and
    they are not free: a value with no unit, no range, no reason per bound, no
    source and no confirmation is an opinion that every margin downstream is
    built out of.
    """
    NODES.append(Node(
        id=nid, label=label, subsystem=subsystem, parent=parent, kind="declared",
        ty=ty, unit=unit, symbol=symbol, value=value, lo=lo, hi=hi,
        rlo=rlo, rhi=rhi, question=q, source=src, owner=owner, confirmed=conf,
        tier=tier, kpis=list(kpis), note=note, expression="%s = %g" % (symbol, value),
        assumptions=[], steps=[], inputs=[], fixtures=[], view=("number", {}), bundles=[]))


def C(nid, label, subsystem, parent, ty, unit, symbol, q, expr, src, owner,
      ins, steps, lo, hi, rlo, rhi, tier="B", assumptions=(), fixtures=(),
      kpis=(), view=("number", {}), bundles=(), kind="computed", note=""):
    """A computed node — one small question with one answer."""
    NODES.append(Node(
        id=nid, label=label, subsystem=subsystem, parent=parent, kind=kind,
        ty=ty, unit=unit, symbol=symbol, question=q, expression=expr, source=src,
        owner=owner, inputs=list(ins), steps=list(steps), lo=lo, hi=hi,
        rlo=rlo, rhi=rhi, tier=tier, assumptions=list(assumptions),
        fixtures=list(fixtures), kpis=list(kpis), view=view,
        bundles=list(bundles), confirmed="", value=None, note=note))


def layer(gid, label, parent, owner, relates=()):
    LAYERS[gid] = dict(id=gid, label=label, parent=parent, owner=owner,
                       relates=list(relates))


# =============================================================================
# The layer files — the 296 rows in the tree that are not nodes.
# Headings, their relations, and subsystem ownership, from which CODEOWNERS is
# generated. Eight files, changed rarely, reviewed by two people: moving a
# branch moves everyone's work.
# =============================================================================
layer("root", "VLEO multipayload programme", "", "systems")

layer("management", "Management layer", "root", "systems",
      [("management", "system", "the cost model can only run over a selected design")])
layer("mgt_customer", "Customer requirement and KPIs", "management", "systems")
layer("mgt_programme", "Programme control", "management", "systems")
layer("mgt_cost", "Cost and value case", "management", "systems")

layer("system", "The system", "root", "systems")
layer("applications", "Applications and KPIs", "system", "systems",
      [("applications", "payload", "every KPI resolves to a payload chain")])
layer("mission", "Mission performance", "system", "systems",
      [("mission", "orbit_geometry", "revisit and coverage are geometry before they are anything else")])
layer("orbit_env", "Orbit and environment", "system", "systems")
layer("orbit_geometry", "Orbit geometry", "orbit_env", "systems")
layer("space_env", "Space environment", "orbit_env", "environment",
      [("space_env", "aero_drag", "density is the input the whole drag chain stands on")])
layer("satellite", "Satellite system", "system", "systems",
      [("satellite", "propulsion", "thruster demand sets the power budget"),
       ("satellite", "power", "the array is both the power source and a drag surface")])
layer("mass_aero", "Mass and aerodynamics", "satellite", "mass")
layer("aero_drag", "Aerodynamics and drag", "satellite", "aero")
layer("subsystems", "Satellite subsystems", "system", "systems")
layer("propulsion", "Propulsion — air-breathing electric", "subsystems", "propulsion",
      [("propulsion", "power", "the thruster is the largest single load on the bus")])
layer("prop_intake", "Air intake", "propulsion", "propulsion")
layer("prop_thruster", "Thruster and power processing", "propulsion", "propulsion")
layer("power", "Power", "subsystems", "power",
      [("power", "thermal", "everything the bus draws has to leave as heat")])
layer("thermal", "Thermal", "subsystems", "thermal")
layer("gnc", "Guidance, navigation and control", "subsystems", "gnc",
      [("gnc", "aero_drag", "aerodynamic torque is the dominant disturbance in this regime")])
layer("comms", "TT&C and downlink", "subsystems", "comms")
layer("payload", "Service payloads", "system", "payload")
layer("pay_optical", "EO optical", "payload", "payload")
layer("pay_rf", "RF geolocation", "payload", "payload")
layer("closure", "Closure — required against achieved", "system", "systems",
      [("closure", "propulsion", "thrust against drag is the closure the design exists to reach")])


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


# =============================================================================
# ORBIT AND GEOMETRY
# =============================================================================
D("orbit_altitude", "Orbit altitude", "orbit", "orbit_geometry", "Length", "Kilometre", "h",
  250.0, 150.0, 450.0,
  "below 150 km the flow stops being free-molecular for a body of this size and every aerodynamic node in the tree is outside its envelope",
  "above 450 km there is not enough air for the air-breathing concept to be the reason for the design",
  "At what altitude does the constellation fly? This is the one parameter the whole architecture is shaped around.",
  "orbitt_case_c1", "systems", "A. Rai / 2026-09-01",
  kpis=["kpi_revisit", "kpi_gsd", "kpi_service_lifetime"],
  note="Move this one value and 54 other nodes become stale. That is the requirement the tool exists to satisfy.")

D("orbit_eccentricity", "Orbit eccentricity", "orbit", "orbit_geometry", "Ratio", "One", "e",
  0.0, 0.0, 0.05,
  "a negative eccentricity is not an orbit",
  "above 0.05 the perigee of a 250 km orbit is below 200 km, where the drag per revolution stops being a small perturbation and the circular-orbit relations used throughout this tree no longer apply",
  "How circular is the orbit?",
  "orbitt_case_c1", "systems", "A. Rai / 2026-09-01",
  note="Moving this reaches 3 nodes. Moving the altitude reaches 54. The difference is the answer to how much of the design a change just invalidated.")

D("orbit_inclination", "Orbit inclination", "orbit", "orbit_geometry", "Angle", "Degree", "i",
  96.6, 0.0, 180.0,
  "inclination is defined on 0..180 degrees",
  "inclination is defined on 0..180 degrees",
  "What is the angle between the orbit plane and the equator?",
  "orbitt_case_c1", "systems", "A. Rai / 2026-09-01")

D("orbit_min_elevation", "Minimum useful elevation angle", "orbit", "orbit_geometry", "Angle", "Degree", "eps",
  10.0, 0.0, 90.0,
  "at the horizon the slant range is longest, the atmosphere is thickest and no useful link or image is obtained; zero is the limit, not a working value",
  "90 degrees is the zenith, at which the access circle has no area",
  "Below what elevation is a target or a ground station no longer usable?",
  "larson_wertz", "systems", "A. Rai / 2026-09-01")

D("orbit_beta_angle", "Beta angle", "orbit", "orbit_geometry", "Angle", "Degree", "beta",
  30.0, -90.0, 90.0,
  "the angle between the orbit plane and the Sun is defined on -90..90",
  "the angle between the orbit plane and the Sun is defined on -90..90",
  "What angle does the Sun make with the orbit plane? It decides eclipse duration and therefore both the power and the thermal design.",
  "larson_wertz", "systems", "A. Rai / 2026-09-01")

D("orbit_mission_duration", "Mission duration", "orbit", "orbit_geometry", "Time", "Year", "T_mis",
  5.0, 0.5, 15.0,
  "below six months the programme cannot amortise a satellite, so it is not the mission being designed",
  "above 15 years the cost model, the degradation model and the battery cycle model are all extrapolated well past their fits",
  "How long must one satellite operate?",
  "orbitt_case_c1", "systems", "A. Rai / 2026-09-01",
  kpis=["kpi_cost_per_year"])

C("orbit_radius", "Geocentric radius", "orbit", "orbit_geometry", "Length", "Metre", "r",
  "How far is the spacecraft from the centre of the Earth?",
  "r = R_earth + h",
  "wgs84", "systems",
  [("h", "orbit_altitude")],
  [("add the WGS-84 equatorial radius to the altitude", "r", "Length", "orbit::radius(h)")],
  6.4e6, 7.0e6,
  "below the Earth's mean radius is not an orbit",
  "above 7000 km is outside the band this tool is scoped to",
  tier="A",
  fixtures=[("250 km", {"h": 250000.0}, 6628137.0, 1e-12, "independent-derivation", "wgs84")])

C("orbit_velocity", "Circular orbital speed", "orbit", "orbit_geometry", "Velocity", "MetrePerSecond", "V",
  "How fast is the spacecraft moving through the atmosphere?",
  "V = sqrt(mu/r)",
  "vallado2013", "systems",
  [("r", "orbit_radius")],
  [("apply the circular two-body speed relation", "v", "Velocity", "orbit::circular_velocity(r)")],
  7000.0, 8200.0,
  "below 7 km/s corresponds to an orbit above 1000 km, outside this tool's band",
  "8.2 km/s exceeds the surface circular speed and is not reachable",
  tier="A",
  assumptions=[("Two-body, circular",
                "at e = 0.05 the speed varies by 5% around the orbit and a drag figure computed at the mean underestimates the perigee pass")],
  fixtures=[("250 km circular", {"r": 6628137.0}, 7754.84549737, 1e-9, "independent-derivation", "vallado2013"),
            ("400 km circular", {"r": 6778137.0}, 7668.55817541, 1e-9, "independent-derivation", "vallado2013")])

C("orbit_period", "Orbital period", "orbit", "orbit_geometry", "Time", "Minute", "T_orb",
  "How long does one revolution take?",
  "T = 2*pi*sqrt(a^3/mu)",
  "vallado2013", "systems",
  [("r", "orbit_radius")],
  [("apply Kepler's third law at the circular radius", "t", "Time", "orbit::period(r)")],
  83.0, 104.0,
  "a period below 5000 s corresponds to an orbit inside the Earth",
  "6200 s corresponds to about 1000 km, outside this tool's band",
  tier="A",
  fixtures=[("250 km circular", {"r": 6628137.0}, 5370.29564631, 1e-9, "independent-derivation", "vallado2013")])

C("orbit_ground_track_speed", "Ground track speed", "orbit", "orbit_geometry", "Velocity", "MetrePerSecond", "V_g",
  "How fast does the sub-satellite point move over the ground?",
  "V_g = V*Re/r",
  "larson_wertz", "systems",
  [("v", "orbit_velocity"), ("r", "orbit_radius")],
  [("scale the orbital speed by the ratio of Earth radius to orbital radius",
    "vg", "Velocity", "orbit::ground_track_speed(v, r)")],
  6000.0, 8000.0,
  "below 6 km/s is outside this tool's altitude band",
  "the ground track can never move faster than the orbital speed",
  tier="A",
  note="Not the orbital speed. The difference is what sets sensor dwell time and therefore signal-to-noise.")

C("orbit_nodal_regression", "Nodal regression rate", "orbit", "orbit_geometry", "AngularRate", "DegreePerDay", "dOmega",
  "How fast does the orbit plane rotate under the second zonal harmonic?",
  "dOmega/dt = -1.5*J2*sqrt(mu)*Re^2*cos(i)/((1-e^2)^2*a^(7/2))",
  "vallado2013", "systems",
  [("r", "orbit_radius"), ("e", "orbit_eccentricity"), ("i", "orbit_inclination")],
  [("evaluate the secular J2 node rate", "d", "AngularRate", "orbit::nodal_regression(r, e.get(), i)")],
  -15.0, 15.0,
  "a regression faster than 15 degrees per day does not occur in this altitude band",
  "a regression faster than 15 degrees per day does not occur in this altitude band",
  tier="A",
  fixtures=[("250 km, 96.6 deg", {"r": 6628137.0, "e": 0.0, "i": 1.68598806}, 2.02216606e-7, 1e-6, "independent-derivation", "vallado2013")])

C("orbit_sun_sync_inclination", "Sun-synchronous inclination", "orbit", "orbit_geometry", "Angle", "Degree", "i_ss",
  "What inclination makes the orbit plane keep pace with the Sun?",
  "cos(i) = -Omega_sun*(1-e^2)^2*a^(7/2)/(1.5*J2*sqrt(mu)*Re^2)",
  "vallado2013", "systems",
  [("r", "orbit_radius"), ("e", "orbit_eccentricity")],
  [("solve the nodal regression relation for the inclination that matches the Sun's apparent motion; refuse when no such inclination exists",
    "i", "Angle",
    "match orbit::sun_synchronous_inclination(r, e.get()) { Some(i) => i, None => return Err(Fault::Degenerate { node: NODE_ID, field: \"r\", reason: \"no inclination gives Sun-synchronous regression at this radius\" }) }")],
  85.0, 115.0,
  "a Sun-synchronous inclination is always retrograde and above about 95 degrees in this band",
  "above 115 degrees no Sun-synchronous solution exists below 2000 km",
  tier="A",
  fixtures=[("250 km circular", {"r": 6628137.0, "e": 0.0}, 1.68420864, 1e-6, "independent-derivation", "vallado2013")])

C("orbit_earth_central_angle", "Earth central half-angle", "orbit", "orbit_geometry", "Angle", "Degree", "lambda",
  "How much of the Earth's surface is in view at the working elevation?",
  "lambda = acos((Re/r)*cos(eps)) - eps",
  "larson_wertz", "systems",
  [("r", "orbit_radius"), ("eps", "orbit_min_elevation")],
  [("apply the spherical Earth access geometry", "l", "Angle", "orbit::earth_central_angle(r, eps)")],
  0.0, 86.0,
  "a negative half-angle means the target is below the horizon",
  "beyond 85 degrees the whole visible hemisphere is in view, which does not occur in this band",
  tier="A",
  fixtures=[("250 km, 10 deg elevation", {"r": 6628137.0, "eps": 0.17453293}, 0.150429309, 1e-6, "independent-derivation", "larson_wertz")])

C("orbit_swath_width", "Ground swath width", "orbit", "orbit_geometry", "Length", "Kilometre", "W",
  "How wide a strip of ground does one pass cover?",
  "W = 2*lambda*Re",
  "larson_wertz", "systems",
  [("lam", "orbit_earth_central_angle")],
  [("convert the Earth-central half-angle into a great-circle width", "w", "Length", "orbit::swath_width(lam)")],
  1.0, 40000.0,
  "a swath below one kilometre would not be a swath",
  "a swath cannot exceed the Earth's circumference",
  tier="A",
  kpis=["kpi_revisit"])

C("orbit_slant_range", "Slant range at working elevation", "orbit", "orbit_geometry", "Length", "Kilometre", "d",
  "How far away is a target or ground station at the edge of the access circle?",
  "d = sqrt(Re^2 + r^2 - 2*Re*r*cos(lambda))",
  "larson_wertz", "systems",
  [("r", "orbit_radius"), ("eps", "orbit_min_elevation")],
  [("apply the law of cosines across the access triangle", "d", "Length", "orbit::slant_range(r, eps)")],
  100.0, 4000.0,
  "the slant range is at least the altitude",
  "beyond 4000 km the geometry is outside this tool's band",
  tier="A")

C("orbit_eclipse_fraction", "Eclipse fraction of the orbit", "orbit", "orbit_geometry", "Ratio", "One", "f_ecl",
  "What fraction of each revolution is spent in the Earth's shadow?",
  "f_ecl = (1/pi)*acos(sqrt(r^2 - Re^2)/(r*cos(beta)))",
  "larson_wertz", "systems",
  [("r", "orbit_radius"), ("beta", "orbit_beta_angle")],
  [("apply the cylindrical shadow model; a full-sun orbit returns zero rather than a fault",
    "f", "Ratio", "orbit::eclipse_fraction(r, beta)")],
  0.0, 0.45,
  "a negative eclipse fraction is not a fraction",
  "no circular orbit in this band spends more than 45% of a revolution in shadow",
  tier="A",
  assumptions=[("Cylindrical shadow, no penumbra",
                "the penumbra adds about 10 seconds per orbit, which matters for a precise thermal transient and not for a power budget")],
  fixtures=[("250 km, beta = 30 deg", {"r": 6628137.0, "beta": 0.52359878}, 0.398283597, 1e-6, "independent-derivation", "larson_wertz")])

C("orbit_decay_rate", "Orbital decay rate", "orbit", "orbit_geometry", "Velocity", "MetrePerSecond", "da_dt",
  "How fast does the orbit fall if nothing compensates the drag?",
  "da/dt = -rho*sqrt(mu*a)/BC",
  "vallado2013", "systems",
  [("rho", "env_mass_density"), ("bc", "aero_ballistic_coefficient"), ("r", "orbit_radius")],
  [("apply the circular-orbit secular decay relation", "d", "Velocity", "orbit::decay_rate(rho, bc.get(), r)")],
  -10.0, 0.0,
  "a decay faster than 10 m/s of semi-major axis per second is re-entry, not an orbit",
  "drag cannot raise an orbit; a positive value here is a sign error upstream",
  tier="B",
  kpis=["kpi_service_lifetime"],
  view=("line", {"over": "orbit_altitude", "points": 60}))

C("orbit_makeup_delta_v", "Drag make-up delta-v per year", "orbit", "orbit_geometry", "Velocity", "MetrePerSecond", "dv_dm",
  "How much delta-v a year does holding this altitude cost?",
  "dv = (D/m)*t",
  "vallado2013", "systems",
  [("a_drag", "aero_drag_acceleration")],
  [("integrate the drag deceleration over one Julian year",
    "dv", "Velocity", "orbit::drag_makeup_delta_v(a_drag, Time::new(31_557_600.0))")],
  0.0, 100000.0,
  "make-up delta-v cannot be negative",
  "above 100 km/s a year no propulsion system of any kind closes, and the answer is that the altitude is wrong",
  tier="B",
  note="This is the number an air-breathing system exists to make free. A stored-propellant design at 250 km spends several kilometres per second a year on it.")

C("orbit_deorbit_delta_v", "Disposal delta-v", "orbit", "orbit_geometry", "Velocity", "MetrePerSecond", "dv_dis",
  "What does the end-of-life disposal manoeuvre cost, as required by the debris mitigation standard?",
  "dv = |V_circ - V_transfer(r, a_t)|",
  "iso24113", "systems",
  [("h", "orbit_altitude")],
  [("lower perigee to a 60 km re-entry interface on a Hohmann transfer",
    "dv", "Velocity", "orbit::deorbit_delta_v(h, Length::from_km(60.0))")],
  0.0, 500.0,
  "a disposal manoeuvre cannot cost negative delta-v",
  "above 500 m/s the disposal budget dominates the design and the altitude choice should be revisited",
  tier="A",
  note="Carried as a design variable rather than discovered at the end, because ISO 24113 makes it a requirement and not a courtesy.")

C("orbit_lifetime_uncontrolled", "Uncontrolled orbital lifetime", "orbit", "orbit_geometry", "Time", "Day", "t_life",
  "If propulsion stops, how long before the satellite re-enters?",
  "t = INT da/(da/dt) from h to 120 km",
  "vallado2013", "systems",
  [("h", "orbit_altitude"), ("bc", "aero_ballistic_coefficient"), ("t_inf", "env_exospheric_temperature")],
  [("integrate the decay rate down to the 120 km model base at fixed atmospheric conditions",
    "t", "Time",
    "orbit::lifetime_estimate(h, Length::from_km(120.0), bc.get(), 64, |z| env::mass_density(z, t_inf))")],
  0.0, 36500.0,
  "a lifetime cannot be negative",
  "above 100 years the 25-year debris rule is violated and the number's precision is irrelevant",
  tier="C",
  assumptions=[("Atmospheric conditions fixed at the current activity for the whole decay",
                "over a solar cycle the density at a given altitude changes by more than an order of magnitude, so this is an order-of-magnitude answer and is reported as one")],
  kpis=["kpi_service_lifetime"])


# =============================================================================
# AERODYNAMICS AND DRAG
# =============================================================================
D("aero_body_length", "Body length", "aero", "aero_drag", "Length", "Metre", "L",
  2.0, 0.3, 10.0,
  "below 0.3 m the vehicle is smaller than a 3U cubesat and the multipayload mission does not fit in it",
  "above 10 m the launch envelope and the free-molecular panel model both stop applying",
  "How long is the vehicle along the flight direction?",
  "orbitt_case_c1", "mass", "A. Rai / 2026-09-01")

D("aero_body_diameter", "Body diameter", "aero", "aero_drag", "Length", "Metre", "D_b",
  0.6, 0.1, 3.0,
  "below 0.1 m no useful payload aperture fits",
  "above 3 m the frontal area makes the air-breathing closure impossible at any altitude in the band",
  "How wide is the vehicle across the flight direction?",
  "orbitt_case_c1", "mass", "A. Rai / 2026-09-01")

D("aero_appendage_area", "Appendage frontal area", "aero", "aero_drag", "Area", "SquareMetre", "A_app",
  0.05, 0.0, 5.0,
  "an area cannot be negative",
  "above 5 m2 of appendage the design is an array with a satellite attached, and the drag closure is decided by the array alone",
  "How much frontal area do the array, antennas and radiators add beyond the body?",
  "orbitt_case_c1", "mass", "A. Rai / 2026-09-01",
  note="Separated from the body area because it is the term a power or thermal decision moves, and the tree has to show that coupling.")

D("aero_cp_cm_offset", "Centre of pressure to centre of mass offset", "aero", "aero_drag", "Length", "Metre", "x_cp",
  0.15, 0.0, 2.0,
  "a zero offset is neutrally stable, which is the limit and not a design",
  "the offset cannot exceed the body length",
  "How far behind the centre of mass does the aerodynamic force act?",
  "orbitt_case_c1", "gnc", "A. Rai / 2026-09-01",
  note="Positive means aft, which is the stable configuration. Arranging that is most of what passive aerostability in VLEO means.")

C("aero_frontal_area", "Total frontal area", "aero", "aero_drag", "Area", "SquareMetre", "A",
  "What area does the flow actually see?",
  "A = pi*D^2/4 + A_app",
  "orbitt_case_c1", "mass",
  [("d", "aero_body_diameter"), ("a_app", "aero_appendage_area")],
  [("add the circular body cross-section to the appendage area",
    "a", "Area", "Area::new(0.25 * pmath::PI * d.get() * d.get() + a_app.get())")],
  0.005, 20.0,
  "below 50 cm2 there is no vehicle",
  "above 20 m2 nothing in this design closes",
  tier="A")

C("aero_speed_ratio", "Molecular speed ratio", "aero", "aero_drag", "Ratio", "One", "s",
  "How hyperthermal is the flow — does the gas arrive as a beam or as a cloud?",
  "s = V/sqrt(2*R*T/M)",
  "sentman1961", "aero",
  [("v", "orbit_velocity"), ("t", "env_local_temperature"), ("m", "env_mean_molar_mass")],
  [("divide the bulk speed by the most probable thermal speed of the local mixture",
    "s", "Ratio", "aero::speed_ratio(v, t, m)")],
  1.0, 30.0,
  "below a speed ratio of one the flow is thermal rather than hyperthermal, and Sentman's relations lose their meaning",
  "above 30 the exponential terms underflow and the relations are numerically dead",
  tier="A",
  fixtures=[("250 km, moderate activity", {"v": 7754.6, "t": 905.8, "m": 0.018720}, 8.6435, 5e-3, "independent-derivation", "sentman1961")])

C("aero_accommodation", "Energy accommodation coefficient", "aero", "aero_drag", "Ratio", "One", "alpha",
  "How completely does an incoming molecule thermalise with the surface before leaving?",
  "alpha = K*n_O*T_inf/(1 + K*n_O*T_inf),  K = 7.5e-17",
  "moe2005", "aero",
  [("n_o", "env_atomic_oxygen_density"), ("t_inf", "env_exospheric_temperature")],
  [("apply the Langmuir adsorption isotherm for atomic oxygen coverage",
    "a", "Ratio", "Ratio::new(aero::accommodation_coefficient(n_o, t_inf))")],
  0.0, 1.0,
  "accommodation is a fraction and cannot be negative",
  "full accommodation is one; above it the reflected molecules would carry away energy the surface does not have",
  tier="B",
  assumptions=[("The surface is covered in adsorbed atomic oxygen and behaves as that coverage dictates",
                "a freshly cleaned or a fluorinated surface adsorbs far less, and its drag differs by tens of per cent from this")],
  note="A design that assumes a fixed accommodation has a margin that moves when the Sun does.")

C("aero_drag_coefficient", "Drag coefficient", "aero", "aero_drag", "Ratio", "One", "Cd",
  "What is the drag coefficient of this body, at this speed ratio, with this surface?",
  "Cd = P/sqrt(pi) + gamma*Q*Z + (gamma/2)*vr*(gamma*sqrt(pi)*Z + P)",
  "doornbos2011", "aero",
  [("s", "aero_speed_ratio"), ("l", "aero_body_length"), ("d", "aero_body_diameter"),
   ("alpha", "aero_accommodation"), ("t_w", "env_wall_temperature"), ("v", "orbit_velocity"),
   ("m", "env_mean_molar_mass"), ("kn", "env_knudsen")],
  [("evaluate Sentman's free-molecular coefficients for the front face and the side walls, referred back to the frontal area",
    "cd", "Ratio",
    "Ratio::new(aero::cylinder_drag_coefficient(s.get(), l, d, alpha.get(), t_w, v, m))")],
  1.5, 5.0,
  "no free-molecular body has a drag coefficient below 1.5; a lower value means the speed ratio or the accommodation is wrong",
  "above 5.0 the geometry is not the slender body this model assumes",
  tier="A",
  assumptions=[("Free-molecular flow — a molecule leaving the surface never returns and never strikes an incoming one",
                "below Kn = 10, which is roughly 150 km for this body; the Knudsen number is an input so that the guard on it is visible here")],
  fixtures=[("250 km cylinder, moderate activity", {"s": 8.6435, "l": 2.0, "d": 0.6, "alpha": 0.9918,
                                                    "t_w": 300.0, "v": 7754.6, "m": 0.018720, "kn": 1.0e4},
             2.85807, 1e-3, "independent-tool", "matlab_legacy")],
  note="Treating this as a constant 2.2 is the single largest avoidable error in a VLEO drag estimate, and it is what most concept studies do.",
  view=("line", {"over": "orbit_altitude", "points": 60}))

C("aero_dynamic_pressure", "Dynamic pressure", "aero", "aero_drag", "Pressure", "Pascal", "q",
  "What pressure does the flow exert per unit of drag coefficient and area?",
  "q = 0.5*rho*V^2",
  "vallado2013", "aero",
  [("rho", "env_mass_density"), ("v", "orbit_velocity")],
  [("form the dynamic pressure of the free stream", "q", "Pressure", "aero::dynamic_pressure(rho, v)")],
  0.0, 1.0,
  "dynamic pressure cannot be negative",
  "above 1 Pa the vehicle is re-entering, not orbiting",
  tier="A")

C("aero_drag_force", "Drag force", "aero", "aero_drag", "Force", "Millinewton", "D",
  "How hard is the atmosphere pulling the satellite backwards?",
  "D = 0.5*rho*V^2*Cd*A",
  "vallado2013", "aero",
  [("q", "aero_dynamic_pressure"), ("cd", "aero_drag_coefficient"), ("a", "aero_frontal_area")],
  [("multiply dynamic pressure by the drag coefficient and the frontal area",
    "d", "Force", "Force::new(q.get() * cd.get() * a.get())")],
  0.0, 10000.0,
  "drag cannot be negative",
  "above 10 N the vehicle is decelerating at a rate no electric propulsion system can answer",
  tier="A",
  kpis=["kpi_service_lifetime"],
  fixtures=[("250 km, baseline vehicle", {"q": 1.9938e-3, "cd": 2.85807, "a": 0.3327},
             1.8957e-3, 1e-3, "independent-derivation", "vallado2013")],
  view=("line", {"over": "orbit_altitude", "points": 60}))

C("aero_drag_acceleration", "Drag deceleration", "aero", "aero_drag", "Acceleration", "MetrePerSecond2", "a_D",
  "What deceleration does the drag produce on this mass?",
  "a_D = D/m",
  "vallado2013", "aero",
  [("d", "aero_drag_force"), ("m", "mass_wet")],
  [("divide the drag force by the wet mass", "a", "Acceleration", "aero::drag_acceleration(d, m)")],
  0.0, 0.1,
  "drag deceleration cannot be negative",
  "above 0.1 m/s2 the orbit decays within hours",
  tier="A")

C("aero_ballistic_coefficient", "Ballistic coefficient", "aero", "aero_drag", "Ratio", "One", "BC",
  "How well does this vehicle resist the atmosphere, per unit of its own mass?",
  "BC = m/(Cd*A)",
  "vallado2013", "aero",
  [("m", "mass_wet"), ("cd", "aero_drag_coefficient"), ("a", "aero_frontal_area")],
  [("divide the wet mass by the product of drag coefficient and frontal area",
    "bc", "Ratio", "Ratio::new(aero::ballistic_coefficient(m, cd.get(), a))")],
  5.0, 500.0,
  "below 5 kg/m2 the vehicle is a sail, and nothing in this band closes",
  "above 500 kg/m2 the vehicle is denser than any spacecraft ever flown",
  tier="A",
  note="Higher is better here. A cubesat is around 50; a slender VLEO platform reaches 150, and that difference is worth tens of kilometres of altitude.")

C("aero_torque", "Aerodynamic disturbance torque", "aero", "aero_drag", "Torque", "NewtonMetre", "T_aero",
  "How much torque does the drag apply about the centre of mass?",
  "T = D*x_cp",
  "larson_wertz", "gnc",
  [("d", "aero_drag_force"), ("x", "aero_cp_cm_offset")],
  [("multiply the drag force by the centre-of-pressure offset",
    "t", "Torque", "aero::aerodynamic_torque(d, x)")],
  0.0, 1.0,
  "a torque magnitude cannot be negative",
  "above 1 N.m no reaction wheel that fits this vehicle can hold attitude",
  tier="A",
  note="In VLEO this dominates gravity gradient and solar pressure by one to three orders of magnitude, which is why a control design carried over from a higher orbit is undersized here.")

C("aero_ao_fluence", "Atomic oxygen fluence", "aero", "aero_drag", "Ratio", "One", "F_AO",
  "How many oxygen atoms strike a forward-facing surface over the mission?",
  "F = n_O*V*t",
  "moe2005", "mass",
  [("n_o", "env_atomic_oxygen_density"), ("v", "orbit_velocity"), ("t", "orbit_mission_duration")],
  [("multiply the atomic oxygen number density by the flight speed and the mission duration",
    "f", "Ratio", "Ratio::new(aero::atomic_oxygen_fluence(n_o, v, t))")],
  0.0, 1.0e26,
  "a fluence cannot be negative",
  "above 1e26 atoms per square metre no known external material survives, so the design is not a design",
  tier="B",
  kpis=["kpi_service_lifetime"],
  note="Polyimide erodes at about 3e-24 cm3 per incident atom, so 1e22 removes roughly 30 micrometres of it.")


# =============================================================================
# PROPULSION — AIR-BREATHING ELECTRIC
# The actual contribution. Everything else in the tree exists to say whether
# this closes.
# =============================================================================
D("prop_intake_area", "Intake mouth area", "prop", "prop_intake", "Area", "SquareMetre", "A_in",
  0.20, 0.01, 5.0,
  "below 100 cm2 the collected flow is below the ionisation threshold of any thruster considered",
  "above 5 m2 the mouth is itself more drag than the thrust it can produce, at every altitude in the band",
  "How large is the opening the intake presents to the flow?",
  "romano2021", "propulsion", "A. Rai / 2026-09-01",
  kpis=["kpi_thrust_margin"],
  note="Both the numerator and part of the denominator of the closure. A bigger mouth collects more and drags more, which is why there is an optimum rather than a maximum.")

D("prop_throat_area", "Intake throat area", "prop", "prop_intake", "Area", "SquareMetre", "A_out",
  0.010, 0.0001, 1.0,
  "below 1 cm2 the throat chokes the flow to the thruster whatever the mouth collects",
  "the throat cannot be larger than the mouth in any useful intake",
  "How large is the opening from the collection chamber into the thruster?",
  "romano2021", "propulsion", "A. Rai / 2026-09-01")

D("prop_eta_geo", "Intake geometric transmission", "prop", "prop_intake", "Ratio", "One", "eta_geo",
  0.90, 0.1, 1.0,
  "below 10% the duct blocks more than it passes and is not an intake",
  "unity means a perfectly open mouth with no structure, which is not manufacturable",
  "What fraction of the mouth is open duct, and what fraction of a hyperthermal beam does that duct pass?",
  "romano2021", "propulsion", "A. Rai / 2026-09-01")

D("prop_beta_backflow", "Duct back-flow transmission", "prop", "prop_intake", "Ratio", "One", "beta",
  0.06, 0.001, 0.5,
  "below 0.001 no manufacturable duct is that good at trapping thermal molecules",
  "above 0.5 the duct traps nothing and the compression ratio collapses to one",
  "What fraction of thermal molecules in the chamber find their way back out of the mouth?",
  "romano2021", "propulsion", "A. Rai / 2026-09-01",
  note="This single number is what an intake design is for. A long, narrow, honeycombed duct makes it small; that is the whole device.")

D("prop_utilisation", "Propellant utilisation efficiency", "prop", "prop_thruster", "Ratio", "One", "eta_u",
  0.40, 0.05, 0.95,
  "below 5% the thruster ionises almost nothing and the concept does not work",
  "above 95% no inductively coupled source has demonstrated utilisation this high on atomic oxygen",
  "What fraction of the collected mass is actually ionised and accelerated?",
  "romano2021", "propulsion", "A. Rai / 2026-09-01",
  kpis=["kpi_thrust_margin"])

D("prop_beam_voltage", "Beam accelerating voltage", "prop", "prop_thruster", "Voltage", "Volt", "V_b",
  300.0, 50.0, 2000.0,
  "below 50 V the exhaust velocity is too low to produce useful thrust from this mass flow",
  "above 2000 V grid erosion by atomic oxygen becomes the life-limiting mechanism, and the design has no demonstrated basis",
  "Through what potential is the ion beam accelerated?",
  "romano2021", "propulsion", "A. Rai / 2026-09-01")

D("prop_div_efficiency", "Beam divergence efficiency", "prop", "prop_thruster", "Ratio", "One", "alpha_div",
  0.97, 0.7, 1.0,
  "below 0.7 the beam is a cloud rather than a beam",
  "unity is a perfectly collimated beam, which does not exist",
  "How much thrust is lost because the beam is not perfectly axial?",
  "romano2021", "propulsion", "A. Rai / 2026-09-01")

D("prop_double_ion_factor", "Doubly-charged ion correction", "prop", "prop_thruster", "Ratio", "One", "alpha_pp",
  0.97, 0.8, 1.0,
  "below 0.8 the plasma is dominated by multiply-charged species and the single-charge model does not apply",
  "unity means no doubly-charged ions at all, which no real source achieves",
  "How much thrust is lost to doubly-charged ions carrying charge without proportional mass?",
  "romano2021", "propulsion", "A. Rai / 2026-09-01")

D("prop_ionisation_cost", "Energy cost per ion", "prop", "prop_thruster", "Ratio", "One", "eps_i",
  250.0, 20.0, 1000.0,
  "below 20 eV per ion is under twice the 13.6 eV ionisation potential of atomic oxygen; no source has approached it",
  "above 1000 eV per ion the ionisation power alone exceeds any plausible bus, whatever the beam does",
  "How much energy does the source spend to make one ion, including every loss?",
  "romano2021", "propulsion", "A. Rai / 2026-09-01",
  note="Not the ionisation potential. A real source spends 2 to 30 times the theoretical minimum on excitation, radiation and wall losses.")

D("prop_other_losses", "Other thruster losses", "prop", "prop_thruster", "Ratio", "One", "L_other",
  0.15, 0.0, 0.6,
  "a loss fraction cannot be negative",
  "above 0.6 the thruster is a heater with a beam attached",
  "What fraction of the thruster's input power goes to losses other than ionisation and the beam?",
  "romano2021", "propulsion", "A. Rai / 2026-09-01")

D("prop_ppu_efficiency", "Power processing unit efficiency", "prop", "prop_thruster", "Ratio", "One", "eta_ppu",
  0.90, 0.5, 0.98,
  "below 50% the power processing unit dissipates more than the thruster uses and the thermal design does not close",
  "above 98% no flown power processing unit has been demonstrated",
  "How efficiently does the power processing unit convert bus power into thruster power?",
  "larson_wertz", "power", "A. Rai / 2026-09-01")

C("prop_incident_flux", "Incident mass flux", "prop", "prop_intake", "MassFlux", "KgPerSecond", "phi",
  "How much mass arrives on a square metre facing the flow each second?",
  "phi = rho*V",
  "romano2021", "propulsion",
  [("rho", "env_mass_density"), ("v", "orbit_velocity")],
  [("multiply the free-stream density by the flight speed",
    "f", "MassFlux", "prop::incident_mass_flux(rho, v)")],
  0.0, 1.0,
  "an incident flux cannot be negative",
  "above 1 kg/m2/s the flow is continuum and the intake model does not apply",
  tier="A",
  note="The ceiling on everything downstream. At 200 km it is about 2 milligrams per square metre per second.")

C("prop_capture_efficiency", "Intake collection efficiency", "prop", "prop_intake", "Ratio", "One", "eta_c",
  "What fraction of the flow entering the mouth actually reaches the thruster?",
  "eta_c = A_out*eta_geo/(A_out + beta*A_in)",
  "romano2021", "propulsion",
  [("a_in", "prop_intake_area"), ("a_out", "prop_throat_area"), ("eta_geo", "prop_eta_geo"),
   ("beta", "prop_beta_backflow"), ("t_c", "env_chamber_temperature"), ("m", "env_mean_molar_mass"),
   ("n", "env_number_density"), ("v", "orbit_velocity")],
  [("balance the hyperthermal inflow against the thermal outflow through the throat and back out of the mouth",
    "r", "Ratio",
    "prop::intake_balance(n, v, a_in, a_out, eta_geo, beta, t_c, m).collection_efficiency")],
  0.0, 1.0,
  "a collection efficiency cannot be negative",
  "an intake cannot deliver more than enters its mouth",
  tier="B",
  assumptions=[("Free-molecular flow throughout the duct and the chamber",
                "if the chamber compresses far enough to become collisional the balance is no longer a simple flux balance, which happens above a compression ratio of roughly 1e4"),
               ("The chamber gas is fully thermalised at the chamber temperature",
                "a short duct passes part of the beam straight through, which raises the delivered flow above this and is not modelled")],
  fixtures=[("IRS-class baseline intake", {"a_in": 0.20, "a_out": 0.010, "eta_geo": 0.90,
                                           "beta": 0.06, "t_c": 600.0, "m": 0.018720,
                                           "n": 2.133e15, "v": 7754.6},
             0.40909, 1e-4, "independent-derivation", "romano2021")],
  kpis=["kpi_thrust_margin"],
  note="Read the relation slowly: the collection efficiency does not depend on the flight speed at all, only on geometry. Speed buys compression, not capture. A design trying to raise capture by flying faster is optimising the wrong term.")

C("prop_compression_ratio", "Intake compression ratio", "prop", "prop_intake", "Ratio", "One", "CR",
  "By how much does the intake concentrate the gas before it reaches the thruster?",
  "CR = 4*V*A_in*eta_geo/(v_bar*(A_out + beta*A_in))",
  "romano2021", "propulsion",
  [("a_in", "prop_intake_area"), ("a_out", "prop_throat_area"), ("eta_geo", "prop_eta_geo"),
   ("beta", "prop_beta_backflow"), ("t_c", "env_chamber_temperature"), ("m", "env_mean_molar_mass"),
   ("n", "env_number_density"), ("v", "orbit_velocity")],
  [("take the chamber-to-free-stream density ratio from the same flux balance",
    "r", "Ratio",
    "prop::intake_balance(n, v, a_in, a_out, eta_geo, beta, t_c, m).compression_ratio")],
  1.0, 1.0e5,
  "an intake cannot dilute the flow; a compression ratio below one means the geometry is inverted",
  "above 1e5 the chamber gas becomes collisional and the free-molecular balance no longer holds",
  tier="B",
  fixtures=[("IRS-class baseline intake", {"a_in": 0.20, "a_out": 0.010, "eta_geo": 0.90,
                                           "beta": 0.06, "t_c": 600.0, "m": 0.018720,
                                           "n": 2.133e15, "v": 7754.6},
             308.09, 1e-3, "independent-derivation", "romano2021")])

C("prop_chamber_density", "Collection chamber number density", "prop", "prop_intake", "NumberDensity", "PerCubicMetre", "n_c",
  "How dense is the gas the thruster is asked to ionise?",
  "n_c = CR*n",
  "romano2021", "propulsion",
  [("n", "env_number_density"), ("cr", "prop_compression_ratio")],
  [("scale the free-stream number density by the compression ratio",
    "nc", "NumberDensity", "NumberDensity::new(n.get() * cr.get())")],
  1.0e12, 1.0e22,
  "below 1e12 per cubic metre no inductively coupled source will strike a discharge",
  "above 1e22 the gas is at atmospheric pressure, which no intake in this band can produce",
  tier="B",
  note="This is the number that decides whether a plasma can be struck at all, and it is why compression matters even though it does not appear in the capture efficiency.")

C("prop_collected_flow", "Collected mass flow", "prop", "prop_intake", "MassFlow", "MgPerSecond", "mdot",
  "How much mass per second reaches the thruster?",
  "mdot = rho*V*A_in*eta_c",
  "romano2021", "propulsion",
  [("rho", "env_mass_density"), ("v", "orbit_velocity"), ("a_in", "prop_intake_area"),
   ("eta_c", "prop_capture_efficiency")],
  [("multiply the incident flux by the mouth area and the collection efficiency",
    "md", "MassFlow", "prop::collected_mass_flow(rho, v, a_in, eta_c)")],
  0.0, 10.0,
  "a mass flow cannot be negative",
  "above 10 mg/s no intake in this band collects, at any mouth size the vehicle can carry",
  tier="B",
  view=("line", {"over": "orbit_altitude", "points": 60}))

C("prop_ion_flow", "Ionised mass flow", "prop", "prop_thruster", "MassFlow", "MgPerSecond", "mdot_i",
  "How much of the collected mass is actually accelerable?",
  "mdot_i = eta_u*mdot",
  "romano2021", "propulsion",
  [("md", "prop_collected_flow"), ("eta_u", "prop_utilisation")],
  [("apply the propellant utilisation efficiency", "mi", "MassFlow", "prop::ion_mass_flow(md, eta_u)")],
  0.0, 10.0,
  "an ion mass flow cannot be negative",
  "cannot exceed the collected flow it comes from",
  tier="B")

C("prop_exhaust_velocity", "Beam exhaust velocity", "prop", "prop_thruster", "Velocity", "MetrePerSecond", "v_e",
  "How fast does the accelerated beam leave the thruster?",
  "v_e = sqrt(2*q*V_b/m_i)",
  "romano2021", "propulsion",
  [("vb", "prop_beam_voltage"), ("m", "env_mean_molar_mass")],
  [("accelerate a singly-charged ion of the local mean mass through the beam potential",
    "ve", "Velocity", "prop::beam_exhaust_velocity(vb, m)")],
  1000.0, 500000.0,
  "below 1 km/s the exhaust is slower than a cold gas thruster and the concept has no advantage",
  "above 500 km/s the relativistic and space-charge assumptions in the relation break down",
  tier="A",
  fixtures=[("300 V, atomic oxygen mixture", {"vb": 300.0, "m": 0.018720}, 55610.0583860, 1e-9, "independent-derivation", "codata2018")],
  note="Air is lighter than xenon, so the same voltage buys a far higher exhaust velocity. It is the one respect in which air-breathing propulsion is easier rather than harder.")

C("prop_thrust", "Thrust", "prop", "prop_thruster", "Force", "Millinewton", "T",
  "How much thrust does the thruster produce from the air it is given?",
  "T = mdot_i*v_e*alpha_div*alpha_pp",
  "romano2021", "propulsion",
  [("mi", "prop_ion_flow"), ("ve", "prop_exhaust_velocity"), ("ad", "prop_div_efficiency"),
   ("ap", "prop_double_ion_factor")],
  [("multiply the ion flow by the exhaust velocity and apply the divergence and multiple-charge corrections",
    "t", "Force", "prop::beam_thrust(mi, ve, ad, ap)")],
  0.0, 1000.0,
  "thrust cannot be negative",
  "above 1 N no air-breathing thruster in this band produces thrust, at any intake size",
  tier="B",
  kpis=["kpi_thrust_margin"],
  view=("line", {"over": "orbit_altitude", "points": 60}))

C("prop_jet_power", "Jet power", "prop", "prop_thruster", "Power", "Watt", "P_jet",
  "How much power ends up in the beam?",
  "P_jet = T^2/(2*mdot_i)",
  "romano2021", "propulsion",
  [("t", "prop_thrust"), ("mi", "prop_ion_flow")],
  [("form the kinetic power of the beam from thrust and ion flow; refuse when the flow is zero",
    "p", "Power",
    "if mi.get() <= 0.0 { return Err(Fault::Degenerate { node: NODE_ID, field: \"mdot_i\", reason: \"no ion flow, so jet power is undefined rather than infinite\" }); } else { prop::jet_power(t, mi) }")],
  0.0, 1.0e5,
  "jet power cannot be negative",
  "above 100 kW no bus in this mass class supplies it",
  tier="A")

C("prop_ionisation_power", "Ionisation power", "prop", "prop_thruster", "Power", "Watt", "P_ion",
  "How much power does making the ions cost, before any of them is accelerated?",
  "P_ion = (mdot_i/m_i)*eps_i*q",
  "romano2021", "propulsion",
  [("mi", "prop_ion_flow"), ("m", "env_mean_molar_mass"), ("eps", "prop_ionisation_cost")],
  [("convert the ion mass flow into ions per second and charge each one its stated energy cost",
    "p", "Power", "prop::ionisation_power(mi, m, eps.get())")],
  0.0, 1.0e5,
  "ionisation power cannot be negative",
  "above 100 kW no bus in this mass class supplies it",
  tier="B")

C("prop_input_power", "Thruster input power", "prop", "prop_thruster", "Power", "Watt", "P_in",
  "How much electrical power does the thruster itself draw?",
  "P_in = (P_jet + P_ion)/(1 - L_other)",
  "romano2021", "propulsion",
  [("pj", "prop_jet_power"), ("pi", "prop_ionisation_power"), ("lo", "prop_other_losses")],
  [("sum the useful terms and inflate them by the other-loss fraction",
    "p", "Power", "prop::thruster_input_power(pj, pi, lo)")],
  0.0, 1.0e5,
  "input power cannot be negative",
  "above 100 kW no bus in this mass class supplies it",
  tier="B")

C("prop_bus_power", "Propulsion demand on the bus", "prop", "prop_thruster", "Power", "Watt", "P_prop",
  "How much power does the propulsion system take from the bus?",
  "P_prop = P_in/eta_ppu",
  "larson_wertz", "propulsion",
  [("pin", "prop_input_power"), ("eta", "prop_ppu_efficiency")],
  [("divide by the power processing unit efficiency",
    "p", "Power", "prop::bus_power_demand(pin, eta)")],
  0.0, 1.0e5,
  "bus demand cannot be negative",
  "above 100 kW no bus in this mass class supplies it",
  tier="B",
  note="The largest single load on the bus, and the reason the power chapter is where an air-breathing design usually fails.")

C("prop_total_efficiency", "Total propulsion efficiency", "prop", "prop_thruster", "Ratio", "One", "eta_T",
  "What fraction of the power drawn from the bus ends up in the beam?",
  "eta_T = P_jet/P_prop",
  "romano2021", "propulsion",
  [("pj", "prop_jet_power"), ("pb", "prop_bus_power")],
  [("divide jet power by bus demand", "e", "Ratio", "prop::total_efficiency(pj, pb)")],
  0.0, 1.0,
  "an efficiency cannot be negative",
  "an efficiency above one is a violation of energy conservation and a sign of an error upstream",
  tier="A")

C("prop_specific_impulse", "Specific impulse", "prop", "prop_thruster", "Time", "Second", "Isp",
  "What is the specific impulse, referred honestly to the collected flow rather than the ionised part?",
  "Isp = T/(mdot*g0)",
  "romano2021", "propulsion",
  [("t", "prop_thrust"), ("md", "prop_collected_flow")],
  [("divide thrust by the weight flow of everything collected, not only what was ionised",
    "i", "Time", "prop::specific_impulse(t, md)")],
  100.0, 50000.0,
  "below 100 s an air-breathing system offers nothing a cold gas thruster does not",
  "above 50000 s the relation is being fed an ion flow that is not physical",
  tier="A",
  note="Referring this to the ion flow alone flatters the number by the reciprocal of the utilisation, which is how brochure figures are made.")

C("prop_thrust_to_drag", "Thrust to drag ratio", "prop", "closure", "Ratio", "One", "T_D",
  "Does the thrust the air produces exceed the drag the air imposes? This is the closure the whole design exists to reach.",
  "T/D",
  "romano2021", "systems",
  [("t", "prop_thrust"), ("d", "aero_drag_force")],
  [("divide thrust by drag; at or above one the orbit holds with no stored propellant",
    "r", "Ratio", "prop::thrust_to_drag(t, d)")],
  0.0, 100.0,
  "a negative ratio means a sign error in thrust or drag",
  "above 100 the vehicle is a launch stage, not a satellite",
  tier="B",
  kpis=["kpi_thrust_margin", "kpi_service_lifetime"],
  view=("line", {"over": "orbit_altitude", "points": 80}),
  note="At or above one the orbit holds indefinitely. Below one the mission has a lifetime rather than an altitude, and every other number in the design is a detail.")


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


# =============================================================================
# TT&C AND DOWNLINK
# =============================================================================
D("com_frequency", "Downlink carrier frequency", "com", "comms", "Frequency", "Gigahertz", "f_c",
  8.2, 0.1, 40.0,
  "below 100 MHz the antenna does not fit on this spacecraft",
  "above 40 GHz rain fade dominates and the ground segment is not the one costed here",
  "On what frequency does the payload data come down?",
  "itu_sa1862", "comms", "A. Rai / 2026-09-01")

D("com_tx_power", "Transmitter radio-frequency output power", "com", "comms", "Power", "Watt", "P_t",
  8.0, 0.1, 200.0,
  "below 0.1 W no link closes from this altitude",
  "above 200 W the transmitter is the largest load on the bus and the thermal design does not close",
  "How much radio-frequency power does the transmitter deliver?",
  "orbitt_case_c1", "comms", "A. Rai / 2026-09-01")

D("com_antenna_diameter", "Spacecraft antenna diameter", "com", "comms", "Length", "Metre", "D_a",
  0.30, 0.02, 2.0,
  "below 2 cm the aperture gain relation is not valid at this frequency",
  "above 2 m the antenna is a drag surface larger than the vehicle",
  "How large is the spacecraft downlink antenna?",
  "orbitt_case_c1", "comms", "A. Rai / 2026-09-01")

D("com_antenna_efficiency", "Antenna aperture efficiency", "com", "comms", "Ratio", "One", "eta_a",
  0.60, 0.2, 0.9,
  "below 0.2 the aperture is badly illuminated and the design is not one considered here",
  "above 0.9 no practical feed achieves it",
  "How efficiently does the antenna use its aperture?",
  "larson_wertz", "comms", "A. Rai / 2026-09-01")

D("com_line_loss", "Transmit line loss", "com", "comms", "Ratio", "One", "L_line",
  1.2, 0.0, 6.0,
  "a loss cannot be negative in decibels here",
  "above 6 dB most of the transmitter output is heating the harness",
  "How much is lost between the transmitter and the antenna, in decibels?",
  "larson_wertz", "comms", "A. Rai / 2026-09-01")

D("com_gs_antenna_diameter", "Ground station antenna diameter", "com", "comms", "Length", "Metre", "D_g",
  5.4, 0.5, 35.0,
  "below 0.5 m no ground station in the costed network is that small",
  "above 35 m is a deep-space aperture, not a commercial ground station",
  "How large is the ground station antenna?",
  "orbitt_case_c1", "comms", "A. Rai / 2026-09-01")

D("com_gs_noise_temperature", "Ground station antenna noise temperature", "com", "comms", "Temperature", "Kelvin", "T_a",
  60.0, 10.0, 300.0,
  "below 10 K requires cryogenic cooling that no commercial station in this network has",
  "above 300 K the antenna is looking at the ground",
  "How much noise does the ground antenna see?",
  "larson_wertz", "comms", "A. Rai / 2026-09-01")

D("com_required_ebn0", "Required energy per bit over noise density", "com", "comms", "Ratio", "One", "EbN0_req",
  4.5, 0.0, 20.0,
  "below 0 dB no practical coded modulation reaches the required bit error rate",
  "above 20 dB the modulation and coding are not the ones assumed here",
  "What energy per bit does the modulation and coding need, in decibels?",
  "ccsds131", "comms", "A. Rai / 2026-09-01")

D("com_implementation_loss", "Implementation loss", "com", "comms", "Ratio", "One", "L_imp",
  1.5, 0.0, 6.0,
  "an implementation loss cannot be negative",
  "above 6 dB the equipment is not flight hardware",
  "How far short of theory does the real modem fall, in decibels?",
  "ccsds131", "comms", "A. Rai / 2026-09-01")

D("com_atmospheric_loss", "Atmospheric and polarisation loss", "com", "comms", "Ratio", "One", "L_atm",
  1.8, 0.0, 15.0,
  "an atmospheric loss cannot be negative",
  "above 15 dB the link is not available at the stated elevation and frequency",
  "How much is lost to the atmosphere, rain and polarisation mismatch, in decibels?",
  "itu_p618", "comms", "A. Rai / 2026-09-01")

D("com_link_margin_required", "Required link margin", "com", "comms", "Ratio", "One", "M_req",
  3.0, 0.0, 12.0,
  "a required margin cannot be negative",
  "above 12 dB the link is being designed for an availability nobody asked for",
  "How much margin must the link carry above threshold, in decibels?",
  "ecss_e_st_50", "comms", "A. Rai / 2026-09-01")

D("com_passes_per_day", "Usable passes per day", "com", "comms", "Ratio", "One", "N_pass",
  9.0, 0.5, 60.0,
  "below half a pass a day the mission cannot be operated",
  "above 60 passes a day exceeds the number of revolutions in a day",
  "How many usable ground station contacts are there each day?",
  "orbitt_case_c1", "operations", "A. Rai / 2026-09-01",
  kpis=["kpi_latency", "kpi_data_volume"])

C("com_antenna_gain", "Spacecraft antenna gain", "com", "comms", "Ratio", "Decibel", "G_t",
  "How much does the spacecraft antenna concentrate the transmitted power?",
  "G = eta*(pi*D/lambda)^2",
  "larson_wertz", "comms",
  [("d", "com_antenna_diameter"), ("f", "com_frequency"), ("e", "com_antenna_efficiency")],
  [("apply the circular aperture gain relation at the carrier wavelength",
    "g", "Ratio", "Ratio::new(comms::aperture_gain_db(d, f, e))")],
  -10.0, 80.0,
  "an aperture antenna does not have less than -10 dBi of gain",
  "above 80 dBi is a deep-space aperture, not a spacecraft antenna",
  tier="A",
  fixtures=[("0.3 m at 8.2 GHz, 60% efficient", {"d": 0.30, "f": 8.2e9, "e": 0.60},
             26.00679804, 1e-8, "independent-derivation", "larson_wertz")])

C("com_gs_gain", "Ground station antenna gain", "com", "comms", "Ratio", "Decibel", "G_r",
  "How much does the ground antenna concentrate the received power?",
  "G = eta*(pi*D/lambda)^2",
  "larson_wertz", "comms",
  [("d", "com_gs_antenna_diameter"), ("f", "com_frequency"), ("e", "com_antenna_efficiency")],
  [("apply the same aperture relation to the ground antenna",
    "g", "Ratio", "Ratio::new(comms::aperture_gain_db(d, f, e))")],
  0.0, 80.0,
  "a ground aperture of this class always has positive gain",
  "above 80 dBi is a deep-space aperture",
  tier="A")

C("com_eirp", "Effective isotropic radiated power", "com", "comms", "Ratio", "Decibel", "EIRP",
  "How much power does the spacecraft appear to radiate towards the ground station?",
  "EIRP = 10*log10(P_t) + G_t - L_line",
  "larson_wertz", "comms",
  [("p", "com_tx_power"), ("g", "com_antenna_gain"), ("l", "com_line_loss")],
  [("add the antenna gain to the transmitter power in decibels and subtract the line loss",
    "e", "Ratio", "Ratio::new(comms::eirp_dbw(p, g.get(), l.get()))")],
  -20.0, 90.0,
  "below -20 dBW no link in this design closes",
  "above 90 dBW the transmitter is not the one described",
  tier="A")

C("com_path_loss", "Free space path loss", "com", "comms", "Ratio", "Decibel", "L_fs",
  "How much of the signal is lost simply by spreading out over the range?",
  "L = (4*pi*d/lambda)^2",
  "larson_wertz", "comms",
  [("d", "orbit_slant_range"), ("f", "com_frequency")],
  [("apply the inverse-square spreading loss at the working slant range",
    "l", "Ratio", "Ratio::new(comms::free_space_path_loss_db(d, f))")],
  100.0, 250.0,
  "below 100 dB the range is shorter than any orbit in this band",
  "above 250 dB the range is not one this tool covers",
  tier="A")

C("com_system_noise_temperature", "System noise temperature", "com", "comms", "Temperature", "Kelvin", "T_s",
  "How much noise does the receiving system contribute?",
  "T_s = T_a/L + T0*(L-1)/L + T0*(F-1)",
  "larson_wertz", "comms",
  [("ta", "com_gs_noise_temperature"), ("ll", "com_line_loss")],
  [("combine the antenna, line and receiver noise contributions at a 290 K physical temperature and a 1.2 dB noise figure",
    "t", "Temperature",
    "comms::system_noise_temperature(ta, ll.get(), 1.2, Temperature::new(290.0))")],
  10.0, 3000.0,
  "below 10 K requires cryogenic cooling not present in this ground segment",
  "above 3000 K the receiving system is not usable",
  tier="A")

C("com_g_over_t", "Ground station figure of merit", "com", "comms", "Ratio", "Decibel", "GT",
  "What is the ground station's sensitivity, as gain over system noise temperature?",
  "G/T = G_r - 10*log10(T_s)",
  "larson_wertz", "comms",
  [("g", "com_gs_gain"), ("t", "com_system_noise_temperature")],
  [("subtract the system noise temperature in decibels from the receive gain",
    "gt", "Ratio", "Ratio::new(comms::g_over_t_db(g.get(), t))")],
  -30.0, 60.0,
  "below -30 dB/K no ground station in this network performs that badly",
  "above 60 dB/K is a deep-space station",
  tier="A")

C("com_cn0", "Carrier to noise density ratio", "com", "comms", "Ratio", "Decibel", "CN0",
  "How strong is the received carrier relative to the noise floor?",
  "C/N0 = EIRP - L_fs - L_atm + G/T - 10*log10(k)",
  "larson_wertz", "comms",
  [("e", "com_eirp"), ("lf", "com_path_loss"), ("la", "com_atmospheric_loss"), ("gt", "com_g_over_t")],
  [("assemble the link equation in decibels",
    "c", "Ratio",
    "Ratio::new(comms::carrier_to_noise_density_db(e.get(), lf.get(), la.get(), gt.get()))")],
  20.0, 120.0,
  "below 20 dB-Hz no link in this design closes",
  "above 120 dB-Hz the geometry is not one this tool covers",
  tier="A")

C("com_data_rate", "Achievable downlink data rate", "com", "comms", "DataRate", "MegabitPerSecond", "R_b",
  "How fast can data come down while keeping the required margin?",
  "R = 10^((C/N0 - EbN0_req - L_imp - M)/10)",
  "larson_wertz", "comms",
  [("c", "com_cn0"), ("er", "com_required_ebn0"), ("li", "com_implementation_loss"),
   ("m", "com_link_margin_required")],
  [("solve the link equation for the rate that leaves exactly the required margin",
    "r", "DataRate",
    "comms::achievable_data_rate(c.get(), er.get(), li.get(), m.get())")],
  0.001, 100000.0,
  "below 1 kbit/s the downlink cannot even carry housekeeping",
  "above 100 Gbit/s no spacecraft modem in this class runs",
  tier="A",
  kpis=["kpi_data_volume"])

C("com_ebn0", "Achieved energy per bit over noise density", "com", "comms", "Ratio", "Decibel", "EbN0",
  "What energy per bit does the link actually deliver at the design rate?",
  "Eb/N0 = C/N0 - 10*log10(R)",
  "larson_wertz", "comms",
  [("c", "com_cn0"), ("r", "com_data_rate")],
  [("subtract the data rate in decibels from the carrier to noise density",
    "e", "Ratio", "Ratio::new(comms::eb_over_n0_db(c.get(), r))")],
  -10.0, 40.0,
  "below -10 dB no coded link closes",
  "above 40 dB the link is overdesigned by orders of magnitude",
  tier="A")

C("com_link_margin", "Link margin", "com", "closure", "Ratio", "Decibel", "M_link",
  "Does the downlink close with margin to spare?",
  "M = Eb/N0 - EbN0_req - L_imp",
  "ecss_e_st_50", "comms",
  [("e", "com_ebn0"), ("er", "com_required_ebn0"), ("li", "com_implementation_loss")],
  [("subtract the requirement and the implementation loss from what the link delivers",
    "m", "Ratio", "Ratio::new(comms::link_margin_db(e.get(), er.get(), li.get()))")],
  -30.0, 40.0,
  "below -30 dB the link is not close to closing and the design is wrong somewhere upstream",
  "above 40 dB the link is overdesigned by orders of magnitude",
  tier="A",
  kpis=["kpi_data_volume"])

C("com_contact_time", "Maximum contact time per pass", "com", "comms", "Time", "Minute", "t_con",
  "How long is one ground station pass?",
  "t = T_orb*lambda/pi",
  "larson_wertz", "comms",
  [("lam", "orbit_earth_central_angle"), ("p", "orbit_period")],
  [("take the fraction of the revolution spent inside the access circle",
    "t", "Time", "comms::max_contact_time(lam, p)")],
  0.0, 60.0,
  "a contact time cannot be negative",
  "above an hour is not a low-orbit pass",
  tier="A",
  assumptions=[("A pass directly overhead",
                "the mean over many passes is roughly 60% of this, so a downlink volume computed from it will not be achieved")])

C("com_pass_volume", "Data volume per pass", "com", "comms", "DataVolume", "Gigabit", "V_pass",
  "How much data comes down in one pass?",
  "V = R*t*eta_link",
  "larson_wertz", "comms",
  [("r", "com_data_rate"), ("t", "com_contact_time")],
  [("multiply rate by contact time and apply a 60% mean-pass and protocol efficiency",
    "v", "DataVolume", "comms::pass_data_volume(r, t, Ratio::new(0.60))")],
  0.0, 1.0e6,
  "a data volume cannot be negative",
  "above a petabit per pass no modem in this class runs",
  tier="B")

C("com_daily_volume", "Daily downlink capacity", "com", "comms", "DataVolume", "Gigabit", "V_day",
  "How much data can the system bring down in a day?",
  "V = V_pass*N_pass*A",
  "larson_wertz", "operations",
  [("v", "com_pass_volume"), ("n", "com_passes_per_day")],
  [("multiply by the usable passes per day at 95% station availability",
    "vd", "DataVolume", "comms::daily_downlink_volume(v, n.get(), Ratio::new(0.95))")],
  0.0, 1.0e7,
  "a data volume cannot be negative",
  "above 10 petabits a day no ground segment in this design receives it",
  tier="B",
  kpis=["kpi_data_volume"])

C("com_doppler", "Doppler shift at closest approach", "com", "comms", "Frequency", "Megahertz", "df",
  "How far does the carrier move in frequency as the satellite passes?",
  "df = f*v/c",
  "larson_wertz", "comms",
  [("f", "com_frequency"), ("v", "orbit_ground_track_speed")],
  [("apply the first-order Doppler relation at the ground track speed",
    "d", "Frequency", "comms::doppler_shift(f, v)")],
  0.0, 10.0,
  "a shift magnitude cannot be negative",
  "above 10 MHz the receiver acquisition range is exceeded and the link never locks",
  tier="A")


# =============================================================================
# SERVICE PAYLOADS
# =============================================================================
D("pay_aperture", "Optical aperture diameter", "pay", "pay_optical", "Length", "Metre", "D_o",
  0.30, 0.02, 1.5,
  "below 2 cm the diffraction limit is coarser than any useful ground sample distance from this altitude",
  "above 1.5 m the telescope does not fit the launch envelope of this class",
  "How large is the telescope's entrance pupil?",
  "orbitt_case_c1", "payload", "A. Rai / 2026-09-01",
  kpis=["kpi_gsd"])

D("pay_focal_length", "Optical focal length", "pay", "pay_optical", "Length", "Metre", "f_o",
  2.4, 0.05, 20.0,
  "below 5 cm the detector-limited sample is coarser than anything useful",
  "above 20 m no folded optical path of this class is that long",
  "What is the effective focal length of the telescope?",
  "orbitt_case_c1", "payload", "A. Rai / 2026-09-01")

D("pay_pixel_pitch", "Detector pixel pitch", "pay", "pay_optical", "Length", "Micrometre", "p_x",
  5.5, 1.0, 30.0,
  "below one micrometre no space-qualified detector has pixels that small",
  "above 30 micrometres the detector-limited sample dominates and the aperture is wasted",
  "How large is one detector pixel?",
  "orbitt_case_c1", "payload", "A. Rai / 2026-09-01")

D("pay_wavelength", "Centre wavelength", "pay", "pay_optical", "Length", "Nanometre", "lam_o",
  550.0, 350.0, 2500.0,
  "below 350 nm the atmosphere absorbs most of the signal",
  "above 2500 nm the detector is a cooled infrared array, which is not the payload described here",
  "At what wavelength does the imager work?",
  "orbitt_case_c1", "payload", "A. Rai / 2026-09-01")

D("pay_pixels_across", "Detector pixels across track", "pay", "pay_optical", "Ratio", "One", "N_x",
  8192.0, 256.0, 40000.0,
  "below 256 pixels the swath is too narrow to be useful",
  "above 40000 pixels no single space detector array is that wide",
  "How many pixels does the detector have across the swath?",
  "orbitt_case_c1", "payload", "A. Rai / 2026-09-01")

D("pay_bits_per_pixel", "Bits per pixel", "pay", "pay_optical", "Ratio", "One", "b_px",
  12.0, 8.0, 16.0,
  "below 8 bits the dynamic range is not usable for radiometry",
  "above 16 bits exceeds the detector's own noise floor and the extra bits carry noise",
  "How many bits does each sample carry?",
  "orbitt_case_c1", "payload", "A. Rai / 2026-09-01")

D("pay_compression", "Image compression ratio", "pay", "pay_optical", "Ratio", "One", "CR_img",
  3.0, 1.0, 20.0,
  "unity is no compression, the limit",
  "above 20:1 the compression is lossy enough to destroy the radiometry the product is sold on",
  "By what factor is the imagery compressed before downlink?",
  "ccsds122", "payload", "A. Rai / 2026-09-01")

D("pay_quantum_efficiency", "Detector quantum efficiency", "pay", "pay_optical", "Ratio", "One", "QE",
  0.75, 0.05, 0.95,
  "below 5% the detector is not one considered here",
  "above 95% no back-illuminated detector reaches it across the band",
  "What fraction of incident photons become photoelectrons?",
  "orbitt_case_c1", "payload", "A. Rai / 2026-09-01")

D("pay_transmission", "Optical transmission", "pay", "pay_optical", "Ratio", "One", "tau_o",
  0.72, 0.1, 0.98,
  "below 10% the optical train is not one considered here",
  "above 98% no multi-element telescope achieves it",
  "What fraction of light entering the aperture reaches the detector?",
  "orbitt_case_c1", "payload", "A. Rai / 2026-09-01")

D("pay_radiance", "At-aperture spectral radiance", "pay", "pay_optical", "Ratio", "One", "L_ap",
  60.0, 0.1, 500.0,
  "below 0.1 W/m2/sr/um the scene is darker than any night-time reference used here",
  "above 500 W/m2/sr/um the scene is brighter than a desert at noon",
  "How bright is the scene at the aperture, per unit wavelength?",
  "modtran_ref", "payload", "A. Rai / 2026-09-01")

D("pay_read_noise", "Detector read noise", "pay", "pay_optical", "Ratio", "One", "n_read",
  8.0, 1.0, 100.0,
  "below one electron no space detector in this class reads that quietly",
  "above 100 electrons the detector is not usable for this product",
  "How many electrons of noise does one read add?",
  "orbitt_case_c1", "payload", "A. Rai / 2026-09-01")

D("pay_power", "Payload orbit-average power", "pay", "payload", "Power", "Watt", "P_pay",
  120.0, 0.0, 3000.0,
  "a payload load cannot be negative",
  "above 3 kW the payload is the spacecraft",
  "How much power does the whole payload set draw, averaged over an orbit?",
  "orbitt_case_c1", "payload", "A. Rai / 2026-09-01",
  kpis=["kpi_power_margin"])

D("pay_rf_bandwidth", "Intercept receiver bandwidth", "pay", "pay_rf", "Frequency", "Megahertz", "B_rf",
  20.0, 0.1, 500.0,
  "below 100 kHz the timing precision is too coarse for any geolocation requirement here",
  "above 500 MHz the digitiser and the downlink neither fit nor close",
  "Over what bandwidth does the geolocation payload observe?",
  "orbitt_case_c2", "payload", "A. Rai / 2026-09-01")

D("pay_rf_snr", "Intercept signal to noise ratio", "pay", "pay_rf", "Ratio", "One", "SNR_rf",
  10.0, 0.1, 1000.0,
  "below 0.1 the signal is not detectable at all",
  "above 1000 the emitter is so strong that geolocation is not the limiting problem",
  "What signal to noise ratio does the intercepted emitter present, as a linear ratio?",
  "orbitt_case_c2", "payload", "A. Rai / 2026-09-01")

D("pay_rf_integration", "Intercept integration time", "pay", "pay_rf", "Time", "Second", "t_int_rf",
  0.010, 1.0e-6, 10.0,
  "below a microsecond the emitter is not observed long enough to time it",
  "above 10 seconds the emitter and the geometry both move during the observation",
  "How long is the emitter observed for one measurement?",
  "orbitt_case_c2", "payload", "A. Rai / 2026-09-01")

D("pay_rf_gdop", "Geolocation geometric dilution", "pay", "pay_rf", "Ratio", "One", "GDOP_rf",
  4.0, 1.0, 50.0,
  "a dilution below one is not geometrically possible",
  "above 50 the formation geometry is degenerate and no solution is produced",
  "How much does the formation geometry amplify timing error into position error?",
  "orbitt_case_c2", "payload", "A. Rai / 2026-09-01",
  kpis=["kpi_geolocation"])

C("pay_gsd_diffraction", "Diffraction-limited ground sample", "pay", "pay_optical", "Length", "Metre", "GSD_d",
  "What is the finest detail the optics can resolve, whatever the detector?",
  "GSD = 1.22*lambda*h/D",
  "larson_wertz", "payload",
  [("h", "orbit_altitude"), ("lam", "pay_wavelength"), ("d", "pay_aperture")],
  [("apply the Rayleigh criterion at the flight altitude",
    "g", "Length", "payload::diffraction_limited_gsd(h, lam, d)")],
  0.001, 1000.0,
  "below a millimetre the relation is being fed an aperture that is not physical",
  "above a kilometre no product in this design is being made",
  tier="A",
  kpis=["kpi_gsd"],
  fixtures=[("250 km, 550 nm, 0.3 m aperture", {"h": 250000.0, "lam": 5.5e-7, "d": 0.30},
             0.55917, 1e-4, "independent-derivation", "larson_wertz")],
  view=("line", {"over": "orbit_altitude", "points": 60}))

C("pay_gsd_detector", "Detector-limited ground sample", "pay", "pay_optical", "Length", "Metre", "GSD_p",
  "What is the finest detail the detector can sample, whatever the optics?",
  "GSD = h*p/f",
  "larson_wertz", "payload",
  [("h", "orbit_altitude"), ("p", "pay_pixel_pitch"), ("f", "pay_focal_length")],
  [("project one pixel onto the ground through the focal length",
    "g", "Length", "payload::detector_limited_gsd(h, p, f)")],
  0.001, 1000.0,
  "below a millimetre the relation is being fed a focal length that is not physical",
  "above a kilometre no product in this design is being made",
  tier="A",
  fixtures=[("250 km, 5.5 um pixel, 2.4 m focal length", {"h": 250000.0, "p": 5.5e-6, "f": 2.4},
             0.572917, 1e-5, "independent-derivation", "larson_wertz")])

C("pay_gsd", "Achieved ground sample distance", "pay", "pay_optical", "Length", "Metre", "GSD",
  "What ground sample distance does the system actually achieve?",
  "GSD = max(GSD_diffraction, GSD_detector)",
  "larson_wertz", "payload",
  [("gd", "pay_gsd_diffraction"), ("gp", "pay_gsd_detector")],
  [("take the worse of the two limits, never the better",
    "g", "Length", "payload::achieved_gsd(gd, gp)")],
  0.001, 1000.0,
  "below a millimetre the inputs are not physical",
  "above a kilometre no product in this design is being made",
  tier="A",
  kpis=["kpi_gsd"],
  note="Taking the better of the two produces a resolution claim the optics cannot deliver and the detector cannot sample, and it survives review because both inputs are individually correct.")

C("pay_swath", "Optical swath width", "pay", "pay_optical", "Length", "Kilometre", "W_o",
  "How wide a strip does one optical pass image?",
  "W = GSD*N_x",
  "larson_wertz", "payload",
  [("g", "pay_gsd"), ("n", "pay_pixels_across")],
  [("multiply the ground sample distance by the across-track pixel count",
    "w", "Length", "payload::optical_swath(g, n.get())")],
  0.1, 1000.0,
  "below 100 m the swath is not a swath",
  "above 1000 km the field of view is not one this telescope has",
  tier="A",
  kpis=["kpi_revisit"])

C("pay_dwell_time", "Per-sample dwell time", "pay", "pay_optical", "Time", "Second", "t_dwell",
  "How long does the sensor have to collect photons from one ground sample?",
  "t = GSD/V_g",
  "larson_wertz", "payload",
  [("g", "pay_gsd"), ("v", "orbit_ground_track_speed")],
  [("divide the ground sample distance by the ground track speed",
    "t", "Time", "payload::dwell_time(g, v)")],
  1.0e-9, 1.0,
  "below a nanosecond no detector integrates",
  "above one second the ground has moved many samples during the integration",
  tier="A",
  note="The constraint low altitude imposes and higher orbits do not: the ground moves under the sensor faster in angular terms, which partly gives back the signal-to-noise the shorter range won.")

C("pay_signal_electrons", "Signal electrons per sample", "pay", "pay_optical", "Ratio", "One", "N_e",
  "How many photoelectrons does one ground sample produce?",
  "N_e = L*Omega*A_px*B*tau*QE*t/(h*c/lambda)",
  "larson_wertz", "payload",
  [("l", "pay_radiance"), ("d", "pay_aperture"), ("f", "pay_focal_length"), ("p", "pay_pixel_pitch"),
   ("tau", "pay_transmission"), ("qe", "pay_quantum_efficiency"), ("t", "pay_dwell_time"),
   ("lam", "pay_wavelength")],
  [("collect photons through the aperture solid angle over the dwell time and convert them at the detector's quantum efficiency, over a 100 nm band",
    "n", "Ratio",
    "Ratio::new(payload::signal_electrons(l.get() * 1.0e6, d, f, p, tau, qe, t, lam, Length::new(1.0e-7)))")],
  0.0, 1.0e9,
  "an electron count cannot be negative",
  "above a billion electrons the well is saturated many times over",
  tier="B")

C("pay_snr", "Optical signal to noise ratio", "pay", "pay_optical", "Ratio", "One", "SNR_o",
  "How clean is one sample?",
  "SNR = N_e/sqrt(N_e + N_dark + n_read^2)",
  "larson_wertz", "payload",
  [("n", "pay_signal_electrons"), ("nr", "pay_read_noise")],
  [("combine shot noise, a nominal 5-electron dark contribution and read noise in quadrature",
    "s", "Ratio",
    "Ratio::new(payload::optical_snr(n.get(), 5.0, nr.get()))")],
  0.0, 10000.0,
  "a signal to noise ratio cannot be negative",
  "above 10000 the detector is far outside its linear range",
  tier="B",
  kpis=["kpi_gsd"])

C("pay_scene_volume", "Data volume per scene", "pay", "pay_optical", "DataVolume", "Gigabit", "V_scene",
  "How much data is one imaged scene?",
  "V = N_x*N_y*b*bands/CR",
  "ccsds122", "payload",
  [("n", "pay_pixels_across"), ("b", "pay_bits_per_pixel"), ("cr", "pay_compression")],
  [("assume a square scene of the across-track width, in four spectral bands",
    "v", "DataVolume",
    "payload::scene_data_volume(n.get(), n.get(), b.get(), 4.0, cr.get())")],
  0.0, 1.0e6,
  "a data volume cannot be negative",
  "above a petabit no single scene in this design is produced",
  tier="A",
  kpis=["kpi_data_volume"])

C("pay_toa_uncertainty", "Time of arrival uncertainty", "pay", "pay_rf", "Time", "Second", "sig_tau",
  "How precisely can the payload time an intercepted signal?",
  "sigma = 1/(2*pi*B*sqrt(2*SNR*B*T))",
  "larson_wertz", "payload",
  [("b", "pay_rf_bandwidth"), ("s", "pay_rf_snr"), ("t", "pay_rf_integration")],
  [("apply the Cramer-Rao lower bound on time-of-arrival estimation",
    "sg", "Time", "payload::toa_timing_uncertainty(b, s.get(), t)")],
  1.0e-15, 1.0,
  "below a femtosecond no receiver in this design times that precisely",
  "above one second the measurement carries no geolocation information",
  tier="A")

C("pay_geolocation_error", "RF geolocation error", "pay", "pay_rf", "Length", "Metre", "e_geo",
  "How accurately can an emitter on the ground be located?",
  "e = c*sigma_tau*GDOP",
  "larson_wertz", "payload",
  [("s", "pay_toa_uncertainty"), ("g", "pay_rf_gdop")],
  [("convert the timing uncertainty into a range uncertainty and apply the geometric dilution",
    "e", "Length", "payload::tdoa_geolocation_error(s, g.get())")],
  0.01, 1.0e7,
  "below a centimetre no time-difference system in this design performs that well",
  "above 10000 km the answer carries no information",
  tier="B",
  kpis=["kpi_geolocation"],
  view=("line", {"over": "pay_rf_bandwidth", "points": 60}))


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


def amend(nid, **kw):
    """Rewire a node after the fact.

    Used only here, and only to close the power-thermal-propulsion loop after
    both of its ends have been declared. Every amendment is visible in this
    file, which is the point: a sheet that was quietly edited by a later pass
    is a sheet nobody can review.
    """
    for n in NODES:
        if n["id"] == nid:
            n.update(kw)
            return
    raise SystemExit("amend: no node %s" % nid)


# The array's end-of-life output now carries the temperature derating, which is
# what turns three independent chains into one system.
amend("pwr_array_power_eol",
      inputs=[("p", "pwr_array_power_bol"), ("f", "pwr_degradation"), ("ft", "pwr_cell_derating")],
      expression="P_eol = P_bol*f_deg*f_T",
      steps=[("apply both the ageing degradation and the temperature derating to the beginning-of-life output",
              "pe", "Power", "power::array_power_eol(p, f) * ft.get()")])

# The bus demand sees what propulsion actually drew, not what it asked for.
amend("pwr_demand",
      inputs=[("pp", "prop_delivered_bus_power"), ("pay", "pay_power"), ("av", "pwr_avionics_load"),
              ("com", "com_tx_power"), ("th", "pwr_thermal_load"), ("lh", "pwr_harness_loss")])

# And the closure compares delivered thrust against drag.
amend("prop_thrust_to_drag",
      inputs=[("t", "prop_delivered_thrust"), ("d", "aero_drag_force")])


# =============================================================================
# KPIs — required against achieved
#
# The closure rule, repeated identically at every boundary. A required row and
# an achieved row, compared the same way whether the boundary is customer to
# system or system to subsystem. That repetition is one of the five ideas here
# that is ours and unproven: the standards do this at review gates, not
# continuously.
# =============================================================================
def KPI(nid, label, req_value, req_unit, req_ty, achieved_node, sense, q, src,
        req_lo, req_hi, req_rlo, req_rhi, note=""):
    D(nid + "_required", label + " — required", "kpi", "mgt_customer", req_ty, req_unit,
      "R_req", req_value, req_lo, req_hi, req_rlo, req_rhi,
      "What does the customer require for " + label.lower() + "?",
      src, "systems", "A. Rai / 2026-09-01", tier="A")
    C(nid, label + " — closure", "kpi", "applications", "Ratio", "One", "M",
      q, "M = (achieved - required)/required, in the sense the requirement is stated",
      src, "systems",
      [("req", nid + "_required"), ("ach", achieved_node)],
      [("compare achieved against required in the declared sense and return the signed fractional margin",
        "m", "Ratio",
        "Ratio::new(mission::closure(req.get(), ach.get(), mission::Sense::%s).margin)" % sense)],
      -100.0, 1000.0,
      "a margin below -10000% means the inputs are not the ones the requirement was written against",
      "a margin above 100000% means the requirement is not the one that binds",
      tier="A", kind="kpi", note=note)


KPI("kpi_revisit", "Mean revisit time", 6.0, "Hour", "Time", "mis_revisit", "AtMost",
    "Does the constellation revisit often enough?", "orbitt_case_c1",
    0.1, 720.0, "below six minutes no constellation in this class achieves it",
    "above 30 days the service is not the one being sold")

KPI("kpi_gsd", "Ground sample distance", 1.0, "Metre", "Length", "pay_gsd", "AtMost",
    "Is the imagery sharp enough?", "orbitt_case_c1",
    0.05, 100.0, "below 5 cm no optical payload in this mass class achieves it",
    "above 100 m the product is not the one being sold")

KPI("kpi_geolocation", "Emitter geolocation accuracy", 500.0, "Metre", "Length",
    "pay_geolocation_error", "AtMost",
    "Are emitters located accurately enough?", "orbitt_case_c2",
    1.0, 100000.0, "below one metre no time-difference system in this class achieves it",
    "above 100 km the product carries no information")

KPI("kpi_latency", "End to end latency", 30.0, "Minute", "Time", "mis_latency", "AtMost",
    "Does the product reach the customer fast enough?", "orbitt_case_c1",
    1.0, 2880.0, "below a minute no ground segment in this design delivers",
    "above two days the product is not the near-real-time one being sold")

KPI("kpi_availability", "Service availability", 0.99, "One", "Ratio", "mis_availability", "AtLeast",
    "Is the service available often enough?", "orbitt_case_c1",
    0.5, 0.99999, "below 50% the service is unavailable more often than not",
    "above five nines no constellation in this class has demonstrated it")

KPI("kpi_data_volume", "Daily downlink volume", 500.0, "Gigabit", "DataVolume",
    "com_daily_volume", "AtLeast",
    "Does enough data reach the ground each day?", "orbitt_case_c1",
    1.0, 1.0e7, "below one gigabit a day the mission returns nothing useful",
    "above 10 petabits a day the ground segment is not the one costed")

KPI("kpi_service_lifetime", "Service lifetime", 5.0, "Year", "Time",
    "orbit_lifetime_uncontrolled", "AtLeast",
    "Does the satellite stay up long enough with no propulsion at all?", "orbitt_case_c1",
    0.1, 25.0, "below five weeks the satellite is not a mission",
    "above 25 years the debris mitigation standard is violated",
    note="Deliberately measured against the uncontrolled lifetime rather than the propelled one, so the answer does not depend on the propulsion system working.")

KPI("kpi_cost_per_year", "Cost per operational year", 45.0, "MillionUsDollar", "Money",
    "cost_per_year", "AtMost",
    "Does a year of service cost what the business case allows?", "orbitt_case_c1",
    0.1, 10000.0, "below 100 thousand a year no programme of this size runs",
    "above 10 billion a year the programme is not the one being designed")

KPI("kpi_thrust_margin", "Thrust to drag closure", 1.0, "One", "Ratio",
    "prop_thrust_to_drag", "AtLeast",
    "Does thrust exceed drag — does the design close at all?", "romano2021",
    0.1, 5.0, "a required ratio below 0.1 is not a closure requirement",
    "above 5 the requirement is not one an air-breathing design is written against",
    note="The one closure the whole programme exists to reach. Everything else is a detail until this is at or above zero margin.")

KPI("kpi_power_margin", "Power margin", 0.20, "One", "Ratio", "pwr_margin", "AtLeast",
    "Is there enough power, with the margin the standard requires?", "ecss_e_st_10_02",
    0.0, 1.0, "a negative required margin is not a requirement",
    "above 100% required margin the design is being asked to double its power for no stated reason")

KPI("kpi_thermal_margin", "Thermal margin", 10.0, "Kelvin", "Temperature", "thm_margin", "AtLeast",
    "Is the spacecraft comfortably inside its temperature limit?", "ecss_e_st_10_02",
    0.0, 100.0, "a negative required margin is not a requirement",
    "above 100 K the requirement is not one this design is written against")

KPI("kpi_mass_margin", "Mass margin", 0.10, "One", "Ratio", "mass_margin", "AtLeast",
    "Does the spacecraft fit its mass allocation with margin?", "ecss_e_st_10_02",
    0.0, 0.9, "a negative required margin is not a requirement",
    "above 90% required margin the allocation is nine times the design")


# =============================================================================
# THE EMITTER
# One folder per node. Everything about a node is in one directory: adding one
# is a copy, deleting one is a remove, its history is the log of a directory,
# and ownership is a path rule.
# =============================================================================
SUBSYS_CRATE = {
    "env": "vleo-mod-env", "orbit": "vleo-mod-orbit", "aero": "vleo-mod-aero",
    "prop": "vleo-mod-prop", "power": "vleo-mod-power", "thm": "vleo-mod-thermal",
    "gnc": "vleo-mod-gnc", "com": "vleo-mod-comms", "pay": "vleo-mod-payload",
    "mass": "vleo-mod-mass", "mis": "vleo-mod-mission", "cost": "vleo-mod-cost",
    "kpi": "vleo-mod-mission",
}


def folder_for(n):
    """Frozen at seed and recorded in the sheet. Editing a label never moves a
    directory."""
    fid = n["id"]
    # KPI rows keep their whole identifier: they share a crate with the mission
    # rows and `kpi_revisit` and `mis_revisit` would otherwise both want the
    # folder `revisit`. Caught by the collision check in main(), which is where
    # a folder collision has to be caught — it is the one thing the frozen-name
    # rule makes possible.
    if fid.startswith("kpi_"):
        return fid
    for pre in ("env_", "orbit_", "aero_", "prop_", "pwr_", "thm_", "gnc_",
                "com_", "pay_", "mass_", "mis_", "cost_"):
        if fid.startswith(pre):
            return fid[len(pre):] or fid
    return fid


def crate_for(n):
    if n["id"].startswith("pwr_"):
        return "vleo-mod-power"
    return SUBSYS_CRATE[n["subsystem"]]


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        f.write(text)


def emit_node(n):
    crate = crate_for(n)
    folder = folder_for(n)
    base = os.path.join(ROOT, "crates", crate, "nodes", folder)
    n["_crate"] = crate
    n["_folder"] = folder
    L = []
    a = L.append
    a("# The sheet. This is the only file in this folder that must be written by")
    a("# hand, and every other artefact here is generated from it. A hand edit")
    a("# outside a numbered HOLE block in model.rs fails the regeneration diff.")
    a('id = %s' % toml_str(n["id"]))
    a('label = %s' % toml_str(n["label"]))
    a('folder = %s   # frozen at seed; renaming the label never moves the directory' % toml_str(folder))
    a('subsystem = %s' % toml_str(n["subsystem"]))
    a('parent = %s' % toml_str(n["parent"]))
    a('kind = %s' % toml_str(n["kind"]))
    a('owner = %s' % toml_str(n["owner"]))
    a('tier = %s' % toml_str(n["tier"]))
    a('state = "published"')
    a("")
    a("[question]")
    a('text = %s' % toml_str(n["question"]))
    if n["note"]:
        a('note = %s' % toml_str(n["note"]))
    a("")
    a("[maths]")
    a('expression = %s' % toml_str(n["expression"]))
    a('source = %s' % toml_str(n["source"]))
    a("")
    for text, fails in n["assumptions"]:
        a("[[assumption]]")
        a('text = %s' % toml_str(text))
        a('fails_when = %s' % toml_str(fails))
        a("")
    a("[output]")
    a("# The node's one answer. The variable id is the node id: one small")
    a("# question, one answer, one folder, one row on the tree.")
    a('symbol = %s' % toml_str(n["symbol"]))
    a('type = %s' % toml_str(n["ty"]))
    a('unit = %s' % toml_str(n["unit"]))
    a('lower = %r' % float(n["lo"]))
    a('upper = %r' % float(n["hi"]))
    a('reason_lower = %s' % toml_str(n["rlo"]))
    a('reason_upper = %s' % toml_str(n["rhi"]))
    a("")
    if n["kind"] == "declared":
        a("[value]")
        a("# A number a person picked. It carries the same discipline as a")
        a("# fixture, because every margin in the design is built out of these.")
        a('number = %r' % float(n["value"]))
        a('confirmed_by = %s' % toml_str(n["confirmed"]))
        a("")
    for name, var in n["inputs"]:
        a("[[input]]")
        a("# The consumer declares what it expects. Assembly refuses a mismatch")
        a("# against what the producer publishes — which is the only place the")
        a("# whole graph is visible and therefore the only place to ask.")
        a('binding = %s' % toml_str(name))
        a('var = %s' % toml_str(var))
        a('type = %s' % toml_str(BY_ID[var]["ty"]))
        a("")
    for i, (text, binds, ty, _body) in enumerate(n["steps"], 1):
        a("[[algorithm.step]]")
        a('number = %d' % i)
        a('text = %s' % toml_str(text))
        a('binds = %s' % toml_str(binds))
        a('type = %s' % toml_str(ty))
        a("")
    if n["kpis"]:
        a("[contributes]")
        a('kpis = [%s]' % ", ".join(toml_str(k) for k in n["kpis"]))
        a("")
    if n["bundles"]:
        a("[data]")
        a('bundles = [%s]' % ", ".join(toml_str(b) for b in n["bundles"]))
        a("")
    kind, opts = n["view"]
    a("[view]")
    a('kind = %s' % toml_str(kind))
    for k, v in opts.items():
        a('%s = %s' % (k, toml_str(v) if isinstance(v, str) else repr(v)))
    a("")
    write(os.path.join(base, "node.toml"), "\n".join(L) + "\n")

    # The holes: the only lines a person or an agent types into the code.
    if n["steps"]:
        H = ["// Seeded hole bodies. `cargo xtask docs` regenerates everything",
             "// around these markers and splices the bodies back in unchanged.",
             "// A hand edit outside a HOLE block fails the regeneration diff.",
             ""]
        for i, (text, binds, ty, body) in enumerate(n["steps"], 1):
            H.append("// ---- HOLE %d : %s -> %s" % (i, text, ty))
            H.append("let %s: %s = %s;" % (binds, ty, body))
            H.append("// ---- end HOLE %d" % i)
            H.append("")
        write(os.path.join(base, "model.rs"), "\n".join(H) + "\n")

    F = ["# Known-good values, and where each came from.",
         "#",
         "# The one rule: an expected value may never be produced by the code",
         "# under test. The schema refuses a row whose provenance is the",
         "# implementation, because a number worked out by the thing being",
         "# checked proves nothing.",
         ""]
    for label, inputs, expect, tol, prov, src in n["fixtures"]:
        F.append("[[fixture]]")
        F.append('label = %s' % toml_str(label))
        F.append('expect = %r' % float(expect))
        F.append('tolerance = %r' % float(tol))
        F.append('provenance = %s' % toml_str(prov))
        F.append('source = %s' % toml_str(src))
        F.append("inputs = { %s }" % ", ".join("%s = %r" % (k, float(v)) for k, v in inputs.items()))
        F.append("")
    write(os.path.join(base, "fixtures.toml"), "\n".join(F) + "\n")


SOURCES = [
    ("jacchia1971", "Jacchia, L. G. — Revised static models of the thermosphere and exosphere with empirical temperature profiles", "SAO Special Report 332, 1971", "current",
     "The temperature profile, the exospheric temperature relation and the diffusive-equilibrium structure the atmosphere node implements."),
    ("us_std_1976", "U.S. Standard Atmosphere, 1976", "NOAA-S/T 76-1562", "current",
     "The 120 km base state the diffusive-equilibrium integration starts from, and the collision cross-section."),
    ("sentman1961", "Sentman, L. H. — Free molecule flow theory and its application to the determination of aerodynamic forces", "Lockheed LMSC-448514, 1961", "current",
     "The free-molecular drag and lift coefficients for a surface element."),
    ("doornbos2011", "Doornbos, E. — Thermospheric density and wind determination from satellite dynamics", "TU Delft, 2011", "current",
     "The form of Sentman's relations used here, and the published residual of empirical thermosphere models against measured drag."),
    ("moe2005", "Moe, K. and Moe, M. M. — Gas-surface interactions and satellite drag coefficients", "Planetary and Space Science 53, 2005", "current",
     "The Langmuir adsorption model for energy accommodation, and why accommodation moves with solar activity."),
    ("romano2021", "Romano, F. et al. — Intake design for an atmosphere-breathing electric propulsion system", "Acta Astronautica, 2021", "current",
     "The intake flux balance, the compression ratio and collection efficiency definitions, and the thruster efficiency chain."),
    ("vallado2013", "Vallado, D. A. — Fundamentals of Astrodynamics and Applications, 4th edition", "Microcosm Press, 2013", "current",
     "The two-body and J2 secular relations, the decay relation and the rocket equation."),
    ("larson_wertz", "Wertz, J. R., Everett, D. F. and Puschell, J. J. — Space Mission Engineering: The New SMAD", "Microcosm Press, 2011", "current",
     "The access geometry, the link budget, the power and thermal budgets, the disturbance torques and the payload relations."),
    ("wgs84", "NIMA TR8350.2 — World Geodetic System 1984", "3rd edition, 2000", "current",
     "The Earth equatorial radius, flattening, gravitational parameter and J2."),
    ("codata2018", "CODATA 2018 recommended values of the fundamental physical constants", "NIST, 2019", "current",
     "Boltzmann's constant, the gas constant, Avogadro's number, the elementary charge and the atomic mass unit."),
    ("igrf2020", "International Geomagnetic Reference Field, 13th generation", "IAGA, 2020", "current",
     "The dipole moment used in the magnetic field approximation."),
    ("iso24113", "ISO 24113 — Space systems: space debris mitigation requirements", "ISO, 2019", "current",
     "The disposal requirement that makes deorbit delta-v a design variable rather than an afterthought."),
    ("ecss_e_st_10_02", "ECSS-E-ST-10-02C — Verification", "ESA-ESTEC, 2018", "current",
     "The margin philosophy, the mass growth allowances and the required-against-achieved closure convention."),
    ("ecss_e_st_50", "ECSS-E-ST-50-05C — Radio frequency and modulation", "ESA-ESTEC, 2011", "current",
     "The link margin convention."),
    ("ccsds131", "CCSDS 131.0-B — TM synchronization and channel coding", "CCSDS, 2021", "current",
     "The required energy per bit for the assumed coded modulation."),
    ("ccsds122", "CCSDS 122.0-B — Image data compression", "CCSDS, 2017", "current",
     "The compression ratio range the imagery budget is written against."),
    ("itu_p618", "ITU-R P.618 — Propagation data and prediction methods for earth-space telecommunication systems", "ITU, 2023", "current",
     "The atmospheric and rain attenuation the link budget carries."),
    ("itu_sa1862", "ITU-R SA.1862 — Frequency bands for Earth exploration-satellite service downlinks", "ITU, 2010", "current",
     "The X-band downlink allocation the frequency is chosen inside."),
    ("noaa_swpc", "NOAA Space Weather Prediction Center — solar and geomagnetic indices", "NOAA, retrieved 2026-09-01", "current",
     "The F10.7 and Kp drivers, distributed as the solar-drivers bundle."),
    ("nist_asd", "NIST Atomic Spectra Database, ionisation energies", "NIST, 2023", "current",
     "The first ionisation energies that set the floor on the energy an ABEP thruster spends per ion."),
    ("ciaaw2021", "CIAAW — Standard atomic weights", "IUPAC, 2021", "current",
     "The molar masses of the atmospheric species."),
    ("nasa_cer", "NASA Cost Estimating Handbook, version 4.0", "NASA, 2015", "current",
     "The form of the cost estimating relations and the learning curve convention."),
    ("modtran_ref", "At-aperture radiance reference case, mid-latitude summer, 30 degree solar zenith", "MODTRAN 6 run, 2026-08-14", "current",
     "The scene radiance the optical signal-to-noise chain is evaluated at."),
    ("orbitt_case_c1", "Orbitt Space — VLEO multipayload reference case C1", "internal, 2026-09-01", "current",
     "Every declared value that is a programme decision rather than a physical constant."),
    ("orbitt_case_c2", "Orbitt Space — VLEO multipayload reference case C2", "internal, 2026-09-01", "current",
     "The comms and PNT case's declared values."),
    ("matlab_legacy", "Orbitt Space — legacy MATLAB feasibility tool, 364 files", "internal, 2025", "current",
     "An independent implementation, used as a second opinion. Its outputs are a parity grid, never a fixture: an implementation cannot supply its own expected values, and this one is an implementation."),
]

CASES = [
    ("nominal", "Nominal — case C1 at the reference altitude",
     "The design point every dossier in this suite is checked against. Case A of "
     "the two reference cases: 250 km, integrated ISR/EO and IoT comms "
     "multipayload, global service, constellation concept.",
     {}),
    ("low_altitude", "Case A — 200 km, drag dominated",
     "Drag-dominated and air-breathing propulsion in play. The propulsion layer "
     "carries thrust-to-drag as its binding target.",
     {"orbit_altitude": 200000.0}),
    ("high_altitude", "Case B — 350 km, downlink dominated",
     "Drag is reduced and both chemical and air-breathing propulsion close. The "
     "propulsion targets relax and downlink and revisit dominate.",
     {"orbit_altitude": 350000.0}),
    ("solar_max", "Solar maximum, storm conditions",
     "The activity level the design must survive rather than the one it is sized "
     "at. Density at 250 km is roughly five times the quiet value.",
     {"env_f107": 250.0, "env_f107a": 250.0, "env_kp": 7.0}),
    ("solar_min", "Solar minimum, quiet conditions",
     "The activity level at which the air-breathing concept is hardest, because "
     "there is least air to breathe.",
     {"env_f107": 70.0, "env_f107a": 70.0, "env_kp": 1.0}),
]

CYCLE = dict(
    nodes=["pwr_cell_derating", "pwr_array_power_eol", "pwr_available", "prop_throttle",
           "prop_delivered_bus_power", "pwr_demand", "thm_dissipation",
           "thm_equilibrium_temperature"],
    converge_on="pwr_demand", tolerance=1e-6, max_iter=80,
    seeds={"thm_equilibrium_temperature": 300.0},
)


def emit_supporting():
    # ---- sources -----------------------------------------------------------
    S = ["# Sources are objects, not strings.",
         "#",
         "# A fixture or a declared value references an identifier here, never a",
         "# free-text citation. Marking one superseded then lists every node that",
         "# depends on it — one query rather than a search.",
         ""]
    for sid, title, where, status, used in SOURCES:
        S += ["[[source]]", "id = %s" % toml_str(sid), "title = %s" % toml_str(title),
              "where = %s" % toml_str(where), "status = %s" % toml_str(status),
              "used_for = %s" % toml_str(used), ""]
    write(os.path.join(ROOT, "sources", "sources.toml"), "\n".join(S) + "\n")

    # ---- layers ------------------------------------------------------------
    tops = {}
    for gid, g in LAYERS.items():
        top = gid
        seen = set()
        while LAYERS.get(top, {}).get("parent") and top not in seen:
            seen.add(top)
            top = LAYERS[top]["parent"]
        tops.setdefault(top if top != "root" else gid, []).append(gid)
    for top, members in sorted(tops.items()):
        L = ["# A layer file: the rows in the tree that are not nodes.",
             "#",
             "# Headings, their parents, the relation edges between groups, and",
             "# subsystem ownership — which CODEOWNERS is generated from. Changed",
             "# rarely and reviewed by two people, because the tree is the",
             "# decomposition and moving a branch moves everyone's work.",
             ""]
        for gid in sorted(members):
            g = LAYERS[gid]
            L += ["[[group]]", "id = %s" % toml_str(g["id"]), "label = %s" % toml_str(g["label"]),
                  "parent = %s" % toml_str(g["parent"]), "owner = %s" % toml_str(g["owner"]), ""]
        for gid in sorted(members):
            for frm, to, why in LAYERS[gid]["relates"]:
                L += ["[[relates]]", "from = %s" % toml_str(frm), "to = %s" % toml_str(to),
                      "why = %s" % toml_str(why), ""]
        write(os.path.join(ROOT, "layers", "%s.toml" % top), "\n".join(L) + "\n")

    # ---- cases -------------------------------------------------------------
    for cid, label, note, supplied in CASES:
        L = ["# A case: per-customer values against one shared architecture.",
             "#",
             "# The architecture is never copied. What the customer did not buy is",
             "# simply not in play, and n copies would mean n fixes and silent",
             "# drift — the anti-clone-and-own rule, ISO/IEC 26580.",
             "",
             "id = %s" % toml_str(cid), "label = %s" % toml_str(label),
             "note = %s" % toml_str(note), ""]
        if supplied:
            L.append("[supply]")
            L.append("# Overrides on top of each declared node's own value.")
            for k, v in sorted(supplied.items()):
                L.append("%s = %r" % (k, float(v)))
            L.append("")
        L += ["[[iterate]]",
              "# A declared cycle. Power becomes heat, heat sets array temperature,",
              "# array temperature sets cell efficiency, cell efficiency sets available",
              "# power, and available power throttles the largest load on the bus.",
              "# A cycle is a property of the design, so it is declared where design",
              "# decisions live. An undeclared one is a named error, not a hung run.",
              "nodes = [%s]" % ", ".join(toml_str(x) for x in CYCLE["nodes"]),
              "converge_on = %s" % toml_str(CYCLE["converge_on"]),
              "tolerance = %r" % CYCLE["tolerance"],
              "max_iter = %d" % CYCLE["max_iter"], ""]
        for k, v in CYCLE["seeds"].items():
            L += ["[[iterate.seed]]", "var = %s" % toml_str(k), "value = %r" % float(v),
                  "source = \"orbitt_case_c1\"",
                  "note = \"A seed decides which fixed point the iteration reaches, so it carries a source like every other declared number.\"", ""]
        write(os.path.join(ROOT, "cases", "%s.toml" % cid), "\n".join(L) + "\n")


BY_ID = {}


def main():
    for n in NODES:
        BY_ID[n["id"]] = n
    seen = {}
    for n in NODES:
        key = (crate_for(n), folder_for(n))
        if key in seen:
            raise SystemExit("folder collision: %s and %s both resolve to %s/nodes/%s"
                             % (seen[key], n["id"], key[0], key[1]))
        seen[key] = n["id"]
    for n in NODES:
        emit_node(n)
    emit_supporting()
    crates = sorted({crate_for(n) for n in NODES})
    print("seeded %d node folders across %d subsystem crates" % (len(NODES), len(crates)))
    for c in crates:
        k = len([n for n in NODES if crate_for(n) == c])
        print("  %-22s %3d nodes" % (c, k))
    print("%d layer groups, %d sources, %d cases" % (len(LAYERS), len(SOURCES), len(CASES)))


if __name__ == "__main__":
    main()
