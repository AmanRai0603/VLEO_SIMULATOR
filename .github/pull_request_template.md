## What this changes

<!-- One node, ideally. A branch is a set of node folders, and a pull request
     that touches thirty of them is a pull request nobody can review. -->

## The two reviews

- [ ] **H1a — completeness.** Every question answered, every declared limit has
      a reason, the interface closes, a source is cited. Any engineer, about ten
      minutes.
- [ ] **H1b — physics.** The relation is right, the source is the right source,
      the declared range is honest. A domain engineer, about twenty-five
      minutes.
- [ ] **H2 — fixtures and holes.** Every expected value came from outside this
      code. The filled holes are a few typed lines each and they say what the
      sheet says.

The author is not an eligible reviewer on either.

## Before asking for review

- [ ] `cargo run -p xtask -- gate <node>` passes
- [ ] `cargo run -p xtask -- docs` leaves no diff
- [ ] `cargo test` passes
- [ ] If a tolerance moved, two reviewers — a tolerance change is a gate change

## Contract

- [ ] This node's published outputs are unchanged, **or** the change is routed
      to the owner of every consumer.
