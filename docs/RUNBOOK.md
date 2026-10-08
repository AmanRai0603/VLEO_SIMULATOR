# Runbook

> **Answer first.** A developer's first day, then where one group's sealed release goes — onto the shared drive and into today's design, with no developer step — and what to do when the gate refuses.
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

## One sealed release, from the drive to today's design

Until the switch-over every change to the design arrives as a group's sealed
release: each node written by its node engineer and signed, the whole sealed by
the subsystem engineer. It goes on the shared drive, in its group's folder,
`groups/<group>/releases/`. Nobody takes it in by hand, and the developer has no
step in it.

When the tool opens on the drive, it builds today's design from every group's
latest sealed release that passes its checks: its seal, its content, and each
node against the design. To see it as everyone does:

    VLEO_DRIVE=<drive> cargo run --release -p vleo-daemon

A refused release is refused whole, and the tool says why: a **conflict** means
the design changed since the release was based on it; a **refused** method or
result means its declaration says an assistant supplied it, or says nothing; a
change to what a node computes needs its de-risking record. The group's part is
built from its last good release, and the tool says which release each group's
part is. The fix goes back to the group, which seals a new release and puts it
beside the old one; the release is never edited. Every check is in
`docs/GROUP_APPS.md`.

A node whose method is new is built into the kernel with
`cargo run -p xtask -- build-node <node>`, and reaches everyone in the next
release of the tool, reviewed as `CONTRIBUTING.md` says.

A row a developer starts without a release — rare, and usually structural —
still starts with `cargo run -p xtask -- new <id> --like <sibling>`, then
`publish`, its method — written by its node engineer — built with `build-node`,
and the same gate; it is
reviewed as `CONTRIBUTING.md` says.

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
4. Its node engineer writes the method from the same source, and
   `cargo run -p xtask -- method <node>` runs their cases through it.
   `cargo test` runs fixtures from the source *and* the parity grid.
   `migrated_from` records the function and line.
5. A disagreement is a finding. Either the method is wrong or the MATLAB was, and
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
