<!-- GENERATED from crates/vleo-sheet/src/method.rs by `cargo run -p xtask -- docs`. Do not edit. -->
# The method language, version 4

> **Answer first.** Every node's relation is written once more as a *method*: a few lines in a
> small fixed language that the tool can check and run. The checker refuses a
> sum of unlike units, a logarithm of a length, an answer of the wrong quantity, and a path
> that ends without an answer or a refusal — before the method runs.
>
> **Kind:** reference · **For:** node engineers and the developer

## Said simply

A method is the recipe for the node's answer, written so a machine can follow it exactly. It
reads the node's inputs by their names, may use the constants below, works out named values
with `let`, and ends every path with `return` (the answer) or `refuse` (the reason it will not
answer). Every number carries its unit: write `6371 [km]`, and the tool works in SI from there.

## Why a method, when my code already works

Your code produced your test cases. The method is a second, independent statement of the same
relation, and the Rust the tool ships is generated from the method. The three are compared on
your cases: if any one of them is wrong, a case disagrees, and it is caught before the change
reaches anyone.

## Statements

| Form | Meaning | Example |
|---|---|---|
| `let NAME = EXPR` | A new named value. Its dimension is whatever the expression's is. | `let r = R_EARTH + h` |
| `let NAME : Quantity = EXPR` | The same, and the checker confirms the expression has that quantity's dimension. | `let v : Velocity = sqrt(MU_EARTH / r)` |
| `const NAME = NUMBER [unit]` | A constant from your source, with its unit. Say where it comes from in a comment. | `const cd = 2.2 [1]   # drag coefficient, Sentman flat plate` |
| `const NAME = [A, B, …] [unit]` | A list from your source, written out, with one unit for every entry. It is written once, at the method's top level, and never changes. Read one entry as NAME[i], counted from 1; its length as len(NAME); every entry with for … in; or use it as a row of interp's table. | `const EDGES = [90, 130, 170] [1]   # the published band edges, sfu` |
| `set NAME = EXPR` | Change a value made with let. Same dimension; inputs and constants cannot be changed. | `set total = total + term` |
| `if COND then … else if COND then … else … end` | Choose. Conditions compare like with like: h < 0 [m], not h < 0 [s]. | `if h < 150 [km] then ⏎   refuse "below the lowest altitude the model covers" ⏎ end` |
| `for NAME = FIRST to LAST … end` | Repeat for whole numbers FIRST..LAST. The count is fixed when written; the loop variable is a pure number. | `for n = 1 to 10 ⏎   set total = total + x ^ n / n ⏎ end` |
| `for NAME in LIST … end` | Repeat once for each entry of a list, in order; NAME holds the entry, in the list's unit. | `for edge in EDGES ⏎   if f107 >= edge then ⏎     set band = band + 1 ⏎   end ⏎ end` |
| `while COND at most N times … end` | Repeat while COND holds — an iteration that settles, or a count the inputs decide. N is the most passes it may take, fixed when written; if COND still holds after N passes the node refuses, saying the loop did not settle, rather than answer with wherever it had got to. | `while abs(r * r - a) > 1e-12 * a at most 40 times ⏎   set r = (r + a / r) / 2 ⏎ end` |
| `refuse "reason"` | The node will not answer here, and says why. A refusal is never a substitute value. | `refuse "the orbit is inside the Earth"` |
| `return EXPR` | The node's answer, in its declared quantity. Every path ends in return or refuse. | `return v` |
| `publish SYMBOL = EXPR` | For a node that publishes several values: one of them, by the member's symbol on the sheet, in that member's quantity. Each member is published once, at the method's top level, before the method returns — so every answer carries every member. A refusal may still come anywhere. | `publish Kp_mean_nominal = kp_from_ap(ap_nominal) + kp_mean_slot_bias(ap_nominal)` |
| `# comment` | Anything after # on a line is for the reader. | `# Vallado (2013), eq. 1-18` |

Operators: `+ - * / ^`, comparisons `< <= > >= == !=`, and `and`, `or`, `not`. A power of a
dimensioned value is written as a number (`r ^ 3`, `a ^ 0.5` when every exponent stays whole).
A bare `0` is zero of any unit; any other number that is not a pure ratio needs its unit.

## Functions

| Function | Units | Meaning |
|---|---|---|
| `sqrt` | halves every exponent (m^2 → m) | square root; refused below zero |
| `cbrt` | thirds every exponent | cube root |
| `abs` | keeps the unit | size without sign |
| `min` | two like values, that unit out | the smaller of two like values |
| `max` | two like values, that unit out | the larger of two like values |
| `hypot` | two like values, that unit out | sqrt(a^2 + b^2), without overflow |
| `fmod` | two like values, that unit out | remainder of a / b, with a's sign |
| `pow` | as ^ | a to the power p — the same as a ^ p |
| `exp` | a pure number in, a pure number out | e to the power x |
| `ln` | a pure number in, a pure number out | natural logarithm; refused at or below zero |
| `log10` | a pure number in, a pure number out | base-10 logarithm |
| `log2` | a pure number in, a pure number out | base-2 logarithm |
| `sin` | a pure number in, a pure number out | sine of an angle (radians; write 30 [deg] for degrees) |
| `cos` | a pure number in, a pure number out | cosine |
| `tan` | a pure number in, a pure number out | tangent |
| `asin` | a pure number in, a pure number out | inverse sine, in radians; refused outside -1..1 |
| `acos` | a pure number in, a pure number out | inverse cosine, in radians; refused outside -1..1 |
| `atan` | a pure number in, a pure number out | inverse tangent, in radians |
| `atan2` | two like values, a pure number out | the angle of (x, y), from y and x of one dimension |
| `sinh` | a pure number in, a pure number out | hyperbolic sine |
| `cosh` | a pure number in, a pure number out | hyperbolic cosine |
| `tanh` | a pure number in, a pure number out | hyperbolic tangent |
| `erf` | a pure number in, a pure number out | the error function |
| `erfc` | a pure number in, a pure number out | 1 - erf(x) |
| `floor` | a pure number in, a pure number out | round down — pure numbers only, since the answer would depend on the unit |
| `ceil` | a pure number in, a pure number out | round up — pure numbers only |
| `round` | a pure number in, a pure number out | round to nearest, halves away from zero — pure numbers only |
| `wrap_2pi` | a pure number in, a pure number out | an angle brought into 0..2π |
| `wrap_pi` | a pure number in, a pure number out | an angle brought into -π..π |
| `interp` | x like the table's x row; the y row's unit out | straight-line lookup in a table: interp(x, XS, YS), each row a list by name or written out as [x1, x2, …] [unit]; held at the ends |
| `len` | a list in, a pure number out | how many entries a list has: len(EDGES) |

## Kernel functions

Relations too long to write as a formula — an integral up the atmosphere, a decay over many orbits — already live in the kernel, reviewed once. A method calls them by name, each argument in the unit shown; the checker holds the units, and the engine runs the kernel itself, so the method and the built node cannot differ. An argument in `[~]` takes any unit, so long as every `[~]` argument of the call is in the same one; a function that can find no answer refuses, in the words shown.

| Call | Answer | Meaning | In the kernel |
|---|---|---|---|
| `thermosphere_o(h [m], T_inf [K])` | `[1/m^3]` | atomic oxygen number density at altitude h, for exospheric temperature T_inf | `env::composition(h, T_inf).o` |
| `thermosphere_number_density(h [m], T_inf [K])` | `[1/m^3]` | every species' number density at altitude h, summed | `env::composition(h, T_inf).total()` |
| `thermosphere_molar_mass(h [m], T_inf [K])` | `[kg/mol]` | the mean molar mass of the gas at altitude h | `env::composition(h, T_inf).mean_molar_mass()` |
| `thermosphere_density(h [m], T_inf [K])` | `[kg/m^3]` | the mass density of the gas at altitude h | `env::mass_density(h, T_inf)` |
| `thermosphere_scale_height(h [m], T_inf [K])` | `[m]` | the density scale height at altitude h | `env::scale_height(h, T_inf)` |
| `orbit_decay_time(h [m], h_end [m], bc [kg/m^2], T_inf [K])` | `[s]` | the time a circular orbit takes to decay from h to h_end, for ballistic coefficient bc, in the thermosphere of T_inf (64 steps) | `orbit::lifetime_estimate(h, h_end, bc, 64, |z| env::mass_density(z, T_inf))` |
| `solar_cycle_mean(t0 [s], t1 [s])` | `[1]` | the solar-cycle analogue's mean F10.7 from mission time t0 to t1 | `env::solar_cycle_analogue_mean(t0, t1), in days` |
| `solar_cycle_max(t0 [s], t1 [s])` | `[1]` | the solar-cycle analogue's highest F10.7 from mission time t0 to t1 | `env::solar_cycle_analogue_max(t0, t1), in days` |
| `kp_from_ap(ap [1])` | `[1]` | Kp on the published three-hour scale for the planetary index ap, between its tabulated thirds | `env::kp_from_ap(ap)` |
| `kp_mean_slot_bias(ap [1])` | `[1]` | the measured offset of a day's mean three-hour Kp from the Kp of its daily Ap | `env::kp_mean_slot_bias(ap)` |
| `kp_peak_slot_bias(ap [1])` | `[1]` | the measured offset of a day's highest three-hour Kp from the Kp of its daily Ap | `env::kp_peak_slot_bias(ap)` |
| `aerodynamic_torque(drag [N], cp_cm_offset [m])` | `[N.m]` | the torque drag makes about the centre of mass, from the centre-of-pressure offset | `aero::aerodynamic_torque(drag, cp_cm_offset)` |
| `drag_acceleration(drag [N], mass [kg])` | `[m/s^2]` | drag deceleration, D/m | `aero::drag_acceleration(drag, mass)` |
| `dynamic_pressure(density [kg/m^3], velocity [m/s])` | `[Pa]` | dynamic pressure, 0.5·rho·V^2 | `aero::dynamic_pressure(density, velocity)` |
| `speed_ratio(velocity [m/s], t [K], molar_mass [kg/mol])` | `[1]` | molecular speed ratio, V over the most probable thermal speed | `aero::speed_ratio(velocity, t, molar_mass)` |
| `accommodation_coefficient(n_atomic_oxygen [1/m^3], t_inf [K])` | `[1]` | energy accommodation coefficient from Langmuir adsorption of atomic oxygen (Moe & Moe 2005) | `aero::accommodation_coefficient(n_atomic_oxygen, t_inf)` |
| `atomic_oxygen_fluence(n_atomic_oxygen [1/m^3], velocity [m/s], duration [s])` | `[1/m^2]` | atomic oxygen fluence over a duration, atoms per square metre | `aero::atomic_oxygen_fluence(n_atomic_oxygen, velocity, duration)` |
| `ballistic_coefficient(mass [kg], drag_coefficient [1], reference_area [m^2])` | `[kg/m^2]` | ballistic coefficient, m/(Cd·A) | `aero::ballistic_coefficient(mass, drag_coefficient, reference_area)` |
| `cylinder_drag_coefficient(s [1], length [m], diameter [m], alpha [1], t_wall [K], velocity [m/s], molar_mass [kg/mol])` | `[1]` | drag coefficient of a right circular cylinder flying axially, Sentman free-molecular, for speed ratio s and accommodation alpha | `aero::cylinder_drag_coefficient(s, length, diameter, alpha, t_wall, velocity, molar_mass)` |
| `aperture_gain_db(diameter [m], frequency [Hz], efficiency [1])` | `[1]` | gain of a circular aperture antenna, in dBi as a pure number | `comms::aperture_gain_db(diameter, frequency, efficiency)` |
| `carrier_to_noise_density_db(eirp_dbw [1], path_loss_db [1], atmospheric_loss_db [1], g_over_t_db [1])` | `[1]` | carrier to noise density ratio, in dB-Hz as a pure number; every argument in dB as a pure number | `comms::carrier_to_noise_density_db(eirp_dbw, path_loss_db, atmospheric_loss_db, g_over_t_db)` |
| `max_contact_time(earth_central_angle [rad], orbital_period [s])` | `[s]` | contact time for one overhead pass | `comms::max_contact_time(earth_central_angle, orbital_period)` |
| `daily_downlink_volume(per_pass [bit], passes_per_day [1], availability [1])` | `[bit]` | data downlinked in a day, from the volume per pass and the passes per day | `comms::daily_downlink_volume(per_pass, passes_per_day, availability)` |
| `achievable_data_rate(cn0_db [1], required_eb_n0_db [1], implementation_loss_db [1], margin_db [1])` | `[bit/s]` | the highest data rate that closes the link at the stated margin; every argument in dB as a pure number | `comms::achievable_data_rate(cn0_db, required_eb_n0_db, implementation_loss_db, margin_db)` |
| `doppler_shift(frequency [Hz], relative_velocity [m/s])` | `[Hz]` | Doppler shift at closest approach, f·v/c | `comms::doppler_shift(frequency, relative_velocity)` |
| `eb_over_n0_db(cn0_db [1], data_rate [bit/s])` | `[1]` | energy per bit over noise density, in dB as a pure number | `comms::eb_over_n0_db(cn0_db, data_rate)` |
| `eirp_dbw(transmit_power [W], antenna_gain_db [1], line_loss_db [1])` | `[1]` | effective isotropic radiated power, in dBW as a pure number | `comms::eirp_dbw(transmit_power, antenna_gain_db, line_loss_db)` |
| `g_over_t_db(receive_gain_db [1], system_noise_temperature [K])` | `[1]` | receiver figure of merit G/T, in dB/K as a pure number | `comms::g_over_t_db(receive_gain_db, system_noise_temperature)` |
| `pass_data_volume(rate [bit/s], contact [s], link_efficiency [1])` | `[bit]` | data volume downlinked in one pass | `comms::pass_data_volume(rate, contact, link_efficiency)` |
| `free_space_path_loss_db(range [m], frequency [Hz])` | `[1]` | free-space path loss, in dB as a pure number | `comms::free_space_path_loss_db(range, frequency)` |
| `system_noise_temperature(antenna_temperature [K], line_loss_db [1], receiver_noise_figure_db [1], physical_temperature [K])` | `[K]` | system noise temperature from the antenna, line and receiver; losses in dB as pure numbers | `comms::system_noise_temperature(antenna_temperature, line_loss_db, receiver_noise_figure_db, physical_temperature)` |
| `link_margin_db(eb_n0_db [1], required_eb_n0_db [1], implementation_loss_db [1])` | `[1]` | link margin against a required Eb/N0, with implementation loss; every value in dB as a pure number | `comms::link_margin_db(eb_n0_db, required_eb_n0_db, implementation_loss_db)` |
| `cost_per_service_unit(programme_cost [USD], service_units [1])` | `[USD]` | cost per unit of delivered service | `cost::cost_per_service_unit(programme_cost, service_units)` |
| `production_run_cost(first_unit [USD], units [1], slope [1])` | `[USD]` | a production run's cost under Wright's learning curve, summed unit by unit; units a whole number, cut toward zero | `cost::production_run_cost(first_unit, units, slope)` |
| `programme_cost(non_recurring [USD], production [USD], annual_operations [USD], years [1])` | `[USD]` | non-recurring plus production plus operations for the years given | `cost::programme_cost(non_recurring, production, annual_operations, years)` |
| `replacement_avoided(satellite_unit_cost [USD], launch_cost [USD], replacements_avoided [1])` | `[USD]` | the saving of satellites that need not be replaced and relaunched | `cost::replacement_avoided(satellite_unit_cost, launch_cost, replacements_avoided)` |
| `cer_inflated(driver [~], a [1], b [1], base_year [1], target_year [1], annual_inflation [1])` | `[USD]` | a cost estimating relationship a·driver^b in millions, in its base year's money, inflated to the target year at the annual rate; years whole numbers | `cost::Cer { a, b, base_year, .. }.evaluate_inflated(driver, target_year, annual_inflation)` |
| `density_uncertainty(kp [1])` | `[1]` | the one-sigma uncertainty the density model claims, as a fraction | `env::density_uncertainty(kp)` |
| `exospheric_temperature(f107 [1], f107a [1], kp [1])` | `[K]` | exospheric temperature from F10.7, its 81-day mean and Kp (Jacchia 1971) | `env::exospheric_temperature(f107, f107a, kp)` |
| `knudsen(mean_free_path [m], characteristic_length [m])` | `[1]` | Knudsen number, lambda/L | `env::knudsen(mean_free_path, characteristic_length)` |
| `thermosphere_temperature(altitude [m], t_inf [K])` | `[K]` | the gas's kinetic temperature at altitude, Bates profile | `env::temperature(altitude, t_inf)` |
| `magnetic_field(r [m], magnetic_latitude [rad])` | `[T]` | the geomagnetic field's magnitude, dipole approximation | `env::magnetic_field(r, magnetic_latitude)` |
| `mean_free_path(n [1/m^3])` | `[m]` | mean free path for a number density | `env::mean_free_path(n)` |
| `along_track_error_from_drag(drag_acceleration_error [m/s^2], duration [s])` | `[m]` | along-track error grown from an unmodelled drag acceleration over a duration | `gnc::along_track_error_from_drag(drag_acceleration_error, duration)` |
| `gravity_gradient_torque(radius [m], inertia_max [kg.m^2], inertia_min [kg.m^2], theta [rad])` | `[N.m]` | gravity-gradient torque at pitch angle theta from local vertical | `gnc::gravity_gradient_torque(radius, inertia_max, inertia_min, theta)` |
| `pointing_to_ground_error(pointing_error [rad], slant_range [m])` | `[m]` | ground position error from a pointing error at a slant range | `gnc::pointing_to_ground_error(pointing_error, slant_range)` |
| `magnetic_torque(residual_dipole [A.m^2], field [T])` | `[N.m]` | torque from a residual dipole in the geomagnetic field | `gnc::magnetic_torque(residual_dipole, field)` |
| `magnetorquer_dipole_required(momentum [N.m.s], field [T], dump_time [s])` | `[A.m^2]` | the dipole a torque rod needs to dump a momentum in a time | `gnc::magnetorquer_dipole_required(momentum, field, dump_time)` |
| `momentum_storage_required(secular_torque [N.m], orbital_period [s], margin [1])` | `[N.m.s]` | momentum a wheel stores against a secular torque for half an orbit, with a margin | `gnc::momentum_storage_required(secular_torque, orbital_period, margin)` |
| `navigation_position_error(user_range_error [m], gdop [1])` | `[m]` | GNSS position error from the receiver's range error (URE) and the geometry | `gnc::navigation_position_error(user_range_error, gdop)` |
| `pointing_error_rss(a [rad], b [rad], c [rad], d [rad])` | `[rad]` | root-sum-square of four pointing error terms | `gnc::pointing_error_rss(&[a, b, c, d])` |
| `solar_pressure_torque(area [m^2], reflectivity [1], cp_offset [m], incidence [rad])` | `[N.m]` | solar radiation pressure torque | `gnc::solar_pressure_torque(area, reflectivity, cp_offset, incidence)` |
| `total_disturbance_torque(aerodynamic [N.m], gravity_gradient [N.m], solar [N.m], magnetic [N.m])` | `[N.m]` | the disturbance torques summed, worst case | `gnc::total_disturbance_torque(aerodynamic, gravity_gradient, solar, magnetic)` |
| `dry_mass(structure [kg], propulsion [kg], power [kg], thermal [kg], avionics [kg], comms [kg], gnc [kg], harness [kg], payloads [kg], system_margin [1])` | `[kg]` | dry mass: the subsystems summed, with the system margin | `mass::dry_mass(structure, propulsion, power, thermal, avionics, comms, gnc, harness, payloads, system_margin)` |
| `mass_margin(limit [kg], actual [kg])` | `[1]` | mass margin against a limit, as a fraction of the limit | `mass::mass_margin(limit, actual)` |
| `wet_mass(dry [kg], propellant [kg])` | `[kg]` | dry plus propellant | `mass::wet_mass(dry, propellant)` |
| `margin_at_least(required [~], achieved [~])` | `[1]` | signed margin of an achieved value that must be AT LEAST the required one, as a fraction of it: (achieved - required) / required, and 0 when nothing is required | `mission::closure(required, achieved, AtLeast).margin` |
| `margin_at_most(required [~], achieved [~])` | `[1]` | signed margin of an achieved value that must be AT MOST the required one, as a fraction of it: (required - achieved) / required, and 0 when nothing is required | `mission::closure(required, achieved, AtMost).margin` |
| `access_area(earth_central_angle [rad])` | `[m^2]` | instantaneous access area on the ground | `mission::access_area(earth_central_angle)` |
| `k_of_n_availability(unit_availability [1], n [1], k [1])` | `[1]` | availability of k of n, binomial; n and k whole numbers, cut toward zero | `mission::k_of_n_availability(unit_availability, n, k)` |
| `instantaneous_coverage_fraction(earth_central_angle [rad])` | `[1]` | fraction of the Earth one satellite sees at an instant | `mission::instantaneous_coverage_fraction(earth_central_angle)` |
| `end_to_end_latency(time_to_downlink [s], downlink_duration [s], ground_processing [s], delivery [s])` | `[s]` | collect, hold, downlink, process and deliver | `mission::end_to_end_latency(time_to_downlink, downlink_duration, ground_processing, delivery)` |
| `mean_revisit_time(swath_width [m], ground_track_speed [m/s], satellites [1])` | `[s]` | mean revisit time by an area-rate argument | `mission::mean_revisit_time(swath_width, ground_track_speed, satellites)` |
| `satellites_for_revisit(target_revisit [s], swath_width [m], ground_track_speed [m/s])` | `[1]` | satellites needed for a revisit target | `mission::satellites_for_revisit(target_revisit, swath_width, ground_track_speed)` |
| `mean_time_to_downlink(passes_per_day [1])` | `[s]` | mean time to the next ground contact | `mission::mean_time_to_downlink(passes_per_day)` |
| `circular_velocity(radius [m])` | `[m/s]` | circular orbital speed, sqrt(mu/r) | `orbit::circular_velocity(radius)` |
| `decay_rate(density [kg/m^3], ballistic_coefficient [kg/m^2], semi_major_axis [m])` | `[m/s]` | rate of change of semi-major axis under drag | `orbit::decay_rate(density, ballistic_coefficient, semi_major_axis)` |
| `deorbit_delta_v(from_altitude [m], target_perigee_altitude [m])` | `[m/s]` | delta-v to lower perigee to a re-entry altitude | `orbit::deorbit_delta_v(from_altitude, target_perigee_altitude)` |
| `drag_makeup_delta_v(drag_acceleration [m/s^2], duration [s])` | `[m/s]` | delta-v to hold altitude against drag for a duration | `orbit::drag_makeup_delta_v(drag_acceleration, duration)` |
| `earth_central_angle(radius [m], min_elevation [rad])` | `[rad]` | Earth-central angle to the horizon at a minimum elevation | `orbit::earth_central_angle(radius, min_elevation)` |
| `eclipse_fraction(radius [m], beta [rad])` | `[1]` | fraction of an orbit in the Earth's shadow, cylindrical model | `orbit::eclipse_fraction(radius, beta)` |
| `ground_track_speed(orbital_speed [m/s], radius [m])` | `[m/s]` | speed of the sub-satellite point | `orbit::ground_track_speed(orbital_speed, radius)` |
| `nodal_regression(semi_major_axis [m], eccentricity [1], inclination [rad])` | `[rad/s]` | secular regression of the node from J2 | `orbit::nodal_regression(semi_major_axis, eccentricity, inclination)` |
| `orbital_period(semi_major_axis [m])` | `[s]` | orbital period, 2·pi·sqrt(a^3/mu) | `orbit::period(semi_major_axis)` |
| `orbit_radius(altitude [m])` | `[m]` | geocentric radius from altitude above the WGS-84 equatorial radius | `orbit::radius(altitude)` |
| `slant_range(radius [m], min_elevation [rad])` | `[m]` | slant range to the edge of the access circle | `orbit::slant_range(radius, min_elevation)` |
| `sun_synchronous_inclination(semi_major_axis [m], eccentricity [1])` | `[rad]` | the inclination at which the node regresses with the Sun; refused where none does; refuses: «no inclination gives Sun-synchronous regression at this radius» | `orbit::sun_synchronous_inclination(semi_major_axis, eccentricity)` |
| `swath_width(earth_central_angle [rad])` | `[m]` | ground swath width for an Earth-central half-angle | `orbit::swath_width(earth_central_angle)` |
| `propellant_mass(dry_mass [kg], delta_v [m/s], exhaust_velocity [m/s])` | `[kg]` | the rocket equation, solved for propellant mass | `orbit::propellant_mass(dry_mass, delta_v, exhaust_velocity)` |
| `exhaust_velocity(specific_impulse [s])` | `[m/s]` | exhaust velocity from specific impulse, Isp·g0 | `orbit::exhaust_velocity(specific_impulse)` |
| `dwell_time(gsd [m], ground_track_speed [m/s])` | `[s]` | integration time per ground sample | `payload::dwell_time(gsd, ground_track_speed)` |
| `tdoa_geolocation_error(timing_uncertainty [s], gdop [1])` | `[m]` | geolocation error from time difference of arrival | `payload::tdoa_geolocation_error(timing_uncertainty, gdop)` |
| `achieved_gsd(diffraction [m], detector [m])` | `[m]` | the ground sample distance achieved: the worse of the two limits | `payload::achieved_gsd(diffraction, detector)` |
| `detector_limited_gsd(altitude [m], pixel_pitch [m], focal_length [m])` | `[m]` | detector-limited ground sample distance, h·p/f | `payload::detector_limited_gsd(altitude, pixel_pitch, focal_length)` |
| `diffraction_limited_gsd(altitude [m], wavelength [m], aperture [m])` | `[m]` | diffraction-limited ground resolution, Rayleigh | `payload::diffraction_limited_gsd(altitude, wavelength, aperture)` |
| `scene_data_volume(pixels_across [1], pixels_along [1], bits_per_pixel [1], bands [1], compression_ratio [1])` | `[bit]` | raw data volume of one optical scene | `payload::scene_data_volume(pixels_across, pixels_along, bits_per_pixel, bands, compression_ratio)` |
| `signal_electrons(radiance [1], aperture [m], focal_length [m], pixel_pitch [m], transmission [1], quantum_efficiency [1], integration_time [s], wavelength [m], bandwidth [m])` | `[1]` | signal electrons per sample; radiance in W/m^2/sr/m as a pure number | `payload::signal_electrons(radiance, aperture, focal_length, pixel_pitch, transmission, quantum_efficiency, integration_time, wavelength, bandwidth)` |
| `optical_snr(signal_electrons [1], dark_electrons [1], read_noise_electrons [1])` | `[1]` | signal to noise ratio: shot, dark and read noise | `payload::optical_snr(signal_electrons, dark_electrons, read_noise_electrons)` |
| `optical_swath(gsd [m], pixels_across_track [1])` | `[m]` | swath from the pixels across track and the ground sample distance | `payload::optical_swath(gsd, pixels_across_track)` |
| `toa_timing_uncertainty(bandwidth [Hz], snr_linear [1], integration [s])` | `[s]` | Cramér-Rao bound on time-of-arrival | `payload::toa_timing_uncertainty(bandwidth, snr_linear, integration)` |
| `array_mass(area [m^2], areal_density [kg/m^2])` | `[kg]` | solar array mass from area and areal density | `power::array_mass(area, areal_density)` |
| `array_power_bol(area [m^2], cell_efficiency [1], packing_factor [1], incidence [rad])` | `[W]` | solar array output at beginning of life | `power::array_power_bol(area, cell_efficiency, packing_factor, incidence)` |
| `array_power_eol(bol [W], degradation [1])` | `[W]` | array output at end of life | `power::array_power_eol(bol, degradation)` |
| `battery_cycles(mission_duration [s], orbital_period [s])` | `[1]` | charge-discharge cycles over a mission, one per orbit | `power::battery_cycles(mission_duration, orbital_period)` |
| `battery_energy_required(eclipse_load [W], eclipse_duration [s], depth_of_discharge [1], discharge_efficiency [1])` | `[J]` | battery energy to carry the eclipse load | `power::battery_energy_required(eclipse_load, eclipse_duration, depth_of_discharge, discharge_efficiency)` |
| `battery_mass(energy [J], specific_energy_wh_per_kg [1])` | `[kg]` | battery mass from energy and specific energy; specific energy in Wh/kg as a pure number | `power::battery_mass(energy, specific_energy_wh_per_kg)` |
| `array_degradation(annual_rate [1], years [1])` | `[1]` | degradation factor after a number of years at an annual rate | `power::degradation(annual_rate, years)` |
| `power_demand(propulsion [W], payload [W], avionics [W], comms [W], thermal [W], harness_loss [1])` | `[W]` | the named loads summed, with the harness loss | `power::power_demand(propulsion, payload, avionics, comms, thermal, harness_loss)` |
| `power_margin(available [W], demand [W])` | `[1]` | power margin as a fraction of the demand | `power::power_margin(available, demand)` |
| `bus_power_demand(thruster_input [W], ppu_efficiency [1])` | `[W]` | power drawn from the bus, after the power processing unit | `prop::bus_power_demand(thruster_input, ppu_efficiency)` |
| `intake_collection_efficiency(free_stream_density [1/m^3], velocity [m/s], intake_area [m^2], throat_area [m^2], eta_geo [1], beta_backflow [1], chamber_temperature [K], molar_mass [kg/mol])` | `[1]` | the intake's collection efficiency, from its particle balance | `prop::intake_balance(free_stream_density, velocity, intake_area, throat_area, eta_geo, beta_backflow, chamber_temperature, molar_mass).collection_efficiency` |
| `intake_compression_ratio(free_stream_density [1/m^3], velocity [m/s], intake_area [m^2], throat_area [m^2], eta_geo [1], beta_backflow [1], chamber_temperature [K], molar_mass [kg/mol])` | `[1]` | the intake's compression ratio, from its particle balance | `prop::intake_balance(free_stream_density, velocity, intake_area, throat_area, eta_geo, beta_backflow, chamber_temperature, molar_mass).compression_ratio` |
| `collected_mass_flow(density [kg/m^3], velocity [m/s], intake_area [m^2], collection_efficiency [1])` | `[kg/s]` | mass flow reaching the thruster | `prop::collected_mass_flow(density, velocity, intake_area, collection_efficiency)` |
| `beam_exhaust_velocity(beam_voltage [V], molar_mass [kg/mol])` | `[m/s]` | exhaust velocity of a beam accelerated through a potential, sqrt(2qV/m) | `prop::beam_exhaust_velocity(beam_voltage, molar_mass)` |
| `incident_mass_flux(density [kg/m^3], velocity [m/s])` | `[kg/s]` | incident mass flux, rho·V | `prop::incident_mass_flux(density, velocity)` |
| `thruster_input_power(jet [W], ionisation [W], other_losses [1])` | `[W]` | electrical power the thruster draws | `prop::thruster_input_power(jet, ionisation, other_losses)` |
| `ion_mass_flow(collected [kg/s], propellant_utilisation [1])` | `[kg/s]` | ion mass flow, from propellant utilisation | `prop::ion_mass_flow(collected, propellant_utilisation)` |
| `ionisation_power(ion_flow [kg/s], molar_mass [kg/mol], epsilon_ev [1])` | `[W]` | power spent making the ions; energy per ion in eV as a pure number | `prop::ionisation_power(ion_flow, molar_mass, epsilon_ev)` |
| `jet_power(thrust [N], ion_flow [kg/s])` | `[W]` | jet power, F^2/(2·m_dot) | `prop::jet_power(thrust, ion_flow)` |
| `specific_impulse(thrust [N], collected_flow [kg/s])` | `[s]` | specific impulse referred to the collected flow | `prop::specific_impulse(thrust, collected_flow)` |
| `beam_thrust(ion_flow [kg/s], exhaust_velocity [m/s], alpha_div [1], alpha_double [1])` | `[N]` | thrust of an ion beam, with divergence and doubly-charged corrections | `prop::beam_thrust(ion_flow, exhaust_velocity, alpha_div, alpha_double)` |
| `total_efficiency(jet [W], bus_power [W])` | `[1]` | jet power over bus power | `prop::total_efficiency(jet, bus_power)` |
| `thrust_to_drag(thrust [N], drag [N])` | `[1]` | thrust over drag | `prop::thrust_to_drag(thrust, drag)` |
| `absorbed_albedo(area [m^2], absorptivity [1], view_factor [1])` | `[W]` | albedo absorbed | `thermal::absorbed_albedo(area, absorptivity, view_factor)` |
| `absorbed_earth_ir(area [m^2], emissivity [1], view_factor [1])` | `[W]` | Earth infrared absorbed | `thermal::absorbed_earth_ir(area, emissivity, view_factor)` |
| `absorbed_solar(area [m^2], absorptivity [1], incidence [rad])` | `[W]` | direct solar absorbed | `thermal::absorbed_solar(area, absorptivity, incidence)` |
| `free_molecular_heating(density [kg/m^3], velocity [m/s], area [m^2], heat_transfer_coefficient [1])` | `[W]` | aerodynamic heating in free-molecular flow | `thermal::free_molecular_heating(density, velocity, area, heat_transfer_coefficient)` |
| `equilibrium_temperature(total_absorbed [W], internal_dissipation [W], radiating_area [m^2], emissivity [1])` | `[K]` | the temperature that balances what is absorbed and dissipated | `thermal::equilibrium_temperature(total_absorbed, internal_dissipation, radiating_area, emissivity)` |
| `required_radiator_area(heat_to_reject [W], emissivity [1], radiator_temperature [K], sink_temperature [K])` | `[m^2]` | radiator area to hold a temperature against a load | `thermal::required_radiator_area(heat_to_reject, emissivity, radiator_temperature, sink_temperature)` |
| `earth_view_factor(radius [m])` | `[1]` | view factor to the Earth from a nadir-facing plate | `thermal::earth_view_factor(radius)` |
| `thermal_margin(predicted [K], limit [K])` | `[K]` | thermal margin, in kelvin, against a limit | `thermal::thermal_margin(predicted, limit)` |

## Constants every method may use

| Name | Value | Unit | Meaning |
|---|---|---|---|
| `PI` | 3.141592653589793e0 | `1` | π |
| `MU_EARTH` | 3.986004418e14 | `m^3/s^2` | Earth's gravitational parameter |
| `R_EARTH` | 6.378137e6 | `m` | Earth's equatorial radius (WGS-84) |
| `R_EARTH_MEAN` | 6.3710088e6 | `m` | Earth's mean radius |
| `F_EARTH` | 3.3528106647474805e-3 | `1` | Earth's flattening (WGS-84) |
| `J2_EARTH` | 1.08262668e-3 | `1` | Earth's J2 zonal harmonic |
| `OMEGA_EARTH` | 7.292115e-5 | `rad/s` | Earth's rotation rate |
| `SIDEREAL_DAY` | 8.616409053e4 | `s` | one sidereal day |
| `G0` | 9.80665e0 | `m/s^2` | standard gravity |
| `SOLAR_CONSTANT` | 1.361e3 | `W/m^2` | total solar irradiance at 1 AU |
| `EARTH_ALBEDO` | 3.06e-1 | `1` | Earth's mean Bond albedo |
| `EARTH_IR` | 2.37e2 | `W/m^2` | Earth's mean outgoing infrared |
| `AU` | 1.495978707e11 | `m` | the astronomical unit |
| `SPEED_OF_LIGHT` | 2.99792458e8 | `m/s` | the speed of light |
| `K_BOLTZMANN` | 1.380649e-23 | `J/K` | the Boltzmann constant |
| `R_UNIVERSAL` | 8.31446261815324e0 | `J/K/mol` | the molar gas constant |
| `N_AVOGADRO` | 6.02214076e23 | `1/mol` | the Avogadro constant |
| `SIGMA_SB` | 5.670374419e-8 | `W/m^2/K^4` | the Stefan–Boltzmann constant |
| `ELEMENTARY_CHARGE` | 1.602176634e-19 | `C` | the elementary charge |
| `ATOMIC_MASS_UNIT` | 1.6605390666e-27 | `kg` | the atomic mass unit |
| `PLANCK` | 6.62607015e-34 | `J.s` | the Planck constant |

## Units

Write a unit in brackets straight after a number: `7.8 [km/s]`, `3.986e14 [m^3/s^2]`,
`30 [deg]`, `1 [1]` for a pure number. Every symbol a sheet may declare is accepted:

`-` · `%` · `dB` · `#` · `bit` · `m` · `km` · `um` · `nm` · `m^2` · `m^3` · `kg` · `g` · `kg/mol` · `s` · `min` · `h` · `d` · `yr` · `m/s` · `km/s` · `m/s^2` · `kg/s` · `mg/s` · `kg/m^3` · `1/m^3` · `N` · `mN` · `uN` · `N.m` · `N.s` · `N.m.s` · `Pa` · `J` · `Wh` · `W` · `kW` · `W/m^2` · `W/m^2/K^4` · `K` · `rad` · `deg` · `rad/s` · `deg/s` · `deg/d` · `arcsec` · `V` · `A` · `Ah` · `T` · `A.m^2` · `C` · `eV` · `Hz` · `MHz` · `GHz` · `bit/s` · `Mbit/s` · `Gbit` · `GB` · `USD` · `MUSD`

and the simple ones combine with `*` or `.`, `/` and whole-number powers `^`.

## The worked example: Circular orbital speed (illustrative)

*illustrative — a worked example, not this node.*

**The method.**

```text
# Vallado (2013), eq. 1-18: the speed of a circular two-body orbit.
if r <= R_EARTH then
  refuse "the orbit is inside the Earth"
end
let v : Velocity = sqrt(MU_EARTH / r)
return v
```

**The node engineer's own code** (MATLAB, `orbit_speed`), which produced the cases below.

```text
function v = orbit_speed(r)
% Circular two-body orbital speed: r in metres, v in metres per second.
mu = 3.986004418e14;          % Earth, m^3/s^2
R  = 6378137.0;               % Earth's equatorial radius, m
if r <= R
    error('orbit_speed:inside', 'the orbit is inside the Earth');
end
v = sqrt(mu / r);
end
```

**The test code** that ran it on each case.

```text
% Runs orbit_speed on each case and prints what goes in the form.
radii = [6628137.0, 6778137.0, 7378137.0];
for k = 1:numel(radii)
    fprintf('%.1f  %.8f\n', radii(k), orbit_speed(radii(k)));
end
try
    orbit_speed(6000000.0);
catch err
    fprintf('6000000.0  refused: %s\n', err.message);
end
```

**The cases**, in SI, and what the method gives for each.

| Case | Inputs | The node engineer's code | The method |
|---|---|---|---|
| 250 km | `r = 6628137` | 7754.84549737 | 7754.845497372695 — agrees with your code |
| 400 km | `r = 6778137` | 7668.55817541 | 7668.558175407055 — agrees with your code |
| 1000 km — the top of the band | `r = 7378137` | 7350.13862961 | 7350.138629613315 — agrees with your code |
| inside the Earth | `r = 6000000` | refuses | refused, as your code does |

## Where the simple version breaks

- **A set is published at the top level.** A node that publishes several values gives each
  with `publish`, once, before it returns — never inside an if or a loop. Work a member
  out with `let` and `if` first, then publish the name.
- **A loop says how often it may run.** `while … at most N times` stops when its condition
  fails, and refuses — by name — if N passes were not enough. It never answers with
  wherever it had got to.
- **A list is read inside its length.** `EDGES[i]` counts from 1; an index that is not a
  whole number, or falls outside the list, stops the method with a fault — it never
  reads the nearest entry instead. Guard it with `if`, or go through the list with
  `for … in`.
- **Tables are written out.** A list holds what your source tabulates, a few dozen entries;
  a lookup into a large data file is a reference-data bundle, not a method.
