# AGENTS.md

> **Answer first.** The rules that do not bend, and the developer's loop: every change to the design arrives as a node form, is checked, applied with its de-risking record, implemented, gated and released. An assistant may help with the code; it never supplies a relation.
>
> **Kind:** reference · **For:** developers and any assistant they run

The root instruction file for the people who maintain this repository — and
for any assistant a developer runs inside it, which works to exactly the same
rules. `areas/*.md` narrow it per area, and the nearer file wins on anything
they disagree about.

This file is reviewed like code, not like documentation. It decides what every
change to the design goes through, so a change to it has the blast radius of a
generator change.

## What this repository is

An integrated design tool for very-low-Earth-orbit spacecraft. One kernel
computes every number; four rings depend inward only:

    vleo-units  →  vleo-core  →  vleo-bus  →  vleo-mod-*  →  the faces
    RING 0         RING 1        RING 2       RING 3

The tree is 1396 rows across four layers. Each row is one small question with
one answer, one folder, and one variable whose id is the row's id.

## Who changes it

**The team uses the tool; the developers maintain it.** A team never edits
this repository. They set the inputs, run, keep and send results — all of it
outside the repository, under `~/.vleo/` — and when the design itself is wrong,
missing or unfinished, the person who knows the answer fills in **the node's
form** (or the form for a new node) and sends it here. The browser cannot
change a node, add one or remove one, by design: a change typed into one copy of
the tool is a change nobody checked, implemented or released.

So every change to the design arrives the same way, and goes through the same
loop:

    1  CHECK     cargo run -p xtask -- intake <form.html>
                 what it would change, field by field; every interface it
                 declares (each input a row that exists, of the quantity the
                 node expects); what cannot be applied — a conflict with a
                 change made since, a relation an assistant supplied.
                 Nothing is written. If it does not pass, it goes back to
                 whoever filled it, with the lines intake printed.
    2  APPLY     cargo run -p xtask -- intake <form.html> --apply
                 into its layer: an existing node's sheet, or a new node built
                 in its place in the tree. Regenerated and gated as one edit,
                 or put back entirely. A change that moves a decision appends
                 its [[version]] — what we believed, what we tested, what we
                 now know, what changed; without that record, only the form's
                 wording goes in (docs/DERISKING.md).
    3  PUBLISH   cargo run -p xtask -- publish <node>
                 a filled seeded row becomes published and its code is
                 generated, with numbered HOLE blocks.
    4  IMPLEMENT cargo run -p xtask -- fill <node> --hole <n> --body - \
                     --by "<who>" --model <model>
                 the few typed lines per hole, composing kernel relations.
    5  EVIDENCE  fixtures.toml — values from outside this code, including any
                 known values the form supplied (intake prints them; it never
                 writes them).
    6  GATE      cargo run -p xtask -- gate && cargo test
    7  RELEASE   commit naming whoever filled the form, review, merge; then
                 cargo run -p xtask -- derisk      (the narrative, regenerated)
                 cargo run -p xtask -- release <version>
                 stamps every version still `next` with the release that ships
                 it. The team gets it in that release, and their saved case
                 carries over on its own.

**An assistant may help at step 4, and anywhere a developer uses one for
ordinary engineering** — the generators, the daemon, the faces, the tests. It
is released on a change only after the form has passed the check at step 1,
and it works to every rule in this file. There is no roster of specialised
agents: the checks enforce the rules, not a prompt. What no assistant may do is
**supply a relation** — intake refuses a form whose relation an assistant
filled, and relation stamping refuses a checkout whose `git config user.name`
is an assistant's. `fill --by --model` records who wrote each hole, so a
significant one written twice by different model families can be compared
with `xtask differential`.

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

## Requirements and closure — the contract between layers

A layer does not read into the layer below it. What crosses is a **closure**: a
requirement, an achieved value, and a margin between them. That is the only
contract there is, so it is the one thing that must not be ambiguous.

**A requirement declares which way it binds.** `sense = "<="` means the achieved
value must stay **under** the bound; `sense = ">="` means it must **reach** it.
Never defaulted, and gate check 7d refuses a written requirement without it —
where a requirement is any row of `kind = "required"` **or** any row some
closure reads as its `req` binding. That second half is taken from the graph
rather than from a naming convention, because a convention can be dodged by
renaming a folder and a contract edge cannot.

The reason is not tidiness. *The design sustains Ap 200* and *the design needs
Ap 200* are the same number and opposite requirements. Read the wrong way, the
closure still computes, still has a plausible sign, and reports a comfortable
margin for a spacecraft that is about to be destroyed. The prior MATLAB was
stricter than this repository here for exactly that reason: its contract
declares an adverse direction per requirement and fails its build without one,
because "defaulting either is how a silently wrong bound gets shipped".

**The sheet's sense and the hole's sense are checked against each other.** A
closure's hole applies `mission::Sense::AtLeast` or `mission::Sense::AtMost`,
and that is what actually runs. Gate check 7e refuses a node whose hole applies
the opposite of what its requirement declares. Two statements of one fact drift,
and this one drifts in the direction nobody looks.

**Not every closure runs in the same direction.** Most of this tree is a promise
— a subsystem must reach what it was asked for. The environment is not promised:
nobody builds the Sun, and a solar requirement is the worst sky the design can
sustain, closing when the achieved sky stays under it. Both senses are correct
and they are opposite, which is the whole reason the field exists.

## Two standards every change is held to

**Every change says which belief broke** — `docs/DERISKING.md`. A node changes
because something tested one of its beliefs and it did not hold; the change
records what we believed, what we tested, what we now know, what it cost, what
changed and which risks it moved, and every version says what it rests on and
what would break it. Risks are registered once, on the risk-register rows of the
management layer, and moved only by versions. The gate refuses a malformed
record (`versions`, V17, V18); the gap pass holds a row that has none.

**Everything that teaches, teaches the same way** — `docs/EXPLAINING.md`. Answer
first; then said simply; then the real thing with its source; then where the
simple version breaks. Every claim says whether it is sourced, derived, declared
or illustrative, and every block what kind of reading it is. Node pages, forms,
results, the manual, figures and these documents are all held to it by check.

## The commands

    cargo run -p xtask -- form <node>|--new    a node's form, to send out
    cargo run -p xtask -- intake <form.html> [--apply [--partial]]
                                            the checker, then the apply
    cargo run -p xtask -- publish <node>    seeded and filled → published
    cargo run -p xtask -- declare <node>    the completion questions, and which
                                            are still open
    cargo run -p xtask -- docs [<node>]     the six per-node generators
    cargo run -p xtask -- assemble          the three assembly generators
    cargo run -p xtask -- gate [<node>]     the checks, in order
    cargo run -p xtask -- fill <node> --hole <n> --body - [--by <who> --model <model>]
                                            splice one hole body
    cargo run -p xtask -- ready [<node>]    has it earned a person's attention
    cargo run -p xtask -- status            what exists, what is blocking
    cargo run -p xtask -- active            what answers, what is undefined, and
                                            what is blocked by a named row
    cargo run -p xtask -- reach             where each answer goes, and which
                                            reach no KPI closure
    cargo run -p xtask -- gap               what the sheets promised and
                                            nothing covers
    cargo run -p xtask -- derisk            the de-risking narrative, regenerated
    cargo run -p xtask -- release <version> [--check]
                                            stamp every `next` version; set the
                                            workspace version
    cargo run -p xtask -- kit               the tool for the team, without the
                                            repository (docs/SHARING.md)
    cargo run -p vleo-cli --bin vleo -- run <node> [--save <file.csv>]
    cargo run -p vleo-cli --bin vleo -- result <file>

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
