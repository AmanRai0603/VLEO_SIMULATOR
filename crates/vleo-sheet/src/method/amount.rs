//! The method language: an amount as a person types it (docs/briefs/F8.md,
//! F8-a, item 2).
//!
//! A value, `250 km` or `250 [km]` or `2.2`; a range, `2.5e-11 to 1.5e-10
//! kg/m^3`, one unit after both ends or each end its own of one dimension;
//! or a spread, `55 ± 10% mN` or `55 +/- 5.5 mN`, whose ends are the nominal
//! less and plus the spread. The number is read as the lexer reads one, the
//! unit by [`parse_unit`] alone, and a spread's ends are worked out as
//! decimals from the digits typed, so `55 ± 10%` is `49.5 to 60.5`, never
//! `49.50000000000001`. This is notation, as a unit's factor is: nothing here
//! computes a quantity of the design (F8's question 64).
//!
//! Refused by name, with the fix, never guessed: a unit [`parse_unit`] does
//! not read, `°C` among them; a comma in a number; ends of different
//! dimensions, or the lower above the upper; a spread in per cent on a field
//! already in `%` (question 65); a bare number in a field whose unit is not a
//! pure number's, unless the field's unit is said (question 66: it is read in
//! the field's unit, and said back); superscripts, and `x10^`.

use super::*;

/// What an amount typed is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Value,
    Range,
    Spread,
}

/// An amount as typed, read: what was typed, its nominal and its ends as
/// decimal text in the unit typed, and that unit's factor to SI and its
/// dimension.
#[derive(Clone, Debug, PartialEq)]
pub struct Amount {
    pub typed: String,
    pub shape: Shape,
    /// The value, or a spread's nominal; `None` for a range.
    pub nominal: Option<String>,
    /// The ends, for a range or a spread.
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
    /// exponent `e` or `E` with an optional sign.
    fn read(s: &str) -> Option<Dec> {
        let s = s.trim();
        let (neg, body) = match s.strip_prefix('-') {
            Some(r) => (true, r),
            None => (false, s.strip_prefix('+').unwrap_or(s)),
        };
        let (digits, exp) = match body.find(['e', 'E']) {
            Some(i) => (&body[..i], body[i + 1..].parse::<i32>().ok()?),
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
        let m: i128 = if all.is_empty() { 0 } else { all.parse().ok()? };
        let e = exp.checked_sub(i32::try_from(frac.len()).ok()?)?;
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
            self.e += 1;
        }
        self
    }
    /// The two at one exponent, the smaller.
    fn aligned(a: Dec, b: Dec) -> Option<(i128, i128, i32)> {
        let e = a.e.min(b.e);
        let up = |d: Dec| -> Option<i128> {
            let k = u32::try_from(d.e - e).ok()?;
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
    /// Decimal text, plain while it is short, else `m.mmme±x`.
    fn text(self) -> String {
        let neg = self.m < 0;
        let digits = self.m.unsigned_abs().to_string();
        let n = i32::try_from(digits.len()).unwrap_or(i32::MAX);
        let point = n + self.e;
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
fn number_then_unit(part: &str) -> Result<(Dec, String, String), String> {
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
    let unit = rest.trim();
    let unit = unit
        .strip_prefix('[')
        .and_then(|u| u.strip_suffix(']'))
        .unwrap_or(unit)
        .trim();
    let d = Dec::read(num).ok_or_else(|| {
        if num.trim().is_empty() {
            format!("«{p}» does not start with a number: write the number first, as 250 km")
        } else {
            format!("«{num}» is not a number")
        }
    })?;
    Ok((d, num.trim().to_string(), unit.to_string()))
}

/// The unit's factor to SI and dimension, read by [`parse_unit`] alone, with
/// the fix for a spelling it refuses.
fn unit_of(unit: &str) -> Result<(f64, Dim), String> {
    let hint = match unit {
        u if u.contains('°') => {
            " — this tool holds temperatures in kelvin, [K]: a Celsius value is the kelvin value less 273.15"
        }
        u if u.contains('µ') || u.contains('μ') => " — write the micro prefix as u, as uN",
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

/// The quantities a dimension may be, for saying an amount back.
fn kind_of(dim: Dim) -> String {
    if dim == Dim::NONE {
        return "a pure number".into();
    }
    let names: Vec<&str> = vleo_units::QUANTITIES
        .iter()
        .copied()
        .filter(|q| vleo_units::quantity_unit(q).is_some_and(|u| u.dim() == dim))
        .collect();
    match names.as_slice() {
        [] => format!("of dimension {}", dim_text(dim)),
        [one] => format!("a {one}"),
        many => format!("a {}", many.join(" or a ")),
    }
}

/// What a person typed in a field whose unit is `field_unit` (empty when the
/// field has none), read as an amount, or why not, with the fix.
pub fn read(typed: &str, field_unit: &str) -> Result<Amount, String> {
    let t = typed.trim();
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
    let spread = ["±", "+/-"].iter().find_map(|s| t.split_once(*s));
    let range = t.split_once(" to ");
    match (spread, range) {
        (Some(_), Some(_)) => Err(format!(
            "«{t}» is both a range and a spread: write one, as 49.5 to 60.5 mN or 55 ± 10% mN"
        )),
        (Some((nominal, by)), None) => spread_of(t, nominal, by, field_unit),
        (None, Some((lo, hi))) => range_of(t, lo, hi, field_unit),
        (None, None) => value_of(t, field_unit),
    }
}

/// The unit an amount's numbers are in: the one typed, else the field's;
/// a bare number in a field with a dimensioned unit is read in that unit and
/// said so (question 66).
fn unit_for(typed_unit: &str, field_unit: &str) -> Result<(String, f64, Dim, bool), String> {
    let (unit, from_field) = match (typed_unit.is_empty(), field_unit.trim().is_empty()) {
        (false, _) => (typed_unit.to_string(), false),
        (true, false) => (field_unit.trim().to_string(), true),
        (true, true) => ("1".to_string(), false),
    };
    let (factor, dim) = unit_of(&unit)?;
    if !field_unit.trim().is_empty() && !typed_unit.is_empty() {
        let (_, want) = unit_of(field_unit.trim())
            .map_err(|e| format!("the field's own unit [{field_unit}] does not read: {e}"))?;
        if want != dim {
            return Err(format!(
                "[{unit}] is {}, and this field is in [{}], {}",
                kind_of(dim),
                field_unit.trim(),
                kind_of(want)
            ));
        }
    }
    Ok((unit, factor, dim, from_field))
}

fn said(nominal: &str, unit: &str, factor: f64, dim: Dim, from_field: bool) -> String {
    let si = Dec::read(nominal)
        .map(|d| d.f64() * factor)
        .map(|v| format!("{v}"))
        .unwrap_or_default();
    let base = if dim == Dim::NONE {
        String::new()
    } else {
        format!(" {}", dim_text(dim))
    };
    format!(
        "{nominal} {unit}{}, {}: {si}{base} in SI",
        if from_field { ", the field's unit" } else { "" },
        kind_of(dim)
    )
}

fn value_of(t: &str, field_unit: &str) -> Result<Amount, String> {
    let (_, num, typed_unit) = number_then_unit(t)?;
    let (unit, factor, dim, from_field) = unit_for(&typed_unit, field_unit)?;
    Ok(Amount {
        typed: t.to_string(),
        shape: Shape::Value,
        nominal: Some(Dec::read(&num).map(Dec::text).unwrap_or(num.clone())),
        lower: None,
        upper: None,
        said: said(&num, &unit, factor, dim, from_field),
        unit,
        factor,
        dim,
    })
}

fn range_of(t: &str, lo: &str, hi: &str, field_unit: &str) -> Result<Amount, String> {
    let (l, _, lu) = number_then_unit(lo)?;
    let (h, _, hu) = number_then_unit(hi)?;
    // One unit after both ends, or each end its own of one dimension.
    let (unit, factor, dim, from_field) =
        unit_for(if hu.is_empty() { &lu } else { &hu }, field_unit)?;
    let (l, h) = if lu.is_empty() || lu == unit {
        (l, h)
    } else {
        let (lf, ld) = unit_of(&lu)?;
        if ld != dim {
            return Err(format!(
                "the lower end is in [{lu}], {}, and the upper in [{unit}], {}: both ends are of one dimension",
                kind_of(ld),
                kind_of(dim)
            ));
        }
        // The lower end said in the upper end's unit: its factor over the
        // upper's, exact when both are powers of ten.
        let ratio = lf / factor;
        let exact = (-30..=30)
            .find(|k| ratio == format!("1e{k}").parse::<f64>().unwrap_or(f64::NAN))
            .map(|k| Dec { m: 1, e: k });
        match exact.and_then(|r| Dec::mul(l, r)) {
            Some(l) => (l, h),
            None => {
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
    let what = format!("{} to {}", l.text(), h.text());
    Ok(Amount {
        typed: t.to_string(),
        shape: Shape::Range,
        nominal: None,
        said: format!(
            "{what} {unit}{}, {}",
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

fn spread_of(t: &str, nominal: &str, by: &str, field_unit: &str) -> Result<Amount, String> {
    let (n, _, nu) = number_then_unit(nominal)?;
    let by = by.trim();
    let (pct, by_rest) = match by.split_once('%') {
        Some((p, rest)) => (Some(p.trim()), rest.trim()),
        None => (None, ""),
    };
    let (spread, unit_text) = match pct {
        Some(p) => {
            let d = Dec::read(p).ok_or_else(|| format!("«{p}» is not a number of per cent"))?;
            let u = if nu.is_empty() {
                by_rest.to_string()
            } else {
                nu.clone()
            };
            let (unit, ..) = unit_for(&u, field_unit)?;
            if unit == "%" {
                return Err(format!(
                    "«{t}» is a spread in per cent of a value in per cent: 10% of 50% or 10 points? Write its ends, as 45 to 55 %"
                ));
            }
            // The spread is the nominal times the per cent over a hundred.
            let s = Dec::mul(n, Dec { m: d.m, e: d.e - 2 })
                .ok_or("the spread is too long a number to work out exactly")?;
            (s, u)
        }
        None => {
            let (s, _, su) = number_then_unit(by)?;
            if !nu.is_empty() && !su.is_empty() && nu != su {
                return Err(format!(
                    "the nominal is in [{nu}] and the spread in [{su}]: write the unit once, after the spread, as 55 ± 5.5 mN"
                ));
            }
            (s, if su.is_empty() { nu.clone() } else { su })
        }
    };
    if spread.m < 0 {
        return Err(format!(
            "the spread in «{t}» is negative: write it as a size, as ± 5"
        ));
    }
    let (unit, factor, dim, from_field) = unit_for(&unit_text, field_unit)?;
    let lo = Dec::sub(n, spread).ok_or("the lower end is too long a number to work out exactly")?;
    let hi = Dec::add(n, spread).ok_or("the upper end is too long a number to work out exactly")?;
    Ok(Amount {
        typed: t.to_string(),
        shape: Shape::Spread,
        said: format!(
            "{} {unit}, from {} to {}{}, {}",
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
