# VLEO Integrated Design Tool

A systems engineering method that keeps a programme in control of its own
design, and the tool that runs it. Built around a very-low-Earth-orbit
multipayload programme with air-breathing electric propulsion — but the problem
it solves is not a VLEO problem.

| | |
|---|---|
| rows in the tree | **250** — 112 a person picked, 138 worked out |
| declared edges | **368** derivation · **53** contribution · **10** relation |
| subsystem crates | **12**, one per team, isolated by the compiler |
| faces | browser · daemon · command line · C ABI · Python wheel · MATLAB |
| deepest declared chain | **25 nodes**, solar flux to cost per year |
| an 80-point sweep of the whole graph | **83 ms** |

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
      vleo-mod-*/     RING 3  twelve subsystem crates, 250 node folders between them
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
    web/                      the shell: three views of one state
    docs/                     including VARIABLES.md — all 250 rows, generated

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

- **120 of the 138 computed rows carry no fixture.** Nothing outside this code
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
    cargo run -p xtask -- assemble             # the eleven assembly validations
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
because it will be paid 250 times.

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
