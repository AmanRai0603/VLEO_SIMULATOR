# Contributing

> **Answer first.** Changes to the code go out as reviewed pull requests into `developer`, one concern per pull request, with the number of reviewers set by what changed; the table below is the only statement of that rule. Changes to the design are not made here: their owners make them in the application. Until the switch-over, the design's one way in is a group's sealed release.
>
> **Kind:** reference + how-to · **For:** the developer and their deputy

## The shape of the work

New to the repository? Read [`docs/HOW_IT_WORKS.html`](docs/HOW_IT_WORKS.html)
first, then [`docs/OPERATING_1_0.md`](docs/OPERATING_1_0.md), section 17, which is
the developer's side of the whole.

The repository holds the code: the kernel and its toolbox, the method
interpreter and the engine, the library that reads, writes and checks every
design file, the application, and their tests. The developer maintains it. The
design is written, checked, sealed and released by the people who own it, in
the application, on the shared drive.

Work reaches the developer as a **request** (W14) or an **issue** raised in the
application, with its evidence. One request, or one concern, per pull request.

## The two branches

| branch | who works into it | how | what it is for |
|---|---|---|---|
| `developer` | the developer and their deputy | a pull request from a working branch | changes to the code |
| `main` | nobody directly | a pull request from `developer` | what is released |

**A release is cut only from `main`, and only from a commit that reached `main`
through a merged pull request.** The release pipeline checks it before anything
is built. Tags are `vX.Y.Z`. `developer` is brought up to date from `main` after
each release by a merge, never a force-push.

**An application release gives the released design's answers unchanged.** A
pull request that moves a line of `baseline/today.csv` says why in its
description, and is either a defect being fixed or a deliberate change to what
the engine does, reviewed as one.

## Who reviews

Ownership of the code is generated into `CODEOWNERS` from
[`areas/teams.toml`](areas/teams.toml): who reviews which crate, page and tool.
It says nothing about who owns which part of the design; that is the
application's, from the group files. Today every review team is one person. GitHub
never lets whoever opened a pull request approve it, so until a review team has a
second member the counts below are met by the developer's deputy, and turned on
in branch protection then, not before, or no pull request can merge.

## How many reviewers, by what changed

This table is the only statement of the rule in the repository. `AGENTS.md`
points here rather than repeating it.

| change | reviewers | why |
|---|---|---|
| `vleo-units` or `vleo-core` | two | everything reads them |
| the method interpreter, or the library's checks | two | a defect there admits or refuses every design file at once |
| the gate, a generator, or a `tools/` script the pipeline runs | two | a gate that can be widened quietly is not a gate, and a generator's defect reaches every row at once |
| **a tolerance**, or a line of `baseline/today.csv` | two | the commonest way a check stops meaning anything is somebody widening it, or recording it again, to get green |
| an instruction file — `AGENTS.md`, `CONTRIBUTING.md`, `areas/*.md` | two | it decides what every change to the code goes through |
| `contract/` — a route, a schema, a file format | two: one frontend, one backend (and the data owner for `formats/`) | it is where the two sides meet |
| publishing a licensed bundle | two | publication is irreversible by design |
| anything else | one | ordinary blast radius |

Whoever opened the pull request is never one of its reviewers.

## What you may not do

- **Put a formula outside `vleo-core`.** The gate fails the build.
- **Call the standard library's `sin`, `cos`, `exp`, `ln` or `powf` in a kernel
  crate.** Route them through `pmath`.
- **Offer a case or fixture whose expected value came from this code.** The
  library and the gate refuse it.
- **Weaken one of the library's checks to let a file through.** A file it
  refuses goes back to its writer, with the refusal.
- **Record today's answers again to get green.** `baseline/today.csv` is
  written again only in the commit of a deliberate change, and reviewed as one.
- **Edit, sign, seal or release anyone's design.** The developer maintains the
  code. A design that needs the code to change arrives as a request.
- **Let an assistant supply a relation.** It may write code. It may transcribe
  a relation a person already wrote when the node declares it `transcribed`,
  names the source and carries the signature of the person who read the copy
  against it.
- **Skip, disable or quarantine a test to get green.**

## Before asking for review

    cargo run -p xtask -- gate                  # the checks, in order
    cargo test                                  # the evidence, today's answers among it
    cargo clippy --workspace --all-targets -- -D warnings
    cargo fmt --all --check

## When a test refuses

    the engine no longer gives the answers on record in baseline/today.csv:
      line 5:
        on record  declared · with data,value,sw_ap_daily_band_drop,12.5635147395496,…
        today      declared · with data,value,sw_ap_daily_band_drop,12.563514739562164,…

**An answer moved.** Find the change that moved it. It is a defect unless the
pull request exists to move it.

## Until the switch-over

The design is still in this repository until the planned day
(`docs/PLAN_1_0.md`, phase H), and a group's sealed release, on the shared
drive, is its one way in.
This section is deleted on that day, with the `maintainer` branch.

- **A group's sealed release** goes on the shared drive, in
  `groups/<group>/releases/`, and not in this repository. The tool takes it into
  today's design when it opens on the drive, or refuses it and says why. The
  release is never edited.
- **`maintainer` goes into `main`** by pull request, like `developer`.
- **Reviewers, for the design while it is here:**

  | change | reviewers |
  |---|---|
  | a method an assistant transcribed | H1b physics, read against its source, by someone other than the person who signed it |
  | fixtures and filled holes | H2, after the machine checks pass |
  | moving a branch in `layers/` | two |

  H1b asks whether the relation, its source and its range are right; H2 whether every
  expected value came from outside this code. The person who wrote the node is
  eligible for none of them.
- **While it is here, you may not** edit a generated file outside a numbered
  `HOLE` block, commit an aggregate, add a guard by hand, apply a change to what
  a node computes without its reason, edit a recorded version, edit a sealed
  release, or move a group's results or tolerances to make a test pass.

## Commit messages

Say what changed and *why the previous shape was wrong*. A commit that says
"fix drag coefficient" and one that says "the drag coefficient was referred to
the wetted area, not the frontal area, so every ballistic coefficient
downstream was low by the ratio of the two" cost the same to write and differ
by an afternoon to the next person.
