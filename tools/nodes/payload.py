#!/usr/bin/env python3
"""Service payloads.

One subsystem's rows, and nothing else. Split out of the seeder so that
changing what this subsystem declares does not mean opening the file that
declares every other one — the same reason each subsystem has its own crate.

The helpers come from `seed_helpers`, which owns the shared lists these
append to. Import order is authoring order, so `seed_tree` decides it and this
file only says what the rows are.
"""

from seed_helpers import *          # noqa: F401,F403  the row helpers: D, C, layer, S, unnamed

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
