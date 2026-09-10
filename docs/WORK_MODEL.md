# Work model

Segregate the work first, then decide who does it.

| family | owns | characteristic failure | verified by |
|---|---|---|---|
| **K** know | requirements, the physics, assumptions, the domain, the algorithm | the right method applied to the wrong regime — and nothing downstream sees it | people, and only people |
| **M** make | implementation, evidence, the interface a node publishes | a wrong constant; fixtures that pass and prove nothing | generation, types, fixtures, mutation |
| **C** compose | subsystem and system runs, campaigns, margins | two nodes each correct, wrong when coupled | the integrator, on prepared evidence |
| **P** platform | generators, the faces, the document, data and bundles | a generator defect reaching every node at once | ordinary software engineering, reviewed twice |
| **O** operate | repository, release, maintenance, the agent fleet | a supply-chain change merged without anyone deciding | configuration, bots, and one approval |

The families are not teams. One engineer works in K and M every day. What the
segregation fixes is **which rules apply**.

## Six interfaces, and exactly what crosses each

If the families are the work packages, the interfaces are where a programme
actually fails. Each is a named artefact with a schema, an owner and an
acceptance condition — not a handover conversation.

| | between | artefact | accepted when |
|---|---|---|---|
| I1 | know → make | the node sheet | it validates, no question is open, and a second engineer has read it in two stages |
| I2 | make → compose | the contract and a stub satisfying it | generated, and the contract difference accepted if the interface moved |
| I3 | compose → know | the impact report | the integrator accepts, rejects or defers, **in writing** |
| I4 | platform → make | the generators and the gate | the golden corpus passes and the nightly regeneration diff is clean |
| I5 | platform → everyone | the faces and the document | golden vectors agree across every face |
| I6 | operate → everyone | the gate verdict, the merge queue, the release | the queue proves the merged state; a person approves the release |

Every interface is a file with a schema, so a malformed handover fails a build
rather than surfacing three weeks later. I1 carries the most: everything MAKE
produces is a function of it.

## Where a person stops the work

Nine decisions, and only two are per node. Gating everything is the documented
failure: a person asked to approve too many things stops evaluating each one.

| | decision | family | how often |
|---|---|---|---|
| H1 | the sheet and the algorithm, in two stages | K | **per node** |
| H2 | fixtures, provenance and the filled holes | M | **per node** |
| H3 | the contract difference | M | only when the semantic-version check fires |
| H4 | common content — symbols, units, frames, constants | K | rare; two reviewers, one outside the family |
| H5 | which interface mismatches are defects | C | per subsystem run |
| H6 | assumption compatibility; accepting a narrowed margin | C | per run |
| H7 | a generator or gate change | P | two reviewers — a defect reaches every node at once |
| H8 | release | O | per release |
| H9 | the monthly review | O | monthly |

Answering the completion questions and deriving fixtures are **work, not
oversight**. They do not count against this budget, and they are what keep the
people capable of making the nine decisions above.

## The six things no agent does, and what covers each

The dividing line is simple: an agent may do anything where being wrong shows up
immediately, and nothing where being wrong looks completely fine.

| | what no agent does | why not | covered by |
|---|---|---|---|
| 1 | state the physics and where it came from | a wrong relation that runs cleanly is the worst failure this system can have | H1b, a second domain engineer |
| 2 | say what the right answer is | a number worked out by the code being tested proves nothing | H2, plus the schema rule on provenance |
| 3 | decide whether two subsystems disagreeing is a defect or a modelling choice | needs to know what the model was for | H5, the integrator |
| 4 | accept a reduced margin | a promise to a customer, not a calculation | H6, the integrator with domain owners |
| 5 | change the generator or the gate | one mistake there is 1329 mistakes | H7, two reviewers |
| 6 | decide what ships and what the customer is told | liability | H8, the feature owner |

Four agent definitions live in `.claude/agents/`. Each has exactly one
prohibition, and what matters is whether that prohibition is enforced by a
machine or by a person:

| agent | may not | enforced by |
|---|---|---|
| declaration drafter | supply mathematics, a reason, or a value | **nothing mechanical** — the physics review is the whole guard, and that is why it exists |
| fixture recorder | produce an expected value | the schema rejects any provenance that is not an external oracle |
| hole filler | write outside a hole | structural: the generator splices, and a body for an undeclared hole is refused |
| diagnostician | fix, or judge acceptability | structural: the report shape has no field for a fix |

The first of those has no mechanical guard, and it is stated rather than hidden.

## Four hooks, four moments

`.claude/settings.json`. Commands are opt-in and skills are opt-in; a hook fires
whether or not anybody remembered it.

| hook | fires | asks |
|---|---|---|
| session start | a session opens | what state is the tree in, what is blocking, is the data store filled |
| path guard | before every write | is this path yours, and is it generated |
| post-edit gate | after every write | format, lint, the declaration check — about a second |
| evidence guard | a session ends | has every touched node's evidence executed |

The last one changes behaviour most: a session cannot end with a node whose
evidence has not run, so leaving the tests until tomorrow stops being available.

**A hook is not a control.** It only fires inside the authoring tool; a terminal
in the same container has none. The authority is server-side — branch
protection, required checks, `CODEOWNERS`, the merge queue. The hooks exist to
fail fast and to teach; the branch rules exist to be true. Both are needed, and
there is **one gate binary** so the two can never disagree about what passing
means.

## The gap pass

The gate asks whether what exists passes. It does not ask whether anything the
sheet promised is **absent** — and the dominant way this class of work fails is
not that it cannot make progress: something plausible gets built, the checks it
chose for itself pass, a confident summary is written, and it stops while real
requirements are still unmet.

Here the requirement is a schema rather than prose, so the difference between
the sheet and the artefacts is a **diff, not a judgement**. It costs nothing and
runs on every node, every night.

    cargo run -p xtask -- gap

Its output is a list, not a verdict — it blocks the second human review rather
than the build. A tree that is 45% evidenced should not have a red build every
night, because that is how a team learns to ignore one.
