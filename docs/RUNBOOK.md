# Runbook

Written from a node that was actually built, not from a specification. Every
step below was performed.

## Day one — the environment nobody configures

    git clone <this repository> && cd vleo_simulator
    cargo run -p vleo-cli --bin vleo -- data sync
    cargo run -p xtask -- gate && cargo test

If that is not green on the first try, nothing else matters. The toolchain is
pinned in `rust-toolchain.toml` and the container in `.devcontainer/`, because
otherwise day seven fails on an environment difference and gets logged as a
template defect.

Then:

    cargo run --release -p vleo-daemon
    # open http://127.0.0.1:7777

## One node, start to merge

    cargo run -p xtask -- new prop_intake_throat --like prop_capture_efficiency
    git switch -c node/prop-intake-throat
    $EDITOR crates/vleo-mod-prop/nodes/intake_throat/node.toml   # the physics

`new --like` clones the *shape* of a sibling and blanks what must be
re-decided. Not a literal copy: a copy drags someone else's source citation and
someone else's domain limits through thirty nodes, and that is how a wrong
reference propagates.

    cargo run -p xtask -- gate prop_intake_throat
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

    cargo run -p xtask -- docs prop_intake_throat    # six artefacts, none typed

Then fill the numbered `HOLE` blocks in `model.rs`. A few typed lines each. The
signature, the unit types, every guard with its reason, the fault construction
and the ordering are already generated.

    cargo test -p vleo-mod-prop
    cargo run -p xtask -- gate prop_intake_throat

**Review 2 (H2)** — fixtures, their provenance, and the filled holes. By then
the node has already survived independent machine verification, so the person
accepts rather than hunts. That is what makes two reviews per node affordable.

    gh pr ready   →   merge   →   published

Two reviews. Everything between them is a command. If a node takes materially
longer, the template has a defect, and it is worth finding: it will be paid 1329
times.

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
| build a fixture the kernel must match | MATLAB — this is its best job | the fixture is committed as a golden vector, with its provenance |
| try a whole alternative architecture | a long-lived spike branch | never merged; findings return as a declaration |

## Two questions every month, above every metric

**How many nodes exist?** If the answer is "we have been improving the
generator", the drift already happened and every week felt productive.

**Is the review still changing anything?** If the two reviews have stopped
producing corrections, that is far more likely to be the review thinning than
the work improving.
