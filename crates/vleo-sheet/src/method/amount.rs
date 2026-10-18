//! The method language: an amount as a person types it (docs/briefs/F8.md,
//! F8-a, item 2).
//!
//! A value, `250 km` or `250 [km]` or `2.2`; a range, `2.5e-11 to 1.5e-10
//! kg/m^3`, one unit after both ends or each end its own of one dimension;
//! or a spread, `55 ± 10% mN` or `55 +/- 5.5 mN`, whose ends are the nominal
//! less and plus the spread. The number is read as the lexer reads one, the
//! unit by [`parse_unit`] alone, and every number worked out (a spread's
//! ends, a range's lower end in the upper end's unit, the value in SI said
//! back) is worked out as a decimal from the digits typed, so `55 ± 10%` is
//! `49.5 to 60.5`, never `49.50000000000001`. This is notation, as a unit's
//! factor is: nothing here computes a quantity of the design (F8's question
//! 64).
//!
//! A bare number in a field with a unit is read in the field's unit, and said
//! so (question 66); in a field with none, as a pure number. A field's unit
//! may be a symbol (`km`) or a machine name (`Kilometre`), as design files
//! hold it.
//!
//! Refused by name, with the fix, never guessed: a unit [`parse_unit`] does
//! not read, `°C` among them; a comma in a number; a number too large or too
//! small to hold; ends of different dimensions, in units that differ by other
//! than a power of ten, written with one end's unit only, or the lower above
//! the upper; a unit after the per-cent unlike the nominal's; a spread in per
//! cent on a field already in `%` (question 65); a unit unlike the field's;
//! superscripts, and `x10^`.

use super::*;

/// What an amount typed is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Value,
    Range,
    Spread,
}

/// An amount as typed, read: what was typed, its nominal and its ends as
/// decimal text in the unit it is held in, and that unit's factor to SI and
/// its dimension.
#[derive(Clone, Debug, PartialEq)]
pub struct Amount {
    pub typed: String,
    pub shape: Shape,
    /// The value, or a spread's nominal; `None` for a range.
    pub nominal: Option<String>,
    /// The ends, for a range or a spread; a range's lower end is said in the
    /// upper end's unit when the two were typed in two.
    pub lower: Option<String>,
    pub upper: Option<String>,
    /// The unit the numbers are in, as written (`km`, `kg/m^3`, `1`).
    pub unit: String,
    pub factor: f64,
    pub dim: Dim,
    /// The amount said back: `250 km, a Length: 250000 m in SI`.
    pub said: String,
}

/// A number as typed, held exactly: `mantissa * 10^exp`.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Dec {
    m: i128,
    e: i32,
}

impl Dec {
    /// The digits of a number as the lexer reads one: digits, one `.`, and an
    /// exponent `e` or `E` with an optional sign. `None` when it is not one,
    /// or reaches beyond what a number holds.
    fn read(s: &str) -> Option<Dec> {
        let s = s.trim();
        let (neg, body) = match s.strip_prefix('-') {
            Some(r) => (true, r),
            None => (false, s.strip_prefix('+').unwrap_or(s)),
        };
        let (digits, exp) = match body.find(['e', 'E']) {
            Some(i) => (&body[..i], body[i + 1..].parse::<i64>().ok()?),
            None => (body, 0),
        };
        if digits.is_empty() || digits.matches('.').count() > 1 {
            return None;
        }
        let (whole, frac) = digits.split_once('.').unwrap_or((digits, ""));
        if whole.is_empty() && frac.is_empty() {
            return None;
        }
        if !whole
            .chars()
            .chain(frac.chars())
            .all(|c| c.is_ascii_digit())
        {
            return None;
        }
        let all = format!("{whole}{frac}");
        let all = all.trim_start_matches('0');
        let all = all.trim_end_matches('0');
        let trailing =
            format!("{whole}{frac}").trim_start_matches('0').len() as i64 - all.len() as i64;
        let m: i128 = if all.is_empty() { 0 } else { all.parse().ok()? };
        let e = exp + trailing - frac.len() as i64;
        let e = i32::try_from(if m == 0 { 0 } else { e }).ok()?;
        Some(
            Dec {
                m: if neg { -m } else { m },
                e,
            }
            .norm(),
        )
    }
    fn norm(mut self) -> Dec {
        if self.m == 0 {
            return Dec { m: 0, e: 0 };
        }
        while self.m % 10 == 0 {
            self.m /= 10;
            self.e = self.e.saturating_add(1);
        }
        self
    }
    /// Whether it lies within what a number holds: finite, and not zero when
    /// its digits are not.
    fn holds(self) -> bool {
        let v = self.f64();
        v.is_finite() && (self.m == 0 || v != 0.0)
    }
    /// The two at one exponent, the smaller.
    fn aligned(a: Dec, b: Dec) -> Option<(i128, i128, i32)> {
        let e = a.e.min(b.e);
        let up = |d: Dec| -> Option<i128> {
            let k = u32::try_from(d.e.checked_sub(e)?).ok()?;
            d.m.checked_mul(10i128.checked_pow(k)?)
        };
        Some((up(a)?, up(b)?, e))
    }
    fn add(a: Dec, b: Dec) -> Option<Dec> {
        let (x, y, e) = Dec::aligned(a, b)?;
        Some(
            Dec {
                m: x.checked_add(y)?,
                e,
            }
            .norm(),
        )
    }
    fn sub(a: Dec, b: Dec) -> Option<Dec> {
        let (x, y, e) = Dec::aligned(a, b)?;
        Some(
            Dec {
                m: x.checked_sub(y)?,
                e,
            }
            .norm(),
        )
    }
    fn mul(a: Dec, b: Dec) -> Option<Dec> {
        Some(
            Dec {
                m: a.m.checked_mul(b.m)?,
                e: a.e.checked_add(b.e)?,
            }
            .norm(),
        )
    }
    fn cmp(a: Dec, b: Dec) -> std::cmp::Ordering {
        match Dec::aligned(a, b) {
            Some((x, y, _)) => x.cmp(&y),
            None => a.f64().total_cmp(&b.f64()),
        }
    }
    fn f64(self) -> f64 {
        format!("{}e{}", self.m, self.e).parse().unwrap_or(f64::NAN)
    }
    /// A unit's factor to SI as the decimal it is written as: the shortest
    /// digits that read back as the same number.
    fn of_factor(f: f64) -> Option<Dec> {
        Dec::read(&format!("{f:e}"))
    }
    /// Decimal text, plain while it is short, else `m.mmme±x`.
    fn text(self) -> String {
        let neg = self.m < 0;
        let digits = self.m.unsigned_abs().to_string();
        let point = digits.len() as i64 + i64::from(self.e);
        let plain = if self.e >= 0 && point <= 21 {
            format!("{digits}{}", "0".repeat(self.e as usize))
        } else if point > 0 && self.e < 0 {
            let p = point as usize;
            format!("{}.{}", &digits[..p], &digits[p..])
        } else if point <= 0 && point > -7 {
            format!("0.{}{digits}", "0".repeat((-point) as usize))
        } else {
            let (head, tail) = digits.split_at(1);
            let mant = if tail.is_empty() {
                head.to_string()
            } else {
                format!("{head}.{tail}")
            };
            format!("{mant}e{}", point - 1)
        };
        if neg {
            format!("-{plain}")
        } else {
            plain
        }
    }
}

/// A number and the unit written after it, if one is: `250 km`, `250 [km]`,
/// `250km`, `55 %`.
fn number_then_unit(part: &str) -> Result<(Dec, String), String> {
    let p = part.trim();
    let b = p.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'-' || b[i] == b'+') {
        i += 1;
    }
    while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.') {
        i += 1;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let save = i;
        i += 1;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        if i < b.len() && b[i].is_ascii_digit() {
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
        } else {
            i = save;
        }
    }
    let (num, rest) = p.split_at(i);
    let d = match Dec::read(num) {
        _ if num.trim().is_empty() => {
            return Err(format!(
                "«{p}» does not start with a number: write the number first, as 250 km"
            ))
        }
        Some(d) if d.holds() => d,
        Some(_) => {
            return Err(format!(
                "«{num}» is too large or too small a number to hold"
            ))
        }
        // A number with an exponent beyond reach is a number all the same:
        // said as too large or too small, never as not one.
        None => {
            let (mant, exp) = num.split_once(['e', 'E']).unwrap_or((num, ""));
            let digits = exp.trim_start_matches(['+', '-']);
            if Dec::read(mant).is_some()
                && !digits.is_empty()
                && digits.chars().all(|c| c.is_ascii_digit())
            {
                return Err(format!(
                    "«{num}» is too large or too small a number to hold"
                ));
            }
            return Err(format!("«{num}» is not a number"));
        }
    };
    Ok((d, bracketless(rest)))
}

/// A unit as written after a number, its brackets taken off.
fn bracketless(unit: &str) -> String {
    let u = unit.trim();
    u.strip_prefix('[')
        .and_then(|u| u.strip_suffix(']'))
        .unwrap_or(u)
        .trim()
        .to_string()
}

/// The unit's factor to SI and dimension, read by [`parse_unit`] alone, with
/// the fix for a spelling it refuses.
fn unit_of(unit: &str) -> Result<(f64, Dim), String> {
    let hint = match unit {
        u if u.contains('°') => {
            " — this tool holds temperatures in kelvin, [K]: a Celsius value is the kelvin value less 273.15"
        }
        u if u.contains('µ') || u.contains('μ') => " — write the micro prefix as u, as uN",
        u if u.split_whitespace().any(|w| w.eq_ignore_ascii_case("to")) || u.starts_with("to") => {
            " — a range is written with to between its ends, a space each side, as 1 to 2 km"
        }
        u if u.chars().any(|c| c.is_ascii_alphabetic())
            && u.chars().last().is_some_and(|c| c.is_ascii_digit())
            && !u.contains('^') =>
        {
            " — write a power with ^, as m^3"
        }
        _ => "",
    };
    parse_unit(unit).map_err(|e| format!("{}{hint}", e.message()))
}

/// A field's unit as the method language writes it: a design file's machine
/// name (`Kilometre`) is its symbol (`km`).
fn field_symbol(field: &str) -> String {
    let f = field.trim();
    match (parse_unit(f), Unit::from_name(f)) {
        (Err(_), Some(u)) => u.symbol().to_string(),
        _ => f.to_string(),
    }
}

/// The quantities a dimension may be, for saying an amount back.
pub(crate) fn kind_of(dim: Dim) -> String {
    if dim == Dim::NONE {
        return "a pure number".into();
    }
    let a = |q: &str| {
        if q.starts_with(['A', 'E', 'I', 'O', 'U']) {
            format!("an {q}")
        } else {
            format!("a {q}")
        }
    };
    let names: Vec<String> = vleo_units::QUANTITIES
        .iter()
        .copied()
        .filter(|q| vleo_units::quantity_unit(q).is_some_and(|u| u.dim() == dim))
        .map(a)
        .collect();
    if names.is_empty() {
        format!("of dimension {}", dim_text(dim))
    } else {
        names.join(" or ")
    }
}

/// What a person typed in a field whose unit is `field_unit` (empty when the
/// field has none), read as an amount, or why not, with the fix.
pub fn read(typed: &str, field_unit: &str) -> Result<Amount, String> {
    let t = typed.trim();
    let field = field_symbol(field_unit);
    if t.is_empty() {
        return Err("nothing is typed: write a value, as 250 km, a range, as 1 to 2 km, or a spread, as 55 ± 10% mN".into());
    }
    if let Some(c) = t.chars().find(|c| "⁰¹²³⁴⁵⁶⁷⁸⁹⁻".contains(*c)) {
        return Err(format!(
            "«{c}» is a superscript: write a power with ^, as m^3, and a power of ten with e, as 1.5e-10"
        ));
    }
    if t.contains("x10^") || t.contains("×10^") || t.contains("*10^") {
        return Err("write a power of ten with e, as 1.5e-10, not x10^".into());
    }
    if t.contains(',') {
        return Err(format!(
            "«{t}» has a comma: write a decimal point, as 1.5, and no thousands separator"
        ));
    }
    if t.contains('−') {
        return Err("«−» is a typographic minus: write a minus as -, as -5".into());
    }
    let spread = ["±", "+/-"].iter().find_map(|s| t.split_once(*s));
    let range = t.split_once(" to ");
    match (spread, range) {
        (Some(_), Some(_)) => Err(format!(
            "«{t}» is both a range and a spread: write one, as 49.5 to 60.5 mN or 55 ± 10% mN"
        )),
        (Some((nominal, by)), None) => spread_of(t, nominal, by, &field),
        (None, Some((lo, hi))) => range_of(t, lo, hi, &field),
        (None, None) => value_of(t, &field),
    }
}

/// The unit an amount's numbers are in: the one typed, else the field's;
/// a bare number in a field with a dimensioned unit is read in that unit and
/// said so (question 66).
fn unit_for(typed_unit: &str, field: &str) -> Result<(String, f64, Dim, bool), String> {
    let (unit, from_field) = match (typed_unit.is_empty(), field.is_empty()) {
        (false, _) => (typed_unit.to_string(), false),
        (true, false) => (field.to_string(), true),
        (true, true) => ("1".to_string(), false),
    };
    let (factor, dim) = unit_of(&unit).map_err(|e| {
        if from_field {
            format!("the field's own unit [{field}] does not read: {e}")
        } else {
            e
        }
    })?;
    if !field.is_empty() && !typed_unit.is_empty() {
        let (_, want) = unit_of(field)
            .map_err(|e| format!("the field's own unit [{field}] does not read: {e}"))?;
        if want != dim {
            return Err(format!(
                "[{unit}] is {}, and this field is in [{field}], {}",
                kind_of(dim),
                kind_of(want)
            ));
        }
    }
    Ok((unit, factor, dim, from_field))
}

/// `n` in `unit` said back: the number, its unit unless it is a pure
/// number's, what it is, and its value in SI, worked out from the digits.
fn said(n: Dec, unit: &str, factor: f64, dim: Dim, from_field: bool) -> String {
    let si = Dec::of_factor(factor)
        .and_then(|f| Dec::mul(n, f))
        .map(Dec::text)
        .unwrap_or_else(|| format!("{}", n.f64() * factor));
    let shown = if unit == "1" {
        String::new()
    } else {
        format!(" {unit}")
    };
    let base = if dim == Dim::NONE {
        String::new()
    } else {
        format!(" {}", dim_text(dim))
    };
    format!(
        "{}{shown}{}, {}: {si}{base} in SI",
        n.text(),
        if from_field { ", the field's unit" } else { "" },
        kind_of(dim)
    )
}

fn value_of(t: &str, field: &str) -> Result<Amount, String> {
    let (n, typed_unit) = number_then_unit(t)?;
    let (unit, factor, dim, from_field) = unit_for(&typed_unit, field)?;
    Ok(Amount {
        typed: t.to_string(),
        shape: Shape::Value,
        nominal: Some(n.text()),
        lower: None,
        upper: None,
        said: said(n, &unit, factor, dim, from_field),
        unit,
        factor,
        dim,
    })
}

fn range_of(t: &str, lo: &str, hi: &str, field: &str) -> Result<Amount, String> {
    let (l, lu) = number_then_unit(lo)?;
    let (h, hu) = number_then_unit(hi)?;
    if !lu.is_empty() && hu.is_empty() {
        return Err(format!(
            "the lower end has its unit [{lu}] and the upper end has none: write the unit after the upper end, as {} to {} {lu}",
            l.text(),
            h.text()
        ));
    }
    // One unit after both ends, or each end its own of one dimension.
    let (unit, factor, dim, from_field) = unit_for(&hu, field)?;
    let l = if lu.is_empty() || lu == unit {
        l
    } else {
        let (lf, ld) = unit_of(&lu)?;
        if ld != dim {
            return Err(format!(
                "the lower end is in [{lu}], {}, and the upper in [{unit}], {}: both ends are of one dimension",
                kind_of(ld),
                kind_of(dim)
            ));
        }
        // The lower end said in the upper end's unit: exact when the two
        // factors are one mantissa at two powers of ten.
        match (Dec::of_factor(lf), Dec::of_factor(factor)) {
            (Some(a), Some(b)) if a.m == b.m => Dec::mul(l, Dec { m: 1, e: a.e - b.e })
                .ok_or("the lower end is too long a number to work out exactly")?,
            _ => {
                return Err(format!(
                    "write both ends in one unit: [{lu}] and [{unit}] differ by a factor that is not a power of ten"
                ))
            }
        }
    };
    if Dec::cmp(l, h) == std::cmp::Ordering::Greater {
        return Err(format!(
            "the lower end {} is above the upper end {}: write the lower first, as {} to {} {unit}",
            l.text(),
            h.text(),
            h.text(),
            l.text()
        ));
    }
    let shown = if unit == "1" {
        String::new()
    } else {
        format!(" {unit}")
    };
    Ok(Amount {
        typed: t.to_string(),
        shape: Shape::Range,
        nominal: None,
        said: format!(
            "{} to {}{shown}{}, {}",
            l.text(),
            h.text(),
            if from_field { ", the field's unit" } else { "" },
            kind_of(dim)
        ),
        lower: Some(l.text()),
        upper: Some(h.text()),
        unit,
        factor,
        dim,
    })
}

fn spread_of(t: &str, nominal: &str, by: &str, field: &str) -> Result<Amount, String> {
    let (n, nu) = number_then_unit(nominal)?;
    let by = by.trim();
    let (spread, unit_text) = match by.split_once('%') {
        Some((p, rest)) => {
            let p = p.trim();
            let d = Dec::read(p).ok_or_else(|| format!("«{p}» is not a number of per cent"))?;
            if d.m < 0 {
                return Err(format!(
                    "the spread in «{t}» is negative: write it as a size, as ± {}%",
                    Dec { m: -d.m, e: d.e }.text()
                ));
            }
            let after = bracketless(rest);
            if !nu.is_empty() && !after.is_empty() && nu != after {
                return Err(format!(
                    "the nominal is in [{nu}] and the spread names [{after}]: write the unit once, as 55 ± 10% mN"
                ));
            }
            let u = if nu.is_empty() { after } else { nu.clone() };
            let (unit, ..) = unit_for(&u, field)?;
            if unit == "%" {
                return Err(format!(
                    "«{t}» is a spread in per cent of a value in per cent: 10% of 50% or 10 points? Write its ends, as 45 to 55 %"
                ));
            }
            // The spread is the nominal's size times the per cent over a
            // hundred.
            let e =
                d.e.checked_sub(2)
                    .ok_or("the spread is too long a number to work out exactly")?;
            let s = Dec::mul(
                Dec {
                    m: n.m.abs(),
                    e: n.e,
                },
                Dec { m: d.m, e },
            )
            .ok_or("the spread is too long a number to work out exactly")?;
            (s, u)
        }
        None => {
            let (s, su) = number_then_unit(by)?;
            if !nu.is_empty() && !su.is_empty() && nu != su {
                return Err(format!(
                    "the nominal is in [{nu}] and the spread in [{su}]: write the unit once, after the spread, as 55 ± 5.5 mN"
                ));
            }
            if s.m < 0 {
                return Err(format!(
                    "the spread in «{t}» is negative: write it as a size, as ± {}",
                    Dec { m: -s.m, e: s.e }.text()
                ));
            }
            (s, if su.is_empty() { nu.clone() } else { su })
        }
    };
    let (unit, factor, dim, from_field) = unit_for(&unit_text, field)?;
    let lo = Dec::sub(n, spread).ok_or("the lower end is too long a number to work out exactly")?;
    let hi = Dec::add(n, spread).ok_or("the upper end is too long a number to work out exactly")?;
    let shown = if unit == "1" {
        String::new()
    } else {
        format!(" {unit}")
    };
    Ok(Amount {
        typed: t.to_string(),
        shape: Shape::Spread,
        said: format!(
            "{}{shown}, from {} to {}{}, {}",
            n.text(),
            lo.text(),
            hi.text(),
            if from_field { ", the field's unit" } else { "" },
            kind_of(dim)
        ),
        nominal: Some(n.text()),
        lower: Some(lo.text()),
        upper: Some(hi.text()),
        unit,
        factor,
        dim,
    })
}
