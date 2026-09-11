#!/usr/bin/env python3
"""Aerodynamics, drag and the air-breathing chain.

One subsystem's rows, and nothing else. Split out of the seeder so that
changing what this subsystem declares does not mean opening the file that
declares every other one — the same reason each subsystem has its own crate.

The helpers come from `seed_helpers`, which owns the shared lists these
append to. Import order is authoring order, so `seed_tree` decides it and this
file only says what the rows are.
"""

from seed_helpers import *          # noqa: F401,F403  the row helpers: D, C, layer, S, unnamed

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
