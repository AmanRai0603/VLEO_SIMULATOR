//! Kernel functions a method may call by name.
//!
//! Some relations are not a formula a person writes in a line: the
//! thermosphere's composition is a five-species integral up the altitude, an
//! orbit's decay time is a quadrature over that density, the solar-cycle
//! analogue is an integral over a stacked cycle. They already live in
//! `vleo-core::physics`, reviewed once, and a method calls them by name — rule
//! 3: every formula lives in the kernel and nowhere else.
//!
//! Each entry says the unit of every argument and of the answer, so the
//! checker holds a call to them exactly as it holds `sqrt`; the interpreter
//! runs the kernel itself, and the translator writes the same call, so the
//! two cannot disagree. A function is added here only as a reviewed change:
//! it is the language growing.

use vleo_core::physics::{env, orbit};
use vleo_units::{Length, Temperature};

pub struct KernelFn {
    pub name: &'static str,
    /// Each argument's name and unit, as written in brackets.
    pub args: &'static [(&'static str, &'static str)],
    /// The answer's unit.
    pub out: &'static str,
    pub meaning: &'static str,
    /// Where it lives in the kernel, for the reader.
    pub kernel: &'static str,
    /// The interpreter's evaluation: every value in SI.
    pub eval: fn(&[f64]) -> f64,
    /// The translation, `{0}`, `{1}`… standing for the arguments.
    pub rust: &'static str,
}

fn o(a: &[f64]) -> f64 {
    env::composition(Length::new(a[0]), Temperature::new(a[1]))
        .o
        .get()
}
fn total(a: &[f64]) -> f64 {
    env::composition(Length::new(a[0]), Temperature::new(a[1]))
        .total()
        .get()
}
fn molar(a: &[f64]) -> f64 {
    env::composition(Length::new(a[0]), Temperature::new(a[1]))
        .mean_molar_mass()
        .get()
}
fn density(a: &[f64]) -> f64 {
    env::mass_density(Length::new(a[0]), Temperature::new(a[1])).get()
}
fn scale(a: &[f64]) -> f64 {
    env::scale_height(Length::new(a[0]), Temperature::new(a[1])).get()
}
fn decay(a: &[f64]) -> f64 {
    let t_inf = Temperature::new(a[3]);
    orbit::lifetime_estimate(Length::new(a[0]), Length::new(a[1]), a[2], 64, |z| {
        env::mass_density(z, t_inf)
    })
    .get()
}
const DAY: f64 = 86_400.0;
fn cycle_mean(a: &[f64]) -> f64 {
    env::solar_cycle_analogue_mean(a[0] / DAY, a[1] / DAY)
}
fn cycle_max(a: &[f64]) -> f64 {
    env::solar_cycle_analogue_max(a[0] / DAY, a[1] / DAY)
}

const H_T: &[(&str, &str)] = &[("h", "m"), ("T_inf", "K")];

#[rustfmt::skip]
pub const KERNEL_FUNCTIONS: &[KernelFn] = &[
    KernelFn { name: "thermosphere_o", args: H_T, out: "1/m^3", eval: o,
        meaning: "atomic oxygen number density at altitude h, for exospheric temperature T_inf",
        kernel: "env::composition(h, T_inf).o",
        rust: "vleo_core::physics::env::composition(vleo_units::Length::new({0}), vleo_units::Temperature::new({1})).o.get()" },
    KernelFn { name: "thermosphere_number_density", args: H_T, out: "1/m^3", eval: total,
        meaning: "every species' number density at altitude h, summed",
        kernel: "env::composition(h, T_inf).total()",
        rust: "vleo_core::physics::env::composition(vleo_units::Length::new({0}), vleo_units::Temperature::new({1})).total().get()" },
    KernelFn { name: "thermosphere_molar_mass", args: H_T, out: "kg/mol", eval: molar,
        meaning: "the mean molar mass of the gas at altitude h",
        kernel: "env::composition(h, T_inf).mean_molar_mass()",
        rust: "vleo_core::physics::env::composition(vleo_units::Length::new({0}), vleo_units::Temperature::new({1})).mean_molar_mass().get()" },
    KernelFn { name: "thermosphere_density", args: H_T, out: "kg/m^3", eval: density,
        meaning: "the mass density of the gas at altitude h",
        kernel: "env::mass_density(h, T_inf)",
        rust: "vleo_core::physics::env::mass_density(vleo_units::Length::new({0}), vleo_units::Temperature::new({1})).get()" },
    KernelFn { name: "thermosphere_scale_height", args: H_T, out: "m", eval: scale,
        meaning: "the density scale height at altitude h",
        kernel: "env::scale_height(h, T_inf)",
        rust: "vleo_core::physics::env::scale_height(vleo_units::Length::new({0}), vleo_units::Temperature::new({1})).get()" },
    KernelFn { name: "orbit_decay_time", args: &[("h", "m"), ("h_end", "m"), ("bc", "kg/m^2"), ("T_inf", "K")], out: "s", eval: decay,
        meaning: "the time a circular orbit takes to decay from h to h_end, for ballistic coefficient bc, in the thermosphere of T_inf (64 steps)",
        kernel: "orbit::lifetime_estimate(h, h_end, bc, 64, |z| env::mass_density(z, T_inf))",
        rust: "vleo_core::physics::orbit::lifetime_estimate(vleo_units::Length::new({0}), vleo_units::Length::new({1}), {2}, 64, |z| vleo_core::physics::env::mass_density(z, vleo_units::Temperature::new({3}))).get()" },
    KernelFn { name: "solar_cycle_mean", args: &[("t0", "s"), ("t1", "s")], out: "1", eval: cycle_mean,
        meaning: "the solar-cycle analogue's mean F10.7 from mission time t0 to t1",
        kernel: "env::solar_cycle_analogue_mean(t0, t1), in days",
        rust: "vleo_core::physics::env::solar_cycle_analogue_mean(({0}) / 86400.0, ({1}) / 86400.0)" },
    KernelFn { name: "solar_cycle_max", args: &[("t0", "s"), ("t1", "s")], out: "1", eval: cycle_max,
        meaning: "the solar-cycle analogue's highest F10.7 from mission time t0 to t1",
        kernel: "env::solar_cycle_analogue_max(t0, t1), in days",
        rust: "vleo_core::physics::env::solar_cycle_analogue_max(({0}) / 86400.0, ({1}) / 86400.0)" },
];

pub fn kernel_function(name: &str) -> Option<&'static KernelFn> {
    KERNEL_FUNCTIONS.iter().find(|f| f.name == name)
}

/// The translation of a call, with its arguments written in.
pub fn translate(f: &KernelFn, args: &[String]) -> String {
    let mut s = f.rust.to_string();
    for (i, a) in args.iter().enumerate() {
        s = s.replace(&format!("{{{i}}}"), a);
    }
    s
}
