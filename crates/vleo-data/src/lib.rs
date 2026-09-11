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
