# VLEO Integrated Design Tool

A systems engineering method that keeps a programme in control of its own
design, and the tool that runs it. Built around a very-low-Earth-orbit
multipayload programme with air-breathing electric propulsion — but the problem
it solves is not a VLEO problem.

| | |
|---|---|
| rows in the tree | **1329** across four layers — 250 specified, 1079 seeded |
| layers 1 and 2 | **228** and **367** rows — CD-06's own, verbatim, from `cd06/tree.json` |
| specified rows | **112** a person picked · **126** worked out · **12** KPI closures |
| declared edges | **463** derivation · **298** contribution · **177** relation |
| subsystem crates | **15**, one per team, isolated by the compiler |
| faces | browser · daemon · command line · C ABI · Python wheel · MATLAB |
| deepest declared chain | **25 nodes**, solar flux to cost per year |
| an 80-point sweep of the whole graph | **83 ms** |

![The tool: the tree and the dependency in one grid — nested boxes on the
diagonal are the tree, marks off it are what reads what, and the routed arrows
leave the selected row and turn down the column of every node it
feeds](docs/img/tool.png)

The tree and the matrix are **one figure, not two panes**. A side-car tree
drifts out of alignment the moment a branch opens, and a tree on its own cannot
show the second relation at all. Closing a box does not hide anything: it
becomes one row, and every edge inside it rolls up onto that row.

---

## The problem this is shaped around

In 1999 a spacecraft was lost because ground software sent impulse in pound-force
seconds and navigation expected newton seconds. Neither side was wrong on its
own. The interface belonged to both and was owned by neither, so nobody checked
it. The board found one root cause and eight contributing ones, across software
validation, staffing, training, communication, systems engineering and
operations. It was not a physics failure.

Three years earlier, Ariane 501 flew a component with a flawless record on the
previous vehicle outside the envelope it had been specified for, and nothing was
re-derived.

Both boards reached past the immediate fault. Neither concluded that someone
should have been more careful.

This repository is one answer to what they described:

- **A unit is a type.** Adding a `Millinewton` to a `Newton` does not compile.
- **An interface is a node, not a meeting.** If it is not on the sheet it is not
  an interface.
- **A guard carries the reason it exists**, because a guard whose reason is not
  written gets deleted by the next person who finds it awkward.
- **Nothing is compared by hand.** A subsystem result is compared against the
  system requirement by the same function at every boundary.

---

## What is here

    crates/
      vleo-units/     RING 0  units and frames as types, constants, portable maths
      vleo-core/      RING 1  every formula, the fault taxonomy, credibility, the resolver
      vleo-bus/       RING 2  the wire contract every face speaks
      vleo-mod-*/     RING 3  nineteen crates — one per owner, 1329 node folders
      vleo-modules/           the facade: the graph tables, compiled in
      vleo-sheet/      BUILD  what a sheet means — read by the generators and the build
      vleo-data/              versioned bundles, a lockfile, verification before use
      vleo-cli/         FACE  run · sweep · campaign · data · selftest
      vleo-daemon/      FACE  the local engine, serving its own interface
      vleo-ffi/         FACE  the C ABI — MATLAB, Simulink, anything that speaks C
      vleo-wasm/        FACE  the demonstration kernel the public URL carries
      vleo-py/          FACE  the Python wheel; MATLAB reaches it out of process
    xtask/                    the generators, the gate, the assembly
    layers/                   the rows that are not nodes: headings, relations, ownership
    cases/                    per-customer values against one shared architecture
    sources/                  sources as objects, not strings
    bundles/                  reference-data recipes
    web/js/                   the shell: one module per concern, no bundler
    docs/                     including VARIABLES.md — all 1329 rows, generated

### One node is one folder

    crates/vleo-mod-prop/nodes/capture_efficiency/
      node.toml      the sheet — the only file here written by hand
      fixtures.toml  known-good values, and where each came from
      model.rs       generated · the whole file, with one HOLE per numbered step
      contract.rs    generated · the untyped adapter the bus calls
      mod.rs         generated · the module wiring
      evidence.rs    generated · the fixtures, as tests
      page.html      generated · the eight tabs
      meta.json      generated · state and hashes, written by the gate

Everything about one node is in one directory: adding one is a copy, deleting
one is a remove, its history is the log of a directory, and ownership is a path
rule.

---

## One crate per owner

A node's folder lives in the crate of the thing that owns it: the management
layer, the system layer, or one of the seventeen subsystem layers. Nineteen
crates, derived from the tree rather than listed — the skeletons, the workspace
members and the facade's dependencies are all generated from it, because a
hand-kept list of crates drifts from the tree it mirrors and nobody notices.

Not per discipline. `aero` and `mass` both answer the mass-and-aero layer's
targets, and a crate split along discipline put one layer's rows in three
places: nine of the seventeen layers were spread across two to eight crates, so
a propulsion engineer filling in propulsion rows had to touch two of them and
`vleo-mod-subsystem` was one crate that seventeen teams would all edit. The
tree said `l3_prop` was one layer with one owner; the folders said otherwise.

This is the isolation rule made enforceable rather than stated. Inside one
crate `use crate::prop::…` from `power` compiles and nothing stops it; across
crates it is a manifest line. **`V12 one crate per owner`** checks it on every
assembly — no group's rows in two crates, no crate holding two layers — so the
structure cannot decay quietly back.

The folder is the identifier, for every row. That used to be three rules, and
the third collided the moment two disciplines answered the same layer:
`pwr_margin` and `thm_margin` both wanted `margin`. One rule cannot collide,
because identifiers cannot.

## The seeder, one file per subsystem

    tools/seed_helpers.py    the row helpers, and the lists they append to
    tools/nodes/env.py       one module per subsystem — the rows, and nothing else
    tools/nodes/aero.py      …twelve of them
    tools/seed_tree.py       the tree, the layers, the emitter, the assembly
    tools/cd06_extract.py    CD-06's tree, out of the document
    tools/cd06_rows.py       …turned into rows
    tools/crate_skeleton.py  one crate per owner, written from the tree

It was one 3,311-line file holding the rows for every subsystem and the
machinery that emits them, so changing what propulsion declares meant opening
the file that declares everything else. Import order is authoring order, which
is the order the rows read in on the tree, so `seed_tree` decides it and each
module only says what its rows are.

## The shell, segmented

The interface is the architecture read in the order it has to be built in, and
each part of that order is a module of its own. There is no bundler: a bundler
is a build step between the source and the thing that runs, and the first time
they disagree the disagreement is invisible.

    web/js/state.js          the one state, and the three graphs derived from it
    web/js/display.js        the one walk — the display rows all three drawings share
    web/js/tree.js           the tree column
    web/js/paths.js          the paths column
    web/js/matrix.js         the matrix, and the routed arrows
    web/js/figure.js         the layer view, and the prose that surrounds it
    web/js/node.js           one node: assembly, connectivity, the sheet
    web/js/run.js            the run, the result and the behaviour sweep
    web/js/architecture.js   the architecture, as a page of the tool
    web/js/app.js            boot, routing, and every event listener

No view module wires a navigation handler. Views render markup carrying `data-`
attributes and `app.js` decides what a click means, so the module graph stays a
tree: app depends on the views, the views depend on the state, and nothing
depends on app.

**Opening a node reads in three named parts, in a fixed order**, because the
order is what has to be repeatable when the next thousand rows are filled in:

1. **assembly** — the folder and its eight files, read off the disk rather than
   asserted, with who writes each one.
2. **connectivity** — what it reads, what it publishes, what reads it, what it
   contributes to and what it crosses to, each labelled with which end declares
   the edge.
3. **the sheet** — the eight generated tabs: question, interface, algorithm,
   generated code, evidence, flags, credibility, design space.

The first tab of the tool is the same thing at programme scale — the node and
its assembly, connectivity as declared, layer by layer, the tree hierarchy and
its connections, and the rings. Every number on it is counted from the tables
the engine walks, so a claim there cannot outlive the thing it describes.

![The architecture view: the eight files every node folder repeats, what is
specified and what is deliberately not](docs/img/architecture.png)

---

## Four layers, one architecture

| layer | rows | source | what it answers |
|---|---|---|---|
| 1 management | **228** | CD-06, verbatim | who wants what, what it costs, what the programme promised |
| 2 the system | **367** | CD-06, verbatim | what the satellite is made of, and what reads what |
| 3 subsystem | 853 across 17 layers | shape from CD-06, rows ours | decomposed until each row is a question one person can answer |
| 4 the run | — | — | one case, one chain hash, one set of numbers |

Layers 1 and 2 are the document's rows and edges, read out of it by
`tools/cd06_extract.py` and committed as `cd06/tree.json`. Nothing in this
repository names a variable those two layers do not already contain: the counts
match the document exactly, and so do the labels.

Layer 3 is different, and the difference is stated rather than smoothed over.
The document specifies it by shape only — which fifteen subsystem layers exist,
how many targets each takes from layer 2, and how many rows each holds — and
never names a row. Two of those three are enough to build it honestly. **The
targets are derivable**: a target equals the parent's variable one for one, and
every one of the fifteen target counts in the document equals the variable count
of the layer-2 group it reports to. Nineteen targets for propulsion because
layer 2 holds nineteen propulsion variables. So each target is that variable's
name, mirrored by an achieved row. What is left is the layer's own working, and
those rows stay `to be named` until somebody names them.

Two of the seventeen layers are additions, labelled as such: orbit geometry, the
space environment and mission performance are layer-2 groups in CD-06 rather
than subsystem layers, and this repository decomposes all three; closure and
cost are ours entirely.

### What is specified, and what is deliberately not

The 250 specified rows are the **engine-sizing chain** — atmosphere,
aerodynamics, intake, thruster, power — carried end to end to the
thrust-against-drag closure, plus one simple reference design case to run it
against. That is enough to exercise every mechanism in the tool: units, guards,
declared cycles, evidence, credibility, the chain hash and the sweep.

The other 1083 rows are seeded on purpose. **The repeatable architecture is the
first deliverable**; content arrives per node, through the authoring loop, and
each one lands in a folder that already exists with an owner already on it. A
decomposition that only exists where someone has already done the work is a
decomposition nobody can plan against.

**Exactly one row in each subsystem layer crosses upward.** That row is the
whole interface between the layer and the system; it is declared on the sheet
(`crosses_to`) and drawn with a teal outline. A layer whose second row starts
reaching upward is a boundary that has stopped being a boundary, and the gate
can see it because the crossing is data rather than convention.

A **case** selects which boxes are in scope. It is never a copy of the tree: two
customers are two cases against one architecture, because a cloned architecture
is two architectures that will disagree, and the disagreement is found late.

![Layer 3: a subsystem decomposed to 738 seeded rows — the structure exists, the
content does not, and the hatched squares say which is which](docs/img/layers.png)

---

## Run it

    cargo run -p vleo-cli --bin vleo -- data sync     # fill the local store, once
    cargo run --release -p vleo-daemon                # then open http://127.0.0.1:7777

The daemon **serves the interface itself**. That is not a convenience: a page on
the internet speaking to a program on the machine has to get past cross-origin
rules, mixed-content policy and private-network preflight, and the last of those
is tightening and can break in a browser update with no change on this side. So
the boundary is removed rather than negotiated — one origin, and it works with
networking disabled entirely.

From the command line:

    vleo run prop_thrust_to_drag
    vleo sweep prop_thrust_to_drag --over orbit_altitude --from 150000 --to 450000
    vleo campaign pwr_margin
    vleo show env_mass_density
    vleo selftest

Everything crossing a boundary is SI. A face converts for display and never for
transport.

---

## What it computes

The closure thread, end to end, at the reference case:

    prop_thrust_to_drag — Thrust to drag ratio
      T_D = 0.464492
      80 ran, 0 blocked, 2 cycle sweeps
      credibility 0 of 4, governed by validation

At or above one, the orbit holds indefinitely with no stored propellant. Below
one, the mission has a lifetime rather than an altitude, and every other number
in the design is a detail. **It does not close at this design point**, and the
tool says so rather than being tuned until it does.

![One node: eight tabs, its answer, the eight credibility factors with the
lowest governing, and the evidence that executed on this run](docs/img/node.png)

![A behaviour sweep of thrust-to-drag against altitude, 80 points over the whole
graph in 83 ms, showing an optimum near 300 km](docs/img/sweep.png)

### Three rows refuse, and each says why

Running everything at the reference case computes 247 of the 250 specified
rows. The other three do not return a number, and none of them is a failure of
the tool — each is a declared limit doing the job it exists for:

| row | what it says |
|---|---|
| `aero_ao_fluence` | **2.02e27 atoms/m²** against a declared ceiling of 1e26, above which no known external material survives. At 250 km the atomic-oxygen flux is 1.28e19 atoms/m²/s, so a ram-facing surface passes that ceiling in **three months**, not five years. |
| `pay_geolocation_error` | **4.8 mm**, below the declared floor of 1 cm — the relation returns a number better than any time-difference system in this design achieves, so the floor refuses it rather than letting an optimistic figure travel downstream. |
| `kpi_geolocation` | blocked, because the row above it never ran. A KPI whose input refused is **unknown**, and it says so instead of quietly substituting. |

The first is a design finding of the same weight as the thrust-to-drag result:
**this design does not close on external materials either.** Widening either
bound would turn the tool green and delete the finding, which is the one thing
the guards exist to prevent.

The physics behind that number is real, not a placeholder:

- **Atmosphere** — diffusive equilibrium above a 120 km base with a Bates
  temperature profile, five species integrated separately by Runge-Kutta in
  altitude. Reproduces NRLMSISE-class densities across 150–500 km at both
  activity extremes. It carries its own uncertainty, 15% quiet and growing with
  geomagnetic activity, because a node that consumes density and does not carry
  that forward is a node whose margin is fictional.
- **Aerodynamics** — Sentman's free-molecular coefficients in Doornbos's form,
  with Langmuir accommodation from local atomic oxygen. Drag coefficient comes
  out at 2.75–3.05 and **moves with solar activity for an unchanged
  spacecraft**. Treating it as a constant 2.2 is the largest avoidable error in
  a VLEO drag estimate, and it is what most concept studies do.
- **Intake** — the flux balance between a hyperthermal inflow and a thermal
  outflow. Read the relation slowly and it says something a design should know:
  **collection efficiency does not depend on flight speed at all**, only on
  geometry. Speed buys compression, not capture.

---

## The rules the whole thing is derived from

**One implementation of every formula.** Every relation lives in `vleo-core`
and nowhere else; the gate fails the build if one appears in a node. No face,
binding or renderer re-derives a number.

**The declaration is the source.** Page, contract, bindings, tests and metadata
are printed from `node.toml`. A hand edit outside a numbered `HOLE` block is
discarded by the next regeneration and fails the regeneration diff — which is
what makes the generated region genuinely owned by the generator rather than
merely labelled that way.

**The engine makes no network calls.** Reference data arrives by an explicit
sync, before the run. A campaign of a million cases makes zero network calls,
and a run in an air-gapped facility is the same run made anywhere else.

**Refuse, never clamp.** Out of domain is an error naming the field, the bound
and the reason, at every face. A value silently corrected is a design that
drifted without anyone deciding to.

**Nothing is stored that can be recomputed.** Credibility, verdicts and margins
are computed on the run. A stored badge is a claim about last March.

**Every number carries its lineage.** Kernel hash, graph hash, case hash, chain
hash, bundle versions, and which endpoint answered.

---

## Two decisions worth reading the reasoning for

### The chain hash, not the case hash

A cache keyed on the case survives a change to the arithmetic under it. Publish
a node, run a case, get a number; rewrite the holes — same interface, same
inputs, different arithmetic — run the same case, and a case-keyed cache hands
back the old number looking current.

    key on the case values           9991  ->  9991   (unchanged, and wrong)
    key on the chain               f5afdb  -> 2054be  (stale, correctly)

The chain hash covers every node the run reached, each one's implementation
content and its published outputs, and the case. That makes staleness
transitive, which is the only definition of staleness that is safe to show a
person: change any node in a chain and every result downstream is **unknown**,
not wrong.

### Portable maths in the kernel

Addition, subtraction, multiplication, division and square root are exactly
specified by IEEE-754 and agree on every target. `sin`, `cos`, `exp`, `ln` and
`powf` are not: a native build calls the platform maths library, a WebAssembly
build calls the one compiled into the module, and they differ in the last bit.

Any node with a trigonometric or exponential term — most of the orbital ones —
would fail a nightly "golden vectors agree bit for bit" gate on the first night,
for a reason that is not a defect. Within a fortnight the team learns to ignore
a red nightly build, which is worse than having no gate.

So `vleo-units::pmath` implements them from exactly-specified operations only.
Determinism is promised; correct rounding to half an ulp is not, and that is
stated rather than implied. Accuracy is verified against an independent oracle
in `crates/vleo-units/tests/pmath_accuracy.rs`. The gate lints for a kernel
crate reaching the platform library.

---

## Where it is honest about what it is not

- **Three of the 250 specified rows refuse rather than return**, for the
  reasons tabulated above. That is two design findings and one correctly
  propagated unknown, not three defects — but a reader who expects 250 numbers
  and counts 247 deserves to be told which three and why.
- **1079 of the 1329 rows are seeded and nothing is specified in them.** The
  folder, the sheet, the row and the dispatch stub exist; the stub returns
  `NotRun`, by name. That is the decomposition written down before anyone has
  been told to fill it in, and it is deliberate: a decomposition that only
  exists where someone has already done the work is a decomposition nobody can
  plan against.
- **108 of the 126 computed rows carry no fixture.** Nothing outside this code
  has agreed with what they compute, so their validation credibility factor is
  zero, which governs the whole vector. `cargo run -p xtask -- gap` lists every
  one. A blank is a statement, not an oversight.
- **The credibility vector's last two factors are proxies**, not measurements,
  and they say so in the source. A score that pretends to measure something it
  cannot is worse than a gap.
- **The atmosphere model is not NRLMSISE-00** and does not claim to be. It
  reproduces the structure and the activity dependence. Fitted corrections
  against measured drag arrive as a reference-data bundle, not as a change to
  the file.
- **Cost estimating relations are order-of-magnitude at concept stage.** Their
  one-sigma residuals are larger than most of the design decisions they would be
  used to compare, and the residual is carried on the relation.
- **Bundles are CSV, not Parquet.** The production format is columnar; the
  trigger for building it is the first bundle that does not fit comfortably in
  memory as text. Until then a format anybody can read in an editor is worth
  more than one that needs a library to inspect.

---

## Working on it

    cargo run -p xtask -- status               # counts by state, and what is blocking
    cargo run -p xtask -- graph                # the three graphs, and what nothing reads
    cargo run -p xtask -- gap                  # what the sheets promised and nothing covers
    cargo run -p xtask -- docs                 # the six per-node generators
    cargo run -p xtask -- gate                 # the checks, in order
    cargo run -p xtask -- assemble             # the twelve assembly validations
    cargo run -p xtask -- variables            # regenerate docs/VARIABLES.md
    cargo run -p xtask -- new <id> --like <sibling>

`cargo xtask gate` is called by the authoring hook, by the pipeline and by hand.
There is one of it: where the hook runs one script and the pipeline runs
another, they drift within a month and the hook becomes theatre.

A new node:

    cargo run -p xtask -- new prop_intake_throat --like prop_capture_efficiency
    $EDITOR crates/vleo-mod-prop/nodes/intake_throat/node.toml   # the physics
    cargo run -p xtask -- docs prop_intake_throat                # six artefacts, none typed
    # fill the numbered HOLE blocks in model.rs — a few typed lines each
    cargo run -p xtask -- gate prop_intake_throat
    cargo test -p vleo-mod-prop

Two human reviews per node, and everything between them is a command. If a node
takes materially longer than that, the template has a defect — worth finding,
because it will be paid 1329 times.

See [`docs/RUNBOOK.md`](docs/RUNBOOK.md) for the full loop and
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for why the rings are shaped the
way they are.

---

## Provenance

Nineteen of the twenty-two structural decisions here trace to something outside
this repository — ECSS and ISO on how engineering data is organised, published
research on which picture can be read, and flight-proven architecture from the
core Flight System and Basilisk on how a kernel and its faces divide. The
sources are objects in `sources/sources.toml`, referenced by identifier and
never by a free-text string, so marking one superseded lists every node that
depends on it.

Five ideas are ours and unproven, and they are marked as such rather than
presented with the rest:

1. The four-layer split with exactly one crossing node between layers.
2. Required-against-achieved closure repeated identically at every boundary.
3. The eight-tab node as the fixed unit of the design.
4. The document as the source, with code generated from it.
5. Credibility scored per node rather than per model, and gating.

Originality would have been the warning sign.
