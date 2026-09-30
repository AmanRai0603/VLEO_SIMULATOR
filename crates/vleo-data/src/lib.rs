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

pub mod crash;
mod whole;
pub use whole::write_whole;

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
        write_whole(&self.root.join("vleo.lock"), o).map_err(|e| e.to_string())
    }
}

/// Where the application keeps the saved case: the inputs a person set.
///
/// Beside the reference data and outside the repository, so changing inputs
/// never changes the tree: `~/.vleo/case/inputs.csv`, or wherever `VLEO_CASE`
/// points. One place, read by the daemon and the command line alike, so the
/// browser and the terminal always run the same case.
pub fn case_path() -> PathBuf {
    case_path_from(|k| std::env::var_os(k))
}

/// [`case_path`], reading the environment through `var`.
pub fn case_path_from(var: impl Fn(&str) -> Option<std::ffi::OsString>) -> PathBuf {
    set_to_something(&var, "VLEO_CASE")
        .unwrap_or_else(|| in_vleo_home(&var, &["case", "inputs.csv"]))
}

/// Where the application keeps saved results: what runs returned, with the
/// inputs they ran on. Outside the repository like the case —
/// `~/.vleo/results/`, or wherever `VLEO_RESULTS` points. Point it at a shared
/// or synced folder and the team keeps one archive (docs/ARCHITECTURE.html,
/// "Who reaches what"): names are unique and every write is whole.
pub fn results_path() -> PathBuf {
    results_path_from(|k| std::env::var_os(k))
}

/// [`results_path`], reading the environment through `var`.
pub fn results_path_from(var: impl Fn(&str) -> Option<std::ffi::OsString>) -> PathBuf {
    set_to_something(&var, "VLEO_RESULTS").unwrap_or_else(|| in_vleo_home(&var, &["results"]))
}

/// How many days a result keeps all its values before it is thinned to its
/// summary: `VLEO_KEEP_DAYS`, 30 when unset or empty. `0` means never thin —
/// `None`. A value that does not read as a whole number of days is refused
/// with why, rather than read as some other number.
///
/// A FOLDER NAMED BY `VLEO_RESULTS` IS KEPT WHOLE unless `VLEO_KEEP_DAYS` is
/// set too. That folder is usually the team's shared archive, and thinning it
/// is a decision for the team, not for whichever laptop happens to start first.
pub fn keep_days() -> Result<Option<u32>, String> {
    keep_days_from(|k| std::env::var_os(k))
}

/// [`keep_days`], reading the environment through `var`.
pub fn keep_days_from(
    var: impl Fn(&str) -> Option<std::ffi::OsString>,
) -> Result<Option<u32>, String> {
    let Some(v) = var("VLEO_KEEP_DAYS").filter(|v| !v.is_empty()) else {
        let shared = set_to_something(&var, "VLEO_RESULTS").is_some();
        return Ok((!shared).then_some(30));
    };
    let v = v.to_string_lossy();
    match v.trim().parse::<u32>() {
        Ok(0) => Ok(None),
        Ok(n) => Ok(Some(n)),
        Err(_) => Err(format!(
            "VLEO_KEEP_DAYS={v} is not a whole number of days (0 keeps every result whole)"
        )),
    }
}

/// Where crash logs go: `~/.vleo/log/`, or wherever `VLEO_LOG` points.
pub fn log_path() -> PathBuf {
    log_path_from(|k| std::env::var_os(k))
}

/// [`log_path`], reading the environment through `var`.
pub fn log_path_from(var: impl Fn(&str) -> Option<std::ffi::OsString>) -> PathBuf {
    set_to_something(&var, "VLEO_LOG").unwrap_or_else(|| in_vleo_home(&var, &["log"]))
}

/// A variable's value as a path, when it is set to something.
///
/// SET BUT EMPTY COUNTS AS UNSET. `VLEO_RESULTS=` left in a shell profile read
/// as the empty path, which is the folder the tool happened to start in — so
/// results were saved there, and the next start from somewhere else could not
/// find them. `VLEO_DATA` already worked this way; now every path does.
fn set_to_something(
    var: &impl Fn(&str) -> Option<std::ffi::OsString>,
    name: &str,
) -> Option<PathBuf> {
    var(name).filter(|v| !v.is_empty()).map(PathBuf::from)
}

/// `~/.vleo/<parts…>`, or `.vleo/<parts…>` beside the program when there is no
/// home folder at all.
fn in_vleo_home(var: &impl Fn(&str) -> Option<std::ffi::OsString>, parts: &[&str]) -> PathBuf {
    let base = home_from(var)
        .map(|h| h.join(".vleo"))
        .unwrap_or_else(|| PathBuf::from(".vleo"));
    parts.iter().fold(base, |p, s| p.join(s))
}

/// The person's home folder: `HOME`, or `USERPROFILE` where there is no `HOME`.
///
/// Windows sets `USERPROFILE` and usually not `HOME`. Reading `HOME` alone put
/// a Windows teammate's case and results beside the program, in the folder the
/// next kit replaces — so the upgrade that was meant to carry them over deleted
/// them. An empty value counts as unset.
pub fn home() -> Option<PathBuf> {
    home_from(|k| std::env::var_os(k))
}

/// [`home`], reading the environment through `var` so it can be tested without
/// changing the process's own.
pub fn home_from(var: impl Fn(&str) -> Option<std::ffi::OsString>) -> Option<PathBuf> {
    ["HOME", "USERPROFILE"]
        .into_iter()
        .filter_map(var)
        .find(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// Where the reference-data store lives: `~/.vleo/data`, or wherever
/// `VLEO_DATA` points. Outside the install directory, so it survives an
/// upgrade rather than being deleted with the application.
pub fn data_path() -> Option<PathBuf> {
    let var = |k: &str| std::env::var_os(k);
    set_to_something(&var, "VLEO_DATA").or_else(|| home().map(|h| h.join(".vleo").join("data")))
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
    /// The day's sunspot number (SESC), which the cycle figures stack.
    pub ssn: Option<f64>,
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
    i32::try_from(days_from_civil(y, m, d)).ok()
}

/// Hinnant's `days_from_civil`, from 2000-01-01.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    // 719468 puts the epoch at 1970-01-01; 10957 shifts it to 2000-01-01.
    era * 146_097 + doe - 719_468 - 10_957
}

/// The calendar date of a day since 2000-01-01: `(year, month, day)`, and its
/// day of the year from 1.
///
/// The other way round from [`days_since_2000`], by the same published
/// algorithm (Hinnant's `civil_from_days`), so a date read in and written back
/// out is the date it was. A figure that groups the record by year, month or
/// day of year asks here rather than keeping a calendar of its own.
pub fn civil_from_days(day: i32) -> (i32, u32, u32, u32) {
    let z = day as i64 + 10_957 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    let of_year = day as i64 - days_from_civil(y, 1, 1) + 1;
    (y as i32, m as u32, d as u32, of_year as u32)
}

fn cell(v: &str) -> Option<f64> {
    let v = v.trim();
    if v.is_empty() {
        None
    } else {
        v.parse().ok()
    }
}

/// One month of `monthly_means.csv`: the mean of each driver over the
/// month, and the 13-month smoothed value the cycles are counted on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonthlyMean {
    /// Days since 2000-01-01 of the date the month is filed under.
    pub day: i32,
    pub f107_mean: Option<f64>,
    pub ap_mean: Option<f64>,
    pub ssn_mean: Option<f64>,
    /// None at the ends, where the smoother would need months the record does
    /// not hold — left empty rather than extrapolated.
    pub f107_smooth: Option<f64>,
    pub ap_smooth: Option<f64>,
    pub ssn_smooth: Option<f64>,
}

/// The months `monthly_means.csv` holds, in its order, read by the column
/// names in its header. A value written as `NaN` is no value.
pub fn read_monthly_means(b: &Bundle) -> Result<Vec<MonthlyMean>, String> {
    if !b.verified {
        return Err(format!(
            "{} is present but does not verify — refusing to read it",
            b.manifest.name
        ));
    }
    let p = b.dir.join("monthly_means.csv");
    let text = fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    let mut cols: Option<Vec<String>> = None;
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split(',').map(str::trim).collect();
        let Some(h) = &cols else {
            cols = Some(f.iter().map(|s| s.to_string()).collect());
            continue;
        };
        let at = |name: &str| -> Option<f64> {
            h.iter()
                .position(|c| c == name)
                .and_then(|i| f.get(i).copied())
                .and_then(cell)
                .filter(|v| !v.is_nan())
        };
        let month = h
            .iter()
            .position(|c| c == "month")
            .and_then(|i| f.get(i).copied())
            .ok_or_else(|| format!("{}: no 'month' column", p.display()))?;
        let day = days_since_2000(month)
            .ok_or_else(|| format!("{}:{}: '{month}' is not a date", p.display(), n + 1))?;
        out.push(MonthlyMean {
            day,
            f107_mean: at("f107_mean"),
            ap_mean: at("ap_mean"),
            ssn_mean: at("ssn_mean"),
            f107_smooth: at("f107_smooth"),
            ap_smooth: at("ap_smooth"),
            ssn_smooth: at("ssn_smooth"),
        });
    }
    if out.is_empty() {
        return Err(format!("{}: no months", p.display()));
    }
    Ok(out)
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
            ssn: at("ssn_sesc").and_then(cell),
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

/// One solar cycle as the record's own table bounds it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolarCycle {
    /// The cycle's number, 23 onward in this record.
    pub n: u32,
    /// Days since 2000-01-01 of the opening minimum. A day belongs to the
    /// cycle from here...
    pub start: i32,
    /// ...up to, and not including, this day. For the cycle still running it
    /// is where the RECORD stops, not where the cycle does.
    pub end: i32,
}

/// The cycles `solar_cycles.csv` lists, in its order.
///
/// Read by the column names in its header, like the daily record, and refused
/// whole on a row that does not say what a cycle is: a cycle guessed from a
/// broken row would put days in the wrong one and nothing downstream could
/// tell.
pub fn read_solar_cycles(b: &Bundle) -> Result<Vec<SolarCycle>, String> {
    if !b.verified {
        return Err(format!(
            "{} is present but does not verify — refusing to read it",
            b.manifest.name
        ));
    }
    let p = b.dir.join("solar_cycles.csv");
    let text = fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    let mut cols: Option<Vec<String>> = None;
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split(',').map(str::trim).collect();
        let Some(h) = &cols else {
            cols = Some(f.iter().map(|s| s.to_string()).collect());
            continue;
        };
        let at = |name: &str| -> Option<&str> {
            h.iter()
                .position(|c| c == name)
                .and_then(|i| f.get(i).copied())
        };
        let bad = |what: &str| format!("{}:{}: {what}", p.display(), n + 1);
        let cycle = at("cycle")
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| bad("no cycle number"))?;
        let date = |k: &str| {
            at(k)
                .and_then(days_since_2000)
                .ok_or_else(|| bad(&format!("no '{k}' date")))
        };
        out.push(SolarCycle {
            n: cycle,
            start: date("start")?,
            end: date("end")?,
        });
    }
    if out.is_empty() {
        return Err(format!("{}: no cycles", p.display()));
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
