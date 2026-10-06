# De-risking: why the design is what it is

> **Answer first.** Every decision in this tool — a node, an input, an output, a model, the
> mathematics, an algorithm, a picture — rests on a belief, and it changes only when that belief
> breaks. So every change records what we believed, what we tested, what we now know, what it
> cost, what changed, and which risks it opened or closed; every version of a node says what it
> rests on and what would break it; and the risk-register rows of the management layer conclude,
> from all of it, how the work below has bought the programme's risks down.
>
> **Kind:** explanation + how-to · **For:** everyone

---

## Said simply

A spacecraft design is a stack of bets. *The drag coefficient is bounded well enough. The solar
flux this mission meets is about this much. This picture shows the margin honestly.* Each bet is
fine until something tests it. When a test says a bet was wrong, the design changes — and the
useful thing to keep is not only the new design but **why the old one was replaced**: what we
thought, what we checked, what we learned. That record is what lets the next person trust the
design, and what lets the programme see its risks going down.

## Now the real thing

### D1 · Every row says what it rests on

A node's `[[version]]` record starts with its first version: what the node **rests on** and what
**would break it**. A published row with no record is an open gap — shown on its page, listed by
the gap pass, and held back from review — because a row that cannot say what would make it wrong
has not been thought about enough to be relied on.

### D2 · A change says which belief broke

A form that changes what a node computes — any field that moves a decision — must carry the
record, one row of the programme's quarterly de-risking narrative:

| column | the question |
|---|---|
| **What we believed** | the belief the node rested on before |
| **What we tested** | the analysis, comparison, measurement or review — and where it is written down |
| **What we now know** | the result: what was wrong or incomplete in the previous version |
| **What it cost** | time, money or effort *(optional)* |
| **What changed** | what this version does differently, and the benefit |
| **Risks opened / closed** | `R-01 L5->L4`, `R-09 closed`, `R-12 opened` *(optional)* |
| **Rests on now** · **Would break if** | the new belief, and what to watch for |

The kind of decision is not asked: intake reads it off what the form changes — an input, the
output, the model (source, assumptions), the maths (the relation), the algorithm, the
visualisation. Wording — the note, the plain-words explanation, the prose of the derivation — is
not a decision and needs no record, because a sentence that needs paperwork to correct never gets
corrected.

Without the record, intake applies only the wording and **withholds every decision, by name**.
With it, the changes go in and a new `[[version]]` is appended to the sheet.

### D3 · A version is a record, and a release names it

Versions count 1, 2, 3 and are never edited. Each keeps the relation and source it stated, so an
earlier version can be read after the sheet has moved on. A version is `release = "next"` until a
release is cut: `cargo run -p xtask -- release <x.y.z>` stamps every unreleased version with the
release that ships it and sets the tool's version to match, and the release pipeline refuses to
ship while any version is unstamped. So a node's page says, for each belief, which release first
carried it.

### D4 · A risk is registered once, and moved only by work

A risk is registered on one of the four risk-register rows of the management layer — technical,
schedule, supply, regulatory — with an id, what could go wrong, a level from L1 to L5, an owner,
and what it would cost. After that it moves only when a node version says so: a risk is reduced
because something was tested, and the version that tested it is the record of how. The gate
refuses a risk registered twice or anywhere else (V17), and a move naming a risk nobody
registered (V18).

**The conclusion.** Each risk-register row's page lists its risks, where each was registered,
where it stands now, and every version across the tree that moved it, with what was learned —
how the work below, taken together, has de-risked the programme.

### D5 · A result keeps the beliefs it rests on

A saved result records the version of every row it ran through. When one of those rows moves to a
new version, the result says so — in the Results view, the report and `vleo result` — because a
number computed on a belief that has since broken is not the number it looks like. A row that had
no recorded belief when the result was saved counts too: its first version is a belief the result
rested on without anyone having written it down. (A result saved before the tool recorded versions
at all cannot say, and does not.)

### D6 · The narrative is generated, never written

`cargo run -p xtask -- derisk` writes `docs/DERISK_NARRATIVE.md` — every recorded change by
release, and every risk as it stands — and `docs/derisking.csv`, the same rows in the columns of
the quarterly de-risking narrative, for a spreadsheet. The pipeline regenerates both and fails if
the committed copy differs.

## Where the simple version breaks

A record is only as honest as the person writing it: the tool checks that every column is filled,
that the risk ids exist and that the version numbers run in order — not that *what we tested*
really tested the belief. That is the reviewer's job, and it is why a change still goes through
the two reviews. And a risk level is a judgement, not a measurement: the tool moves it only where
a version says so, but the level itself is the owner's call.

## Common wrong idea

*That the record is paperwork added after the engineering.* It is the engineering's conclusion
written down: a change that cannot say what it learned has not yet learned it, and a design whose
history is only its current state cannot tell a reviewer which of its beliefs have ever been
tested.

## Try it — the worked example

`sw_central_expectation` (the centre of the F10.7 design value) carries its real history, taken
from the commits and the port record that made each change:

- **v1** rested on the record's long-term mean, 114.84 sfu, being the right centre for any mission.
- **v2** — *tested* against the record by cycle phase; *learned* the constant was 26 per cent high
  for a five-year mission from 2027; *changed* to the cycle analogue over the mission's own dates.
  Moved R-01 L4 → L3.
- **v3** — *tested* the wrap against the completed cycles' own peaks; *learned* cycles repeat
  their shape and not their size, and a grid that did not divide the period left a 1.2 sfu step;
  *changed* the amplitude and the grid. Moved R-01 L3 → L2.

Open the row in the tool and read its *de-risking* tab; open *Technical risk* under the risk
register and read the same history as the risk's conclusion; run
`cargo run -p xtask -- derisk` and read `docs/DERISK_NARRATIVE.md`.

---

## How to

**As a node engineer.** Fill the node's form as usual. If your change moves what the node computes,
fill *Why it is changing* — the form says as you type which decisions your changes move and what
is still missing. For a new node, say what it rests on and what would break it. To register a
risk, fill the form of the risk-register row it belongs to.

**As a developer.**

    cargo run -p xtask -- intake <form.html>          # the record is checked with everything else
    cargo run -p xtask -- intake <form.html> --apply  # appends the [[version]]
    cargo run -p xtask -- derisk                      # the narrative, regenerated
    cargo run -p xtask -- release 0.2.0               # stamps `next`, sets the version
    cargo run -p xtask -- release 0.2.0 --check       # what the release pipeline runs
