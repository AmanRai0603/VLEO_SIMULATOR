#!/usr/bin/env python3
"""TT&C and downlink.

One subsystem's rows, and nothing else. Split out of the seeder so that
changing what this subsystem declares does not mean opening the file that
declares every other one — the same reason each subsystem has its own crate.

The helpers come from `seed_helpers`, which owns the shared lists these
append to. Import order is authoring order, so `seed_tree` decides it and this
file only says what the rows are.
"""

from seed_helpers import *          # noqa: F401,F403  the row helpers: D, C, layer, S, unnamed

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
