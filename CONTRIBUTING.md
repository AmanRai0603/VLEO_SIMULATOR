# Contributing

> **Answer first.** Changes to the design come in as node forms and go out as reviewed pull requests, one form per pull request, with the number of reviewers set by what changed. The table below is the only statement of that rule.
>
> **Kind:** reference + how-to · **For:** developers

## The shape of the work

Contributions to the design come from the team that uses the tool, as **node
forms** — one HTML file per node, or per new node, filled by whoever knows the
answer. The developers maintain the repository: check each form, apply it,
implement and evidence it, and release. The loop is in [`AGENTS.md`](AGENTS.md);
the team's side of it is in the tool's own Manual tab.

A branch is **a set of node folders**, usually one form's worth, named for whoever
filled the form: `form/<author>/<node>`. `xtask take` creates it from a fresh
main — it is never named by hand. Because every artefact is derived from those
folders, a branch fully determines its own document and its own engine — so
checking one out and running it is one command rather than a procedure.

    git switch form/ana-sharma/sw-ap-design
    cargo run -p xtask -- docs && cargo run --release -p vleo-daemon

One form per pull request. A pull request touching thirty node folders is a
pull request nobody can review. `take` names whoever filled the form in the
commit (`Form-by:`); attach the filled form, or the output of `xtask intake` on
it, to the pull request — it is the record of what was asked for.

**A form branch merges only with its author's approval of the exact build they
tried** — the preview's Approve file, recorded by `xtask approve` and checked by
the pipeline's *the author approved this exact change*. Anything pushed after
an approval needs a new preview and a new approval. The whole loop, step by
step, is `docs/roles/maintainer.html`.

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

## How many reviewers, by what changed

This table is the only statement of the rule in the repository. `AGENTS.md`
points here rather than repeating it: two copies of a review policy are two
policies within a month, and they had already begun to differ.

| change | reviewers | why |
|---|---|---|
| a generator, the gate, `vleo-sheet`, or a `tools/` script | two | a defect there reaches all 1396 rows at once |
| `vleo-units` or `vleo-core` | two | everything reads them |
| an instruction file — `AGENTS.md`, `areas/*.md` | two | it decides what every change to the design goes through, so it is reviewed like code |
| **a tolerance** | two | the commonest way a gate stops meaning anything is somebody widening one to get green, so a tolerance change is a gate change |
| publishing a licensed bundle | two | publication is irreversible by design |
| moving a branch in `layers/` | two | the tree is the decomposition, and moving a branch moves everyone's work |
| a node sheet — a form applied, or a new node | H1a completeness, then H1b physics | above |
| fixtures and filled holes | H2, after the machine stages pass | above |
| anything else | one | ordinary blast radius |

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
- **Apply a form `xtask intake` has not passed.** A conflict goes back to
  whoever filled it; it is never resolved by overwriting the change made since.
- **Apply a change to what a node computes without its reason.** Intake
  withholds it; a hand edit that skips intake still owes the node a
  `[[version]]` saying which belief broke (`docs/DERISKING.md`).
- **Edit a recorded version.** A version is a record; a correction is the next
  version.
- **Let an assistant supply a relation.** It may write a hole's body; the
  relation comes from the form and carries a person's name.
- **Skip, disable or quarantine a test to get green.**

## Before asking for review

    cargo run -p xtask -- intake <form>   # the form passes the checker
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
