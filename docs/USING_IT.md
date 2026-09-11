# Using it

This is the page to read first. It is about doing the work — opening the tool,
filling a row, getting a number you can defend — not about how the system
checks itself. Where a check matters it is mentioned in one line, at the moment
you would actually meet it.

Everything below was run against this repository. The outputs are real.

---

## 1 · Ten minutes

```
cargo run --release -p vleo-daemon        # the tool, at http://127.0.0.1:7777
```

Open it. You get the tree on the left and a panel on the right. Four layers:

| layer | what it holds | rows | of those, written |
|---|---|---|---|
| 1 | management — the programme's own view | 174 | 0 |
| 2 | the system — what the spacecraft must do | 319 | 0 |
| 3 | subsystem — seventeen of them | 836 | 250 |
| 4 | the run — what a single evaluation produced | — | — |

`cargo run -p xtask -- status` prints that table, and a second one by
subsystem. Today 250 of 1329 rows carry content and 1079 are seeded shape
waiting to be filled — which is the state this tool is designed to be useful
in, not a defect.

Each subsystem reaches the layer above through exactly one node — seventeen
`l3_*_interface` rows for the seventeen subsystems, and three customer rows
from management into the system layer. That is what lets you answer "where did
this number come from" by walking upwards without ever leaving the tree, and
what stops a change in one subsystem reaching another by an unnoticed side
door.

The same thing from a terminal, which is often faster:

```
cargo run -p vleo-cli --bin vleo -- list prop        # every propulsion row
cargo run -p vleo-cli --bin vleo -- show prop_throat_area
cargo run -p vleo-cli --bin vleo -- run  prop_throat_area
```

`run` prints the number, what it is made of, and what refused:

```
prop_throat_area — Intake throat area
  How large is the opening from the collection chamber into the thruster?

  A_out = 0.0100000 m^2
  credibility 1 of 4, governed by uncertainty

  1 ran, 0 blocked, 0 cycle sweep(s)

  provenance
    data   solar-drivers@2026.09.04#54784567db11b868
    kernel b29a9e · graph b3eb04 · case 9663b2 · chain 434916
```

Three things there are worth knowing on day one.

**"1 ran, 0 blocked"** is always printed, including the blocked count and the
names. A row that has no content yet does not vanish and does not substitute a
default — it comes back as `NotRun` with its own name on it. Most of the tree
is like that today, which is exactly why the blocked count is never hidden.

**"credibility 1 of 4"** is the lowest of eight factors, and it names which one
is governing. It is never stored anywhere — it is worked out fresh from the
chain each time, so it cannot go stale.

**The chain hash** identifies this exact number: the kernel, the graph, the
case, and every input that reached it. Two runs with the same chain hash are
the same run. This is the thing to quote in a report.

---

## 2 · Asking a question the tool was not set up to answer

Change one number and watch what moves:

```
vleo run prop_capture_efficiency --set prop_throat_area=0.5
```

Sweep it instead:

```
vleo sweep prop_capture_efficiency --over prop_throat_area --from 0.005 --to 0.02 --points 4

# prop_capture_efficiency against prop_throat_area — 4 points
# A_out              prop_capture_efficiency note
0.00500000           0.264705882              ok
0.0100000            0.409090909              ok
0.0150000            0.500000000              ok
0.0200000            0.562500000              ok
# 4 ran, 0 refused. Refusals are recorded, never dropped.
```

You can only set a number a person declared. Ask for one the tool works out and
it tells you so rather than quietly ignoring you:

```
vleo run gnc_along_track_error --set env_density_uncertainty=0.5
vleo: 'env_density_uncertainty' is computed from its inputs, so a supplied
value would be overwritten the moment it is evaluated. Set one of the declared
numbers it reads instead — `vleo show env_density_uncertainty` lists them.
```

`vleo show <node>` lists what a row reads, so the message is a pointer rather
than a dead end.

Run every stored case against a row, or every fixture in the tree against this
build:

```
vleo cases              # what customers' cases supply
vleo campaign kpi_thrust_margin
vleo selftest
```

---

## 3 · Filling a row — the loop you will live in

A row is a folder. Only one file in it is written by hand.

```
crates/vleo-mod-prop/nodes/prop_throat_area/
  node.toml      ← the sheet. yours.
  model.rs       generated, except the numbered holes
  contract.rs    generated
  evidence.rs    generated — the fixture tests, plus three properties
  fixtures.toml  the known-good values
  meta.json      generated
  mod.rs         generated
  page.html      generated
```

### 3.1 Start it

```
cargo run -p xtask -- new prop_intake_mouth --like prop_throat_area
```

`--like` clones the *shape* of a sibling and blanks everything that must be
decided again. It is deliberately not a copy — a copy drags a stale source
citation through thirty rows:

```
label = ""            # REQUIRED — re-decide, do not inherit
[question]
text = ""             # REQUIRED — re-decide, do not inherit
[maths]
expression = ""       # REQUIRED — re-decide, do not inherit
source = ""           # REQUIRED — re-decide, do not inherit
```

### 3.2 Ask what is still open

```
cargo run -p xtask -- declare prop_intake_mouth --source papers/romano2021.pdf
```

It prints every field that is still blank, with what cannot be emitted without
it, and stops when there are none:

```
  ?  what one question does it answer
     question — without it: an equation with no question gets reused for the wrong thing
  ?  cited where — book, paper, page
     source — without it: this is the claim everything else rests on
  ...
6 question(s) open. Generation refuses until they are answered.
```

The open set is computed from the same list `xtask docs` refuses on, so there
is never a question that blocks generation and is not on this page.

### 3.3 Fill the sheet

Seven things, and they are all questions a person has to answer:

| field | what it is | why it is required |
|---|---|---|
| `question` | the one question this row answers, in a sentence | an equation with no question gets reused for the wrong thing |
| `expression` | the relation, as written in the source | so a reviewer can compare it against the paper |
| `source` | a key in `sources/` | this is the claim everything else rests on |
| `symbol`, `type`, `unit` | the answer's identity | a unit mismatch becomes a compile error, not a runtime surprise |
| `lower`, `upper` | where the relation is valid | outside it the answer is a refusal, by name |
| `reason_lower`, `reason_upper` | why each bound is there | a guard with no written reason gets deleted by whoever next finds it awkward |
| `value` + `confirmed_by` | for a declared number: who picked it | every margin in the design is built out of these |
| `[maths] confirmed_by` | who supplied the relation | an agent may never supply mathematics, and without a name nothing can tell whether one did |

Two more that are decisions rather than drafting:

- **`criticality`** — `minor` or `significant`. Significant means two reviewers
  and the hole filled twice by different model families. The default is minor;
  raising it is done on purpose, because a person asked to approve too many
  things stops evaluating each one.
- **`migrated_from`** — set it when the node exists in the MATLAB tool. Its
  numbers then go in `parity.csv` beside the node and never in `fixtures.toml`.

### 3.4 Generate

```
cargo run -p xtask -- docs prop_intake_mouth
docs: 1 node(s), 6 artefact(s) written
```

Six generators run, and none of them reads another row — which is what makes
1329 rows 1329 independent pieces of work rather than one large one.

While any field is still open it refuses instead, names them, and writes
nothing:

```
  refused prop_intake_mouth — nothing to generate from: label, question,
          expression, source, reason_lower, reason_upper
```

That refusal is the mechanism. It turns ambiguity from something an implementer
settles quietly into a blocking item on an engineer's screen.

### 3.5 See what the gate says

```
cargo run -p xtask -- gate prop_intake_mouth

  FAIL schema — required fields blank: label, question, expression, source,
       reason_lower, reason_upper
  ok   inputs
  FAIL declared-value — a declared value needs a number, a source and who
       confirmed it
  ok   contract
  FAIL sources — these cite nothing in sources/:
  ok   regenerate
  note gap-pass — no question stated; no relation stated; no source cited; a
       declared limit has no reason; a declared value with nobody's
       confirmation against it
```

Every one names the field. This is the design: an open decision becomes a line
on your screen rather than something an implementer settles quietly at two in
the afternoon.

### 3.6 Write the maths

You do not open `model.rs`. The body of each numbered hole goes in as text:

```
echo 'let e: Length = gnc::along_track_error_from_drag(
    Acceleration::new(a.get() * s.get()), Time::from_days(1.0));' \
  | cargo run -p xtask -- fill prop_intake_mouth --hole 1 --body -
```

`fill` is the only thing in this system that puts text into a generated file.
It refuses, before writing anything: a hole the sheet does not declare, a body
carrying its own `HOLE` marker, a guard, an early return, and a platform maths
call. Then it re-reads the file and proves the body landed.

This is why agent C has no write tool at all. An agent handed the file and told
not to stray is not constrained, it is asked — and the same applies to a person
in a hurry.

What the hole looks like once it is in:

```rust
pub fn evaluate(a: Acceleration, s: Ratio) -> Result<Length, Fault> {
    // ---- HOLE 1 : propagate the drag acceleration error over one day -> Length
    let e: Length = gnc::along_track_error_from_drag(
        Acceleration::new(a.get() * s.get()), Time::from_days(1.0));
    // ---- end HOLE 1
```

Everything outside the markers is regenerated, so an edit there is discarded
the next time anybody runs `docs`, and the gate catches it before that happens.
`tools/agent_lanes.py --holes-only HEAD~1` names any `model.rs` line changed
outside a hole, by anyone.

The signature is already correct. `Acceleration`, `Ratio` and `Length` are
distinct types, so adding a mass to a length does not compile. A hole body is
usually two or three lines that compose relations already in `vleo-core`.

### 3.7 Get evidence

A number the code produced is not evidence that the code is right.
`fixtures.toml` holds values from somewhere else — a paper, a measurement, a
MATLAB function somebody trusts:

```toml
[[fixture]]
label = "IRS-class baseline intake"
expect = 0.40909
tolerance = 0.0001
provenance = "independent-derivation"
source = "romano2021"
inputs = { a_in = 0.2, a_out = 0.01, eta_geo = 0.9, beta = 0.06, t_c = 600.0,
           m = 0.01872, n = 2133000000000000.0, v = 7754.6 }
```

`inputs` is keyed by **symbol**, not by node id — the symbols `vleo show` lists
under "reads". Every input the relation takes must be supplied; leave one out
and the gap pass names it.

`provenance` is one of `independent-derivation`, `published-source`,
`independent-tool` or `physical-bound`. The two it refuses are `self-snapshot`
and `agent-generated`: an expected value may never come from the code under
test. That is the one rule the whole evidence model rests on, and it is worth
knowing before you are tempted.

**Three properties come with the fixture, generated from the declared domain.**
One per cent either side of the known-good point the node must still answer;
every answer it gives must be finite and inside its declared domain, with no
input scaling making it panic; and the same inputs must give a bit-identical
answer. They catch discontinuity, a panic, non-determinism, and a domain
declared tighter than the physics. They do not catch a relation wrong in shape
that stays inside its domain — that is what a fixture and H2 are for.

### 3.8 Ask whether a person should look yet

```
cargo run -p xtask -- ready prop_intake_mouth
cargo test -p vleo-mod-prop
```

`ready` runs the gate, then the gap pass, then what criticality demands, and
stops at the first that is not clean. A node it holds is a node where a person
would be doing first-pass defect-finding — work a machine does better, faster
and free. When it says `1 of 1 … waiting on H2`, the node has earned a review.

Across the whole tree it counts what is holding rather than listing every row:

```
ready: 0 of 250 node(s) have passed every machine stage and are waiting on H2
250 held, by what is holding them:
    250  the relation has nobody's name against it
    120  no fixture — nothing outside this code has agreed with it
```

Then commit. The message form is checked (§6).

---

## 4 · Where the work can be handed off

Seven agents exist. You do not have to use any of them — every step above is
something you can do yourself — but each one takes a piece that is either
tedious or easy to get subtly wrong.

The useful way to think about them: **each agent is defined by what it cannot
do.** That is not a rule written in its prompt, which the same model would be
deciding whether to make an exception to. It is a list of paths, checked
against the diff afterwards:

```
tools/agent_lanes.py --agent hole-filler
tools/agent_lanes.py --agent test-author --since HEAD~1
tools/agent_lanes.py --list
```

| you have | ask | it gives you | it cannot |
|---|---|---|---|
| a paper, a measurement, a MATLAB function | **declaration-drafter** | a filled sheet, plus the questions that must be answered first | supply maths, a reason, or a value |
| expected values a person derived | **fixture-recorder** | those rows with provenance, and which domain edges have no case behind them | produce an expected value itself |
| a sheet that passed review | **hole-filler** | the few lines inside each numbered hole | write outside a hole, or add a guard |
| a declared property nothing tests | **test-author** | a property test over the declared domain | invent an expected value |
| a run that flipped and nobody knows why | **diagnostician** | which node, which change, which margin moved, and the smallest reproducing case | fix anything, or judge acceptability |
| ordinary engineering on the tool | **systems-backend** | generators, kernel plumbing, the runner, the server | edit a sheet or a hole |
| the web face or a figure | **frontend-visualisation** | the shell, tree, matrix, panels, plots | invent a style token, or ship a figure nobody rendered |

Two of these were written for this domain (drafter, recorder) and the rest are
adopted definitions, pinned in this repository rather than referenced upstream.

### What to expect

They are ordinary help, not oracles. The useful pattern is: ask for one step,
read what came back, run the gate. Three things are worth doing every time:

1. **Run the lane check.** `tools/agent_lanes.py --agent <name> --since HEAD~1`
   says whether the diff stayed where it was supposed to. The hole-filler's
   check reads the diff itself, not just the paths, because its lane includes
   `model.rs` and a path check alone would pass a rewrite of the whole file.
2. **Look at what it did not do.** A fixture recorder that reports no uncovered
   edges on a node with two bounds has not looked.
3. **Check a test can fail.** A test that passes against a deliberately broken
   implementation is not testing anything. Break the hole body, run the test,
   see red, put it back. It takes a minute and it is the difference between
   evidence and decoration.

`docs/AGENT_EVIDENCE.md` records what each one actually produced when it was
first used here, including the defect that scrutiny found.

---

## 4b · The rest of the commands

Everything `xtask` does that the walkthrough above did not need. All of it is
reporting or regeneration; none of it decides anything.

| command | what it does |
|---|---|
| `cargo run -p xtask -- assemble` | the three assembly generators — the index, the document fragments, the graph tables |
| `cargo run -p xtask -- status` | counts by layer and by subsystem, and what is blocking |
| `cargo run -p xtask -- gap` | what every sheet promised and nothing yet covers |
| `cargo run -p xtask -- graph` | the three graphs, their sizes, the deepest chain, and the crate-direction check |
| `cargo run -p xtask -- variables` | regenerate `docs/VARIABLES.md` — every variable, its unit, its bounds and the reason for each |
| `cargo run -p xtask -- codeowners` | regenerate `CODEOWNERS` from the owner field on each layer group |
| `cargo run -p xtask -- bundle publish <dir>` | hash a bundle's payload and write the result into its manifest |
| `cargo run -p xtask -- bundle verify` | re-check every hash in `bundles/` |

`variables` and `codeowners` write files that are committed, so run them after
a change that moves a bound or an owner, and commit what they produce. The
regeneration diff in the pipeline catches it if you forget.

---

## 5 · What runs without being asked

| when | what | where |
|---|---|---|
| you open a session | the tree's state, what is blocking, whether reference data is present | `.claude/hooks/session-start.sh` |
| you save a sheet or a fixture file | the gate on that node, refusing the edit if it fails | `.claude/hooks/post-edit.sh` |
| you save any `.rs` file | formatting | same file |
| you write a commit message | the message form | `tools/githooks/commit-msg` |
| every push | build, regenerate, gate, test, both profiles, no-std | `.github/workflows/gate.yml` |
| every push | an advisory review that cannot fail the build | same file, `review` job |
| every night | six passes over the whole tree, the ledger, and yesterday's state | `.github/workflows/nightly.yml` |
| weekly | a dependency bot, on its own branch, majors excluded | `.github/dependabot.yml` |
| on a tag | prove, build, and one human approval | `.github/workflows/release.yml` |

A hook is not a control — it only fires inside the tool that installed it. Every
rule above also runs in the pipeline, from the same file, which is why the two
can never drift apart.

---

## 6 · Reference data

Bundles are versioned, immutable, and verified before use. Nothing fetches
during a run: a run either has verified data on disk or refuses to start.

```
vleo data sync          # reconcile the local store
vleo data list          # what is installed
vleo data verify        # re-check every hash
```

Publishing your own:

```
cargo run -p xtask -- bundle publish bundles/my-data/2026.09.11
```

The manifest needs a name, a version, a provenance, a `licence_until` date, a
`stale_after_days`, and a file list. Leave any of them out and the bundle does
not load — with a message naming the field. Publishing twice from the same
input gives the same hash, which is what makes verification mean anything.

---

## 7 · Committing

```
type(scope): a sentence saying what changed
```

`tools/commit_message.py --types` prints the nine types and every scope (scopes
are read from the crates on disk, so they are never out of date). The prefix is
machine-readable for one reason: release notes are read off the log rather than
written.

```
tools/commit_message.py --types
tools/release_notes.py --version 0.2.0
```

If the hook is not firing, it has not been pointed at the tracked folder:

```
cargo xtask setup
```

That sets `core.hooksPath`, reads it back to check nothing overrode it, and
refuses if a hook is not executable — git skips a non-executable hook silently,
which looks exactly like a hook that passed.

---

## 8 · When something has gone wrong

| symptom | first thing to try |
|---|---|
| a number moved and nobody expected it | `vleo run <node>` and read the chain — every input is listed with its own credibility |
| "the committed artefacts differ from what the sheets generate" | you edited outside a hole. `cargo run -p xtask -- docs` and look at the diff |
| a row returns `NotRun` | it has no content yet. `cargo run -p xtask -- status` says how many are like it |
| the gate refuses a fixture | a fixture disagreement is a physics disagreement. Take it to the node owner; do not widen the tolerance |
| a sweep row says `refused` | the value left the declared domain. The message names the bound and its reason |
| the daemon shows the wrong tree | an old process. It is the sheet hash that would refuse a stale page, but a stale *process* has its own copy — check the port |

Two commands answer most of it:

```
cargo run -p xtask -- gate && cargo test     # must be green
cargo run -p xtask -- status                 # what exists, what is blocking
cargo run -p xtask -- gap                    # what every sheet promised and nothing covers
```

---

## 9 · Where to read next

| you want | read |
|---|---|
| why the four rings, and what may depend on what | `docs/ARCHITECTURE.md` |
| who decides what, and which changes need two reviewers | `docs/WORK_MODEL.md` |
| what each agent actually produced when it was first used | `docs/AGENT_EVIDENCE.md` |
| every variable, its unit, its bounds and their reasons | `docs/VARIABLES.md` — generated |
| the sheet field by field, in full | `docs/NODE_AUTHORING.md` |
| what to do when the tool is down | `docs/RUNBOOK.md` |
