# The system model

> **Answer first.** Any system is defined here the same way: one kind of block, broken down from the top to any depth, whose requirements flow down and whose answers flow back up to decide every margin. This page is that model, the spacecraft as it is broken down today, and the published practice each part rests on.
>
> **Kind:** explanation + reference · **For:** everyone

This page is the design's architecture: how a question becomes a tree of
blocks, how the blocks connect, and how the tree decides whether the design
closes. The code's architecture, the four rings, is `docs/ARCHITECTURE.md`.
How it is operated, who holds what and where every file sits, is
`docs/OPERATING_1_0.md`; how the work reaches 1.0.0 is `docs/PLAN_1_0.md`.

It is a living page. The model in the first six sections changes rarely and by
review, like a rule. The breakdown of the spacecraft changes as groups break
their branches down; when it does, this page follows.

## 1 · Flows down, closes up

A design starts as one question at the top and is broken into the things that
decide it, and each of those into the things that decide it, until every piece
is something a person can state, look up or calculate.

- **Requirements flow down.** A parent hands a child a bound and the direction
  it binds in: `>=` must reach it, `<=` must stay under it.
- **Answers flow up.** Only the bottom calculates. An achieved value is never
  typed at the top; it arrives from below.
- **A closure decides.** A requirement, the value achieved, and the margin
  between them. A change anywhere re-closes every margin above it, and a red
  margin names the block that caused it.

*Said simply:* the top decides what is needed, the bottom decides what is
possible, and the closure says whether the two meet.

## 2 · One block, any depth

There is one kind of element, the **block**. The team calls it a node. There
are three relations between blocks:

| relation | means |
|---|---|
| **contains** | a parent holds its children, to any depth |
| **connects** | an output of one block is an input of another; every wire |
| **closes** | a requirement against an achieved value, with a direction and a margin |

A block's **behaviour** is exactly one of:

| behaviour | the block's answer is |
|---|---|
| method | its pseudocode, run on its inputs (`docs/PSEUDOCODE.md`) |
| children | whatever its children give at its outputs |
| stated | a value a person states, with its source |
| lookup | a table and how to read it |
| open | not decided yet: a draft, refused by name if a run reaches it |

A block stops being broken down when its behaviour is a method, a stated
value, a lookup, or a wire from another group.

**A block sees inside its children only through their inputs and outputs.**
This is the rule "a layer reads the one below it only through a closure",
applied at every depth instead of at three fixed floors.

### Breaking a node down later

The node a group calls its last break is only the last break *so far*. Any
node can be broken down later, and nothing above it has to change.

1. **Its outside stays.** The same identity, the same ports, the same contract.
   Everything that reads it reads the same ports, so no other node and no other
   group is edited. A breakdown that also changes a port is a contract change,
   and is treated as one.
2. **Its behaviour becomes its children.** The method, value or table it had is
   kept as its estimate. The answer from its children is shown beside it, and
   the difference is a margin like any budget's.
3. **Its cases stay, and test the breakdown.** The children together must
   reproduce the node's own cases within their tolerance. If they do not, a case
   or the breakdown is wrong, and the version that broke it down says which.
4. **Its children belong to the same group,** unless another team will own
   them. Then the node becomes a **mount**, a new group hangs there, and the
   node's group is to it what the systems group is to a subsystem group.
5. **It can be folded back.** A broken-down node can return to a method, for
   speed or for an earlier stage of a study. Its children stay in the release
   history.

*Said simply:* breaking a node down replaces what is inside the box, never the
box.

**Depth and perspective are different things.** Management, the system and
the subsystems are *perspectives*: what kind of question a branch answers.
They are tags on a branch, not floors. A block six levels inside Propulsion is
still a subsystem block. The run, where a case is set, is the bottom of
every branch: its inputs, not its rows.

## 3 · What a block holds

Six parts, and each fact is written once.

1. **Identity:** the question it answers, its owner, its group, its version.
2. **Ports:** its inputs and outputs. Each has a type, a unit, a range and the
   reason for the range. Types: a number with a unit, a whole number, a choice
   (low, moderate, elevated, high), yes or no, a list, a table, a time series,
   an uncertain value, text, a file. Ports can be bundled, so a group passes
   one driver set instead of ten wires.
3. **Behaviour:** one of the five above. The equation on the page is drawn from
   the method, and the range refusals are generated from the ports. Neither is
   written twice.
4. **Evidence:** cases, each an input, an expected answer, a tolerance and
   where the answer came from. An expected value never comes from the code
   under test.
5. **Explanation:** said simply, the theory, each assumption and when it
   fails, the sources.
6. **History:** every version: what was believed, what was tested, what is now
   known, what changed (`docs/DERISKING.md`). Sign-offs, each against the
   content it was given for.

## 4 · Every value carries its state, its maturity and, while open, its range

**State.** Every port, at every level, is in one of four states:

| state | means | example |
|---|---|---|
| decided | a value stated at this level, with who, when and why | design altitude 250 km |
| allocated | a bound handed to a child | propulsion mass at most 12 kg |
| open | to be decided below, with an owner and the gate it is due by | thruster efficiency, open until the design review |
| achieved | computed from the children | 11.3 kg |

So the top can decide what it must and move on, without inventing what it
cannot know yet. A block may start with an estimate, such as propulsion being
15 per cent of dry mass, and be broken down later. The estimate stays as the
budget, the children compute the real value, and the difference is the margin.
This is how mass and power budgets are run on space programmes.

**Maturity.** Every value is estimated, calculated or measured. A closure
demands the margin its least mature input needs, so an early estimate cannot
look as safe as a measurement. Mass control on space programmes grades every
item this way and carries a growth allowance by grade (ANSI/AIAA S-120A). VLEO
already does this for mass alone, citing ECSS; the model does it for every
value.

**Range.** An open value carries the range it may still take, not a blank. Every
closure then says one of three things: it closes for the whole range, so the
decision can wait; it closes for part of it, so decide soon, and here is the
crossing; or it fails for all of it, so change the design rather than the
timing. This is set-based design: carry the set of acceptable values and
narrow it as knowledge arrives (Sobek, Ward and Liker, 1999).

**Which value to decide first.** Each closure ranks the open values behind it by
how far each one moves its margin: a tornado, one input at a time, and later
variance-based indices that include interactions (Saltelli et al., 2008). The
widest bar is the decision that matters most.

*Where this breaks:* a range drawn too narrow, or a maturity claimed too high,
makes a closure look settled. Ranges and maturities are stated, sourced and
signed like any other value.

## 5 · Connections, and the N2

The **N2** of a block is the matrix of its children: each child on the
diagonal, an output along its row, an input down its column, and a mark where
one feeds the other. A mark above the diagonal feeds forward. A mark below it
feeds back, and is a loop. It is a design structure matrix (Eppinger and
Browning, 2012), and it is drawn from the wires, never separately.

- **Every block has its own N2**, so the matrix opens to any depth. The main
  app already draws the whole tree's N2 with the hierarchy as nested boxes on
  the diagonal.
- **A mark that leaves a box** is a connection that crosses a boundary, and
  that block needs a port for it.
- **A loop belongs to the smallest block that contains it.** It is declared
  there, with what must settle and how tightly, and iterated there. An
  undeclared loop is refused by name, never run forever. VLEO declares its one
  architectural loop today in `layers/cycles.toml`: power becomes heat, heat
  sets the array temperature, which sets the power available, which throttles
  the thruster. OpenMDAO puts each solver on the group that holds the cycle in
  the same way (Gray et al., 2019).
- **The matrix advises the breakdown.** Reordered, it exposes clusters of
  blocks that mostly talk to each other: candidates for one block and one
  owner.

## 6 · Groups, valves, and the daily design

A **group** owns a branch: its blocks, its people, its releases. The branch
mounts on a block of the branch above it, and that **mount** is the contract
between the two groups: the ports the child group must answer, the
requirements handed to it with their direction, and the inputs it is given.

The top of the tree is owned the same way. The management perspective is the
programme's branch and the system perspective is the systems branch. They are
groups like any other, and the subsystem groups mount on their blocks. So
there is one mechanism from the top to the bottom, and no special layer.

### Every mount is a valve

Requirements flow down the tree and answers come back up, and every mount is
a **valve** between the two. The owner of the branch above sets what flows
down through it: the allocation, the bound, the direction. They also decide
what comes back up: the release they accept and pass on.

```text
                 MANAGEMENT  (programme manager)
                      │  customers, KPIs, gates, margin policy
                      ▼
   ═══════════  MAIN VALVE  (system engineer)  ═══════════
        │             │             │             │
        ▼             ▼             ▼             ▼
     valve         valve         valve         valve      each a subsystem engineer,
   Propulsion      Power         Solar          …         the system engineer of their own system
        │             │             │
       ...          nodes         nodes                   down to the last break, a node
```

- **The system engineer is the main valve,** between the management side above
  and the subsystems below. Everything the spacecraft is passes through it, both
  ways.
- **Every subsystem engineer is the system engineer of their own system.** They
  set what flows down to their nodes, or to a group mounted below them, and
  decide what goes back up. Their role is the system engineer's, one level
  down, and the same app serves both.
- **It goes to any depth,** to the last break: a node.

*Said simply:* the design is one system broken down into systems, each with an
owner who controls what flows in and out of it, down to the smallest part.

### Today's design, and the released design

Every valve has two views of what is below it:

| | made from | for | changes |
|---|---|---|---|
| **today's** | the latest signed work of everything below | seeing, every day, how everyone's latest work fits together | whenever anyone below signs or seals |
| **released** | what the owner chose to pass on, signed | what every decision, review and gate refers to | when the owner releases |

At the top, **today's design** is every group's latest sealed release, combined
and run. Anyone can open it, every day, and it shows how the whole spacecraft
stands with everyone's latest work. **The released design** is the one the
system engineer releases: the one everyone works from, and the programme
decides on.

The same holds at every valve. A subsystem's today is its node engineers'
latest signed nodes; its release is what its subsystem engineer seals.

### The cycle

1. **The system sets the frame:** where each group mounts, what is allocated to
   it, the shared cases.
2. **Each subsystem breaks its branch down** and hands nodes to its node
   engineers, setting the frame of its own system.
3. **Node engineers write their nodes,** and sign each day's work.
4. **Each subsystem sees today's group,** tries it inside today's design, and
   seals a release when it is right. It can see the result of its change, but
   cannot release it past the main valve.
5. **Everyone sees today's design every day,** and discusses what it shows.
6. **The system engineer integrates,** sees exactly where anything breaks, and
   **releases the design database:** the one everyone works from.
7. **The programme decides last,** against its customers, gates and risks. Its
   decisions are a release of the programme's branch, taken into the next cycle
   through the main valve.

### A parameter belongs to the level that decides it

A parameter is a stated node, owned like any other. The programme owns the
margin policy by maturity, the confidence the design is held to, the gate
criteria and the customers' KPIs. The system owns the allocations, the system
margins and the shared cases. A subsystem owns its own design choices. Nobody
below can change a parameter above; they can only ask.

## 7 · What sits in the database, and what in the code

The design is data. The code makes it run, shows it and checks it.

| in the database (changes the design) | in the code (makes it run) |
|---|---|
| blocks: question, ports, behaviour, cases, explanation, history | the method interpreter and the toolbox it may call, with units |
| wires, mounts and closures | the engine: order, run, iterate loops, refuse and block by name |
| stated values, ranges, maturities, states | range verdicts, tornadoes, sweeps |
| loops declared on their blocks | the pages: block page, tree, N2, map, charts |
| cases, set for a run | the checks, signing and verifying |
| sign-offs, seals, versions | one library that reads and writes every file, in the tool and in the browser |

**The design never passes through the code.** It is written, checked,
combined, signed and published in the apps, by the people who own it. The
checks the apps run are code, the same code the tool runs, so a design passes
the same checks wherever it is opened. The decisions are data.

Only three kinds of change need the code, and each goes to the developer as a
request:

- **a new kind of maths:** a toolbox function a method may call (rule 3,
  `AGENTS.md`);
- **a new kind of thing:** a port type, a behaviour, a picture, a check;
- **a fault in the engine or an app.**

A new use of maths, a new node, a new group, a new wire, a deeper breakdown or
a new requirement is data, and never waits for a developer.

## 8 · The spacecraft, broken down

This is the tree as it stands, and where it is proposed to go one level
deeper. The tree itself is held by the layer files in the repository until
1.0.0, and by the programme and systems groups' files on the shared drive
after it. This section is a picture of it.
When the two disagree, the files are right and this page is stale.

### Today

    VLEO multipayload programme
    ├─ Management · Orbitt Space
    │   customers 1–3 (each: requirement, KPI definition, concept decision,
    │   cost & price, value proposition) · programme control (life cycle,
    │   review gates, schedule, risk register, configuration) · mission
    │   segments (space, launch, ground, user) · standards & compliance ·
    │   supply chain · Orbitt capability
    ├─ The system · VLEO multipayload
    │   service level the customer needs (comms, EO & ISR, PNT: each a
    │   required/achieved KPI pair) · mission concept · mission performance ·
    │   orbit and environment · satellite system (mass & aero, structure,
    │   multi-payload) · satellite subsystems (keep it in orbit, point it,
    │   know where it is, power it, keep it cool, command and downlink it,
    │   carry the payloads) · service payloads
    └─ 18 subsystem groups, each mounted below

### Where each group mounts

Today each subsystem group hangs from the root and reaches its system block by
a relation. On the block model it mounts on that block, and the tree becomes
one tree.

| group | mounts on |
|---|---|
| l3_prop · Propulsion, ICP plasma thruster | sys_propulsion |
| l3_orbmaint · Orbit maintenance | sys_orbit_maintenance |
| l3_acs · Attitude control sizing | sys_attitude_control_sizing |
| l3_atthw · Attitude hardware | sys_attitude_hardware |
| l3_pointing · Pointing error budget | sys_pointing_error_budget |
| l3_navsense · Navigation sensing | sys_navigation_sensing |
| l3_navod · Navigation and orbit determination | sys_navigation_and_orbit_determination |
| l3_power · Power | sys_power |
| l3_thermal · Thermal | sys_thermal |
| l3_fsw · Flight software | sys_avionics_and_software |
| l3_ttc · TT&C and downlink | sys_tt_and_c_and_downlink |
| l3_payload · Payload | sys_payload_interface |
| l3_multipay · Multi-payload | sys_multi_payload |
| l3_massaero · Mass and aero | sys_mass_and_aero |
| l3_struct · Structure | sys_structure |
| l3_x_envorbit · Environment and orbit | sys_orbit_and_environment |
| l3_solar · Solar weather | sys_space_environment, feeding l3_x_envorbit |
| l3_x_closure · Closure and cost | sys_service_level_the_customer_needs |

### Proposed: one level deeper

The next level each group may open, in the order a spacecraft product tree
usually goes from subsystem to equipment. Each is an **open** block until its
group's subsystem engineer breaks it down, and none computes until it does. A
group may break down differently; its subsystem engineer decides.

| group | proposed children |
|---|---|
| Propulsion | thruster head · power processing unit · propellant storage and feed |
| Orbit maintenance | drag make-up · collision avoidance · end-of-life disposal |
| Attitude control sizing | disturbance torques, aerodynamic first · actuator sizing · momentum management |
| Attitude hardware | attitude sensors · actuators · attitude electronics |
| Pointing error budget | knowledge error · control error · stability |
| Navigation sensing | GNSS receiver · GNSS antenna |
| Navigation and orbit determination | orbit determination · ephemeris propagation · accuracy budget |
| Power | solar array · battery · power conditioning and distribution · harness |
| Thermal | radiators · heaters and their control · insulation · thermal interfaces |
| Flight software | on-board computer · modes and autonomy · fault detection and recovery · data handling and storage |
| TT&C and downlink | command and telemetry link · payload downlink · ground passes |
| Payload and multi-payload | each payload's accommodation: power, data, pointing, mass |
| Mass and aero | mass budget · drag area and shape · centre of mass and inertia |
| Structure | primary structure · secondary structure · mechanisms and deployments · launch adapter interface |
| Environment and orbit | atmospheric density · orbit geometry and lifetime · atomic oxygen and radiation |
| Solar weather | already broken down: 65 blocks, F10.7, Ap and Kp design levels, cycle, forecast, storms |
| Closure and cost | the 12 KPI closures · cost per operational year |

## 9 · Where this model breaks

- **Breaking down well is the engineer's skill.** The tool can say when to
  stop; it cannot say what decides a question. That is why the subsystem
  engineer and the system engineer both see every breakdown.
- **Moving a branch moves everyone's work.** A change to a mount is the system
  engineer's, and every group it reaches sees it in a preview before it is
  released (`docs/OPERATING_1_0.md`, section 11, W7).
- **A breakdown can move a value its cases never tested.** Children that
  reproduce every case of the node they replace may still answer differently
  between those cases. Intake's impact list shows every value of another group
  that moved, so the change is seen, not assumed away.
- **Unlimited depth invites over-breaking.** A block that is one formula should
  stay one block. The stop rule in section 2 is the guard.
- **A block without a method cannot show its curve.** On 1.0.0 the blocks whose
  relation is still compiled code are marked so, and the plan migrates them
  group by group (`docs/PLAN_1_0.md`).
- **One number per row runs deep in the engine.** Choices, lists, tables and
  time series as real ports are engine work, not only a new page.

## 10 · References

What each part of the model rests on.

| part | rests on |
|---|---|
| recursion: a system's elements are systems, and the same processes apply at every level | ISO/IEC/IEEE 15288:2023; [SEBoK, Recursion](https://sebokwiki.org/wiki/Recursion_(glossary)) |
| perspectives, not floors | the Arcadia method: operational, system, logical and physical perspectives ([Roques, *Systems Architecture Modeling with the Arcadia Method*](https://www.iste.co.uk/book.php?id=1261)) |
| blocks, ports, connections, requirements, verification cases, and an API to exchange them | OMG SysML v2 and the Systems Modeling API & Services, adopted 2025 ([OMG](https://www.omg.org/news/releases/pr2025/07-21-25.htm)) |
| N2 as a design structure matrix; partitioning, clustering | Eppinger and Browning, *Design Structure Matrix Methods and Applications*, MIT Press, 2012 ([MIT](https://stuff.mit.edu/people/eppinger/SDE-MIT/DSM_Book.html)) |
| feedback below the diagonal; a solver on the group that holds the cycle | Gray et al., *OpenMDAO*, Structural and Multidisciplinary Optimization 59, 2019 ([U. Michigan](https://mdolab.engin.umich.edu/bibliography/Gray2019a)); [OpenMDAO N2 details](https://openmdao.org/docs/latest/features/model_visualization/n2_details/n2_details.html) |
| ranges, not blanks; decide when the information exists | Sobek, Ward and Liker, *Toyota's principles of set-based concurrent engineering*, MIT Sloan Management Review, 1999 ([MIT SMR](https://sloanreview.mit.edu/article/toyotas-principles-of-setbased-concurrent-engineering)) |
| margins by maturity: estimated, calculated, actual | ANSI/AIAA S-120A-2015, mass properties control for space systems ([NASA NTRS](https://ntrs.nasa.gov/citations/20130014265)) |
| which input moves a result: tornado, then variance-based indices | Saltelli et al., *Global Sensitivity Analysis: The Primer*, Wiley, 2008 ([overview](https://en.wikipedia.org/wiki/Variance-based_sensitivity_analysis)) |
| a model's ports described as data any tool can read | FMI 3.0 ([press release](https://fmi-standard.org/assets/FMI_3.0_Press_Release.pdf)) |
| the document is the program: change an input and what depends on it re-runs | reactive notebooks such as marimo ([README](https://fossies.org/linux/marimo/README.md)) |
| one file as the application's document | SQLite as an application file format ([sqlite.org](https://sqlite.org/appfileformat.html)) |
