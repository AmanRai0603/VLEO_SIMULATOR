//! Reference data — a package manager, not a query interface.
//!
//! # Why there is no database on the physics path
//!
//! Treating reference and operational data as one problem is what makes the
//! data question look hard. Separated, each has an obvious answer:
//!
//! | | reference data | operational data |
//! |---|---|---|
//! | is | solar drivers, coefficients, fitted biases, validated cases | identity, entitlement, the run ledger |
//! | changes | slowly, by publishing a new version | constantly |
//! | needed | by every single evaluation | occasionally, never during one |
//! | lives in | immutable versioned bundles, synced to a local store | a database behind an interface |
//! | if the network is down | everything still works on what is already local | history pauses; nothing else |
//!
//! Only one of the two is on the physics path, and it never crosses the
//! network at run time. The consequence worth stating once: a campaign of a
//! million cases makes zero network calls, and a run made in an air-gapped
//! facility is the same run made anywhere else.
//!
//! # The mechanism
//!
//! Named, versioned, immutable bundles; a manifest with a content hash; a
//! lockfile recording what is installed; and one command that reconciles the
//! two. The same pattern every language ecosystem converged on, and the same
//! one Orekit already uses for exactly this class of data.
//!
//! **Synchronisation and evaluation never overlap.** A run either has verified
//! data on disk or refuses to start. It does not fetch, wait, retry, or fall
//! back silently.
//!
//! # The format, and what it is not
//!
//! Bundles are a manifest plus comma-separated values, content-addressed with
//! the same FNV-1a the kernel uses for its chain hash. The intended production
//! format is columnar — Parquet, read in process, with no database server to
//! install, run, back up or version. It is deliberately not built yet: the
//! trigger is the first bundle that does not fit comfortably in memory as
//! text, and until then a format anybody can read in a text editor is worth
//! more than one that needs a library to inspect.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use vleo_core::hash::Hasher;

/// What a bundle says about itself.
#[derive(Clone, Debug, Default)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    /// FNV-1a over every payload file, in sorted order. A tampered coefficient
    /// file silently changing every result is the one failure the credibility
    /// system cannot detect on its own.
    pub content_hash: String,
    /// Where the data came from, as a source identifier. A manifest without
    /// provenance fails publication.
    pub provenance: String,
    /// The date beyond which the licence does not permit use. Read locally, so
    /// expiry works with no network — the only honest way to time-limit data
    /// without demanding a connection.
    pub licence_until: String,
    /// The age beyond which the input-pedigree credibility factor drops.
    pub stale_after_days: u32,
    pub files: Vec<String>,
    pub note: String,
}

/// A bundle as it sits in the store.
#[derive(Clone, Debug)]
pub struct Bundle {
    pub manifest: Manifest,
    pub dir: PathBuf,
    pub verified: bool,
    /// Why it did not verify, when it did not.
    pub refusal: Option<String>,
}

/// The local store: a folder of verified files, read directly. There is no
/// service to run.
#[derive(Debug, Default)]
pub struct Store {
    pub root: PathBuf,
    pub bundles: BTreeMap<String, Bundle>,
    pub lock: BTreeMap<String, (String, String)>,
}

/// Where bundles come from. One code path; only the source differs, which is
/// the whole point — internal use is not a special case with its own path, so
/// nothing has to be re-engineered when it scales outward.
#[derive(Clone, Debug)]
pub enum Source {
    /// Shipped inside the installer — coefficients and climatology. A fresh
    /// install runs before any sync.
    Shipped(PathBuf),
    /// A folder, or a signed bundle set on a drive. The air-gapped
    /// configuration, and a supported one rather than a special build.
    File(PathBuf),
}

impl Store {
    /// Open the store. Never creates network state and never fetches.
    pub fn open(root: &Path) -> Store {
        Store {
            root: root.to_path_buf(),
            ..Default::default()
        }
    }

    /// Reconcile the store against a source and write the lockfile.
    ///
    /// Idempotent by construction: syncing twice changes nothing the second
    /// time, because a published version is never modified and a correction is
    /// a new version.
    pub fn sync(&mut self, source: &Source) -> Result<usize, String> {
        let from = match source {
            Source::Shipped(p) | Source::File(p) => p.clone(),
        };
        if !from.is_dir() {
            return Err(format!("{} is not a directory", from.display()));
        }
        let mut n = 0usize;
        let mut names: Vec<PathBuf> = fs::read_dir(&from)
            .map_err(|e| format!("{}: {e}", from.display()))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        names.sort();
        for bundle_dir in names {
            let mut versions: Vec<PathBuf> = fs::read_dir(&bundle_dir)
                .map_err(|e| format!("{}: {e}", bundle_dir.display()))?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect();
            versions.sort();
            for v in versions {
                let b = load_bundle(&v)?;
                if !b.verified {
                    return Err(format!(
                        "{}@{} failed verification: {}. A result computed from unverifiable data is not a degraded result, it is not a result.",
                        b.manifest.name,
                        b.manifest.version,
                        b.refusal.clone().unwrap_or_default()
                    ));
                }
                // Copy it into the store. Without this, sync verified the
                // source, wrote a lockfile and reported success over a store
                // that stayed empty — so `data sync` and `data list`
                // contradicted each other, and a run that needed a bundle
                // found nothing where the lockfile said something was.
                //
                // Into name/version, so two versions of one bundle coexist and
                // a published version is never overwritten.
                let dest = self.root.join(&b.manifest.name).join(&b.manifest.version);
                copy_bundle(&v, &dest, &b.manifest)?;

                // Re-read from the store and verify there, not at the source.
                // A copy that lost a byte is exactly the failure this whole
                // mechanism exists to catch, and checking the original again
                // would not catch it.
                let installed = load_bundle(&dest)?;
                if !installed.verified {
                    return Err(format!(
                        "{}@{} verified at the source and not after the copy: {}. \
                         Something changed the bytes in between.",
                        b.manifest.name,
                        b.manifest.version,
                        installed.refusal.clone().unwrap_or_default()
                    ));
                }

                self.lock.insert(
                    installed.manifest.name.clone(),
                    (
                        installed.manifest.version.clone(),
                        installed.manifest.content_hash.clone(),
                    ),
                );
                self.bundles
                    .insert(installed.manifest.name.clone(), installed);
                n += 1;
            }
        }
        self.write_lock()?;
        Ok(n)
    }

    /// Load whatever is already on disk, verifying every hash before use.
    pub fn load(&mut self) -> Result<usize, String> {
        if !self.root.is_dir() {
            return Ok(0);
        }
        let mut n = 0;
        let mut dirs: Vec<PathBuf> = fs::read_dir(&self.root)
            .map_err(|e| format!("{}: {e}", self.root.display()))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();
        for d in dirs {
            let mut versions: Vec<PathBuf> = fs::read_dir(&d)
                .map_err(|e| format!("{}: {e}", d.display()))?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect();
            versions.sort();
            if let Some(v) = versions.last() {
                let b = load_bundle(v)?;
                self.bundles.insert(b.manifest.name.clone(), b);
                n += 1;
            }
        }
        Ok(n)
    }

    /// Which bundles are present and verified — the resolved handle the kernel
    /// is given. The kernel never opens a path or a socket.
    pub fn verified_names(&self) -> Vec<String> {
        self.bundles
            .values()
            .filter(|b| b.verified)
            .map(|b| b.manifest.name.clone())
            .collect()
    }

    /// The exact versions and hashes, for the run manifest. Two machines with
    /// the same lockfile produce the same numbers; without it, "one kernel
    /// everywhere" is true and "the same numbers everywhere" still is not.
    pub fn versions(&self) -> Vec<String> {
        self.bundles
            .values()
            .map(|b| {
                format!(
                    "{}@{}#{}",
                    b.manifest.name, b.manifest.version, b.manifest.content_hash
                )
            })
            .collect()
    }

    fn write_lock(&self) -> Result<(), String> {
        let mut o = String::from(
            "# vleo.lock — exactly which versions are installed, with their hashes.\n\
             # Committed alongside a study. Two machines with the same lockfile\n\
             # produce the same numbers.\n\n",
        );
        for (name, (version, hash)) in &self.lock {
            o.push_str(&format!(
                "[[bundle]]\nname = \"{name}\"\nversion = \"{version}\"\nhash = \"{hash}\"\n\n"
            ));
        }
        fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        fs::write(self.root.join("vleo.lock"), o).map_err(|e| e.to_string())
    }
}

/// `YYYY-MM-DD`, checked for shape rather than trusted.
///
/// No calendar arithmetic: this says the field is a date, not that the date
/// exists. That is enough to stop `licence_until = "soon"`, which is the
/// failure worth stopping — it parses as TOML and never expires.
fn is_iso_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| matches!(i, 4 | 7) || c.is_ascii_digit())
}

/// Read one bundle and verify it before it is usable.
pub fn load_bundle(dir: &Path) -> Result<Bundle, String> {
    let mp = dir.join("manifest.toml");
    let text = fs::read_to_string(&mp).map_err(|e| format!("{}: {e}", mp.display()))?;
    let v: toml::Value = text.parse().map_err(|e| format!("{}: {e}", mp.display()))?;
    let g = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    let mut m = Manifest {
        name: g("name"),
        version: g("version"),
        content_hash: g("content_hash"),
        provenance: g("provenance"),
        licence_until: g("licence_until"),
        stale_after_days: v
            .get("stale_after_days")
            .and_then(|x| x.as_integer())
            .unwrap_or(0) as u32,
        note: g("note"),
        files: Vec::new(),
    };
    for f in v.get("files").and_then(|f| f.as_array()).unwrap_or(&vec![]) {
        m.files.push(f.as_str().unwrap_or("").to_string());
    }
    // Publishing without these is impossible rather than forbidden. A rule that
    // says "always fill in the licence" is a rule somebody skips on the day
    // they are in a hurry, and the bundle that results looks exactly like a
    // good one. Refusing to load it is the only version of the rule that holds.
    for (field, value) in [
        ("name", m.name.as_str()),
        ("version", m.version.as_str()),
        ("provenance", m.provenance.as_str()),
        ("licence_until", m.licence_until.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!(
                "{}: a manifest with no {field} fails publication",
                mp.display()
            ));
        }
    }
    if m.files.is_empty() {
        return Err(format!(
            "{}: a manifest that lists no files has nothing to hash, so its \
             content hash would mean nothing",
            mp.display()
        ));
    }
    // A date, so that expiry can be read locally with no network. Checked for
    // shape here rather than trusted: `licence_until = "soon"` parses as TOML
    // and would silently never expire.
    if !is_iso_date(&m.licence_until) {
        return Err(format!(
            "{}: licence_until is {:?}, which is not a YYYY-MM-DD date — an \
             unparseable expiry is an expiry that never arrives",
            mp.display(),
            m.licence_until
        ));
    }
    if m.stale_after_days == 0 {
        return Err(format!(
            "{}: stale_after_days is missing or zero. It decides when the \
             input-pedigree factor drops, and a missing one defaults to a \
             number nobody chose",
            mp.display()
        ));
    }
    let computed = hash_files(dir, &m.files)?;
    let verified = computed == m.content_hash;
    let refusal = if verified {
        None
    } else {
        Some(format!(
            "content hash is {computed}, the manifest claims {}",
            m.content_hash
        ))
    };
    Ok(Bundle {
        manifest: m,
        dir: dir.to_path_buf(),
        verified,
        refusal,
    })
}

/// Put a verified bundle in the store: the manifest and every file it lists.
///
/// Only what the manifest names. A file sitting in the source directory that no
/// manifest lists is not part of the bundle — it is not hashed, so copying it
/// would install something nothing verified.
fn copy_bundle(from: &Path, to: &Path, m: &Manifest) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    fs::copy(from.join("manifest.toml"), to.join("manifest.toml"))
        .map_err(|e| format!("{}: {e}", to.join("manifest.toml").display()))?;
    for f in &m.files {
        let dst = to.join(f);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        fs::copy(from.join(f), &dst).map_err(|e| format!("{}: {e}", dst.display()))?;
    }
    Ok(())
}

/// FNV-1a over every payload file, in the order the manifest lists them.
pub fn hash_files(dir: &Path, files: &[String]) -> Result<String, String> {
    let mut h = Hasher::new();
    let mut sorted: Vec<&String> = files.iter().collect();
    sorted.sort();
    for f in sorted {
        let p = dir.join(f);
        let bytes = fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        h.write_str(f);
        h.write_bytes(&bytes);
    }
    Ok(format!("{:016x}", h.finish()))
}

/// One row of a driver table.
#[derive(Clone, Copy, Debug)]
pub struct DriverRow {
    /// Days since 2000-01-01, so the table needs no calendar library and no
    /// clock — the kernel has neither.
    pub day: i32,
    pub f107: f64,
    pub f107a: f64,
    pub ap: f64,
}

/// Read the solar driver table out of a bundle.
pub fn read_drivers(b: &Bundle) -> Result<Vec<DriverRow>, String> {
    if !b.verified {
        return Err(format!(
            "{} is present but does not verify — refusing to read it",
            b.manifest.name
        ));
    }
    let p = b.dir.join("drivers.csv");
    let text = fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if f.len() < 4 {
            return Err(format!("{}:{}: expected four columns", p.display(), n + 1));
        }
        out.push(DriverRow {
            day: f[0]
                .parse()
                .map_err(|_| format!("{}:{}: bad day", p.display(), n + 1))?,
            f107: f[1]
                .parse()
                .map_err(|_| format!("{}:{}: bad F10.7", p.display(), n + 1))?,
            f107a: f[2]
                .parse()
                .map_err(|_| format!("{}:{}: bad F10.7A", p.display(), n + 1))?,
            ap: f[3]
                .parse()
                .map_err(|_| format!("{}:{}: bad Ap", p.display(), n + 1))?,
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// The solar-weather record
// ---------------------------------------------------------------------------

/// One day of the observed record.
///
/// The eight three-hourly Kp are kept as the eight they are rather than
/// averaged on the way in. A daily mean and a daily peak are different
/// questions — the peak is what a storm looks like and the mean is what a
/// daily Ap can support — and a reader that collapsed them here would decide
/// that for every caller.
#[derive(Clone, Copy, Debug)]
pub struct SolarDay {
    /// Days since 2000-01-01. The same epoch `DriverRow` uses, so the two
    /// tables join without a calendar library and without a clock.
    pub day: i32,
    pub f107: Option<f64>,
    pub ap: Option<f64>,
    /// The eight three-hourly Kp, 00-03z first. `None` where the record has a
    /// gap, which is not the same as zero and must not become it.
    pub kp: [Option<f64>; 8],
    pub kp_max: Option<f64>,
}

/// Days since 2000-01-01 from `YYYY-MM-DD`.
///
/// Public because a caller with a date needs the same epoch the tables use,
/// and two implementations of one calendar is one implementation too many.
///
/// Written out rather than taken from a date crate: this is the only calendar
/// arithmetic the store does, a dependency for it would be the largest thing in
/// the crate, and the algorithm is a published one (Howard Hinnant's
/// `days_from_civil`), not something invented here.
pub fn days_since_2000(s: &str) -> Option<i32> {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let n = |a: usize, z: usize| -> Option<i64> { s[a..z].parse::<i64>().ok() };
    let (y, m, d) = (n(0, 4)?, n(5, 7)?, n(8, 10)?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    // 719468 puts the epoch at 1970-01-01; 10957 shifts it to 2000-01-01.
    i32::try_from(era * 146_097 + doe - 719_468 - 10_957).ok()
}

fn cell(v: &str) -> Option<f64> {
    let v = v.trim();
    if v.is_empty() {
        None
    } else {
        v.parse().ok()
    }
}

/// Read the daily observed record out of a `solar-weather` bundle.
///
/// Refuses an unverified bundle for the same reason `read_drivers` does: a
/// number whose bytes were not checked is not evidence, and reading it anyway
/// is how an unverified file ends up under a published result.
pub fn read_solar_days(b: &Bundle) -> Result<Vec<SolarDay>, String> {
    if !b.verified {
        return Err(format!(
            "{} is present but does not verify — refusing to read it",
            b.manifest.name
        ));
    }
    let p = b.dir.join("observed_daily.csv");
    let text = fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;

    let mut cols: Option<Vec<String>> = None;
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split(',').map(str::trim).collect();
        // The header names the columns, and every read below goes through it.
        // Reading by position would survive a column being inserted and return
        // the wrong quantity, silently, which is the failure this file format
        // exists to avoid.
        let Some(h) = &cols else {
            cols = Some(f.iter().map(|s| s.to_string()).collect());
            continue;
        };
        let at = |name: &str| -> Option<&str> {
            h.iter()
                .position(|c| c == name)
                .and_then(|i| f.get(i).copied())
        };
        let date = at("date").ok_or_else(|| format!("{}: no 'date' column", p.display()))?;
        let day = days_since_2000(date)
            .ok_or_else(|| format!("{}:{}: '{date}' is not a date", p.display(), n + 1))?;
        let mut kp = [None; 8];
        for (i, slot) in kp.iter_mut().enumerate() {
            *slot = at(&format!("kp_{:02}z", i * 3)).and_then(cell);
        }
        out.push(SolarDay {
            day,
            f107: at("f107").and_then(cell),
            ap: at("ap_planetary").and_then(cell),
            kp,
            kp_max: at("kp_max").and_then(cell),
        });
    }
    if out.is_empty() {
        return Err(format!("{}: no rows", p.display()));
    }
    // The record is a time series and everything downstream will bisect it. A
    // file that arrived out of order would make every lookup silently wrong, so
    // it is checked once here rather than assumed at every call site.
    if out.windows(2).any(|w| w[1].day <= w[0].day) {
        return Err(format!(
            "{}: rows are not in strictly increasing date order",
            p.display()
        ));
    }
    Ok(out)
}

/// The row for one day, or `None` if the record does not cover it.
///
/// Binary search rather than a scan: the caller is a design window asking for
/// three hundred and sixty-five consecutive days, and a scan per day is a scan
/// of ten thousand rows three hundred and sixty-five times.
pub fn solar_day(rows: &[SolarDay], day: i32) -> Option<&SolarDay> {
    rows.binary_search_by_key(&day, |r| r.day)
        .ok()
        .map(|i| &rows[i])
}

/// Every row in `[from, to]`, inclusive, as a slice of the record.
///
/// A slice, not a copy: a mission window is a contiguous run of the table and
/// there is no reason for the caller to own a second copy of it.
///
/// **The record has a hole in it.** 2017-01-01 to 2017-09-30 is absent — 273
/// days, and the source's own metadata calls the record "gap-free". A window
/// overlapping that range comes back shorter than the days asked for, and
/// nothing here treats that as an error, because a shorter window is the
/// truthful answer. A caller sizing a design against it wants
/// [`days_missing`] first.
pub fn solar_window(rows: &[SolarDay], from: i32, to: i32) -> &[SolarDay] {
    if from > to {
        return &[];
    }
    let a = rows.partition_point(|r| r.day < from);
    let b = rows.partition_point(|r| r.day <= to);
    &rows[a..b]
}

/// How many days of `[from, to]` the record does not have.
///
/// Zero means the window is complete. Anything else is the number of days a
/// caller would be averaging over without knowing it — which for the 2017 hole
/// is nine months, and is the difference between a design window and most of a
/// design window.
pub fn days_missing(rows: &[SolarDay], from: i32, to: i32) -> i32 {
    if from > to {
        return 0;
    }
    let asked = to - from + 1;
    asked - solar_window(rows, from, to).len() as i32
}
