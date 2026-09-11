#!/usr/bin/env python3
"""Orbit and geometry.

One subsystem's rows, and nothing else. Split out of the seeder so that
changing what this subsystem declares does not mean opening the file that
declares every other one — the same reason each subsystem has its own crate.

The helpers come from `seed_helpers`, which owns the shared lists these
append to. Import order is authoring order, so `seed_tree` decides it and this
file only says what the rows are.
"""

from seed_helpers import *          # noqa: F401,F403  the row helpers: D, C, layer, S, unnamed

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
