# What each agent was asked, and what it actually did

A roster entry is a claim. This is the evidence behind it: the task each agent
was given, what it returned, and what an independent recount said about that.
Every number below was re-derived without the agent's help, because an agent
checking its own work is the failure this whole system is arranged against.

Re-run any row. If an agent stops behaving like its row, the row is what has to
change.

---

## F · diagnostician — the AO fluence refusal

**Asked.** `aero_ao_fluence` refuses at the nominal case. Gather the evidence:
which node, what its inputs were, which bound it broke and by how much, what
the reason on that bound says, and the smallest reproducing command. Do not
change anything, do not propose a fix, do not say whether it is acceptable.

**Returned.** Attribution to *contract* — the node's own declared `upper = 1e26`
fires against three inputs each comfortably inside its own range. The smallest
reproduction. The reason string. A hand-check of `n_O·V·t` against the printed
output. Explicit elimination of data, code, hole, case and input bounds, each
with the check that ruled it out. And a breadth check across all five cases.

**Recounted independently.**

| case | F_AO | over the 1e26 ceiling |
|---|---|---|
| high_altitude | 3.029e26 | 3.0x |
| solar_min | 9.961e26 | 10.0x |
| nominal | 2.019e27 | 20.2x |
| solar_max | 3.179e27 | 31.8x |
| low_altitude | 6.048e27 | 60.5x |

Its claim was "blocked in all five, smallest margin ~3x at high_altitude, up to
~60x at low_altitude". Confirmed exactly.

**Lane.** `tools/agent_lanes.py --agent diagnostician` → *nothing changed*. It
holds the strictest lane in the roster (`never = ["**"]`) and stayed inside it.

**One thing it got wrong, and what changed because of it.** While gathering
evidence it found a second defect — two folders claiming the identifier
`prop_throat_area` — which was real and worth finding. It then reported the
folder as *"pre-existing, not created by this session"*, inferring that from
`git status` showing it untracked. That was wrong: the folder had been created
minutes earlier by `cargo xtask new`. Untracked means not committed; it says
nothing about age or origin.

The finding was right and the provenance was invented, which is the more
dangerous half — it sends the next person looking in the wrong place. The
definition now carries a *What you may never claim* section naming this case
and the commands that would have answered it (`git log -1 --format=%ci`,
`stat`, the reflog).

---

## B · fixture recorder — unbacked range edges in propulsion

**Asked.** Which declared range edges among the published propulsion nodes have
no fixture behind them? Report only — invent nothing, edit nothing.

**Returned.** A per-node table of declared range, fixture count and which edges
are unexercised, plus the observation that even the three nodes that *do* carry
a fixture have it at an interior operating point, so no edge is covered by any
of them.

**Recounted independently.** 3 nodes carry a fixture
(`prop_capture_efficiency`, `prop_compression_ratio`, `prop_exhaust_velocity`),
3 fixtures in total, **0 of them sitting at a declared bound**. Its
shape and its conclusion were right.

**Lane.** It edited nothing, as asked — it holds Edit but was told to report,
and did.

**What the recount added.** The agent's node count included a folder that had
been created during the run, which is how the duplicate-identifier defect
surfaced a second time, independently of the diagnostician. Two agents on
unrelated tasks both tripped over the same broken thing. That is the argument
for the lane checker and the gate being cheap to run: the defect was found
twice before anybody went looking for it.

---

## The defect both of them surfaced

`cargo xtask new <id> --like <sibling>` kept its own copy of the folder rule,
stripping the subsystem prefix, after the seeder's rule had changed to *the
folder is the identifier*. So the first command a new person runs produced two
folders claiming one id.

The gate caught it immediately and by name —

    two nodes claim the identifier 'prop_throat_area' — the second is
    .../crates/vleo-mod-prop/nodes/throat_area

— which is the system working. But a rule that lives in two places will
disagree again, so it now lives in one. Fixed in `5355548`, retested with the
command that produced it.
