# AGENTS.md

The root instruction file. Everything below applies to every agent and every
person working in this repository; `areas/*.md` narrow it per area, and the
nearer file wins on anything they disagree about.

This file is reviewed like code, not like documentation. It shapes what seven
agents produce, so a change to it has the blast radius of a generator change.

## What this repository is

An integrated design tool for very-low-Earth-orbit spacecraft. One kernel
computes every number; four rings depend inward only:

    vleo-units  →  vleo-core  →  vleo-bus  →  vleo-mod-*  →  the faces
    RING 0         RING 1        RING 2       RING 3

The tree is 1329 rows across four layers. Each row is one small question with
one answer, one folder, and one variable whose id is the row's id.

## The five rules that do not bend

**1 · The sheet is the only source.** `node.toml` is written by hand. Every
other file in a node folder is generated from it. A hand edit outside a
numbered `HOLE` block in `model.rs` is discarded by the next `xtask docs` and
fails the regeneration diff in the gate.

**2 · An expected value may never come from the code under test.** The gate
refuses a fixture whose provenance is `self-snapshot` or `agent-generated`.
This is the one external oracle in the entire system; everything else compares
the software against itself.

**3 · Every formula lives in `vleo-core::physics` and nowhere else.** A
relation inlined in a node is a relation nobody can review or reuse. Adding one
to the kernel is a reviewed change to a crate every node reads.

**4 · Portable maths only.** `vleo_core::units::pmath`, never the standard
library's transcendentals. The kernel crates are `no_std`, so `f64::cos` does
not exist there and the compiler refuses; in a hole body the splice and then
the gate refuse it by name.

**5 · A refusal is never a substitution.** A row with no content returns
`NotRun` under its own name. A run always prints "n ran, m blocked" and names
the blocked. A sweep records refused points; it never drops them.

## The commands

    cargo run -p xtask -- declare <node>    the completion questions, and which
                                            are still open
    cargo run -p xtask -- docs [<node>]     the six per-node generators
    cargo run -p xtask -- assemble          the three assembly generators
    cargo run -p xtask -- gate [<node>]     the checks, in order
    cargo run -p xtask -- fill <node> --hole <n> --body -
                                            splice one hole body
    cargo run -p xtask -- ready [<node>]    has it earned a person's attention
    cargo run -p xtask -- status            what exists, what is blocking
    cargo run -p xtask -- gap               what the sheets promised and
                                            nothing covers
    cargo run -p vleo-cli --bin vleo -- run <node>

One command must be green before anything is pushed:

    cargo run -p xtask -- gate && cargo test

## Testing

`cargo test --workspace`. A fixture disagreement is a physics disagreement, not
a build failure — it goes to the node owner, and the tolerance is never the
thing to change.

A test that passes against a deliberately broken implementation is not testing
anything. Before claiming a test is load-bearing, break what it covers, watch
it go red, and put it back.

## Review standards

How many reviewers a change needs is stated once, in
[`CONTRIBUTING.md`](CONTRIBUTING.md). Read it there. It is not repeated here
because it was, and the two copies had already begun to differ — one listed
tolerance changes and bundle publication, the other listed `tools/` scripts and
instruction files, and neither was complete.

## House rules

Write code that reads like the code around it. Match the comment density and
the naming of the file you are in.

Commit messages: `type(scope): a sentence saying what changed`.
`tools/commit_message.py --types` prints the types and every valid scope. The
commit-msg hook and the pipeline run the same script.

Say what you did not do. A report that lists only what worked is a report
somebody has to re-derive.

## Your lane

Every agent has a list of paths it may change and a list it may never touch,
in `agents/lanes.toml`. It is checked against the diff, not asserted in a
prompt:

    tools/agent_lanes.py --agent <name> --since HEAD~1

Each lane also records what actually enforces its prohibition — `full`,
`partial` or `none`. Read your own before you start. Where it says `none`, a
person is the only thing between you and a defect, and that is worth knowing.

`agents/provenance.toml` says where each definition came from, who owns it,
what it falls back to, and which model runs it. A checker never runs the model
family of the thing it checks: a model given its own reasoning to grade
approves it. `tools/instruction_lint.py` refuses a pair whose models match.
