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
