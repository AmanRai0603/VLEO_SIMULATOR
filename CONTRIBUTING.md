# Contributing

## The shape of the work

A branch is **a set of node folders**. Because every artefact is derived from
those folders, a branch fully determines its own document and its own engine —
so checking one out and running it is one command rather than a procedure.

    git switch node/prop-intake-throat
    cargo run -p xtask -- docs && cargo run --release -p vleo-daemon

One node per pull request, ideally. A pull request touching thirty node folders
is a pull request nobody can review.

## Two reviews, and the author is not eligible for either

| | asks | who | roughly |
|---|---|---|---|
| **H1a** completeness | is every question answered, does every limit have a reason, does the interface close, is a source cited | any engineer | 10 min |
| **H1b** physics | is the relation right, is the source the right source, is the declared range honest | a domain engineer | 25 min |
| **H2** fixtures and holes | did every expected value come from outside this code; do the filled holes say what the sheet says | a domain engineer, *after* the machine checks pass | |

Splitting H1 is the only lever that moves the schedule without hiring: roughly
half the review load leaves the person who cannot be duplicated.

By H2 the node has already survived the gate, the gap pass and its own
fixtures, so the reviewer **accepts** rather than hunts. That is what makes two
reviews per node affordable.

## What needs two reviewers

- A change to a generator, the gate, or `vleo-sheet`. A defect there reaches
  every node at once.
- A change to `vleo-units` or `vleo-core`: everything reads them.
- **A tolerance change.** The commonest way a gate stops meaning anything is
  somebody widening a tolerance to get green, so a tolerance change is a gate
  change.
- Publishing a licensed bundle. Publication is irreversible by design.
- Moving a branch in `layers/`: the tree is the decomposition, and moving a
  branch moves everyone's work.

## What you may not do

- **Edit a generated file outside a numbered `HOLE` block.** It is discarded by
  the next regeneration and fails the regeneration diff.
- **Commit an aggregate.** The assembled document, the index, the module lists
  and the graph tables are built. Every node would touch them, so every merge
  would conflict in generated content nobody is allowed to edit.
- **Put a formula outside `vleo-core`.** The gate fails the build.
- **Call the standard library's `sin`, `cos`, `exp`, `ln` or `powf` in a kernel
  crate.** Route them through `pmath`, or the nightly cross-face gate fails for
  a reason that is not a defect.
- **Offer a fixture whose expected value came from this code.** The schema
  refuses it.
- **Add a guard by hand.** Guards are generated from the declared domain with
  their reasons attached; one added by hand has no reason and will be deleted.
- **Skip, disable or quarantine a test to get green.**

## Before asking for review

    cargo run -p xtask -- gate <node>     # the checks, in order
    cargo run -p xtask -- docs            # must leave no diff
    cargo test                            # the evidence
    cargo clippy --workspace --all-targets -- -D warnings

## When the gate refuses

    FAIL  fixtures   2 of 3 rows pass
      row 2: expected 2.645, got 2.641, tolerance 1e-3

**A physics disagreement, not a build failure.** It goes to the node owner. The
tolerance is not the thing to change.

## Commit messages

Say what changed and *why the previous shape was wrong*. A commit that says
"fix drag coefficient" and one that says "the drag coefficient was referred to
the wetted area, not the frontal area, so every ballistic coefficient
downstream was low by the ratio of the two" cost the same to write and differ
by an afternoon to the next person.
