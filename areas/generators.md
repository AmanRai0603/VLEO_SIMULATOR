# areas/generators.md

The schema, the nine generators, the gate and the golden corpus.

Applies to `crates/vleo-sheet/**`, `xtask/**`, `tools/**`.
Agent I works here. Agents A, B, C, E never do.

## Why a change here needs two reviewers

There are 1375 rows. A defect in a generator is in all of them at once, and it
arrives everywhere on the same commit. One heavily reviewed component beats
1375 lightly reviewed ones, but only if it is actually treated as one — so
`H7` is two reviewers, and the advisory review job says so unprompted when it
sees a diff here.

## The nine

Six run per node and read nothing but that node's sheet, which is what makes
1375 rows 1375 independent acts:

| generator | emits |
|---|---|
| model | `model.rs` — the whole file, with numbered `HOLE` blocks |
| contract | `contract.rs` — outputs, units, guarantees, domain, faults |
| module | `mod.rs` |
| evidence | `evidence.rs` — the fixture table, executable |
| page | `page.html` — the node's fragment of the document |
| metadata | `meta.json` |

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

A field added to `node.toml` without a reader is a field that rots. A field read
without being declared in the loader is a field that silently defaults. Change
both, in the same commit, or neither.

## The rules a generator obeys

**Deterministic.** Same sheet in, same bytes out, on any machine in any month.
A generator whose output depends on iteration order, a clock or a path fails
the byte-stability check at random, and within a fortnight nobody reads the
regeneration diff either.

**Formatted before comparison.** Emit text, run it through `gate::formatted`,
then compare. Writing unformatted text and formatting afterwards makes every
run report a change.

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
