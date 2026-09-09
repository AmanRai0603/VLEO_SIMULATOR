//! RING 2 — the bus.
//!
//! One struct per message: the wire contract. A face may depend on this crate
//! and never on a module, which is why adding a face cannot change a result.
//!
//! Publish and subscribe are checked at compile time rather than at run time.
//! The core Flight System proved the pattern with CCSDS packets and a runtime
//! subscribe; here the type is the contract, so a wiring error is caught by the
//! compiler rather than on the bench.
//!
//! `no_std` with `alloc`, so the identical message types reach the rig and a
//! flight target. A schema that only exists on the desktop is a schema that
//! will drift from the one that flies.
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use vleo_core::credibility::CredVec;
use vleo_core::fault::Fault;
use vleo_core::hash::Hasher;
use vleo_core::resolver::Mode;
use vleo_core::value::SlotStatus;

/// What a face sends in.
///
/// Inputs are immutable and evaluation is pure, so a case fully determines a
/// run. That is what makes staleness computable rather than remembered.
#[derive(Clone, Debug, Default)]
pub struct Case {
    /// The stored case this run started from — `nominal`, `solar_max`, and so on.
    pub base: String,
    /// Overrides, as `(node id, SI value)`. Always SI: everything crossing this
    /// boundary is in the canonical unit of its type, and a face converts for
    /// display and never for transport.
    pub supply: Vec<(String, f64)>,
    /// Which node, and how much of the graph around it.
    pub target: String,
    pub mode: RunMode,
    /// The bundles the face verified before the run started, by name.
    ///
    /// A resolved handle, not a path: `evaluate` reads the local store and
    /// nothing else, and it cannot reach a filesystem or a socket to find out.
    /// Synchronisation and evaluation are separate moments — a network failure
    /// stops the store getting fresher and stops nothing else.
    pub data: Vec<String>,
    /// The exact versions and hashes, carried into the run manifest so a
    /// result can be reproduced next year by syncing the same versions.
    pub data_versions: Vec<String>,
}

/// The three run modes, in the wire form.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RunMode {
    /// This node only. Disabled, and it names what is missing, if a dependency
    /// has never run.
    Alone,
    /// The dependency closure of one node. The mode that produces a defensible
    /// number.
    #[default]
    Branch,
    /// Everything buildable. Unbuilt nodes stay blocked and are listed —
    /// n ran, m blocked, and the blocked ones named, always.
    All,
}

impl RunMode {
    pub fn to_mode(self, target: u16) -> Mode {
        match self {
            RunMode::Alone => Mode::Alone(target),
            RunMode::Branch => Mode::Branch(target),
            RunMode::All => Mode::All,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            RunMode::Alone => "alone",
            RunMode::Branch => "branch",
            RunMode::All => "all",
        }
    }
    pub fn from_name(s: &str) -> RunMode {
        match s {
            "alone" => RunMode::Alone,
            "all" => RunMode::All,
            _ => RunMode::Branch,
        }
    }
}

impl Case {
    /// The case hash. Part of the run's identity, and never the whole of it —
    /// keying a cache on this alone is the defect this system exists to
    /// prevent, reappearing one level up.
    pub fn hash(&self) -> u64 {
        let mut h = Hasher::new();
        h.write_str(&self.base);
        h.write_str(&self.target);
        h.write_str(self.mode.name());
        let mut sorted: Vec<&(String, f64)> = self.supply.iter().collect();
        sorted.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        for (k, v) in sorted {
            h.write_str(k);
            h.write_f64(*v);
        }
        h.finish()
    }
}

/// One value coming back.
#[derive(Clone, Debug)]
pub struct ValueOut {
    pub id: String,
    pub symbol: String,
    pub label: String,
    /// Always SI. The face converts for display.
    pub value: f64,
    pub unit: &'static str,
    pub status: SlotStatus,
    pub cred: CredVec,
    pub governing: &'static str,
}

/// A node that could not run, and why. Never hidden: the answer is always
/// "n ran, m blocked", and the blocked ones are named.
#[derive(Clone, Debug)]
pub struct BlockedOut {
    pub id: String,
    pub kind: &'static str,
    pub message: String,
}

/// One fixture's verdict, executed on the run rather than looked up.
#[derive(Clone, Debug)]
pub struct VerdictOut {
    pub node: String,
    pub label: String,
    pub expected: f64,
    pub got: f64,
    pub relative_error: f64,
    pub tolerance: f64,
    pub passed: bool,
    pub provenance: &'static str,
    pub source: &'static str,
}

/// What a face gets back.
#[derive(Clone, Debug, Default)]
pub struct Results {
    pub values: Vec<ValueOut>,
    pub blocked: Vec<BlockedOut>,
    pub verdicts: Vec<VerdictOut>,
    pub manifest: RunManifest,
    /// Bulk output — a sweep or a field — as raw values. Never serialised as
    /// text: serialising a field costs more than computing it.
    pub series: Vec<Series>,
}

/// A swept output, for the design-space views.
#[derive(Clone, Debug, Default)]
pub struct Series {
    pub x_id: String,
    pub y_id: String,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    /// Points the sweep refused, with the reason. A campaign in which some rows
    /// quietly used a substituted value is a campaign whose conclusion is
    /// unknown, so refusals are reported rather than dropped.
    pub refused: Vec<(f64, String)>,
}

/// Every number carries its lineage. Without these it is not a result, it is a
/// rumour.
#[derive(Clone, Debug, Default)]
pub struct RunManifest {
    pub node: String,
    pub mode: &'static str,
    /// Identifies the engine build.
    pub kernel: String,
    /// Identifies the graph the engine was built against.
    pub graph: String,
    /// Identifies the case.
    pub case: String,
    /// Identifies the whole chain: every node reached, each one's
    /// implementation content and published outputs, and the case. This is the
    /// cache key, and it is why rewriting a node's arithmetic correctly
    /// invalidates every result downstream of it.
    pub chain: String,
    /// Which reference-data bundles answered, and at which version.
    pub data: Vec<String>,
    /// Which engine answered — the demonstration kernel, the local daemon, or a
    /// team server. Every result records it, so a divergence between two
    /// endpoints is a fault that raises a flag rather than an argument.
    pub endpoint: String,
    pub ran: usize,
    pub blocked_count: usize,
    pub iterations: u32,
    /// The credibility of the answer, and which factor is holding it down.
    pub cred: CredVec,
    pub governing: &'static str,
    /// True when the run can be reproduced from this manifest alone.
    pub reproducible: bool,
}

/// The wire form of a refusal.
pub fn fault_message(f: &Fault) -> String {
    use core::fmt::Write;
    let mut s = String::new();
    let _ = write!(&mut s, "{f}");
    s
}

/// Format a number to a stated number of significant figures, switching to
/// scientific notation where fixed notation would round the value away.
///
/// One implementation, in the layer every face already depends on, because a
/// number formatted two ways is a number two people will read differently. A
/// density of 6.6e-11 printed to six decimal places is `0.000000`, which is not
/// a rounding — it is a different claim.
pub fn format_significant(v: f64, digits: usize) -> String {
    use alloc::format;
    use vleo_units::pmath;
    if v == 0.0 {
        return String::from("0");
    }
    if !pmath::is_finite(v) || pmath::is_nan(v) {
        return String::from(if v > 0.0 { "+inf" } else { "not a number" });
    }
    let a = pmath::abs(v);
    if !(1.0e-3..1.0e6).contains(&a) {
        format!("{:.*e}", digits.saturating_sub(1), v)
    } else {
        let exp = pmath::floor(pmath::log10(a)) as i32;
        let decimals = (digits as i32 - 1 - exp).clamp(0, 12) as usize;
        format!("{:.*}", decimals, v)
    }
}

/// Convert a value out of SI into the unit a node declared, and format it.
pub fn present(value_si: f64, unit: vleo_units::Unit, digits: usize) -> (String, &'static str) {
    let (v, sym) = vleo_units::unit::present(value_si, unit);
    (format_significant(v, digits), sym)
}
