# Authoring a node

## What a node is, in plain words

One small question with one answer. *Does the air the intake swallows produce
enough thrust to hold the orbit?*

To answer it properly, seven things are written down once:

| | written down | why it has to be there |
|---|---|---|
| 1 | the question, in ordinary words | if it cannot be said plainly, it is not one node |
| 2 | the relation, and the page it came from | this is the claim everything else rests on |
| 3 | inputs and outputs, with units | millinewtons, not "thrust" |
| 4 | the range where it is valid, **with a reason per limit** | this is the Ariane 501 failure |
| 5 | the steps, numbered, in words | the shape the code will take |
| 6 | known-good answers from somewhere that is not our own code | a number from the thing being tested proves nothing |
| 7 | how much we trust it, and why | so a reader can weigh the result |

Once those exist, **eight artefacts are printed automatically**: the
implementation scaffold, the contract, the module wiring, the test harness, the
documentation fragment, the metadata, the graph entry and the binding. That is
the entire point of the design. Write the physics once; get eight artefacts.

## The sheet

```toml
id = "prop_capture_efficiency"
label = "Intake collection efficiency"
folder = "capture_efficiency"   # frozen at seed; renaming the label never moves it
subsystem = "prop"
parent = "prop_intake"
kind = "computed"               # computed | declared | required | achieved | kpi
owner = "propulsion"
tier = "B"

[question]
text = "What fraction of the flow entering the mouth actually reaches the thruster?"

[maths]
expression = "eta_c = A_out*eta_geo/(A_out + beta*A_in)"
source = "romano2021"           # an identifier in sources/, never a free string

[[assumption]]
text = "Free-molecular flow throughout the duct and the chamber"
fails_when = "above a compression ratio of roughly 1e4 the chamber becomes collisional"

[output]
symbol = "eta_c"
type = "Ratio"
unit = "One"
lower = 0.0
upper = 1.0
reason_lower = "a collection efficiency cannot be negative"
reason_upper = "an intake cannot deliver more than enters its mouth"

[[input]]
binding = "a_in"
var = "prop_intake_area"
type = "Area"                   # what the consumer expects; assembly refuses a mismatch

[[algorithm.step]]
number = 1
text = "balance the hyperthermal inflow against the thermal outflow"
binds = "r"
type = "Ratio"
```

### Every field earns its place

- **`folder` is frozen at seed.** Editing a label must never move a directory:
  renaming a heading would otherwise appear in version control as hundreds of
  deletes and adds and would conflict with every open branch.
- **`source` is an identifier**, not a citation string. Marking a source
  superseded then lists every node that depends on it — one query rather than a
  search.
- **`fails_when` is not optional.** An assumption with no failure condition is a
  sentence, not an assumption, and the schema refuses it.
- **`reason_lower` and `reason_upper` are not optional.** A guard whose reason
  is not written gets deleted by the next person who finds it awkward.
- **The consumer declares the input `type`.** Assembly refuses a type that
  disagrees with what the producer publishes — and assembly is the only place
  the whole graph is visible, so it is the only place the question can honestly
  be asked.

### The five fields this page used to omit

Found by checking what the loader reads against what this page explains — the
loader took forty-five fields from a sheet and fifteen were not on this page.
Most of those fifteen belong to `layers/`, `cases/` or `sources/` rather than to
a node. These five are yours.

- **`[maths] confirmed_by`** — who supplied the relation, and when. An agent may
  never supply mathematics, and without a name nothing can tell whether one
  did. A relation with nobody against it is a gap, so the node cannot reach H2.
  It does not make the formula right; it makes it somebody's, which is what H1b
  needs to be a review rather than a reading.
- **`criticality`** — `minor` or `significant`, defaulting to minor. Significant
  means two reviewers and the hole filled twice by different model families.
  Everything cannot be significant: a person asked to approve too many things
  stops evaluating each one, so raising it is done on purpose.
- **`migrated_from`** — the prior MATLAB function and line, when this node was
  translated from it. Its numbers then go in `parity.csv` beside the node and
  **never** in `fixtures.toml`: an implementation cannot supply its own expected
  values, and that is an implementation. A disagreement between the two is a
  finding about one of them, not a check either has passed.
- **`note`** — anything a reader of the node page needs that is not one of the
  fields above: what the relation is usually misused for, what a number is
  sensitive to, why an obvious simplification was not taken. Optional, and the
  one place on the sheet where prose is the right answer.
- **`[contributes] kpis`** — which key performance indicators this node's answer
  feeds. This is the contribution graph, declared by the variable rather than by
  the KPI, and it is what makes coverage a fact rather than a search. The gate
  refuses a KPI named here that does not exist.

`layer`, `order`, `crosses_to` and `folder` are on the sheet and are **not
yours**: they are frozen at seed, and the tree's shape is a reviewed change to
`layers/`, not a field edit.

## Declared values

Two thirds of the tree is numbers a person picked. They are cheaper than a
computed node and they are **not free**: every margin in the design is built out
of them.

```toml
kind = "declared"
[value]
number = 250.0
confirmed_by = "A. Rai / 2026-09-01"
```

A declared value with no source cannot reach `specified`. A confirmation older
than the review cadence is listed by the monthly review — a question, never a
block.

## The holes

The generator emits the whole file. Only the body of each numbered step is
yours:

```rust
// ---- HOLE 1 : balance the hyperthermal inflow against the thermal outflow -> Ratio
let r: Ratio = prop::intake_balance(n, v, a_in, a_out, eta_geo, beta, t_c, m)
    .collection_efficiency;
// ---- end HOLE 1
```

**Compose relations that already exist in `vleo-core::physics`.** Every formula
lives there and nowhere else; the gate fails the build if one appears in a node.
If the relation you need is not in the kernel, that is a reviewed change to the
crate every node reads — not an inline.

**Use `pmath`, never the standard library's transcendental functions.** The gate
lints for it, and the reason is in the README.

**Do not add a guard.** Guards are generated from the declared domain, with
their reasons attached. A guard added by hand is a guard with no reason, and it
will be deleted.

A hand edit anywhere outside a `HOLE` block is discarded by the next
regeneration and fails the regeneration diff. That is what makes the generated
region genuinely owned by the generator rather than merely labelled that way.

## Fixtures

```toml
[[fixture]]
label = "IRS-class baseline intake"
expect = 0.40909            # SI, always
tolerance = 0.0001          # relative
provenance = "independent-derivation"
source = "romano2021"
inputs = { a_in = 0.2, a_out = 0.01, eta_geo = 0.9, beta = 0.06, t_c = 600.0, ... }
```

The one rule: **an expected value may never be produced by the code under
test.** The schema refuses `self-snapshot` and `agent-generated`, and the gate
fails on either.

| provenance | highest tier | what it cannot detect |
|---|---|---|
| `independent-derivation` | A+ | an error common to both routes |
| `published-source` | A | an error in the source itself |
| `independent-tool` | B | an error the tool shares with this one |
| `physical-bound` | C | a wrong constant that still conserves |

Fixtures execute **on the run**, not only in the test suite. A verdict shown on
a page has to have come from a run: a badge read out of a field is a claim about
last March.

## What the eight tabs say when empty

The empty state is the teaching surface. Most of these are read by somebody
about to fill their first node, and "no data" teaches nothing.

| tab | empty state |
|---|---|
| question & mathematics | Not yet specified. Needs a question, an expression and a source. |
| interface | Inputs and outputs are declared. Units are not — a unit is a decision. |
| algorithm | No steps yet. Each step becomes one hole in the generated code. |
| generated code | Nothing generated — the sheet is incomplete. |
| evidence | No known-good numbers yet. A number from our own code does not count. |
| flags | Not run. |
| credibility | Tier not set — this decides how much evidence the gate demands. |
| design space | No limits declared. Every limit needs a reason, not just a bound. |
