# areas/generators.md

> **Answer first.** The schema, the generators and the gate: one defect here is in every row at once, so every change here needs two reviewers.
>
> **Kind:** reference + explanation · **For:** developers

The schema, the generators, the gate and the golden corpus.

Applies to `crates/vleo-sheet/**`, `xtask/**`, `tools/**`.
Maintained by the developers. Until the sheets go, the plan a group release is
taken in by lives here too (`src/template.rs`), read from the same field table
as the sheet. The generators leave the build in phase E, and
their checks move into the library.

## Why a change here needs two reviewers

There are 1401 rows. A defect in a generator is in all of them at once, and it
arrives everywhere on the same commit. One heavily reviewed component beats
1401 lightly reviewed ones, but only if it is actually treated as one — so
`H7` is two reviewers, and the advisory review job says so unprompted when it
sees a diff here.

## The generators

No generator writes code: every relation is the node's method, in its sheet,
run by the interpreter, and nothing in a node's file is generated. One runs
per node, writes nothing, and reads nothing but that node's sheet, which is
what makes 1401 rows 1401 independent acts:

| generator | emits |
|---|---|
| page | the node's fragment of the document — rendered from the sheet when it is opened, never written to the design |

Three run at assembly and may combine and refuse, never decide — a decision
taken during assembly is a decision nobody reviewed:

| generator | emits |
|---|---|
| index | the tree the faces read |
| document | the assembled page fragments |
| graph | the three graph tables |

The gap pass is the deterministic one: it diffs what the sheet promised against
what the artefacts contain. It costs nothing because the requirement is a
schema rather than prose.

## The schema is the contract

A field added to the sheet without a reader is a field that rots. A field read
without being declared in the loader is a field that silently defaults. Change
both, in the same commit, or neither.

## The rules a generator obeys

**Deterministic.** Same sheet in, same bytes out, on any machine in any month.
A generator whose output depends on iteration order, a clock or a path fails
the byte-stability check at random, and within a fortnight nobody reads the
regeneration diff either.

**No cross-node reads.** A per-node generator that reads a sibling is an
assembly generator wearing the wrong name.

## The golden corpus

`cargo test --workspace`, in both profiles. Link-time optimisation changes
inlining, which can move a floating-point result, so the profile that ships is
the profile that must be proven.

## Changing the gate

Every check is a `Check::pass`, `Check::fail` or `Check::note`, in a fixed
order, and the order is part of the design: the schema check runs before the
checks that would report nonsense on an unfilled sheet.

`Check::note` does not block. Promoting a note to a fail is a policy change and
needs the same two reviewers as the code.
