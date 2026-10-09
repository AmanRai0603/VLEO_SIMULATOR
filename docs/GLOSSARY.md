# Glossary

> **Answer first.** The words this repository uses in a sense of its own, each in one or two sentences, with the document that defines it properly.
>
> **Kind:** reference · **For:** everyone

Every term here has a fuller definition somewhere else; this page only says
what it means here and where to read more. When the two disagree, the linked
page is right and this one is out of date.

---

## The design

**Row** — one small question with one answer: one node file in `design/`, one
variable whose id is the row's id. The tree has 1401 of them. *Node* means the
same thing, from the code's side. → `README.md`

**Layer** — which level of the design a row belongs to: 1 management, 2 the
system, 3 a subsystem, 4 the run. A layer reads the one below it only through
a closure. → `AGENTS.md`

**Sheet** — the row's specification, as text held in its node file in
`design/`; `cargo run -p xtask -- sheet <node>` prints it. Nothing in the
repository writes it. → `docs/NODE_AUTHORING.md`

**Seeded / published / deprecated** — a row's `state`. *Seeded*: the row
exists, and nobody has specified it yet. It cannot run and is reported, not
failed. *Published*: specified and able to run. *Deprecated*: kept
so old results still read, and not used. → `docs/NODE_AUTHORING.md`

**Declared / computed** — a declared row holds a value a person picked. A
computed row works its answer out from its inputs. Two thirds of the tree is
declared. → `docs/VARIABLES.md`

**Closure** — what crosses between layers: a requirement, an achieved value,
and the margin between them. It is the only contract between layers.
→ `AGENTS.md`

**Sense** — which way a requirement binds. `<=` means the achieved value must
stay under the bound; `>=` means it must reach it. It is never defaulted.
→ `AGENTS.md`

**KPI** — one of the twelve closures a customer's result is judged on, such as
mass margin, revisit or lifetime. `xtask reach` shows which answers reach one.

**Case** — the one set of inputs every run uses. It has two halves: the
*condition* (the orbit, the environment and the solar weather the design flies
in) and the *customer* (everything they choose). A new customer is new values,
never a change to the tree. It is kept under `~/.vleo/`, outside the
repository. → the manual, *Set the inputs — the case*

---

## The three parts

**Frontend** — what people see and touch: the pages, charts and forms in the
browser, and as they arrive the animation player and the 3D viewer. HTML, CSS
and JavaScript, in `web/`. It draws; it never computes a number.
→ `docs/ARCHITECTURE.html`

**Backend** — the engine: the Rust kernel that computes every number, the
1,396 rows, and the local server that answers the browser. In `crates/`.
→ `docs/ARCHITECTURE.html`

**Data (the part)** — what is kept: the design in git, reference data in
bundles, and each person's case, results and crash logs under `~/.vleo/`.
Reached only through `vleo-data`. → `docs/ARCHITECTURE.html`

**Contract** — what the frontend and backend both depend on and nothing else:
the HTTP API under `/v1` and a few file formats. Changed only with both sides
reviewing, so each side can otherwise change freely.
→ `contract/README.md`; `docs/ARCHITECTURE.html`, section 2

**Mock engine** — `tools/mock_engine.py`: the page served against the
answers recorded in `contract/examples`, with no Rust built. For working on
the page; every number on it is a recorded example, and the page says so.
→ `contract/README.md`

**Crash log** — one small text file per bug the tool hits, in `~/.vleo/log/`,
naming the line of code. The file to send to whoever maintains the tool.
→ `docs/ARCHITECTURE.html`, section 9

**Saved result / kept once** — a run or a sweep kept as a folder: its values,
its sweep's points, and a report page to send. The row, its inputs, the
engine, the tree and the data are one key, so a question already kept is
shown from its result — named on screen — rather than run again, and saved
only once. → `docs/ARCHITECTURE.html`, section 6

**Pinned / thinned** — a saved result's tier. *Pinned*: every value kept for
good. *Thinned*: older than `VLEO_KEEP_DAYS` (30) and not pinned, so the tool
kept the inputs it ran on, its answer and what could not run, and let the
other values go — it says so, and asking its question again runs it. A folder
named by `VLEO_RESULTS` is kept whole unless `VLEO_KEEP_DAYS` is set too.
→ `docs/ARCHITECTURE.html`, section 6

**Figure description** — what a result draws, said by the engine: the kind
(line, scatter, bar, heatmap, animation, 3D scene), its axes and units, every
series and the row it shows, and every refused point with why. A page draws
it; it never invents its own picture of the numbers.
→ `crates/vleo-modules/src/figure.rs`, `contract/schemas/result.json`

**Lesson** — a row's `lesson.toml`: stations of text with a
claim each, equations with their source, "try it" widgets that name input and
output rows for the engine to answer, check-yourself questions. Content only;
the page draws it from its component library. Outside the sheet hash.
→ `docs/LESSONS.md`

**Lesson form** — a row's lesson as one HTML file the node engineer who knows the
row fills anywhere, checked as they type by the gate's own lesson check, and
sent back; `xtask lesson check` checks a filled one, and the lesson goes into the
design with its group's release. → `docs/LESSONS.md`

**Readers' folder** — every row's page and every lesson as plain pages, built
by `xtask readers` for a shared drive or an internal web server; the engine
runs in the page to answer a lesson's widgets. → `docs/SHARING.md`, section 4

**Page template** — `web/page.html`: the one page every file the tool writes
is filled from — forms, a saved result, the role guides, a row's page, the
readers' pages — with four slots. What each page puts in it is its parts file
in `web/pages/`: every word and tag, by name, which the generator chooses and
fills; a test refuses a generator that writes a tag of its own.
→ `crates/vleo-sheet/src/shell.rs`

**Component library** — `web/js/components.js`: every part a page is built
from, by name — station, equation, widget, check-yourself, claim tag,
references, figure. A page that needs something new gets it added there once.
→ `docs/ARCHITECTURE.html`, section 5

**Trace** — the record a writing `xtask` command leaves in
`target/xtask-trace/`: its command line, the commit, and each step with its
time and what it found or why it stopped. `xtask trace` shows the last.
→ `docs/PIPELINE.md`

---

## The code

**Ring** — one of the four dependency levels:
`vleo-units` → `vleo-core` → `vleo-bus` → `vleo-modules`. Each ring depends only
inward. → `docs/ARCHITECTURE.md`

**Kernel** — `vleo-core::physics`, where every formula lives and nowhere else.
→ `AGENTS.md`, rule 3

**pmath** — `vleo_core::units::pmath`, the portable maths every computation
uses instead of the standard library's. It gives the same bits on every
platform. → `AGENTS.md`, rule 4

**Method** — a row's calculation in the pseudocode language, written by the
person who knows it. The method interpreter runs it when the engine runs the
design, and nothing translates it into Rust.
→ `docs/PSEUDOCODE.md`

**Author case** — an input and the answer its node engineer expects, given with a
method. The method is tested against it, in the interpreter. → `docs/PSEUDOCODE.md`

**NotRun** — what a row with no content returns, under its own name. A refusal
is never a substitute value. → `AGENTS.md`, rule 5

---

## Evidence

**Fixture** — an expected value from outside this code, in a row's
`fixtures.toml`, checked on every run. → `docs/NODE_AUTHORING.md`

**Provenance** — where a fixture's value came from. In order of strength:
independent derivation, published source, another tool, physical bound. The
gate refuses `self-snapshot` and `agent-generated`. `xtask status` counts rows
by their best one. → `docs/NODE_AUTHORING.md`

**Theory** — why the relation is the relation, step by step. It is
documentation, not specification, so it is outside the sheet hash.
→ `docs/NODE_AUTHORING.md`

**Gap** — something a sheet promised that nothing covers yet. `xtask gap` lists
them; it is a list, not a verdict. → `docs/NODE_AUTHORING.md`

**Version (de-risking)** — a `[[version]]` record on a sheet: what we believed,
what we tested, what we now know, what changed. It says `next` until the design's
release that carries it names it. → `docs/DERISKING.md`

**Risk register** — the risks, registered once on the management layer's rows.
Only versions move them. → `docs/DERISKING.md`

**Bundle** — a published, hashed set of reference data (for example, the solar
weather record). Publishing one cannot be undone; `xtask bundle verify` re-checks
every hash. → the manual, *Reference data*

---

## Changing the design

**Node file** — `<node>.vnode`: one node's content, written only by its node
engineer, in the node application. A node changes here. → `docs/GROUP_APPS.md`

**Sealed release** — `<group>-<version>.vleo`: a group's structure with every
node's content in it, signed, and sealed by its subsystem engineer. It goes on
the shared drive, in `groups/<group>/releases/`, and is taken into today's
design when it passes its checks. It is never edited. → `docs/GROUP_APPS.md`

**Today's design** — the design as the tool builds it when it opens on the
shared drive (`VLEO_DRIVE`): every group's latest sealed release that passes
its checks — its seal, its content, each node against the design — and, for a
group whose latest is refused, its last good one, with the reason. Nobody takes
a release in by hand. → `docs/OPERATING_1_0.md`, section 5

**Gate** — `xtask gate`: the checks every change must pass, in order. With
`cargo test`, it is the one command that must be green before a push.
→ `AGENTS.md`

---

## Branches and releases

**`developer`** — the branch the software goes to: generators, daemon, faces
and tests, by pull request. → `CONTRIBUTING.md`

**`maintainer`** — the branch the design's changes were taken in on. Nothing
goes into it now: a group's sealed release reaches today's design from the
shared drive. It is deleted on the switch-over day. → `CONTRIBUTING.md`

**`main`** — what ships. It moves only by pull requests from `developer`,
`maintainer` and `release/<version>`. → `CONTRIBUTING.md`

**Release** — a commit a merged pull request put on `main`, tagged
`v<version>`. The release workflow refuses anything else before it builds.
→ `docs/RELEASE_SETUP.md`

**Kit** — the tool for everyone, without the repository: one folder per system,
and the one `.whl` for every laptop. → `docs/SHARING.md`
