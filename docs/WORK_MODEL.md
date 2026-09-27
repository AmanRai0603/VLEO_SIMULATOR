# Work model

> **Answer first.** The team uses the tool and the developers maintain it, and only three things cross between them: a node form in, a release out, and results anyone can keep. Inside the developers' side the work divides by what goes wrong, and nine decisions stop for a person.
>
> **Kind:** explanation · **For:** everyone

Who does what: the team that uses the tool, the developers who maintain it,
and exactly what crosses between them.

## Two sides, one line between them

**The team uses the tool.** Engineers on the programme, a payload team, a
customer's engineer. They set the inputs, run, keep and send results, and read
the design. None of that needs a checkout, and none of it touches the
repository: the case and the results are kept on their own machine, under
`~/.vleo/`.

**The developers maintain it.** They own the repository: the tree, the kernel,
the generators, the faces, the checks and the releases. Every change to the
design passes through their hands, and their job is to keep the tool right and
to upgrade it with time.

The line between the two is the line between *using* the design and *changing*
it, and exactly three things cross it:

| | crosses | from → to | what it is | accepted when |
|---|---|---|---|---|
| X1 | **a node form** | team → developers | one HTML file per node, or per new node: every question the sheet answers, filled by whoever knows | `xtask intake` passes it — no conflict, no refusal, every interface a row that exists, of the quantity expected |
| X2 | **a release** | developers → team | the tool, with every applied form in it | the gate is green, the reviews are done, and a person approves the release |
| X3 | **a result** | team → anyone | a run's values and the inputs it ran on, as a CSV or a report page | it reads back — nothing runs, so it says the same after any release |

The case — the inputs — does not cross at all. It is the team's, it is kept on
their machine, and a release carries it over on its own: what still applies is
kept, new inputs take their defaults, and anything that can no longer be used
is set aside by name.

**The browser never changes the design.** It cannot edit a node, add one or
remove one. A change typed into one copy of the tool is a change nobody
checked, implemented, evidenced or released, and everyone else would be running
something different. A team member who knows the answer fills the form; that is
the whole of the contribution, and it leaves a record of who asked, what
changed and why.

## The developer's loop

Every form goes through the same seven steps, in [`AGENTS.md`](../AGENTS.md):
check, apply, publish, implement, evidence, gate, release. The first is the one
that protects everything after it — a form is read and its interfaces checked
before anything is written or anyone is asked to implement it. A form that
does not pass goes back to whoever filled it with the lines the checker
printed, never "fixed up" on the way in.

## The work itself, segregated

Inside the developers' side, the work divides by what goes wrong, and so by
which rules apply:

| family | owns | characteristic failure | verified by |
|---|---|---|---|
| **K** know | requirements, the physics, assumptions, the domain, the algorithm — most of it arriving in forms | the right method applied to the wrong regime — and nothing downstream sees it | people, and only people |
| **M** make | implementation, evidence, the interface a node publishes | a wrong constant; fixtures that pass and prove nothing | generation, types, fixtures, mutation |
| **C** compose | subsystem and system runs, campaigns, margins | two nodes each correct, wrong when coupled | the integrator, on prepared evidence |
| **P** platform | generators, the faces, the document, data and bundles | a generator defect reaching every node at once | ordinary software engineering, reviewed twice |
| **O** operate | repository, release, maintenance | a supply-chain change merged without anyone deciding | configuration, bots, and one approval |

The families are not teams. One engineer works in K and M every day. What the
segregation fixes is **which rules apply**.

## Six interfaces, and exactly what crosses each

If the families are the work packages, the interfaces are where a programme
actually fails. Each is a named artefact with a schema, an owner and an
acceptance condition — not a handover conversation.

| | between | artefact | accepted when |
|---|---|---|---|
| I1 | know → make | the node sheet, written from a form | it validates, no question is open, and a second engineer has read it in two stages |
| I2 | make → compose | the contract and a stub satisfying it | generated, and the contract difference accepted if the interface moved |
| I3 | compose → know | the impact report | the integrator accepts, rejects or defers, **in writing** |
| I4 | platform → make | the generators and the gate | the golden corpus passes and the nightly regeneration diff is clean |
| I5 | platform → everyone | the faces and the document | golden vectors agree across every face |
| I6 | operate → everyone | the gate verdict, the merge queue, the release | the queue proves the merged state; a person approves the release |

Every interface is a file with a schema, so a malformed handover fails a build
rather than surfacing three weeks later. I1 carries the most: everything MAKE
produces is a function of it — which is why the form that feeds it is checked
before it is applied.

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

## Assistants, and the six things none of them does

A developer may use any assistant for ordinary engineering, and to write a
hole's body — `xtask fill --by --model` records who wrote each one. There is no
roster of specialised agents with lanes of their own: the rules are enforced by
the checks, which apply to an assistant's change exactly as to anyone's. The
dividing line is simple: an assistant may do anything where being wrong shows up
immediately, and nothing where being wrong looks completely fine.

| | what no assistant does | why not | covered by |
|---|---|---|---|
| 1 | state the physics and where it came from | a wrong relation that runs cleanly is the worst failure this system can have | intake refuses a relation a form says an assistant supplied; relation stamping refuses an assistant's name; H1b, a second domain engineer |
| 2 | say what the right answer is | a number worked out by the code being tested proves nothing | H2, plus the schema rule on provenance |
| 3 | decide whether two subsystems disagreeing is a defect or a modelling choice | needs to know what the model was for | H5, the integrator |
| 4 | accept a reduced margin | a promise to a customer, not a calculation | H6, the integrator with domain owners |
| 5 | change the generator or the gate unreviewed | one mistake there is 1396 mistakes | H7, two reviewers |
| 6 | decide what ships and what the customer is told | liability | H8, the feature owner |

The first depends in part on honesty: a form says whether an assistant helped,
and the checker can only act on what it says. That is stated rather than
hidden, and it is why H1b exists.

## Four hooks, four moments

`.claude/settings.json`. They fire for any edit made inside an assistant's
session in a checkout, whether or not anybody remembered them.

| hook | fires | asks |
|---|---|---|
| session start | a session opens | what state is the tree in, what is blocking, is the data store filled |
| path guard | before every write | is this path generated |
| post-edit gate | after every write | format, lint, the declaration check — about a second |
| evidence guard | a session ends | has every touched node's evidence executed |

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
