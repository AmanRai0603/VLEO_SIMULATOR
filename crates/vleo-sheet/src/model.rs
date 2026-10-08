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

/// One line of the derivation, and the mathematics it arrives at.
///
/// The `expression` on a sheet is the relation as a machine needs it: one line,
/// no reason. That is enough to generate code from and not enough to review, and
/// a reviewer who cannot reconstruct why the line is that line has to take it on
/// trust — which is the failure the whole review structure exists to prevent.
/// These are the intermediate statements between the question and the
/// expression, in the order somebody would build them at a board.
#[derive(Clone, Debug, Default)]
pub struct TheoryStep {
    /// The sentence. Prose, because the reader being served here is the one who
    /// does not yet know the subject.
    pub text: String,
    /// The mathematics this sentence arrives at, if the sentence arrives at any.
    /// Optional on purpose: the first line of most derivations states what is
    /// being assumed about the world, and there is no formula for that.
    pub math: String,
}

/// Why the relation is the relation.
///
/// Documentation, not specification — and therefore deliberately outside the
/// sheet hash. Correcting a sentence here must not invalidate a generated
/// artefact or a cached result, because the moment it does, nobody corrects the
/// sentence.
///
/// It is separate from `note` and from the assumptions, which answer different
/// questions. `note` says how to read the answer's encoding. An assumption says
/// where the relation stops being true. This says where the relation came from,
/// which is the one thing a reader cannot recover from any other field.
#[derive(Clone, Debug, Default)]
pub struct Theory {
    /// The physical or statistical reason this node computes what it computes,
    /// before any formula appears.
    pub why: String,
    pub steps: Vec<TheoryStep>,
    /// What the answer means once it is in hand, and what it does not mean. The
    /// place to say that a number is descriptive rather than predictive, or that
    /// it is a bound rather than an expectation.
    pub reading: String,
}

impl Theory {
    /// Nothing written. Distinguished from "written and short" so a page can say
    /// which of the two it is looking at.
    pub fn is_empty(&self) -> bool {
        self.why.trim().is_empty() && self.steps.is_empty() && self.reading.trim().is_empty()
    }
}

/// The node said simply, and where saying it simply stops being true.
///
/// The station pattern of `docs/EXPLAINING.md`: every node is read first in
/// plain words, then as the real relation, then at the place the plain version
/// breaks. The relation and its theory are the second of those; these are the
/// first and the third, plus the wrong idea a reader most often brings.
///
/// Documentation, outside the sheet hash, for the same reason as [`Theory`].
#[derive(Clone, Debug, Default)]
pub struct Explain {
    /// What the row works out and why it matters, with no symbol and no word a
    /// newcomer would have to look up.
    pub simply: String,
    /// Where the plain version stops being true.
    pub breaks: String,
    /// The wrong idea a reader most often brings to this row, and what is true
    /// instead. Optional: not every row has a common misconception.
    pub wrong: String,
    /// Who wrote the plain words, as they would be named on the page. A draft
    /// says so — an assistant's wording is marked as one, for the owner to
    /// confirm — because words that look like the owner's and are not are the
    /// quietest way a page misleads (E15).
    pub by: String,
}

impl Explain {
    pub fn is_empty(&self) -> bool {
        self.simply.trim().is_empty()
            && self.breaks.trim().is_empty()
            && self.wrong.trim().is_empty()
    }
}

/// The relation once more, in the method language (`docs/PSEUDOCODE.md`).
///
/// A third statement of the node, beside the expression a reviewer reads and
/// the author's own code: small and strict enough that every construct has one
/// translation, so the Rust that ships can be generated from it and the
/// author's cases can check all three against each other.
///
/// IN the sheet hash when present: the generated code is translated from it,
/// so a different method is a different node. A sheet with none is hashed
/// exactly as before.
#[derive(Clone, Debug, Default)]
pub struct Method {
    pub text: String,
    /// Who wrote it, as intake stamps it. The same rule as the relation: an
    /// assistant may never be the one who supplied it.
    pub by: String,
    /// For a method copied from a relation that was already written — the
    /// code it replaced — where it was copied from. Empty for a method its
    /// author wrote.
    pub transcribed_from: String,
    /// The person who read the copy against what it was copied from and signs
    /// it. Empty until they do: the method runs, and is held to what it was
    /// copied from, but a release refuses it (AGENTS.md, rule 6).
    pub checked_by: String,
}

/// A row whose answer is a table (`[lookup]`): one input read along it to the
/// row's one output.
///
/// IN the sheet hash when present: the table is what the row computes. Both
/// columns in SI — the input's unit for `x`, the output's for `y` — as a
/// case's numbers are.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lookup {
    /// The binding of the input the table is read along.
    pub by: String,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    /// `"linear"`, or `"log"` for an answer that varies over decades.
    pub read: String,
}

/// A row whose answer is its children's (`[children]`): each output is one of
/// its inputs, a port of a child in `group`. Whatever else the row says — its
/// method, its relation — stays as its estimate.
///
/// IN the sheet hash when present.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChildrenOf {
    /// The group whose children answer.
    pub group: String,
    /// For each output, primary first, the binding of the input that answers
    /// it.
    pub from: Vec<String>,
}

/// The node author's own implementation — the code that produced their test
/// cases — and the script that ran it.
///
/// Evidence, outside the sheet hash. It is kept so the cases can be rerun and
/// so a reviewer can read what the numbers came from; it is never compiled
/// into the tool.
#[derive(Clone, Debug, Default)]
pub struct AuthorCode {
    /// Who wrote the code, as they sign it.
    pub name: String,
    /// MATLAB, Octave, Python, C and so on — see `form::LANGUAGES`.
    pub language: String,
    /// Which function in `code` is this node.
    pub entry: String,
    pub code: String,
    /// The script that ran `code` on each case and printed the results.
    pub test_code: String,
    /// Where and how it was run: the tool and version, the machine, the date.
    pub how_run: String,
}

impl AuthorCode {
    pub fn is_empty(&self) -> bool {
        self.code.trim().is_empty() && self.test_code.trim().is_empty()
    }
}

/// Flight software that belongs to this node, kept with its test.
///
/// Stored, not built: the tool keeps the code and its test beside the relation
/// they implement so the two are reviewed and versioned together. Running them
/// on the flight computer, or in the loop with hardware, is a later decision.
#[derive(Clone, Debug, Default)]
pub struct Flight {
    /// The file name as it lives in the flight tree, `attitude_ctl.c`.
    pub name: String,
    pub language: String,
    /// What it does on board, and which part of this node it implements.
    pub purpose: String,
    pub code: String,
    pub test_code: String,
    /// What the test gave when it was last run, and where.
    pub test_result: String,
}

/// One version of a node: what it rested on, and — from the second version on —
/// which belief broke to make it, what was tested, and what changed.
///
/// A node changes because a belief broke, never because somebody preferred
/// another shape. So every version after the first answers the questions of
/// the de-risking narrative, one row per change: what we believed, what we
/// tested, what we now know, what it cost, what changed in the plan, and which
/// risks that opened or closed. It also says what THIS version rests on and
/// what would break it, which is the next version's first question asked in
/// advance.
///
/// Written once, from the change that carried it, and never edited after: a version is a record. `release` is `next` until a
/// release is cut, when `xtask release` stamps it with the tool's version.
#[derive(Clone, Debug, Default)]
pub struct Version {
    pub n: u32,
    /// The tool release this version first shipped in, or `next`.
    pub release: String,
    pub date: String,
    pub by: String,
    /// Which kinds of decision moved: see `derisk::ABOUT`.
    pub about: Vec<String>,
    pub believed: String,
    pub tested: String,
    /// What we now know — the issue with the previous version.
    pub learned: String,
    pub cost: String,
    /// What this version changes, and the benefit.
    pub changed: String,
    /// Risk moves, as `R-01 L5->L4`, `R-09 closed` or `R-12 opened`.
    pub risks: Vec<String>,
    /// What this version believes.
    pub rests_on: String,
    /// What would break that belief — the condition to watch.
    pub breaks_if: String,
    /// The relation as this version states it, and its source, kept so an
    /// earlier version can be read after the sheet has moved on.
    pub relation: String,
    pub source: String,
}

/// One risk in the programme's register.
///
/// Registered on one of the risk-register rows of the management layer — which
/// row says which kind of risk it is — and moved only by node versions: a risk
/// is reduced or closed because something was tested and a belief changed, and
/// the version that did it is the record of how.
#[derive(Clone, Debug, Default)]
pub struct Risk {
    pub id: String,
    pub title: String,
    /// `L1` (least) to `L5`, when it was registered.
    pub level: String,
    pub owner: String,
    pub since: String,
    /// What could go wrong, and what it would cost if it did.
    pub why: String,
}

/// One extra variable a node publishes beyond its primary answer.
///
/// The rule everywhere else is one row, one question, one answer, and the
/// variable id is the node id. This is the declared exception, and it exists
/// for one shape of thing: a conclusion that is a SET rather than a number.
/// The solar subsystem's product is five driver scenarios, each carrying five
/// quantities; splitting that at the seam would mean layer 2 reassembling
/// something layer 3 already had, and four crossings would break the rule that
/// a subsystem has exactly one.
///
/// The variable id is `<node id>.<id>`. A node id never contains a dot, so an
/// extra can never collide with a row, and a reader seeing one knows without
/// being told that it is part of another row's answer rather than a row.
///
/// A sheet that declares none of these is unaffected in every way, including
/// its hash: the primary answer is still emitted first and still sits at the
/// same index in the variable table.
#[derive(Clone, Debug, Default)]
pub struct Publish {
    /// The suffix. The full variable id is `<node id>.<id>`.
    pub id: String,
    pub symbol: String,
    pub label: String,
    pub ty: String,
    pub unit: String,
    pub lower: f64,
    pub upper: f64,
    pub reason_lower: String,
    pub reason_upper: String,
    pub port: PortSheet,
}

/// What an output says of its value beside the number (docs/SYSTEM_MODEL.md,
/// section 4): its state, its maturity, whether it is a parameter and whose,
/// and, while open, who owns it and the gate it is due by. Each is what the
/// sheet wrote, or empty; [`Sheet::port_state`] says what an empty state
/// means. Outside the sheet hash: it is what is believed of the value, not
/// what the row computes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PortSheet {
    /// `decided`, `allocated`, `open` or `achieved`.
    pub state: String,
    /// `estimated`, `calculated` or `measured`.
    pub maturity: String,
    /// `programme`, `system` or `subsystem`: the level that owns it, when it
    /// is a parameter.
    pub parameter: String,
    pub open_owner: String,
    pub open_due: String,
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
    /// Which published variable this expected value is for, by the `[[publishes]]`
    /// id. Empty means the node's own answer, which is every row that has one
    /// answer. A row whose conclusion is a set must say which member a fixture
    /// is about; the gate refuses a name that is not one of its publishes.
    pub variable: String,
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
    /// An assistant may never supply mathematics, and that prohibition needs
    /// something mechanical behind it: an invented formula that runs cleanly is
    /// the worst failure this system can have, and it looks exactly like a
    /// cited one. A name here is the authorship record the completion
    /// questions are supposed to produce — so an assistant cannot supply a
    /// relation without forging a person's attribution, which is a different
    /// and much larger thing than filling a blank field.
    ///
    /// It does not make the formula right. It makes the formula *somebody's*,
    /// which is what H1b needs in order to be a review rather than a reading.
    pub relation_by: String,
    pub assumptions: Vec<Assumption>,
    /// Where the relation came from, for a reader rather than for the compiler.
    /// Outside the sheet hash: see [`Theory`].
    pub theory: Theory,
    /// The row said simply, where that breaks, and the wrong idea. Outside the
    /// sheet hash: see [`Explain`].
    pub explain: Explain,
    /// The method, if one has been written. See [`Method`].
    pub method: Method,
    /// The table it is read from, when its answer is a lookup. See [`Lookup`].
    pub lookup: Option<Lookup>,
    /// The children that answer it, when its answer is theirs. See
    /// [`ChildrenOf`].
    pub children: Option<ChildrenOf>,
    /// The author's own code, which produced `cases`. Evidence, outside the hash.
    pub author: AuthorCode,
    /// The author's test cases, from their own code: in SI, by input binding.
    /// Outside the hash, like fixtures.
    pub cases: Vec<crate::method::Case>,
    /// Flight software kept with this node. Outside the hash.
    pub flight: Vec<Flight>,
    /// Every recorded version, oldest first. Outside the sheet hash: a record
    /// of why the node is what it is, not part of what it computes.
    pub versions: Vec<Version>,
    /// The risks this row registers. Only the rows under `mgt_risk_register`
    /// hold any; the gate refuses them anywhere else.
    pub risks: Vec<Risk>,
    pub symbol: String,
    pub ty: String,
    pub unit: String,
    pub lower: f64,
    pub upper: f64,
    pub reason_lower: String,
    pub reason_upper: String,
    /// What its primary output says of its value. See [`PortSheet`].
    pub port: PortSheet,
    /// Extra variables published alongside the primary answer. Empty for all
    /// but the rows whose conclusion is a set; see [`Publish`].
    pub publishes: Vec<Publish>,
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
    /// The folder exists, the row is on the tree, every tab opens and each
    /// says what goes in it. Nothing is generated from it and it cannot run.
    /// This is the normal state of most of a tree for most of a programme, so
    /// it is a reported state rather than a failing one.
    pub fn is_seeded(&self) -> bool {
        self.state == "empty" || self.state.is_empty()
    }
    /// What its answer is (docs/SYSTEM_MODEL.md, "behaviour"): `open`,
    /// `lookup`, `children`, `stated`, `method` or `built-in`, the first of
    /// them that holds.
    pub fn behaviour(&self) -> &'static str {
        if self.is_seeded() {
            "open"
        } else if self.lookup.is_some() {
            "lookup"
        } else if self.children.is_some() {
            "children"
        } else if self.is_declared() {
            "stated"
        } else if !self.method.text.trim().is_empty() {
            "method"
        } else {
            "built-in"
        }
    }
    /// Where an output's value stands: what the sheet says, or, when it says
    /// nothing, what its row is — open for a row not decided yet, decided for
    /// a stated value, allocated for a requirement, achieved for anything
    /// computed. The same reading the upgrade to the one schema makes
    /// (`vleo_files::upgrade`).
    pub fn port_state(&self, port: &PortSheet) -> &'static str {
        match port.state.as_str() {
            "decided" => "decided",
            "allocated" => "allocated",
            "open" => "open",
            "achieved" => "achieved",
            _ if self.is_seeded() => "open",
            _ => match self.kind.as_str() {
                "declared" => "decided",
                "required" => "allocated",
                _ => "achieved",
            },
        }
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
    /// The block it is declared on: a loop belongs to the smallest block
    /// that holds it. Empty for one declared in a layer file's own
    /// `[[iterate]]`, or a case's, on no block.
    pub on: String,
    pub nodes: Vec<String>,
    pub converge_on: String,
    pub tolerance: f64,
    pub max_iter: u32,
    pub seeds: Vec<(String, f64)>,
}

/// The case: the one multipayload design, and the shape of its inputs.
///
/// There is one, not one per customer — see `cases/multipayload.toml`. The
/// values a person works with are stored by the application outside the
/// repository; what is here is which inputs describe the CONDITION the design
/// flies in. Every other declared input is CUSTOMER.
#[derive(Clone, Debug, Default)]
pub struct Case {
    pub id: String,
    pub label: String,
    pub note: String,
    pub supply: Vec<(String, f64)>,
    pub cycles: Vec<CycleSpec>,
    /// The inputs in the Condition group, by row id. V16 refuses a name that
    /// is not a declared, published input.
    pub conditions: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Source {
    pub id: String,
    pub title: String,
    pub where_: String,
    pub status: String,
    pub used_for: String,
}
