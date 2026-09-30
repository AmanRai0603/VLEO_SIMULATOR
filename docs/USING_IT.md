# Using it

> **Answer first.** The walkthrough, with real outputs: run the design on your inputs, keep and send results, ask for a node to change, and — as a developer — take that request to a release. The manual in the tool is the reference; this is the worked tour.
>
> **Kind:** tutorial · **For:** everyone

This is the page to read first. It is about doing the work — opening the tool,
running the design on your inputs, keeping what it said, asking for the design
to change, and, for a developer, taking that request all the way to a release —
not about how the system checks itself. Where a check matters it is mentioned in one line, at the moment
you would actually meet it.

Everything below was run against this repository. The outputs are real.

**The manual in the tool is the reference; this page is the walkthrough.**
Start the tool and press **? Manual** at the top of the page, or open
`http://127.0.0.1:7777/#manual`. It lists every task, in the browser and in the
terminal, for a user and for a developer, with each command ready to copy, and
it says what cannot be done by hand and what to do instead. It is tested
against the code, so a command or button it names exists. This page
does something the manual does not: it drives the tool through worked examples
and shows what it printed, and it explains why the tool behaves as it does.
Where the two differ on a name or a flag, the manual is the one that is checked.

---

## 1 · Ten minutes

```
cargo run --release -p vleo-daemon        # the tool, at http://127.0.0.1:7777
```

Open it. You get the tree on the left and a panel on the right, and **? Manual**
in the row of tabs above them. Four layers:

| layer | what it holds | rows | of those, written |
|---|---|---|---|
| 1 | management — the programme's own view | 174 | 0 |
| 2 | the system — what the spacecraft must do | 321 | 7 |
| 3 | subsystem — sixteen subsystems and two additions | 901 | 313 |
| 4 | the run — what a single evaluation produced | — | — |

`cargo run -p xtask -- status` prints that table, and a second one by
subsystem. On 26 September 2026 320 of 1396 rows carry content and 1076 are
seeded shape waiting to be filled — which is the state this tool is designed to
be useful in, not a defect. One subsystem is written all the way through, solar
weather — 56 of its 65 rows answer, 2 are still seeded and 7 are retired — and
section 2b drives it.

Each subsystem reaches the layer above through exactly one node — eighteen
`l3_*_interface` rows for the eighteen groups in layer 3, and three customer rows
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
    data   solar-drivers@2026.09.04#54784567db11b868 · solar-weather@2026.09.14#f3557eb8443bfdaa
    kernel 1ad1ae · graph adb684 · case 9663b2 · chain 28e13d
    mode branch · endpoint local-cli
```

Three things there are worth knowing on day one.

**"1 ran, 0 blocked"** is always printed, including the blocked count and the
names. A row that has no content yet does not vanish and does not substitute a
default — it comes back as `NotRun` with its own name on it. Most of the tree
is like that today, which is exactly why the blocked count is never hidden. A
written row whose relation nobody has derived yet does the same, and says so:
`INACTIVE — the relation is stated and never derived`, with what the sheet still
needs. `cargo run -p xtask -- active` lists every row in that state.

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
vleo run sw_ap_design --set sw_storm_design_level=2
  Ap_design = 80.0000 -
```

Sweep it instead:

```
vleo sweep sw_ap_design --over sw_storm_design_level --from 1 --to 3 --points 3

# sw_ap_design against sw_storm_design_level — 3 points
# G_design           sw_ap_design           note
1.00000              48.0000000               ok
2.00000              80.0000000               ok
3.00000              132.000000               ok
# 3 ran, 0 refused. Refusals are recorded, never dropped.
```

The numbers a solar panel draws come the same way — the answer its picture
reads, from the same function, so a figure can be checked or kept from a script:

```
vleo figure density
{"ok":true,"figure":"density","bundle":"solar-weather@2026.09.14","days":10296.0,"r":0.206670332572184,"median_f107":104.0,"median_ap":7.0,"below_both_pct":27.758352758352757,"storm_ap":26.0,"storm_deciles":10.0}
```

From Python it is a dict: `vleo.figure("growth", v="ap", by="cycle")["change"]`.
The figure ids and their keys are in the manual, under `/v1/figures/solar/<id>`.

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

### On the case — its inputs, from the browser or a CSV

There is **one case**, the multipayload design. A customer, or the sky it must
survive, is a set of input values rather than a folder — so a new customer is
never a commit, and the tree does not grow with the order book. Its 128 inputs
come in two halves: the **condition** the design flies in — orbit, environment,
solar weather, listed in `cases/multipayload.toml` — and everything the
**customer** chooses. Each defaults to the value declared on its own sheet.

In the browser, the **Inputs** tab lists every one with its value, default and
range. Change them there, or upload a CSV: the file is shown before anything is
saved, and a file with any row that cannot be applied — not an input, the wrong
unit, outside its range — is refused whole with each bad row named. The saved
case is kept **outside the repository**, in `~/.vleo/case/inputs.csv` or
wherever `VLEO_CASE` points, so git never sees it; download it to keep a copy.
Every run, sweep and figure is on that case, and the run panel says of each
input whether it is the default, the saved case, or your own what-if edit —
which stays in the browser until you save it to the case.

From a terminal the same files work:

```
$ vleo inputs --defaults > my_case.csv      # the template: fill in `value`
$ vleo run sw_ap_design --inputs cases/examples/storm_level_2.csv
  inputs: cases/examples/storm_level_2.csv, 1 changed from default
  Ap_design = 80.0000 -

$ vleo run sw_ap_design --defaults          # the design as declared
  Ap_design = 132.000 -
```

With neither flag a run uses the saved case, and says so on its first lines.
A blank value in a file means the default, so a file only has to name what it
changes; `--set` has the last word over all of it.

Files outlive versions of the tool. Each one it writes carries a `#! template`
line naming the inputs it was written for; when the tree has changed since, the
file is carried over instead of refused:

```
$ vleo run sw_ap_design --inputs cases/examples/from_an_older_tool.csv
  inputs: …, 2 changed from default; carried over from an older version of the
  tool: 126 new input(s) at their default, 1 value(s) set aside
  set aside: retired_payload_mass = 12 kg — is not a row in this tree
  Ap_design = 80.0000 -

$ vleo inputs --inputs old.csv > new.csv    # the same file, in today's template
```

The saved case is carried over by itself the first time an updated tool reads
it: the old file is kept beside it as `inputs.before-<template>.csv`, the new
one records every value it set aside, and `vleo cases` and the Inputs page say
what happened until the case is saved again.

Compare the defaults, the saved case and any case files against a row, or check
that every fixture declaration in the tree is sound — `cargo test` is what
executes them:

```
vleo cases              # the case, and how its inputs divide
vleo campaign l3_solar_ach_01 --inputs cases/examples/storm_level_2.csv
vleo selftest
```

### Keeping what a run said

A run worth remembering is saved as a **result**: every value it returned,
every row it blocked on, and every input it ran on. `--save <file.csv>` writes
it where you say; `--keep` keeps it in the results folder the browser shows —
one folder per result, with its report page — and keeps each question once. A
question already kept there is not run again: `vleo run` and `vleo sweep` show
the saved answer and say which result it is; `--again` runs it anyway.

```
$ vleo run sw_ap_design --inputs cases/examples/storm_level_2.csv --save storm2.csv
$ vleo result docs/examples/sw_ap_design.result.csv
sw_ap_design — a saved result, 2026-09-27T13:29:20Z
  80  credibility 1 of 4, governed by mathematics
  2 ran, 0 blocked · mode branch · chain cd9ab9 · kernel 1ad1ae · graph adb684

  1 input(s) changed from their defaults:
    sw_storm_design_level                2

  2 value(s) returned:
    sw_storm_design_level                             2       cred 1
    sw_ap_design                                     80       cred 1
```

`vleo result` runs nothing: it shows what the run returned when it was saved,
so it reads the same after any release. `--html <file>` writes it as a report
page to send to somebody without the tool; the report carries the CSV inside
it, and uploading either gives the result back whole.

In the browser the same thing is **save this result** under any run, and the
**Results** tab: every result kept on this machine — under `~/.vleo/results/`,
or wherever `VLEO_RESULTS` points, never in the checkout — shown as it was,
compared two at a time with the inputs that differ listed first, downloaded as
CSV or report, or made the case again with **use its inputs as the case**.

---

## 2b · Changing an input and watching the answer move

The worked example, on the subsystem that is written all the way through. Every
number below is what the commands print.

### What you are allowed to change

Only **declared** rows — the ones where a person picked a number. Everything
else works its answer out during the run, so a supplied value would be
overwritten the moment it was evaluated. The tool refuses by name rather than
accepting it quietly:

```
$ vleo run sw_f107_design --set sw_central_expectation=150
vleo: 'sw_central_expectation' is computed from its inputs, so a supplied value
would be overwritten the moment it is evaluated. Set one of the declared
numbers it reads instead — `vleo show sw_central_expectation` lists them.
```

That refusal exists because the silent version is worse: a sweep over a
computed row draws a flat line and reports "0 refused", which reads exactly
like a real result.

`vleo show <node>` lists what a row reads. For the solar design flux the
declared numbers behind it are the mission **epoch**, the mission **duration**,
and today's **flux**.

### The sun moves, so the launch date is an input

```
$ vleo run sw_f107_design
  F107_design = 200.143 -
```

That is the flux a five-year mission opening on 1 January 2027 should be sized
to. Move the launch date and nothing else:

| launch | `sys_mission_requirements_mission_epoch` | `F107_design` |
|---|---|---:|
| 2018-01-01 | `--set ...=568080000` | 228.332 |
| **2027-01-01 (declared)** | `--set ...=852076800` | **200.143** |
| 2032-11-08 | `--set ...=1036800000` | 258.230 |

```
vleo run sw_f107_design --set sys_mission_requirements_mission_epoch=1036800000
```

Fifty-eight solar flux units between two launch dates for the same spacecraft,
because one flies the declining half of cycle 25 and the other flies into the
next maximum. A `Time` crosses the interface in **seconds** whatever unit the
sheet declares it in, which is why the epoch is 852076800 and not 9862.

### It changes how much room the design has

`l3_solar_ach_01` is what the subsystem hands upward: the room the sustained
F10.7 requirement of 260 has left, against the long-window design flux.

```
$ vleo run l3_solar_ach_01 --set sys_mission_requirements_mission_epoch=1036800000
  M_f107_long = 0.376314 -

  the chain behind this number
    l3_solar_req_01                               260.000 -          cred 1
    sw_f107_design_long                           162.158 -          cred 0
    l3_solar_ach_01                              0.376314 -          cred 0
```

At the declared epoch the long-window flux is 104.07 and the margin is 60 per
cent. Slip the launch to late 2032 and the flux is 162.16 and the margin 38 per
cent — more than a third of the room gone from one date, with nothing else
changed. It still passes: none of the five solar closures fails at any of the
three launch dates above, and the tool does not pretend otherwise. Nothing was
tuned to make that happen and nothing hides it.

### Mission length is not monotone, and that is the point

```
$ for y in 0.5 2 5 10 15; do vleo run sw_f107_design --set sys_mission_requirements_mission_duration=...; done
```

| mission | `F107_design` |
|---|---:|
| 0.5 yr | 162.547 |
| 2 yr | 188.326 |
| 5 yr | 200.143 |
| 10 yr | 181.080 |
| **15 yr** | 220.727 |

A longer mission is not reliably a worse sky. Ten years is quieter than five
because the window averages across a whole cycle; fifteen is worse again
because it reaches into the next maximum. A tool that fitted a rising curve
through these would be smoothing away the one feature a mission planner is
paid to notice.

### The same thing with a picture

`cargo run --release -p vleo-daemon`, then **Solar weather** beside the tree.
Eleven tabs over the record the rows argue about, every control recomputing from
the bundle:

- **Design** — the return curve against what the vehicle is built for, with the
  crossing marked. That crossing is `sw_design_safe_duration`, and the picture
  and the row are reading the same engine.
- **Forecast → by calendar year** — how the published outlook scored, per year.
  The line breaks at years too thin to score rather than joining across them.
- **Repeatability** — every cycle stacked on phase, with cycle 25 dashed
  because it is still running.

Open any solar row from the tree and its node page carries the same argument in
eleven tabs, including **the relation, moving** and **de-risking**: the input crosses its declared
domain, the answer moves, and the guards are drawn as the walls they are. That
animation is the engine's own sweep, so a picture that disagrees with the row
is impossible.

---

## 3 · A change to the design, from form to release

Nobody changes the design from the tool. The person who knows what a node should
say — a payload team, a customer's engineer, a reviewer — fills in **that
node's form** and sends it to the developers, who check it, apply it, implement
it and release it. This section follows one form through, first from the side
that fills it and then from the side that maintains the repository.

### 3.1 The form goes out

In the tool, every node's page has *the node form* tab, and the **Forms** tab
has every form in one place, including **the form for a new node**. From a
terminal:

```
$ cargo run -p xtask -- form sw_ap_design --out sw_ap_design.node-form.html
$ cargo run -p xtask -- form --new --out new-node.node-form.html
```

It is one HTML file that opens anywhere with no connection. It asks every
question in §3.5 with why each is asked, shows what the node reads and feeds and
the known values that already hold it, lists every row it could read — typing an
input's row shows what that row is and its quantity — and saves a filled copy of
itself. The form for a new node also asks where it goes (its parent, and so its
layer) and what kind of row it is. It can be filled by hand or given to an
assistant: the content is a plain block of text near the end.

The person who filled it sends the file back. They can check it first on the
Forms tab — the same checker as below, writing nothing.

### 3.2 It comes back: check it

A developer's first act is the checker. It writes nothing.

```
$ cargo run -p xtask -- intake docs/examples/sw_ap_design.node-form.html
sw_ap_design — a node form filled by A. Example (Payload team), 2026-09-27; assistant: none
  node.toml is still the version the form was made from

  APPLY    note                         «…» → «What the vehicle is built to survive, …»
  APPLY    assumption 5 · added         «» → «text = The storm level is read on the NOAA G scale, …»
  APPLY    de-risking · version 2       «» → «model — Adds the assumption that the level is a NOAA G-scale level, …»

why it is changing — the decisions it moves: model
  what did we believe                      That the storm level needed no scale named: …
  what did we test                         Compared the row's thresholds with the NOAA Space Weather Scales …
  what do we now know                      The thresholds are the NOAA scale's exactly, but nothing on the row …
  …
  → recorded as version 2, released with the next release

still for the developer to settle:
  · known value 1 cites 'NOAA Space Weather Scales, the G2 row: Kp 6, ap 80', which is not in sources/sources.toml — …

3 change(s) can be applied, 0 cannot.
…
[[fixture]]
label = "G2 storm"
expect = 80
tolerance = 1e-6
provenance = "published-source"
source = "NOAA Space Weather Scales, the G2 row: Kp 6, ap 80"
inputs = { g_level = 2.0 }
# the form gave the inputs as: sw_storm_design_level = 2
```

The assumption is a decision about the model, so the form had to say why it
changes — and it does, so the change goes in with its version. Without the
*why it is changing* answers, intake would apply only the note and name the
assumption as withheld (§3.11).

```
$ cargo run -p xtask -- intake docs/examples/new-node.node-form.html
a new node — sw_ap_design_margin under l3_solar, a computed row — a node form filled by A. Example (Solar team), 2026-09-27; assistant: none

  APPLY    new · id                     «» → «sw_ap_design_margin»
  APPLY    new · parent                 «» → «l3_solar»
  APPLY    new · kind                   «» → «computed»
  APPLY    label                        «» → «Ap design margin over the storm level»
  …
  APPLY    input 1 · added              «» → «binding = ap, var = sw_ap_design, type = Ratio»
  APPLY    algorithm 1 · added          «» → «text = Divide the design Ap by the storm level., binds = margin, type…»
  APPLY    de-risking · version 1       «» → «sw_ap_design is the design Ap at the declared storm level, so their r…»

interfaces — what each input reads:
  connects ap               ← sw_ap_design                       Ratio in One

why it is changing — the decisions it moves: node
  …
  → recorded as version 1, released with the next release

still for the developer to settle:
  · the relation cites 'derived here from sw_ap_design', which is not in sources/sources.toml — …

17 change(s) can be applied, 0 cannot.
```

Intake compares three versions — the node when the form was made, what the
filler made of it, and the node now — so a field the repository changed
meanwhile is a CONFLICT, never overwritten. **Every interface is checked**: an
input must name a row that exists (or a named output of one), of the quantity
the node expects, and one that does not says what the row it names actually is.
For a new node the id must be free and the parent a group. If the filler says an
assistant helped with the relation, the relation, its steps and its derivation
are REFUSED: a person derives them. A numbered step removed from the middle is
refused, because each number is a hole holding somebody's Rust.

A form that does not pass goes back to whoever filled it with those lines. It is
never fixed up on the way in. What is *still for the developer to settle* does
not stop it: a work cited that `sources/` does not list yet is added there
before the row is published (the gate refuses it then, V8), and a known value
is printed as the `[[fixture]]` it would be — the answer converted from the
node's unit to SI, the inputs to the node's own binding names — ready for
§3.9, or the inputs left as a comment where one does not say its unit.

### 3.3 Apply it

```
$ git switch -c node/sw-ap-design-margin
$ cargo run -p xtask -- intake docs/examples/new-node.node-form.html --apply
```

For an existing node `--apply` writes `node.toml`. For a new node it builds the
folder in its place in the tree, on the shape of a sibling of the same kind —
the closest one: under the same group, and with the same id prefix where there
is one — deliberately not a copy, because a copy drags a stale source citation
through thirty rows. What the form does not ask (subsystem, owner, criticality,
tier) it takes from that sibling and lists for you to confirm; what is the
sibling's own (its history, its risks, the KPIs it contributes to) it does not
take at all. It writes the form into it, makes room in the tree's order (every
later row's `order` moves up one, which regenerates nothing), and gates the
**whole tree**, because a new row changes what the tree connects. Either way it regenerates and gates as
one edit, or puts everything back. It stamps your name on any relation it
changes, and refuses while anything is blocked unless you add `--partial`.
Known values in the form are printed as `[[fixture]]` blocks for §3.9; they are
never written by intake. Anything the form left blank is printed as still open.

Then `git diff`. From here on the node is the developer's, and the rest of this
section is the loop you will live in.

A row is a folder. Only one file in it is written by hand — by intake, or in
your editor:

```
crates/vleo-mod-prop/nodes/prop_throat_area/
  node.toml      ← the sheet. what a form writes.
  model.rs       generated, except the numbered holes
  contract.rs    generated
  evidence.rs    generated — the fixture tests, plus three properties
  fixtures.toml  the known-good values
  meta.json      generated
  mod.rs         generated
  page.html      generated
```

A developer's own structural row, with no form behind it, still starts from a
sibling: `cargo run -p xtask -- new <id> --like <sibling>`, which blanks every
field that must be decided again.

### 3.4 Ask what is still open

```
cargo run -p xtask -- declare prop_intake_mouth
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

### 3.5 What the sheet holds

Seven things, and they are all questions a person has to answer — which is why
they are the form's questions, and why a gap left in the form is still open
here:

| field | what it is | why it is required |
|---|---|---|
| `question` | the one question this row answers, in a sentence | an equation with no question gets reused for the wrong thing |
| `expression` | the relation, as written in the source | so a reviewer can compare it against the paper |
| `source` | a key in `sources/` | this is the claim everything else rests on |
| `symbol`, `type`, `unit` | the answer's identity | a unit mismatch becomes a compile error, not a runtime surprise |
| `lower`, `upper` | where the relation is valid | outside it the answer is a refusal, by name |
| `reason_lower`, `reason_upper` | why each bound is there | a guard with no written reason gets deleted by whoever next finds it awkward |
| `value` + `confirmed_by` | for a declared number: who picked it | every margin in the design is built out of these |
| `[maths] confirmed_by` | who supplied the relation | an assistant may never supply mathematics, and without a name nothing can tell whether one did |

Two more that are decisions rather than drafting:

- **`criticality`** — `minor` or `significant`. Significant means two reviewers
  and the hole filled twice by different model families. The default is minor;
  raising it is done on purpose, because a person asked to approve too many
  things stops evaluating each one.
- **`migrated_from`** — set it when the node exists in the MATLAB tool. Its
  numbers then go in `parity.csv` beside the node and never in `fixtures.toml`.

### 3.6 Publish and generate

A seeded row, filled, becomes published — and that is what makes its code
generated:

```
cargo run -p xtask -- publish prop_intake_mouth
```

It refuses, naming every reason, while the row is not ready. After that, any
change to the sheet is regenerated with:

```
cargo run -p xtask -- docs prop_intake_mouth
docs: 1 node(s), 6 artefact(s) written
```

Six generators run, and none of them reads another row — which is what makes
1396 rows 1396 independent pieces of work rather than one large one.

While any field is still open it refuses instead, names them, and writes
nothing:

```
  refused prop_intake_mouth — nothing to generate from: label, question,
          expression, source, reason_lower, reason_upper
```

That refusal is the mechanism. It turns ambiguity from something an implementer
settles quietly into a blocking item on an engineer's screen.

### 3.7 See what the gate says

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

### 3.8 Write the maths

You do not open `model.rs`. The body of each numbered hole goes in as text:

```
echo 'let e: Length = gnc::along_track_error_from_drag(
    Acceleration::new(a.get() * s.get()), Time::from_days(1.0));' \
  | cargo run -p xtask -- fill prop_intake_mouth --hole 1 --body - --by "A. Developer" --model <model>
```

`fill` is the only thing in this system that puts text into a generated file.
It refuses, before writing anything: a hole the sheet does not declare, a body
carrying its own `HOLE` marker, a guard, an early return, and a platform maths
call. Then it re-reads the file and proves the body landed.

So an assistant can write the body — give it the sheet and the hole, take back
the lines — and never needs the file. An assistant handed the file and told not
to stray is not constrained, it is asked, and the same applies to a person in a
hurry. `--by` and `--model` record who wrote the body and with what (§4b).

What the hole looks like once it is in:

```rust
pub fn evaluate(a: Acceleration, s: Ratio) -> Result<Length, Fault> {
    // ---- HOLE 1 : propagate the drag acceleration error over one day -> Length
    let e: Length = gnc::along_track_error_from_drag(
        Acceleration::new(a.get() * s.get()), Time::from_days(1.0));
    // ---- end HOLE 1
```

Everything outside the markers is regenerated, so an edit there is discarded
the next time anybody runs `docs`, and the gate's regeneration diff catches it
before that happens, by anyone.

The signature is already correct. `Acceleration`, `Ratio` and `Length` are
distinct types, so adding a mass to a length does not compile. A hole body is
usually two or three lines that compose relations already in `vleo-core`.

### 3.9 Get evidence

A number the code produced is not evidence that the code is right.
`fixtures.toml` holds values from somewhere else — a paper, a measurement, a
MATLAB function somebody trusts, or the known values a form supplied, with where
they came from:

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

### 3.10 Ask whether a person should look yet

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
ready: 16 of 320 node(s) have passed every machine stage and are waiting on H2
304 held, by what is holding them:
    297  the relation has nobody's name against it
    246  other
    124  no fixture — nothing outside this code has agreed with it
      7  significant, with fewer than two checks behind it
```

Then commit, naming whoever filled the form. The message form is checked (§7).
After review and merge, the next release carries the node to the team — and
their saved case carries over on its own, with any new input at its default.

### 3.11 Why it changed, and the release that ships it

If the form moved what the node computes, intake appended a `[[version]]` to
the sheet — what we believed, what we tested, what we now know, what changed,
what the node rests on now and what would break it — marked `next`. Without
that record, intake applied only the form's wording and named every decision
it withheld. `sw_central_expectation` carries three real versions as the worked
example; its *de-risking* tab reads them newest first, and the *Technical
risk* row under the risk register shows the risk they moved.

```
cargo run -p xtask -- derisk            # docs/DERISK_NARRATIVE.md and docs/derisking.csv
cargo run -p xtask -- release 0.2.0     # every `next` stamped 0.2.0; the workspace set to 0.2.0
cargo run -p xtask -- release 0.2.0 --check
```

The release pipeline runs the last line and refuses while any version is still
`next`. A result saved before the release keeps the versions it ran on, and
from then on says which of its beliefs have broken since. The rules are in
`docs/DERISKING.md`.

---

## 4 · Using an assistant

A developer may use any assistant, for anything a developer does — once the form
has passed the checker. There is no roster of specialised agents, each with its
own lane: the rules are held by the checks, and the checks apply to an
assistant's change exactly as to anyone's.

| the step | what an assistant can do | what holds it |
|---|---|---|
| filling a form | help whoever fills it with the words — the form asks whether it did | intake refuses the relation, its steps and its derivation from a form an assistant helped with |
| a hole body (§3.8) | return the few typed lines, as text | `fill` is the only way into `model.rs`, refuses a guard, an early return or a platform maths call, and records `--by` and `--model` |
| evidence (§3.9) | turn a value a person derived into a `[[fixture]]` with its provenance | the schema refuses `self-snapshot` and `agent-generated`: an expected value may never come from the code under test |
| a relation's name | nothing | relation stamping and `confirm` refuse a name that is an assistant's |
| the tool itself | ordinary engineering — generators, daemon, faces, tests | the gate, `cargo test`, the regeneration diff and review, as for anyone |

They are ordinary help, not oracles. Three things are worth doing every time:

1. **Read the diff.** `git diff` after an assistant's change, before the gate,
   so you know what the gate is being asked about.
2. **Look at what it did not do.** A fixture that covers one edge of a node with
   two bounds has not looked at the other.
3. **Check a test can fail.** A test that passes against a deliberately broken
   implementation is not testing anything. Break the hole body, run the test,
   see red, put it back. It takes a minute and it is the difference between
   evidence and decoration.

---

## 4b · The rest of the commands

Everything `xtask` does that the walkthrough above did not need. All of it is
reporting or regeneration; none of it decides anything.

| command | what it does |
|---|---|
| `cargo run -p xtask -- assemble` | the three assembly generators — the index, the document fragments, the graph tables |
| `cargo run -p xtask -- status` | counts by layer and by subsystem, and what is blocking |
| `cargo run -p xtask -- active [<subsystem>] [--defined]` | which rows answer, which are undefined, and which are blocked by a named row; `--defined` names every function that carries a derivation |
| `cargo run -p xtask -- reach [<subsystem>]` | where each answer goes — how many reach a KPI closure, and which answer and are read by nothing |
| `cargo run -p xtask -- gap` | what every sheet promised and nothing yet covers |
| `cargo run -p xtask -- graph` | the three graphs, their sizes, the deepest chain, and the crate-direction check |
| `cargo run -p xtask -- variables` | regenerate `docs/VARIABLES.md` — every variable, its unit, its bounds and the reason for each |
| `cargo run -p xtask -- codeowners` | regenerate `CODEOWNERS` from the owner field on each layer group |
| `cargo run -p xtask -- bundle publish <dir>` | hash a bundle's payload and write the result into its manifest |
| `cargo run -p xtask -- bundle verify` | re-check every hash in `bundles/` |
| `cargo run -p xtask -- setup` | point git at `tools/githooks` on this clone |

`variables` and `codeowners` write files that are committed, so run them after
a change that moves a bound or an owner, and commit what they produce. The
regeneration diff in the pipeline catches it if you forget.

### Putting a name against a relation

`cargo run -p xtask -- confirm --list` prints every written relation with nobody's
name against it, grouped by the owner who owes one. On 26 September that is
**183 of the 185 published rows that carry a relation** — every declared value
already carries a confirmation and only two relations do, which is why `ready`
holds almost the whole written tree.

`cargo run -p xtask -- confirm <node> --by "<your name>"` puts one there. It
prints the question, the relation, the source, the assumptions and the declared
range first, because a name put against a relation nobody re-read is a keystroke
rather than a confirmation. Then it writes `<name> / <today>` into `[maths]` and
reads the sheet back through the loader to prove it landed.

Three things it refuses, and the first is the point of the field:

- **a name that belongs to an assistant.** An assistant may never supply
  mathematics, and this field is the only thing that can tell whether one did.
  The refused identities are one list in `vleo-sheet`, shared by `confirm`,
  intake and relation stamping, so the three cannot disagree.
- **a declared value.** Its confirmation lives under `[value]` and it already
  has one.
- **a relation that is already confirmed.** Changing an attribution is a review
  decision, not a command.

There is no flag that confirms many at once, and that absence is deliberate: a
person asked to approve thirty things at a keystroke is not approving any of
them. One relation, one reading, one name.

### Two commands that ask whether the evidence is real

Everything else in the gate proves the tests pass. These two ask the other
question, and it is a different one: a test that passes against a wrong number
proves nothing, and there is no way to tell the two apart by reading.

`cargo run -p xtask -- mutate [<node>]` moves the node's answer by twice its own
loosest fixture tolerance and requires the node's tests to notice. The size is
taken from the node rather than fixed, because a fixed perturbation asks an
arbitrary question — a first run at a tenth of a percent reported five rows as
unevidenced whose fixtures declare half a percent, which was a finding about the
number chosen and not about those rows. A survivor with a fixture is a finding:
something claims to check this and does not. A survivor with no fixture is a gap
already counted against that row, and is reported as a count rather than named.

Over the whole tree on 26 September: 183 rows are mutated, and all 59 that
have a fixture are killed. The other 124 have no fixture. Two rows with several
outputs, `l3_solar_interface` and `sw_kp_scenarios`, cannot be perturbed this
way and are named as such. Nothing in the tree has evidence that fails to catch
an error larger than the evidence's own claim.

`cargo run -p xtask -- differential <node>` runs every body recorded for a hole
against that node's evidence. Bodies are recorded by `fill --by <who> --model <model>`, which
refuses a second body for a hole from the model that wrote the first: two bodies
from one model are one body written twice, because a model handed its own
reasoning to check approves it. A significant node refuses an unattributed body
outright. If two recorded bodies disagree, that is a finding for the node owner
— at least one reading of the sheet is wrong, or the sheet says less than its
author thought — and never something to settle by keeping the body that passes.

What neither closes is the vendor half of the rule: `--model` separates models,
not training, so two models from one vendor count as two. Closing it is a
developer's choice of assistants, not more code.

---

## 5 · What runs without being asked

| when | what | where |
|---|---|---|
| an assistant's session opens | the tree's state, what is blocking, whether reference data is present | `.claude/hooks/session-start.sh` |
| you save a sheet or a fixture file | the gate on that node, refusing the edit if it fails | `.claude/hooks/post-edit.sh` |
| you save any `.rs` file | formatting | same file |
| you write a commit message | the message form | `tools/githooks/commit-msg` |
| every pull request, and every push to `main` | build, regenerate, gate, test, both profiles, no-std | `.github/workflows/gate.yml` |
| every pull request, and every push to `main` | an advisory review that cannot fail the build | same file, `review` job |
| every night | six passes over the whole tree, the ledger, and yesterday's state | `.github/workflows/nightly.yml` |
| weekly | a dependency bot, on its own branch, majors excluded | `.github/dependabot.yml` |
| on a tag | prove, build, and one human approval | `.github/workflows/release.yml` |

A hook is not a control — it only fires inside the tool that installed it. Every
rule above also runs in the pipeline, from the same file, which is why the two
can never drift apart.

One gap in that is deliberate. A push to a branch with **no pull request open**
runs nothing in the pipeline — the local gate is the only check until a pull
request exists, and from then on every push to it runs the whole thing. The
pipeline used to run on every push to every branch, which meant each commit was
tested twice, once for the push and once for the pull request carrying the same
sha, at roughly seventeen billed minutes a time. That doubling ran the
repository out of its monthly Actions allowance on 15 September 2026, and jobs
stopped being given a runner at all. The rule to take from it: nothing merges
without a pull request, so testing what cannot yet be merged bought nothing and
cost half the budget.

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
| a row is written and still does not answer | its relation is stated and never derived. `cargo run -p xtask -- active` says which rows are in that state and how many others are waiting on each |
| a subsystem looks finished and nothing seems to use it | `cargo run -p xtask -- reach` — a subsystem can answer on every row it has and be wired to nothing. It names the crossing and who reads it |
| the gate refuses a fixture | a fixture disagreement is a physics disagreement. Take it to the node owner; do not widen the tolerance |
| a sweep row says `refused` | the value left the declared domain. The message names the bound and its reason |
| a form's check says CONFLICT | the design changed that field after the form was drawn. Nothing is overwritten: send a fresh form with the answer carried across |
| a form's check says an interface does not connect | the input names no row, or a row of another quantity. The line names the row and what it actually is |
| an uploaded case or result is refused | every refused row is named with why. A case is all or nothing; a file from an older release is carried over rather than refused |
| the daemon shows the wrong tree | an old process. It is the sheet hash that would refuse a stale page, but a stale *process* has its own copy — check the port |

Two commands answer most of it:

```
cargo run -p xtask -- gate && cargo test     # must be green
cargo run -p xtask -- status                 # what exists, what is blocking
cargo run -p xtask -- active                 # what answers, and what is in the way
cargo run -p xtask -- reach                  # where the answers go, and which go nowhere
cargo run -p xtask -- gap                    # what every sheet promised and nothing covers
```

---

## 9 · Where to read next

| you want | read |
|---|---|
| how to do any one thing, in the browser or the terminal, and what cannot be done by hand | **? Manual**, in the tool — source `docs/manual.toml` |
| why the four rings, and what may depend on what | `docs/ARCHITECTURE.md` |
| who does what — the team and the developers, and what crosses between them | `docs/WORK_MODEL.md` |
| one form from arrival to release, and a developer's first day | `docs/RUNBOOK.md` |
| every variable, its unit, its bounds and their reasons | `docs/VARIABLES.md` — generated |
| the sheet field by field, in full | `docs/NODE_AUTHORING.md` |
| what to do when the tool is down | `docs/RUNBOOK.md` |
