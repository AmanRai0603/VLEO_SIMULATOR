//! What a file says about itself: its kind, its version, what it was based on
//! and who wrote it — and the rules each of those follows.
//!
//! docs/OPERATING_1_0.md, sections 10 and 13. Every file names its kind, so a
//! group release and a released design, which both end `.vleo`, are never
//! taken for each other: [`open_as`] refuses one opened as the other, by name.
//! Every file names what it was based on, so "what changed since" always has
//! an answer.

use std::fmt;

use crate::error::{Error, ErrorKind};
use crate::model::File;

/// This format's number: `user_version` in every file.
pub const FORMAT: i64 = 2;

/// The kinds of file, as section 10 lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Node,
    Group,
    GroupRelease,
    Preview,
    PreviewAnswer,
    Issue,
    Daily,
    Design,
    Case,
    Results,
    Key,
}

impl Kind {
    pub const ALL: [Kind; 11] = [
        Kind::Node,
        Kind::Group,
        Kind::GroupRelease,
        Kind::Preview,
        Kind::PreviewAnswer,
        Kind::Issue,
        Kind::Daily,
        Kind::Design,
        Kind::Case,
        Kind::Results,
        Kind::Key,
    ];

    /// The name `meta.file_kind` holds.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Node => "node",
            Kind::Group => "group",
            Kind::GroupRelease => "group release",
            Kind::Preview => "preview",
            Kind::PreviewAnswer => "preview answer",
            Kind::Issue => "issue",
            Kind::Daily => "daily snapshot",
            Kind::Design => "released design",
            Kind::Case => "case",
            Kind::Results => "results",
            Kind::Key => "key",
        }
    }

    pub fn from_name(name: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.name() == name)
    }

    /// The keys a file of this kind must have in its meta, beside the three
    /// every file has, each with the rule its value follows.
    pub fn required(self) -> &'static [(&'static str, Rule)] {
        use Rule::*;
        match self {
            Kind::Node => &[
                ("group_id", Name),
                ("block_uid", Name),
                ("revision", Count),
                ("contract_version", Count),
                ("writer", Name),
            ],
            Kind::Group => &[
                ("group_id", Name),
                ("writer", Name),
                ("based_on", DesignOrNone),
            ],
            Kind::GroupRelease => &[
                ("group_id", Name),
                ("version", Release),
                ("previous", ReleaseOrNone),
                ("based_on", DesignOrNone),
                ("sealed_by", Name),
                ("fingerprint", Digest),
            ],
            Kind::Preview => &[
                ("group_id", Name),
                ("version", Release),
                ("based_on", Design),
            ],
            Kind::PreviewAnswer => &[
                ("group_id", Name),
                ("version", Release),
                ("answering_group", Name),
                ("writer", Name),
            ],
            Kind::Issue => &[("number", Count), ("group_id", Name), ("writer", Name)],
            Kind::Daily => &[("date", Date), ("based_on", DesignOrNone)],
            Kind::Design => &[
                ("version", Design),
                ("previous", DesignOrNone),
                ("released_by", Name),
                ("oldest_application", Application),
            ],
            Kind::Case => &[("name", Name)],
            Kind::Results => &[("name", Name)],
            Kind::Key => &[("person", Name)],
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The rule a meta value follows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// Any text that is not empty.
    Name,
    /// 1, 2, 3 …: a node's revision or contract version, an issue's number.
    Count,
    /// A group release: major.minor, such as 1.2.
    Release,
    /// A release, or empty for a group's first.
    ReleaseOrNone,
    /// A released design: year.month.sequence, such as 2026.11.2.
    Design,
    /// A released design, or empty before the first.
    DesignOrNone,
    /// An application: major.minor.patch, such as 1.0.0.
    Application,
    /// A day: YYYY-MM-DD.
    Date,
    /// A SHA-256, as 64 hexadecimal digits.
    Digest,
}

impl Rule {
    /// Whether `value` follows this rule.
    pub fn holds(self, value: &str) -> bool {
        match self {
            Rule::Name => !value.trim().is_empty(),
            Rule::Count => value.parse::<u64>().is_ok_and(|n| n >= 1) && !value.starts_with('0'),
            Rule::Release => ReleaseVersion::parse(value).is_some(),
            Rule::ReleaseOrNone => value.is_empty() || ReleaseVersion::parse(value).is_some(),
            Rule::Design => DesignVersion::parse(value).is_some(),
            Rule::DesignOrNone => value.is_empty() || DesignVersion::parse(value).is_some(),
            Rule::Application => {
                let p: Vec<&str> = value.split('.').collect();
                p.len() == 3 && p.iter().all(|x| number(x).is_some())
            }
            Rule::Date => {
                let b = value.as_bytes();
                b.len() == 10
                    && b[4] == b'-'
                    && b[7] == b'-'
                    && digits(&value[0..4]).is_some()
                    && digits(&value[5..7]).is_some_and(|m| (1..=12).contains(&m))
                    && digits(&value[8..10]).is_some_and(|d| (1..=31).contains(&d))
            }
            Rule::Digest => value.len() == 64 && value.bytes().all(|c| c.is_ascii_hexdigit()),
        }
    }

    /// What a person reads when a value does not follow the rule.
    pub fn says(self) -> &'static str {
        match self {
            Rule::Name => "a name",
            Rule::Count => "a whole number from 1",
            Rule::Release => "a release version, major.minor such as 1.2",
            Rule::ReleaseOrNone => "a release version such as 1.2, or nothing",
            Rule::Design => "a design version, year.month.sequence such as 2026.11.2",
            Rule::DesignOrNone => "a design version such as 2026.11.2, or nothing",
            Rule::Application => "an application version such as 1.0.0",
            Rule::Date => "a date, YYYY-MM-DD",
            Rule::Digest => "a SHA-256, 64 hexadecimal digits",
        }
    }
}

/// A run of decimal digits, leading zeros and all, as in a date.
fn digits(s: &str) -> Option<u64> {
    (!s.is_empty() && s.bytes().all(|c| c.is_ascii_digit())).then(|| s.parse().ok())?
}

/// A run of decimal digits, without a leading zero unless it is 0 itself.
fn number(s: &str) -> Option<u64> {
    if s.is_empty() || !s.bytes().all(|c| c.is_ascii_digit()) || (s.len() > 1 && s.starts_with('0'))
    {
        return None;
    }
    s.parse().ok()
}

/// A group release's version (section 13): major when anything another group
/// reads changes, minor otherwise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReleaseVersion {
    pub major: u64,
    pub minor: u64,
}

impl ReleaseVersion {
    pub fn parse(text: &str) -> Option<ReleaseVersion> {
        let (a, b) = text.split_once('.')?;
        let v = ReleaseVersion {
            major: number(a)?,
            minor: number(b)?,
        };
        (v.major >= 1 || v.minor >= 1).then_some(v)
    }

    /// The release after this one: a new major when what another group reads
    /// has changed, a new minor when it has not.
    pub fn next(self, others_read_changed: bool) -> ReleaseVersion {
        if others_read_changed {
            ReleaseVersion {
                major: self.major + 1,
                minor: 0,
            }
        } else {
            ReleaseVersion {
                major: self.major,
                minor: self.minor + 1,
            }
        }
    }
}

impl fmt::Display for ReleaseVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// A released design's version (section 13): year.month.sequence, one per
/// release by the system engineer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DesignVersion {
    pub year: u64,
    pub month: u64,
    pub sequence: u64,
}

impl DesignVersion {
    pub fn parse(text: &str) -> Option<DesignVersion> {
        let mut p = text.split('.');
        let v = DesignVersion {
            year: number(p.next()?)?,
            month: number(p.next()?)?,
            sequence: number(p.next()?)?,
        };
        (p.next().is_none()
            && (2000..=9999).contains(&v.year)
            && (1..=12).contains(&v.month)
            && v.sequence >= 1)
            .then_some(v)
    }

    /// The design released after this one in `year` and `month`: the next
    /// sequence in the same month, or the first of a new one.
    pub fn next(self, year: u64, month: u64) -> DesignVersion {
        if (year, month) == (self.year, self.month) {
            DesignVersion {
                sequence: self.sequence + 1,
                ..self
            }
        } else {
            DesignVersion {
                year,
                month,
                sequence: 1,
            }
        }
    }
}

impl fmt::Display for DesignVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.year, self.month, self.sequence)
    }
}

impl File {
    /// A new, empty file of `kind`, written by `app`.
    pub fn new(kind: Kind, app: &str) -> File {
        let mut f = File::default();
        f.meta.insert("file_kind".into(), kind.name().into());
        f.meta.insert("format".into(), FORMAT.to_string());
        f.meta.insert("written_by_app".into(), app.into());
        f
    }

    /// The kind this file says it is, refused if it says none or one there is
    /// not.
    pub fn kind(&self) -> Result<Kind, Error> {
        let name = self.meta.get("file_kind").map(String::as_str).unwrap_or("");
        Kind::from_name(name).ok_or_else(|| {
            Error::new(
                ErrorKind::Malformed,
                if name.is_empty() {
                    "the file does not say what kind of file it is".to_string()
                } else {
                    format!("the file says it is a {name:?}, which is not a kind of file there is")
                },
            )
        })
    }

    /// Everything the meta must say for this kind, each by its rule.
    pub fn check_meta(&self) -> Result<Kind, Error> {
        let kind = self.kind()?;
        let mut wrong = Vec::new();
        if self.meta.get("format").map(String::as_str) != Some(&FORMAT.to_string()) {
            wrong.push(format!("format is not {FORMAT}"));
        }
        if !Rule::Name.holds(self.meta.get("written_by_app").map_or("", String::as_str)) {
            wrong.push("written_by_app does not name the application that wrote it".into());
        }
        for (key, rule) in kind.required() {
            match self.meta.get(*key) {
                None => wrong.push(format!("{key} is missing — it should be {}", rule.says())),
                Some(v) if !rule.holds(v) => {
                    wrong.push(format!("{key} is {v:?}, which is not {}", rule.says()))
                }
                Some(_) => {}
            }
        }
        if wrong.is_empty() {
            Ok(kind)
        } else {
            Err(Error::new(
                ErrorKind::Malformed,
                format!(
                    "this {kind} does not say what it must: {}",
                    wrong.join("; ")
                ),
            ))
        }
    }
}

/// `file`, refused unless it is a `kind` whose meta says what it must. This is
/// how the application opens a file: a group release opened as a released
/// design is refused, by name, and so is the reverse.
pub fn open_as(file: File, kind: Kind) -> Result<File, Error> {
    let is = file.check_meta()?;
    if is != kind {
        return Err(Error::new(
            ErrorKind::WrongKind,
            format!("this is a {is}, not a {kind}"),
        ));
    }
    Ok(file)
}
