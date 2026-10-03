<!-- GENERATED from crates/vleo-sheet/src/method.rs by `cargo run -p xtask -- docs`. Do not edit. -->
# The method language, version 2

> **Answer first.** Every node's relation is written once more as a *method*: a few lines in a
> small fixed language that the tool can check, run and translate. The checker refuses a
> sum of unlike units, a logarithm of a length, an answer of the wrong quantity, and a path
> that ends without an answer or a refusal — before any code exists.
>
> **Kind:** reference · **For:** node authors and developers

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
| `set NAME = EXPR` | Change a value made with let. Same dimension; inputs and constants cannot be changed. | `set total = total + term` |
| `if COND then … else if COND then … else … end` | Choose. Conditions compare like with like: h < 0 [m], not h < 0 [s]. | `if h < 150 [km] then ⏎   refuse "below the lowest altitude the model covers" ⏎ end` |
| `for NAME = FIRST to LAST … end` | Repeat for whole numbers FIRST..LAST. The count is fixed when written; the loop variable is a pure number. | `for n = 1 to 10 ⏎   set total = total + x ^ n / n ⏎ end` |
| `while COND at most N times … end` | Repeat while COND holds — an iteration that settles, or a count the inputs decide. N is the most passes it may take, fixed when written; if COND still holds after N passes the node refuses, saying the loop did not settle, rather than answer with wherever it had got to. | `while abs(r * r - a) > 1e-12 * a at most 40 times ⏎   set r = (r + a / r) / 2 ⏎ end` |
| `refuse "reason"` | The node will not answer here, and says why. A refusal is never a substitute value. | `refuse "the orbit is inside the Earth"` |
| `return EXPR` | The node's answer, in its declared quantity. Every path ends in return or refuse. | `return v` |
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
| `interp` | x like the table's x row; the y row's unit out | straight-line lookup in a table: interp(x, [x1, x2, …] [unit], [y1, y2, …] [unit]); held at the ends |

## Kernel functions

Relations too long to write as a formula — an integral up the atmosphere, a decay over many orbits — already live in the kernel, reviewed once. A method calls them by name, each argument in the unit shown; the checker holds the units, and the engine runs the kernel itself, so the method and the built node cannot differ.

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

*illustrative — a worked example, not this node.* The same example sits behind **Show the example** on every node form.

**The method.**

```text
# Vallado (2013), eq. 1-18: the speed of a circular two-body orbit.
if r <= R_EARTH then
  refuse "the orbit is inside the Earth"
end
let v : Velocity = sqrt(MU_EARTH / r)
return v
```

**The author's own code** (MATLAB, `orbit_speed`), which produced the cases below.

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

| Case | Inputs | The author's code | The method |
|---|---|---|---|
| 250 km | `r = 6628137` | 7754.84549737 | 7754.845497372695 — agrees with your code |
| 400 km | `r = 6778137` | 7668.55817541 | 7668.558175407055 — agrees with your code |
| 1000 km — the top of the band | `r = 7378137` | 7350.13862961 | 7350.138629613315 — agrees with your code |
| inside the Earth | `r = 6000000` | refuses | refused, as your code does |

## Where the simple version breaks

- **One answer per method.** A node that publishes a set of values (a few rows do) keeps its
  hand-written hole for now; the method language answers one quantity.
- **No iteration to convergence.** A loop runs a fixed count. A solver that stops when it
  converges is marked for a developer, who writes it, and your cases still decide.
- **Tables are written out.** A lookup into a large data file is a reference-data bundle,
  not a method.
