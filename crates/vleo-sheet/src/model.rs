//! What a sheet is, as data.

use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct Assumption {
    pub text: String,
    /// The condition under which the assumption stops holding. Not optional:
    /// an assumption with no failure condition is a sentence, not an
    /// assumption, and the schema refuses it.
    pub fails_when: String,
}

/// One declared input.
#[derive(Clone, Debug, Default)]
pub struct Input {
    pub binding: String,
    pub var: String,
    pub ty: String,
}

#[derive(Clone, Debug, Default)]
pub struct Step {
    pub number: u32,
    pub text: String,
    /// The name the step's result is bound to.
    pub binds: String,
    /// Its quantity type. Both are needed by the scaffold, which is why an
    /// unfilled one blocks generation rather than producing a file that does
    /// not compile.
    pub ty: String,
}

#[derive(Clone, Debug, Default)]
pub struct Fixture {
    pub label: String,
    pub expect: f64,
    pub tolerance: f64,
    pub provenance: String,
    pub source: String,
    pub inputs: Vec<(String, f64)>,
}

#[derive(Clone, Debug, Default)]
pub enum View {
    /// A formatted number with its unit, verdict and provenance. The default,
    /// and what all but a handful of nodes want.
    #[default]
    Number,
    Line {
        over: String,
        points: u32,
    },
    Heatmap {
        over_x: String,
        over_y: String,
        points: u32,
    },
    Bar {
        y: String,
    },
}

#[derive(Clone, Debug, Default)]
pub struct Sheet {
    pub id: String,
    pub label: String,
    pub folder: String,
    pub subsystem: String,
    pub parent: String,
    pub kind: String,
    pub owner: String,
    pub tier: String,
    pub state: String,
    pub question: String,
    pub note: String,
    pub expression: String,
    pub source: String,
    pub assumptions: Vec<Assumption>,
    pub symbol: String,
    pub ty: String,
    pub unit: String,
    pub lower: f64,
    pub upper: f64,
    pub reason_lower: String,
    pub reason_upper: String,
    pub value: Option<f64>,
    pub confirmed_by: String,
    /// The derivation graph, declared by the consumer, because knowing its
    /// inputs is what changes *this* node's implementation. Each entry is the
    /// binding name, the variable it reads, and the quantity type the consumer
    /// expects — assembly refuses a type that disagrees with the producer.
    pub inputs: Vec<Input>,
    pub steps: Vec<Step>,
    pub kpis: Vec<String>,
    pub bundles: Vec<String>,
    pub view: View,
    pub fixtures: Vec<Fixture>,
    pub crate_name: String,
    pub dir: PathBuf,
    /// Hash of the sheet's semantic content. A page whose sheet hash differs
    /// from the engine's refuses to run and says so, which makes a stale face
    /// detectable rather than merely wrong.
    pub sheet_hash: u64,
    /// Hash of the filled hole bodies. Part of the chain hash, so a cached
    /// result notices when the arithmetic under it changed even though the
    /// interface did not.
    pub impl_hash: u64,
}

impl Sheet {
    pub fn is_declared(&self) -> bool {
        self.kind == "declared"
    }
    /// The module path, which is the node identifier.
    pub fn module_path(&self) -> String {
        format!("{}::{}", self.subsystem, self.folder)
    }
    pub fn rust_ident(&self) -> String {
        let mut s = self.folder.replace(['-', '.'], "_");
        if s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(true) {
            s.insert(0, 'n');
        }
        s
    }
}

/// One relation edge between two groups, declared in a layer file because
/// neither of its ends is a node.
#[derive(Clone, Debug, Default)]
pub struct Relation {
    pub from: String,
    pub to: String,
    pub why: String,
}

#[derive(Clone, Debug, Default)]
pub struct Group {
    pub id: String,
    pub label: String,
    pub parent: String,
    pub owner: String,
}

#[derive(Clone, Debug, Default)]
pub struct CycleSpec {
    pub nodes: Vec<String>,
    pub converge_on: String,
    pub tolerance: f64,
    pub max_iter: u32,
    pub seeds: Vec<(String, f64)>,
}

#[derive(Clone, Debug, Default)]
pub struct Case {
    pub id: String,
    pub label: String,
    pub note: String,
    pub supply: Vec<(String, f64)>,
    pub cycles: Vec<CycleSpec>,
}

#[derive(Clone, Debug, Default)]
pub struct Source {
    pub id: String,
    pub title: String,
    pub where_: String,
    pub status: String,
    pub used_for: String,
}
