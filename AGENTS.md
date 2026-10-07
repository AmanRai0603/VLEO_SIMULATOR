# AGENTS.md

> **Answer first.** This repository holds the code that runs the design, not the design. The developer maintains the engine, the one library that reads and checks every design file, and the application; the people who own the design write it, check it and release it in that application, on the shared drive. Six rules about the code do not bend, and the first is that every formula lives in the kernel. An assistant may help write the code; it never supplies a relation.
>
> **Kind:** reference · **For:** the developer, their deputy, and any assistant they run

The root instruction file for the people who maintain this repository, and for
any assistant a developer runs inside it, which works to the same rules.
`areas/*.md` narrow it per area, and the nearer file wins where they disagree.

This file is reviewed like code. It decides what every change to the code goes
through, so a change to it has the blast radius of a change to the gate.

Before your first change, read [`docs/HOW_IT_WORKS.html`](docs/HOW_IT_WORKS.html)
in a browser, and the three pages that say what 1.0 is:
[`docs/SYSTEM_MODEL.md`](docs/SYSTEM_MODEL.md),
[`docs/OPERATING_1_0.md`](docs/OPERATING_1_0.md) and
[`docs/PLAN_1_0.md`](docs/PLAN_1_0.md).

## What this repository is

The code of an integrated design tool for very-low-Earth-orbit spacecraft. One
kernel computes every number; four rings depend inward only:

    vleo-units  →  vleo-core  →  vleo-bus  →  vleo-mod-*  →  the faces
    RING 0         RING 1        RING 2       RING 3

Around them: the method interpreter, the library that reads, writes and checks
every design file, and the application, installed and as a page.

**What it does not hold, from the switch-over on:** the design. The programme's,
the systems' and every group's files live on the shared drive, written by their
owners in the application (`docs/OPERATING_1_0.md`, section 15). Until the
switch-over the design is still here, and the section *Until the switch-over*
below says how it changes.

## Who does what

Everyone uses the application. Each role has a deputy who may act for it.

| role | owns | in this repository |
|---|---|---|
| **programme manager** | the programme's branch, and the decision | nothing |
| **system engineer** | the systems' branch; the main valve between the programme and the subsystems; the one who releases the design | nothing |
| **subsystem engineer** | one group's branch; the valve above its nodes. Every owner of a branch is the system engineer of that branch | nothing |
| **node engineer** | one or more nodes in a group | nothing |
| **developer** | the code: the engine, the library, the application, their tests and their releases | everything here |

The design reaches the developer only as a **request** (W14): something the code
cannot yet do, raised in the application with the evidence. The developer
answers it with an application release. The developer never edits, signs,
seals or releases anyone's design.

## The six rules that do not bend

**1 · Every formula lives in `vleo-core::physics` and nowhere else.** A relation
inlined anywhere else is a relation nobody can review or reuse. Adding one to
the kernel is a reviewed change to a crate every node reads. A design reaches
the kernel only through a node's method, run by the interpreter, or through a
built-in relation kept in code by its node's id until its group writes a method.

**2 · Portable maths only.** `vleo_core::units::pmath`, never the standard
library's transcendentals. The kernel crates are `no_std`, so `f64::cos` does
not exist there and the compiler refuses; in an interpreted method the
interpreter has only `pmath`.

**3 · A refusal is never a substitution.** A row with no content returns
`NotRun` under its own name. A run always prints "n ran, m blocked" and names
the blocked. A sweep records refused points; it never drops them.

**4 · An expected value never comes from the code under test.** The library
refuses a case whose provenance is `self-snapshot` or `agent-generated`, and so
does the gate. It is the one external oracle in the system; everything else
compares the software against itself.

**5 · An application release gives the released design's answers unchanged.**
`baseline/today.csv` holds what the engine answers today, and `cargo test`
fails when an answer moves. A change to the code that moves an answer is a
defect, whatever else it improves. From the switch-over on, the record is taken
from the current released design when an application release is prepared (W15).

**6 · No assistant supplies a relation.** It may write code: the engine, the
library, the application, their tests. It may transcribe a relation a person
already wrote, when the node declares it `transcribed`, names the source and
carries the signature of the person who read the copy against it. The library
refuses a node whose declaration says an assistant supplied its method or its
results, or says nothing, and refuses a signature made by an assistant's name.

## The design's rules, which the code enforces

These are the people's rules, described for them in `docs/OPERATING_1_0.md`.
The developer's part is that the library checks each one, the same installed
and in the page, and that a check is never weakened to let a file through. Each
is a check with a test that a file breaking it is refused, by name.

Most of these checks are built in phases C and D of the plan. Until they are,
the gate and intake hold the ones they hold today (the sense of a requirement,
the provenance of a case, the assistant rules, the seal, the de-risking record),
and the others are not yet enforced by anything. That is said here so nobody
takes the table for a list of what is checked now.

| the rule | refused when |
|---|---|
| one writer per file | a file is signed by anyone but its assigned writer |
| a signature checks through the chain | it does not check against the key its parent file registered |
| an expected value never comes from the code under test | rule 4 above |
| no relation supplied by an assistant | rule 6 above |
| every requirement says which way it binds | a requirement, or any row a closure reads as its bound, has no `sense` |
| the sheet's sense and the method's agree | a closure's method applies the opposite of what its requirement declares |
| a node reads its children only through their ports | a method reads a child's value that is not a port |
| a release is never edited | one byte differs from what was sealed |
| a parameter is changed only by the level that owns it | a file below that level sets it |
| every valve's owner controls what passes it | a release integrates without its subsystem engineer's seal |
| the design is released by the system engineer alone | a released design is not signed by the registered system engineer |
| today's design is never taken for a released one | a screen, file or result does not say which it is |
| every change says which belief broke | a change to what a node computes carries no version record (`docs/DERISKING.md`) |

**Not every closure runs in the same direction.** `sense = "<="` means the
achieved value must stay under the bound; `sense = ">="` means it must reach it.
Never defaulted. *The design sustains Ap 200* and *the design needs Ap 200* are
the same number and opposite requirements; read the wrong way, the closure
still computes and reports a comfortable margin for a spacecraft that is about
to be destroyed.

## Everything that teaches, teaches the same way

`docs/EXPLAINING.md`. Answer first; then said simply; then the real thing with
its source; then where the simple version breaks. Every claim says whether it
is sourced, derived, declared or illustrative, and every block what kind of
reading it is. The application's screens, guides, results, figures and these
documents are all held to it by check.

## The developer's loop

    1  REQUEST    a request (W14) or an issue arrives, with its evidence
    2  BRANCH     a working branch from `developer`
    3  CHANGE     the code, and its tests — each new test shown red against a
                  deliberately broken implementation before it is trusted
    4  GATE       cargo run -p xtask -- gate && cargo test
    5  REVIEW     a pull request into `developer`, reviewed as CONTRIBUTING.md says
    6  RELEASE    `developer` into `main` by pull request; on `main`,
                  cargo run -p xtask -- ship <version>
                  the release gives the released design's answers unchanged
                  (rule 5), and names the oldest design it can run

One command must be green before anything is pushed:

    cargo run -p xtask -- gate && cargo test

## Until the switch-over

The design is still in this repository, and the loop that changes it stays in
use exactly as it is today, so nothing changes for anyone before the planned day
(`docs/PLAN_1_0.md`, phase H). This section is deleted on that day.

- **A change to one node** arrives as its form, and goes through `xtask take`:
  its own branch `form/<name>/<node>` from `maintainer`, checked, applied with
  its de-risking record, gated, tested, committed naming the person who filled
  it, pushed, previewed, and merged only with their approval of that build
  (`docs/roles/maintainer.html`).
- **A group's work** arrives as its sealed release, and goes through
  `group-intake`, `group-build`, `group-test`, `group-deliver` and
  `group-accept` on its own branch `group/<group>-<version>` from `maintainer`.
  The release is never edited there; a fix goes back to the group
  (`docs/GROUP_APPS.md`).
- **The node sheet is the only source.** `node.toml` is written by intake from
  a form; every other file in a node folder is generated from it, and a hand
  edit outside a numbered `HOLE` block fails the regeneration diff.
- **`design/` is the design as its files,** converted from the sheets and held
  equal to them by a test. A change to a sheet converts it again in the same
  commit: delete `design/`, then `cargo run -p xtask -- convert --out design`
  (`take` and `group-build` do it themselves).
- **`xtask explain <command>`** says what each of those commands reads, writes
  and checks, and how to undo it (`docs/PIPELINE.md`).

## The commands

    cargo run -p xtask -- gate [<node>]      the checks, in order
    cargo run -p xtask -- explain [<command>]
                                             what a command reads, writes and
                                             checks, how to undo it, where its
                                             code is
    cargo run -p xtask -- method-wasm        rebuild the checker the pages carry
    cargo run -p xtask -- group-app [--check]
                                             the pages built from the checker
    cargo run -p xtask -- kit                the application, without the
                                             repository
    cargo run -p xtask -- ship <version>     the release branch, stamped and
                                             proved
    cargo run -p xtask -- pipeline [--check] docs/PIPELINE.md, from the table
    VLEO_BASELINE=write cargo test -p vleo-cli --test today_s_answers_are_on_record
                                             today's answers recorded again, in
                                             the same commit as a deliberate
                                             change to the design (`take` and
                                             `group-build` do it themselves)
    cargo run -p vleo-cli --bin vleo -- run <node> [--save <file.csv>] [--keep]

The commands of today's loop are in `docs/PIPELINE.md` until the switch-over.
Every command that writes prints numbered steps, says on a stop why, what state
the files are in and how to retry, leaves a trace in `target/xtask-trace/`, and
takes `--dry-run`.

## Testing

`cargo test --workspace`. A disagreement with a case is a physics disagreement,
not a build failure. It goes to the node's engineer, and the tolerance is never
the thing to change.

A test that passes against a deliberately broken implementation is not testing
anything. Before claiming a test is load-bearing, break what it covers, watch
it go red, and put it back.

## Review standards

How many reviewers a change needs is stated once, in
[`CONTRIBUTING.md`](CONTRIBUTING.md), and not repeated here.

## House rules

Write code that reads like the code around it. Match the comment density and
the naming of the file you are in.

Use the role names above and no others: programme manager, system engineer,
subsystem engineer, node engineer, developer, and each one's deputy. A
forecast's *lead time* is not a role, and keeps its name.

Commit messages: `type(scope): a sentence saying what changed`.
`tools/commit_message.py --types` prints the types and every valid scope. The
commit-msg hook and the pipeline run the same script.

Say what you did not do. A report that lists only what worked is a report
somebody has to re-derive.
