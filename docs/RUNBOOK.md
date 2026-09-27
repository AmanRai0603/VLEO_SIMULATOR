# Runbook

> **Answer first.** A developer's first day, then one node form taken from arrival to release — check, apply, publish, implement, evidence, gate, review — and what to do when the gate refuses.
>
> **Kind:** how-to · **For:** developers

Written from a node that was actually built, not from a specification. Every
step below was performed.

## Day one — the environment nobody configures

    git clone <this repository> && cd vleo_simulator
    cargo xtask setup
    cargo run -p vleo-cli --bin vleo -- data sync
    cargo run -p xtask -- gate && cargo test

`setup` points git at `tools/githooks`; git will not follow a committed hooks
path on its own, because a hook that ran because it was cloned would be
arbitrary code from a pull request. The authoring commands warn until it is
done.

If that is not green on the first try, nothing else matters. The toolchain is
pinned in `rust-toolchain.toml` and the container in `.devcontainer/`, because
otherwise day seven fails on an environment difference and gets logged as a
template defect.

Then:

    cargo run --release -p vleo-daemon
    # open http://127.0.0.1:7777

## One form, arrival to release

Almost every change to the design arrives as a filled node form from somebody
on the team — `docs/examples/` has two, one for an existing node and one for a
new one. Start by checking it; nothing is written by checking.

    cargo run -p xtask -- intake sw_ap_design_margin.node-form.html
      APPLY    new · id       «» → «sw_ap_design_margin»
      APPLY    new · parent   «» → «l3_solar»
      …
      interfaces — what each input reads:
        connects ap  ← sw_ap_design    Ratio in One
      16 change(s) can be applied, 0 cannot.

A **conflict** means the design changed under the form since it was drawn; a
**refused** relation means the form says an assistant supplied it; an interface
that does not connect names the row and what it actually is. Any of those goes
back to whoever filled it, with those lines — it is never fixed up on the way
in. When it is clean:

    git switch -c node/sw-ap-design-margin
    cargo run -p xtask -- intake sw_ap_design_margin.node-form.html --apply

For an existing node this writes `node.toml`; for a new one it builds the
folder in its place in the tree, on the shape of a sibling of the same kind —
not a literal copy, because a copy drags someone else's source citation and
domain limits along — and gates the whole tree. Either way it regenerates and
gates as one edit, or puts everything back. What the form left blank is printed
as still open.

    cargo run -p xtask -- gate sw_ap_design_margin
      ok    schema
      ok    inputs
      note  gap-pass — no fixture: nothing outside this code has agreed with it

**Review 1**, in two stages, by someone who is not the author:

- **H1a — completeness.** Is every question answered, does every declared limit
  have a reason, does the interface close, is a source cited at all? Any
  competent engineer, about ten minutes.
- **H1b — physics.** Is the relation right, is the source the right source, is
  the declared range honest? A domain engineer, about twenty-five minutes.

Splitting it is the only lever that moves the schedule without hiring, because
roughly half the review load leaves the person who cannot be duplicated.

    cargo run -p xtask -- publish sw_ap_design_margin    # the code is generated

Then fill the numbered `HOLE` blocks in `model.rs`, through `xtask fill` — by
hand or with an assistant, recorded with `--by` and `--model`. A few typed lines
each. The signature, the unit types, every guard with its reason, the fault
construction and the ordering are already generated. Record the known values the
form supplied in `fixtures.toml`, with where they came from — intake prints them
as `[[fixture]]` blocks and never writes them.

    cargo test -p vleo-mod-solar
    cargo run -p xtask -- gate sw_ap_design_margin

**Review 2 (H2)** — fixtures, their provenance, and the filled holes. By then
the node has already survived independent machine verification, so the person
accepts rather than hunts. That is what makes two reviews per node affordable.

    commit naming whoever filled the form   →   merge   →   release

Two reviews. Everything between them is a command. If a node takes materially
longer, the template has a defect, and it is worth finding: it will be paid 1396
times. The team gets the node in the next release; their saved case carries
over on its own, with any new input at its default.

A row a developer starts without a form — rare, and usually structural — still
starts with `cargo run -p xtask -- new <id> --like <sibling>` and goes through
the same gate and reviews.

## When the gate refuses

    FAIL  fixtures   2 of 3 rows pass
      row 2: expected 2.645, got 2.641, tolerance 1e-3
      source: Singh & Rao 2019, table 4, "200 km nominal"

**A physics disagreement, not a build failure.** Take it to the node owner. Do
not widen the tolerance — the commonest way a gate stops meaning anything is
somebody widening one to get green, so a tolerance change is a gate change and
needs two reviewers.

## Bringing the legacy MATLAB across

There is eighteen months of working MATLAB, and most of the computed rows exist
there in some form. It runs into this system's own rule: an expected value may
not come from the implementation being tested, and the MATLAB *is* an
implementation.

1. Take the next node in the thread, not the next file. Order by thread.
2. Fill the sheet **from the source the MATLAB was built from** — the paper, the
   textbook, the measurement — never from the MATLAB.
3. Export the MATLAB output over a grid into `parity.csv` beside the node. A
   **second opinion**, never a fixture.
4. Fill the holes. The gate runs fixtures from the source *and* the parity grid.
   `migrated_from` records the function and line.
5. A disagreement is a finding. Either the Rust is wrong or the MATLAB was, and
   both classes have been found before.

That turns eighteen months of existing work from a liability under the
provenance rule into the strongest verification asset the programme has.

## Choose the first thread so it closes a loop

Fill by thread, not by subsystem. Six months of filling by subsystem leaves
hundreds of nodes and a tool that computes nothing anyone asked for.

    density → drag → intake → thrust → margin        ~20 nodes, one closure answer
      + its requirement rows, upward                 +10, the management layer's first real number
      + a second thread sharing nodes with the first +15, the first real composition

The blocked count on the Run control is the early warning: if it is not falling,
the order is wrong.

## Where to explore without polluting anything

| to | do it in | what keeps it clean |
|---|---|---|
| try a physics idea before declaring a node | a notebook against the wheel | nothing enters the repository until it is a declaration |
| sweep a design and look at the shape | the interface, or `vleo sweep` | runs carry an exploration channel |
| keep what a run said, to compare later | *save this result*, or `vleo run … --save` | results are kept under `~/.vleo/results/`, never in the checkout |
| build a fixture the kernel must match | MATLAB — this is its best job | the fixture is committed as a golden vector, with its provenance |
| try a whole alternative architecture | a long-lived spike branch | never merged; findings return as a declaration |

## Two questions every month, above every metric

**How many nodes exist?** If the answer is "we have been improving the
generator", the drift already happened and every week felt productive.

**Is the review still changing anything?** If the two reviews have stopped
producing corrections, that is far more likely to be the review thinning than
the work improving.
