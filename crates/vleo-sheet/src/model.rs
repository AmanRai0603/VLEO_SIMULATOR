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
    /// Which layer this row lives in, inherited from its parent group and
    /// carried on the row so a face never has to walk the tree to find out.
    pub layer: u8,
    /// Where the row sits among its siblings, as whoever wrote them ordered
    /// them.
    ///
    /// Folders sort alphabetically, and a tree that reads alphabetically is a
    /// tree nobody wrote: it puts `Achievable lifetime` before `Specific
    /// impulse` and reverses the order the source put them in. The order is
    /// part of what the source said, so it is carried rather than recovered.
    pub order: u32,
    /// The node that crosses between this layer and the one above.
    ///
    /// Empty for an ordinary row. Exactly one node crosses between any two
    /// layers, and it is the only route in: a subsystem is reached through its
    /// interface node, never by reaching into it.
    pub crosses_to: String,
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
    /// Who supplied this relation, and when.
    ///
    /// Agent A may never supply mathematics, and that prohibition had nothing
    /// mechanical behind it: an invented formula that runs cleanly is the worst
    /// failure this system can have, and it looked exactly like a cited one.
    /// A name here is the authorship record the completion questions are
    /// supposed to produce — so an agent cannot supply a relation without
    /// forging a person's attribution, which is a different and much larger
    /// thing than filling a blank field.
    ///
    /// It does not make the formula right. It makes the formula *somebody's*,
    /// which is what H1b needs in order to be a review rather than a reading.
    pub relation_by: String,
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
    /// Criticality: `"minor"` or `"significant"`.
    ///
    /// Set by the engineer who wrote the sheet, and it decides two things: how
    /// many people read the node, and whether the hole is filled twice by
    /// different model families and the two compared numerically over the
    /// declared domain. Everything cannot be significant — a person asked to
    /// approve too many things stops evaluating each one — so the default is
    /// `"minor"` and raising it is a decision somebody makes on purpose.
    pub criticality: String,
    /// The prior implementation this node was translated from, if there is one.
    ///
    /// Eighteen months of working MATLAB exists and most rows with mathematics
    /// exist there in some form. Its outputs may never be fixtures: an
    /// implementation cannot supply its own expected values, and that is an
    /// implementation. So it is recorded here and its numbers go in
    /// `parity.csv` beside the node, where a disagreement is a finding about
    /// one of the two rather than a check either has passed. The check numbers
    /// still come from the paper or measurement the MATLAB was built from.
    pub migrated_from: String,
    /// How far the prior implementation may disagree before the grid is a
    /// finding.
    ///
    /// Defaults to 1e-4. Not a physics tolerance and not negotiable downward
    /// per row: a MATLAB grid is an export, printed to five or six significant
    /// figures, so a tighter default would fail on the printing rather than on
    /// the mathematics. Anything looser is hiding a disagreement, and the
    /// resolution for a disagreement is to find which of the two is wrong, not
    /// to widen this.
    pub parity_tolerance: f64,
    /// Which way a requirement binds: `"<="` or `">="`.
    ///
    /// A requirement row states a bound, and a bound is meaningless until it
    /// says which side of it is safe. "The design sustains Ap 200" and "the
    /// design needs Ap 200" are the same number and opposite requirements, and
    /// a closure computed without knowing which would report a comfortable
    /// margin for a spacecraft that is about to be destroyed.
    ///
    /// Left blank on every row that is not a requirement. The prior MATLAB
    /// already declared this per requirement and failed its build without it:
    /// "a requirement with no driver quantity, or with an undeclared adverse
    /// direction, FAILS THE BUILD. Defaulting either is how a silently wrong
    /// bound gets shipped."
    pub sense: String,
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
    /// Seeded, and nobody has specified it yet.
    ///
    /// The folder exists, the row is on the tree, the eight tabs open and each
    /// says what goes in it. Nothing is generated from it and it cannot run.
    /// This is the normal state of most of a tree for most of a programme, so
    /// it is a reported state rather than a failing one.
    pub fn is_seeded(&self) -> bool {
        self.state == "empty" || self.state.is_empty()
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
    /// Which of the four layers this heading belongs to.
    ///
    /// 1 management · 2 the system · 3 subsystem · 4 the run. Exactly one node
    /// crosses between any two layers; nothing else is shared, so no layer can
    /// be reasoned about wrongly from another.
    pub layer: u8,
    /// Where the heading sits among its siblings, as it was written.
    pub order: u32,
    /// Drawn as a nested box on the diagonal of the matrix.
    ///
    /// A mark inside a box is coupling that subtree owns; a mark outside it
    /// crosses a boundary, and that difference is the finding. A heading that
    /// is not a box is a label rather than a scope.
    pub is_box: bool,
    /// The colour family the branch is drawn in. Not styling: it is what makes
    /// a branch findable on a tree of thirteen hundred rows.
    pub tone: String,
    /// The cases this branch is in play for. Empty means every case.
    ///
    /// The architecture is never copied. `n` copies means `n` fixes and silent
    /// drift, so a case filters what is in play rather than forking the tree —
    /// which is the anti-clone-and-own rule, ISO/IEC 26580.
    pub cases: Vec<String>,
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
