//! UTC calendar dates from Unix seconds, by arithmetic alone.
//!
//! Every face stamps what it writes — a saved result, a sheet's date — and
//! they used to ask the `date` program for it. Windows has no such program, so
//! on the platform most of the team uses every stamp came back empty and two
//! results saved in one session took the same file name. This is the one
//! conversion they all use now: whole days to a civil date by Howard Hinnant's
//! `civil_from_days`, which is exact for every day of the proleptic Gregorian
//! calendar, with no clock, no allocation and no platform.

use core::fmt;

/// One instant as a UTC calendar date and time of day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Civil {
    pub year: i64,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl Civil {
    /// The UTC date and time `secs` seconds after 1970-01-01T00:00:00Z.
    pub fn from_unix(secs: i64) -> Civil {
        let days = secs.div_euclid(86_400);
        let rest = secs.rem_euclid(86_400);
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        let year = yoe + era * 400 + i64::from(month <= 2);
        Civil {
            year,
            month,
            day,
            hour: (rest / 3600) as u32,
            minute: (rest % 3600 / 60) as u32,
            second: (rest % 60) as u32,
        }
    }

    /// The date alone, as `YYYY-MM-DD`.
    pub fn date(self) -> Date {
        Date(self)
    }

    /// Seconds after 1970-01-01T00:00:00Z — the inverse of [`Civil::from_unix`],
    /// by Hinnant's `days_from_civil`.
    pub fn to_unix(self) -> i64 {
        let y = self.year - i64::from(self.month <= 2);
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = i64::from(self.month);
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        let days = era * 146_097 + doe - 719_468;
        days * 86_400
            + i64::from(self.hour) * 3600
            + i64::from(self.minute) * 60
            + i64::from(self.second)
    }

    /// A stamp as this module writes it — `YYYY-MM-DDTHH:MM:SSZ`, or the date
    /// alone — read back. Anything else is `None`, never a guess.
    pub fn parse(text: &str) -> Option<Civil> {
        let t = text.trim();
        let (date, time) = match t.split_once('T') {
            Some((d, rest)) => (d, Some(rest.strip_suffix('Z')?)),
            None => (t, None),
        };
        let mut d = date.split('-');
        let year: i64 = d.next()?.parse().ok()?;
        let month: u32 = d.next()?.parse().ok()?;
        let day: u32 = d.next()?.parse().ok()?;
        if d.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
            return None;
        }
        let (hour, minute, second) = match time {
            Some(tm) => {
                let mut p = tm.split(':');
                let h: u32 = p.next()?.parse().ok()?;
                let m: u32 = p.next()?.parse().ok()?;
                let s: u32 = p.next()?.parse().ok()?;
                if p.next().is_some() || h > 23 || m > 59 || s > 60 {
                    return None;
                }
                (h, m, s)
            }
            None => (0, 0, 0),
        };
        let c = Civil {
            year,
            month,
            day,
            hour,
            minute,
            second,
        };
        // A day that does not exist — 31 April — comes back as another date.
        (Civil::from_unix(c.to_unix()) == c).then_some(c)
    }
}

/// `YYYY-MM-DDTHH:MM:SSZ`.
impl fmt::Display for Civil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}T{:02}:{:02}:{:02}Z",
            self.date(),
            self.hour,
            self.minute,
            self.second
        )
    }
}

/// A [`Civil`] shown as its date alone.
#[derive(Clone, Copy, Debug)]
pub struct Date(Civil);

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}",
            self.0.year, self.0.month, self.0.day
        )
    }
}

#[cfg(test)]
mod round_trip {
    use super::Civil;

    #[test]
    fn a_stamp_reads_back_to_the_second_it_was_written() {
        for secs in [
            0_i64,
            951_782_400,
            1_790_000_000,
            4_102_444_799,
            -86_400,
            1_709_164_800,
        ] {
            let c = Civil::from_unix(secs);
            assert_eq!(c.to_unix(), secs, "{c}");
            let written = alloc_free(c);
            assert_eq!(Civil::parse(&written), Some(c), "{}", &*written);
        }
        assert_eq!(
            Civil::parse("2026-09-29").map(|c| c.to_unix()),
            Some(1_790_640_000)
        );
        for bad in [
            "2026-04-31",
            "2026-13-01",
            "2026-09-29T25:00:00Z",
            "yesterday",
            "2026-09-29T10:00:00",
        ] {
            assert_eq!(Civil::parse(bad), None, "{bad} was read");
        }
    }

    /// The Display form without allocating, as this crate is `no_std`.
    fn alloc_free(c: Civil) -> heapless_str::S {
        let mut s = heapless_str::S::default();
        let _ = core::fmt::write(&mut s, format_args!("{c}"));
        s
    }

    mod heapless_str {
        #[derive(Default)]
        pub struct S {
            buf: [u8; 32],
            len: usize,
        }
        impl core::fmt::Write for S {
            fn write_str(&mut self, t: &str) -> core::fmt::Result {
                let b = t.as_bytes();
                self.buf[self.len..self.len + b.len()].copy_from_slice(b);
                self.len += b.len();
                Ok(())
            }
        }
        impl core::ops::Deref for S {
            type Target = str;
            fn deref(&self) -> &str {
                core::str::from_utf8(&self.buf[..self.len]).unwrap()
            }
        }
    }
}
