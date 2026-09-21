# Architecture

## Four rings, one direction

    RING 0   vleo-units     units and frames as types, constants, portable maths
    RING 1   vleo-core      every formula, faults, credibility, the resolver
    RING 2   vleo-bus       the wire contract every face speaks
    RING 3   vleo-mod-*     nineteen crates, one per owner
             vleo-modules   the facade: the graph tables, compiled in
    FACES    wasm · daemon · cli · console · ffi · py

A ring may call inward and never outward. So *the kernel cannot draw* is a
compile error rather than a review comment: `vleo-core` has no renderer, no
filesystem crate and no allocator anywhere in its dependency tree, and it has
exactly one dependency, which is a leaf.

## Why a crate per subsystem, not folders in one

Inside a single crate, `use crate::prop::…` from `power` compiles and the
isolation rule is unenforced. Separate crates make it a manifest line: a sibling
a crate did not declare will not compile.

The faces still take one dependency on the facade, and the compiler still sees
nineteen units, so they build in parallel.

One per *owner*, not per discipline: a crate split along discipline puts one
subsystem layer's rows in three places, and `V12 one crate per owner` fails the
assembly if any group's rows end up in two crates or any crate holds two layers.

This was found by building it the other way first. The shared-crate version
passed every other check in the design while quietly making its central claim
false.

## The shell is a tree of modules, not a bundle

    dom.js  ← state.js ← display.js ← tree.js  ┐
                      ↖ figure.js ← paths.js   ├─ app.js
                      ↖ node.js  ← run.js      ┘
                      ↖ architecture.js

Every module depends inward and nothing depends on `app.js`. A view that could
navigate would have to import the router that imports it, and a cycle in a
hundred-line module is a cycle nobody notices until it is a thousand. Views
render markup carrying `data-` attributes; `app.js` owns every listener and
decides what a click means.

The modules are served individually by the daemon rather than bundled. A
bundler is a build step between the source and the thing that runs, and the
first time they disagree the disagreement is invisible — which is the same
argument that puts the graph tables in the build rather than in a file the
engine reads at run time.

## Three graphs, never merged

| graph | edge | says | used for | changes |
|---|---|---|---|---|
| derivation | variable → node | this is computed from that | execution | every node added |
| contribution | variable → KPI | this feeds that target | coverage | when requirements move |
| relation | group → group | these bear on each other | navigation, impact | rarely |

Execute the union and a KPI gets computed as though it were derived. Check
coverage against the union and a navigation link counts as evidence.

**Every edge is declared exactly once, by the end the edge changes.**

- A derivation edge by the *consuming* node, because knowing its inputs is what
  changes that node's implementation. The producer is unaffected by who consumes
  it.
- A contribution edge by the *contributing variable*, because a KPI does not
  compute; the variable is the thing that must be traceable.
- A relation edge by the *layer file*, because neither end is a node.

Everything else — who consumes this, what feeds a KPI, what a change reaches —
is derived on assembly and never stored, so it cannot go stale.

## The graph is compiled in

If the resolver read the graph from a file at run time, the engine and the graph
could disagree: an engine built on Tuesday walking Wednesday's graph. So the
tables are generated from the sheets at build time and compiled in. Adding an
edge is therefore a rebuild, which is correct — it changes what the engine
computes, so it should go through the gate.

The front end reads the same tables through the daemon, so the picture and the
execution cannot diverge.

## Nodes call nothing

A node is a pure function of its declared inputs. It has **no way to name a
peer**.

Real closure is cyclic: power becomes heat, heat sets array temperature, array
temperature sets cell efficiency, cell efficiency sets available power, and
available power throttles the largest load on the bus. If implementations
called each other, the crate graph would have to mirror that dataflow, crate
cycles are forbidden, and this one loop would force propulsion, power and
thermal to be merged into one crate.

They are not merged. The loop is **data**, declared in the case, and the
resolver relaxes it to a stated convergence criterion. An undeclared cycle is a
named error rather than a hung resolver.

    [[iterate]]
    nodes = ["pwr_cell_derating", "pwr_array_power_eol", "pwr_available",
             "prop_throttle", "prop_delivered_bus_power", "pwr_demand",
             "thm_dissipation", "thm_equilibrium_temperature"]
    converge_on = "pwr_demand"
    tolerance = 1e-6
    max_iter = 80

A cycle is a **cross-branch property**: two individually clean branches can
form one, so the acyclicity check belongs to the merged state and never to a
per-node gate.

### One consequence found by running it

Credibility rolls up by taking the minimum factor by factor. That is right
along a chain and wrong inside a fixed point: a loop reads its own outputs, so
one zero is absorbing and every member ends at zero however many sweeps run —
an artefact of the propagation rule meeting a cycle, not a statement about the
design. Inside a converged fixed point the members do not weaken each other, so
the vectors are cleared and settled from the converged state.

## Seven planes, four lanes

| plane | browser | desktop | console | campaign |
|---|---|---|---|---|
| 1 author | identical — one sheet | | | |
| 2 project | identical — one generator | | | |
| 3 supply | a form | the same form | a panel | a case file |
| 4 run | a call into WebAssembly | a call to the installed engine | nothing crosses | no button at all |
| 5 kernel | identical — one crate, the same one the rig and the pipeline run | | | |
| 6 return | object and typed array | object and buffer | registers | columns, then a file |
| 7 shown | the node tabs | the node tabs | bench rate | nothing draws |

Three planes are identical, and they are the three that matter: authored once,
generated once, computed by one crate.

Plane 5 carries the strongest rule in the design — no files, no clock, no
drawing — and it binds the resolver too. The resolver may cache, because a hash
of inputs is not a clock.

## The boundary is crossed exactly twice

In with a case, out with scalars and raw buffers. That is the whole reason it
can be tested.

## An input is edited in the face and run in three stages

The tool is one application: a tree of questions, a kernel that answers them,
and a face that is how a person drives it. A `declared` row is where that
driving starts — it is a number somebody chose, so it is the one kind of row a
reader may change. A `computed` row is never editable, because its answer is
its algorithm applied to its inputs; the way to move it is to move what it
reads.

**The edit does not touch the repository.** It travels as `set=<id>:<si>` on
the run, which is `Case::supply`, the mechanism the engine has always had for
answering a question about a design other than the one on disk. The sheet keeps
its number and its `confirmed_by` line, and a session of this leaves
`git status` empty. That is the first of the five rules holding: the sheet is
the only source, and a face that wrote to it would be a second one.

Pressing update then runs **three stages, in this order**, because they answer
three different questions and a single run answers only the last:

| stage | mode | the question it answers |
|---|---|---|
| 1 · alone | `alone` | did the value I typed reach the engine |
| 2 · branch by branch | `branch`, once per branch | what does each thing that reads this row now say |
| 3 · everything | `all` | what does the whole design now say |

**Stage two is the one that needs a definition.** A branch is the dependency
closure of one row. A row is in as many branches as there are rows that read
it — mission duration is read, directly or at a distance, by 69 — and most of
those branches are nested inside each other, so running all of them would
compute the same rows many times over to produce a longer list.

So the branches offered are the **maximal active** ones. *Active* means every
row in the closure is `published`: a branch that stops at a row nobody has
filled in cannot give an output, and offering it as though it might is how a
reader learns to ignore the list. *Maximal* means no other candidate reads it,
directly or at any distance. Running the maximal set computes every row in
every active branch containing the input, and computes none of them twice. For
mission duration that is 28 branches rather than 69.

**Active is not the same as will succeed, and the difference is not a flaw in
the test.** Every row being published says the tree is filled in. It says
nothing about whether a value lands inside its own declared domain on this
case. `aero_ao_fluence` is eleven published rows and refuses anyway, because
the fluence it computes is above its own upper limit. That refusal is a real
answer about the design, so stage two prints it.

**A refusal is reported against the baseline, so it is attributed correctly.**
A row that was already refusing before the edit is not evidence about the edit,
and colouring it as a new failure is a false alarm — which teaches a reader to
skim past the one that matters. A row that *had* a number and now does not is
the finding, and it is the one shown as such: at its upper bound
`com_antenna_diameter` takes `kpi_data_volume` from 197.919 to blocked.

Each stage draws as it lands, because stage two is one engine call per branch
and a reader should not watch a blank panel while they run.

## The front end is assembled exactly like the back end

|  | back end | front end |
|---|---|---|
| per node | `model.rs` | `page.html` |
| assembled by | a generated module list | a generated index |
| into | one crate | one document |
| touched when a node is added | one new file | one new file |

An earlier draft had per-node Rust files assembled into a crate and a *single*
document. That asymmetry is wrong for a measurable reason: five engineers adding
a node on the same afternoon produce four merge conflicts in one document — in
generated content nobody is allowed to hand-edit — and zero with a fragment
each.

It is the same rule already stated for the module list and the index: **never
commit an aggregate**. The document was an aggregate hiding in plain sight.

## What is committed, and what is built

**Committed** — every `node.toml`, every `fixtures.toml`, the hole bodies, and
the per-node generated artefacts. The last of those is deliberate: a committed
`model.rs` is diffable and reviewable, and a generator bug is visible in a
pull request rather than only in a build log.

**Never committed** — the assembled document, the index, the module lists, the
graph tables, the compiled engine. Every node would touch them, so every merge
would conflict in generated content nobody is allowed to edit.

One sentence: **per-node artefacts are committed, everything that combines them
is built.**
