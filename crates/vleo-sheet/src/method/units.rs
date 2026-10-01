//! The method language: dimensions: a unit's seven exponents, and reading a unit from text.

use super::*;

fn dim_arr(d: Dim) -> [i8; 7] {
    [d.m, d.kg, d.s, d.a, d.k, d.mol, d.cd]
}

fn dim_from(a: [i8; 7]) -> Dim {
    Dim::new(a[0], a[1], a[2], a[3], a[4], a[5], a[6])
}

pub(super) fn dim_mul(a: Dim, b: Dim) -> Dim {
    let (x, y) = (dim_arr(a), dim_arr(b));
    dim_from(std::array::from_fn(|i| x[i] + y[i]))
}

pub(super) fn dim_div(a: Dim, b: Dim) -> Dim {
    let (x, y) = (dim_arr(a), dim_arr(b));
    dim_from(std::array::from_fn(|i| x[i] - y[i]))
}

/// `a` raised to `num/den`, when every exponent stays whole.
pub(super) fn dim_pow(a: Dim, num: i32, den: i32) -> Option<Dim> {
    let x = dim_arr(a);
    let mut out = [0i8; 7];
    for i in 0..7 {
        let p = x[i] as i32 * num;
        if p % den != 0 {
            return None;
        }
        out[i] = i8::try_from(p / den).ok()?;
    }
    Some(dim_from(out))
}

pub(super) fn is_none(d: Dim) -> bool {
    d == Dim::NONE
}

/// A dimension as a person reads it: `m^3/s^2`, or `dimensionless`.
pub fn dim_text(d: Dim) -> String {
    if is_none(d) {
        return "dimensionless".into();
    }
    let names = ["m", "kg", "s", "A", "K", "mol", "cd"];
    let x = dim_arr(d);
    let part = |pos: bool| {
        let mut v = Vec::new();
        for i in 0..7 {
            let e = x[i];
            if (pos && e > 0) || (!pos && e < 0) {
                let e = e.abs();
                v.push(if e == 1 {
                    names[i].to_string()
                } else {
                    format!("{}^{e}", names[i])
                });
            }
        }
        v.join(".")
    };
    let (num, den) = (part(true), part(false));
    match (num.is_empty(), den.is_empty()) {
        (false, true) => num,
        (true, false) => format!("1/{den}"),
        _ => format!("{num}/{den}"),
    }
}

/// The dimension of a quantity type a sheet declares, such as `Velocity`.
pub fn quantity_dim(name: &str) -> Option<Dim> {
    vleo_units::quantity_unit(name).map(|u| u.dim())
}

/// Read a unit written in brackets: `km`, `m^3/s^2`, `W/m^2/K^4`, `1/m^3`.
///
/// Returns the factor to SI and the dimension. Any symbol a sheet may declare
/// is accepted whole, and the simple ones combine with `*` or `.`, `/` and a
/// whole-number `^`.
pub fn parse_unit(text: &str) -> Result<(f64, Dim), Error> {
    let t = text.trim();
    if t.is_empty() {
        return Err(Error::new(
            ErrorKind::Malformed,
            "an empty unit — write [1] for a pure number",
        ));
    }
    if t == "1" {
        return Ok((1.0, Dim::NONE));
    }
    for name in Unit::NAMES {
        let u = Unit::from_name(name).expect("a name from the list");
        if u.symbol() == t {
            return Ok((u.si_factor(), u.dim()));
        }
    }
    let atom = |a: &str| -> Result<(f64, Dim), Error> {
        if a == "1" {
            return Ok((1.0, Dim::NONE));
        }
        if a == "mol" {
            return Ok((1.0, Dim::new(0, 0, 0, 0, 0, 1, 0)));
        }
        for name in Unit::NAMES {
            let u = Unit::from_name(name).expect("a name from the list");
            let s = u.symbol();
            if s == a && !s.contains(['/', '.', '^']) {
                return Ok((u.si_factor(), u.dim()));
            }
        }
        Err(Error::new(
            ErrorKind::Malformed,
            format!("«{a}» is not a unit this tool knows"),
        ))
    };
    let mut factor = 1.0;
    let mut dim = Dim::NONE;
    let mut divide = false;
    let mut rest = t;
    loop {
        let end = rest.find(['*', '.', '/']).unwrap_or(rest.len());
        let term = &rest[..end];
        let (base, exp) = match term.split_once('^') {
            Some((b, e)) => (
                b,
                e.parse::<i32>().map_err(|_| {
                    Error::new(
                        ErrorKind::Malformed,
                        format!("«^{e}» in [{t}] is not a whole-number power"),
                    )
                })?,
            ),
            None => (term, 1),
        };
        let (f, d) =
            atom(base.trim()).map_err(|e| Error::new(e.kind(), format!("{e} (in [{t}])")))?;
        let d = dim_pow(d, exp, 1).ok_or_else(|| {
            Error::new(ErrorKind::Malformed, format!("[{t}] is too large a power"))
        })?;
        let f = pmath::powi(f, exp);
        if divide {
            factor /= f;
            dim = dim_div(dim, d);
        } else {
            factor *= f;
            dim = dim_mul(dim, d);
        }
        if end == rest.len() {
            break;
        }
        divide = &rest[end..end + 1] == "/";
        rest = &rest[end + 1..];
    }
    Ok((factor, dim))
}
