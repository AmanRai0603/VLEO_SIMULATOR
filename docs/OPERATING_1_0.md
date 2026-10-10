# How 1.0.0 operates

> **Answer first.** On 1.0.0 everyone has one application and one shared drive. Each person looks after their own part of the design database in it every day, and sees how everyone's latest work fits together in **today's design**, rebuilt from the drive whenever they open it. Requirements flow down through valves and answers come back up. The system engineer is the main valve, and is the only one who releases the design that decisions are made on. The developer maintains only the code.
>
> **Kind:** explanation + reference · **For:** everyone

The model is `docs/SYSTEM_MODEL.md`; the phases that build this are
`docs/PLAN_1_0.md`. Until 1.0.0 ships, today's mechanism is the one in use
(`docs/GROUP_APPS.md`, `docs/DRIVE_SETUP.md`), and this page is the target.

*Said simply:* look after your own part, look at the whole every day, and talk
about what it shows. Only the system engineer releases, so every decision is
made on one design.

## 1 · The shape of the work

**Valves.** Every mount in the tree is a valve (`docs/SYSTEM_MODEL.md`,
section 6). Requirements flow down through it, and answers come back up through
it. The owner of the branch above controls both.

```text
                 MANAGEMENT  (programme manager)
                      │  customers, KPIs, gates, margin policy
                      ▼
   ═══════════  MAIN VALVE  (system engineer)  ═══════════
        │             │             │             │
        ▼             ▼             ▼             ▼
     valve         valve         valve         valve      subsystem engineers:
   Propulsion      Power         Solar          …         each the system engineer of their own system
        │             │             │
       ...          nodes         nodes                   node engineers, at the last break
```

**The daily rhythm.**

| when | who | does |
|---|---|---|
| every day | everyone | opens the application; sees *my work* and today's design; works on their own part; signs what is ready |
| when a group's work is right | its subsystem engineer | seals a release; it joins today's design for everyone at once |
| every day, or as everyone agrees | everyone together | looks at today's design's health map and discusses what it shows |
| when today's design is right | the system engineer | releases it: the design everyone decides on |
| after a release | the programme manager | decides on it: gates, risks, customers, KPIs |

**The cycle** each piece of work goes round:

1. The system sets the frame.
2. Subsystems break their branches down.
3. Nodes are written.
4. Subsystems seal.
5. Everyone sees today's design.
6. The system engineer releases.
7. The programme decides, and its decisions go back in through the main valve.

## 2 · Roles

The roles follow the tree, so the organisation and the design have the same
shape. Every owner of a branch is the system engineer of that branch.

| role | is the valve for | owns | releases |
|---|---|---|---|
| **programme manager** | the management side | customers, KPIs, cost, gates, risks, margin policy; the people and their keys; the drive | programme decisions, as releases of the programme's branch |
| **system engineer** | the main valve: the whole spacecraft | where each group mounts, allocations, system budgets and margins, shared cases | **the design**: the only release everyone works from |
| **subsystem engineer** | their own system, at any depth | their group's breakdown, its parameters, its node engineers' assignments | their group's releases, into today's design |
| **node engineer** | the last break | their nodes | signed revisions of their nodes |
| **developer** | none: not in the design | the code only | tool releases |

**Deputies.** The programme file names a deputy for the programme manager, the
system engineer and every subsystem engineer. A deputy signs with their own
key, and the record says so.

One person may hold several roles. Today Aman Rai is programme manager, system
engineer and the subsystem engineer of Solar.

## 3 · What it replaces

Each piece is kept, changed or retired, once. Nothing is laid over the old.

| today | on 1.0.0 | |
|---|---|---|
| "lead", "author", "team", "user" | subsystem engineer, node engineer; everyone uses the application | changed |
| a group app, a node app and a separate tool | **one application**, with a workspace for each role | changed |
| each node as its file in `design/` in the repository, the source | the node in its group's signed files on the shared drive; the repository holds no design | changed |
| the tree's headings in the group files in `design/` in the repository | the programme's and the systems' own files | changed |
| the developer takes a release in with `xtask`, builds, tests and delivers it | the system engineer integrates it in the application, which runs the same checks | changed |
| a test application per release, accepted by the subsystem engineer | today's design, which every group sees as soon as it seals | changed |
| `design.vleo` built from the repository by the developer | today's design, rebuilt from the drive on opening; the released design, released by the system engineer | changed |
| a sign-off as a typed name | a signature with the person's own key | changed |
| `acceptances/` and group branches in the repository | the integration record inside each released design | retired |
| the single-node form and its loop (`form`, `intake`, `take`, `approve`, `preview`, `queue`) | none: every change is a node signed and a group sealed | retired |
| carrying a group as CSV and Markdown (`groups/SPEC.toml`, `tools/group_db.mjs`) | none: the files are the only carrier | retired |
| sealed releases, one writer per file, the de-risking record | the same | kept |
| lessons and their form | the same, until a later release folds them into the node page | kept |

## 4 · The application

**One application, for everyone.** It has one node page, one health map, one
engine and one way of working. What a person can change is what they own;
everything else they can see, run and question, but not change.

**One installed program.** It is the engine: it opens the database folder,
reads every file there, builds today's design, runs it, and makes every check
and signature, on the computer it is installed on. Its screens open in a
browser window on that computer, and the browser only draws what it answers.
The database folder is the one beside the program by default, or any folder it
is pointed at, such as the shared drive (section 15), read and written
directly. There is no page that runs on its own.

**Who you are** is your key: open it once on each computer, with your
passphrase. From the programme file and the group files the application knows
your roles, and opens on **My work**.

### The workspaces

| workspace | for | holds |
|---|---|---|
| **My work** | everyone | everything that needs you now: your nodes behind, failing or unsigned; requests and issues addressed to you; previews to answer; releases waiting at your valve; decisions due |
| **Node** | node engineer | one node, edited in its live page |
| **System** | the owner of a branch: a subsystem engineer for their system, the system engineer for the spacecraft | the branch, its valve below and above, today's view of it, and its release |
| **Programme** | programme manager | mission health, gates, risks, the organisation, decisions |
| **Explore** | everyone | today's design or any released one: run, sweep, compare, the health map, trace to cause |

### Node workspace

| on screen | |
|---|---|
| left | the node's six parts, each marked done, missing or failing |
| centre | the live node page: answer, curve, cases on the curve, the method running line by line |
| right | the source the node comes from, open beside it |

- **Write:** question; ports with type, unit, range, state and maturity;
  behaviour; cases; explanation (section 7 says how data comes in).
- **See:** its neighbours and their values in today's design; its own health;
  any two revisions compared.
- **Ask:** for a contract change or a breakdown.
- **Sign** the day's work.

### System workspace

The same workspace at every valve. The subsystem engineer opens it on their
system, and the system engineer on the spacecraft.

| on screen | |
|---|---|
| left | the branch as a tree, each part coloured by its state in today's view |
| centre | the map, the N2, or a sheet of every node, at any depth |
| right | what is selected, in detail |
| top | the valve: what flows in from above (the mount, its allocations) and what is waiting from below (signed nodes, sealed releases) |

- **Frame:** break the branch down; add, wire and assign its parts; set what
  flows down to each. The sheet sets up many nodes at once.
- **Today's view:** the branch built from everyone's latest signed work below,
  run inside today's design, with its health map, closures, range verdicts and
  tornadoes.
- **Impact:** every value outside the branch that its changes move.
- **Release:**
  - a subsystem engineer *seals* their group, and it joins today's design;
  - the system engineer *releases* the design.
- **People:** who works on what, and what is overdue.
- **Compare** any two of anything.

The system engineer's System workspace has more:
- every budget (mass, power, thrust against drag, pointing, link, thermal),
  each across releases;
- previews and their answers;
- requests for the developer.

### Programme workspace

| on screen | |
|---|---|
| centre | mission health: each customer's KPIs closed or not, with margins; cost against target |
| left | gates: what each asks, and which branches meet it |
| right | risks: the register, what moved each one, the de-risking narrative |
| tab | the organisation: the tree coloured by owner; each person's role, branch, nodes, issues, overdue work and deputy |

- **See:** the released design at the programme level, and today's beside it.
  Every failure is traced down through the system to its group and node.
- **Decide:** at a gate, on a risk, on a customer or a KPI. Each decision is
  signed into the programme's branch and goes in through the main valve.
- **People:** members, roles, deputies and keys.

### Explore workspace

- **Open** today's design, any released design, or two side by side.
- **Run** cases and sweeps.
- **See** the health map at every level, with trace to cause.
- **Keep** results, each naming the design and engine that made them.
- It changes nothing in the design.

## 5 · Today's design

The application builds today's design on opening, from the drive:
- the latest sealed release of every group that passes its checks;
- for a group whose latest is refused, its last good release, marked red with
  the reason;
- the frame from the systems branch, and the programme's branch.

Everyone who opens it builds it from the same files, so everyone sees the same
design. The application says which releases it used and when it built it. If a
computer's copy of the drive is behind, it says so.

Once a day, the system engineer's application keeps a snapshot in `daily/`, so
any day can be looked at again and compared.

Today's design is for seeing and discussing. The released design is for
deciding. The application never lets one be mistaken for the other: every
screen says which it shows.

## 6 · The health map: where exactly it breaks

Every workspace that shows a design colours every node, group and closure:

| state | means |
|---|---|
| **closes** | answered, and every closure it feeds holds its margin |
| **tight** | closes, but within the margin its maturity demands, or only for part of an open range |
| **fails** | a closure it decides does not hold |
| **refused** | the node refused, and says why, such as a value outside its range |
| **blocked** | it cannot run because a node it reads refused or is open; it names that node |
| **open** | not decided yet |
| **unproven** | its cases are not reproduced, it is unsigned, or it is behind its contract |

- **It rolls up and drills down,** valve by valve. A group is as bad as its worst
  node, and the spacecraft as bad as its worst group. One click goes down a
  level, and the N2 there shows the red wires.
- **Trace to cause.** Click a failing closure. The application walks down
  through what decides it, ranks the contributors by how far each moves the
  margin, and stops at the nodes that cause it. It names their group, their
  engineer, and what would make it close.
- **Raise an issue** from any red mark, addressed to a group and a node. It
  closes with the release that resolves it.

## 7 · Bringing data in, as simply as an engineer can give it

Whatever the engineer already has, the application takes it as it is. Nobody
learns a format.

| the engineer has | they | it becomes |
|---|---|---|
| a formula in a paper, a book or their notes | type it as written: `D = 0.5 * rho * v^2 * Cd * A` | the method; inputs found, units asked, the equation drawn |
| a table in a spreadsheet | copy it from Excel or Google Sheets and paste | a lookup table, or cases |
| a worked example in the source | paste its numbers | a case, marked as coming from that source |
| results from their own MATLAB, Python or spreadsheet | save them as CSV and drop the file in | cases and results; the code itself kept as the record |
| a single number with a source | type it with its unit: `250 km`, `55 mN/kW`, `2.2` | a stated value, converted to SI, with its source |
| a number they are unsure of | type a range or a spread: `2.5e-11 to 1.5e-10`, `55 ± 10%` | an open value with its range |
| a PDF, a picture, a datasheet | drop it beside the node | the node's source, linked to its page |
| many nodes to set up | fill the System workspace's sheet, a row each, or paste whole columns | nodes, each opening in its own page |
| something close to what they need | *start from a similar node*, or a template for its kind | a copy, ready to change |

Every field shows an example of what goes in it, and says in plain words what is
wrong as it is typed. Units are read as people write them and converted to SI
behind the scenes. An Excel file can be dropped in directly once a reader for it
is vetted into the application, the way SQLite is today (`web/vendor/`).

## 8 · Making it easy to use

1. **My work first.** It always opens on what needs you now.
2. **One next step, named.** *Sign revision 4*, *Seal Solar 1.2*, *Release
   design 2026.11.2*.
3. **Checked as you type, in plain words, with the fix.** *Case 3 expects
   177.22 sfu and the method gives 175.90: the multiplier differs.* Never a code
   or a hash.
4. **Live.** Answers, curves, cases, closures and the health map move as you
   edit.
5. **Nothing is lost.** Every save a revision, every step undoable, every older
   file kept, and a refusal never deletes.
6. **One application, one page, one vocabulary.** The same node page in every
   workspace, one word for each thing (`docs/GLOSSARY.md`). People and dates are
   shown, not fingerprints.
7. **Pictures first, examples always.** A template for each kind of node, a
   worked example beside every step, the guide for the screen one click away.
8. **It works where people are.** On the shared drive or a folder on the
   computer, online or not. It says plainly when the folder cannot be reached
   or a file in it cannot be saved, and what to do.

These follow Nielsen's usability heuristics, and the ideals of local-first
software: the data is the organisation's, on its own machines, usable offline,
and readable without the vendor.

## 9 · Parameters and measures, by level

A parameter is a stated node, owned by the level that decides it. A level below
can see it and ask for it to change, never change it.

| level | parameters it owns | measures it watches |
|---|---|---|
| programme | each customer's KPIs; the margin policy by maturity; the confidence the design is held to; gate criteria; cost targets | KPIs closed per customer; cost against target; gate readiness; risks open and moved; issues by owner and age; built-in relations still in code |
| system | allocations to each group; system budgets and margins; shared cases; declared loops | each budget's margin, today and across releases; groups whose latest release was refused; previews unanswered; closures failing, tight and refused |
| subsystem | what flows to its own parts; its design choices, ranges and stated values | its nodes proven of total; open and overdue nodes; issues on the group; contract changes in flight |
| node | its inputs' ranges and its cases | its checks, its cases, both ends of its range |

## 10 · The files

Every file is one SQLite database in one schema. Its `meta` says what kind it
is, its version, what it was based on, and who wrote it.

| file | name | holds | written by | changes after |
|---|---|---|---|---|
| **node file** | `<node>.vnode` | one node, all six parts | its node engineer | yes: each save a revision |
| **group file** | `<group>.vgroup` | the branch, its members and their public keys, who is assigned what at which contract version, its mount read-only, its base | its subsystem engineer; for the programme and the systems, their owners | yes |
| **group release** | `<group>-<version>.vleo` | the group file and every node, every signature, release notes, the seal; for the programme, its signed decisions | its owner, sealing | never |
| **preview** | `preview-<group>-<version>.vleo` | the released design with one release in it, when that release moves other groups' values | the system engineer | never |
| **preview answer** | `<group>-<version>.<answering group>.answer` | *fine*, or *object, because…*, signed | the answering subsystem engineer | never |
| **issue** | `<number>-<group>.vissue` | what is wrong, where, the evidence from the health map, whom it is addressed to | whoever raises it | never; closed by the release that resolves it |
| **daily snapshot** | `design-<date>.daily.vleo` | today's design as built that day, and the releases it used | the system engineer's application | never |
| **released design** | `design-<version>.vleo` | every integrated release, mounted into one tree; the integration record; the tool version and toolbox it needs; the system engineer's signature | the system engineer | never |
| **case** | `<name>.vcase` | the inputs for a run | anyone | yes |
| **results** | `<name>.vleor` | a run's answers, and the design and engine that made them | the application | never |
| **key** | `<person>.vkey` | a person's private key, locked by their passphrase | the person | kept by the person, never on the shared drive |

A group release and a released design both end `.vleo`; each file's `meta` says
which it is, and the application refuses one opened as the other, by name.
Comments, requests and changes are kept inside the node or group file they were
written on, so nothing a person writes is lost between versions.

**One writer per file.** Nothing is merged by two people. The application
refuses to sign for anyone but the file's assigned writer, and integration
refuses a signature from anyone the group file did not assign.

## 11 · The work, step by step

Steps W1 to W10 are the cycle; W11 to W16 can happen at any time. Each step says
who, in which workspace, which file, what is checked, and the message that goes
with it.

### W1 · The system sets the frame

1. **System engineer**, System workspace on the spacecraft: sets the mounts, the
   allocations handed to each group with their direction, the system budgets and
   margins, and the shared cases.
2. **To start a new group:** adds its mount, then *Start group*. That writes
   `groups/<group>/` with its group file, mount, base, and its subsystem engineer
   from the programme file.
3. **Programme manager:** shares a new group's folder; its subsystem engineer
   registers their node engineers' keys.
4. A change to the frame shows at once in today's design, and goes out with the
   next release (W8).

Message, to a new group's subsystem engineer: *"<group> is set up. Open it in
the application."*

### W2 · A subsystem sets its own frame

1. **Subsystem engineer**, System workspace on their system: breaks the branch
   down. Each new node starts open, with its question and ports; the sheet sets up
   many at once.
2. Assigns each node a node engineer, and issues the node files into `nodes/`.

Message: *"Your node <node> is in your My work. Due by <gate>."*

### W3 · A node is written, every day

1. **Node engineer**, Node workspace: brings the data in (section 7) and writes
   the node in its live page. It runs as it is written, inside today's design.
2. Each save is a new revision in place. If the file moved on since it was
   opened, the application stops and overwrites nothing.
3. Signs the day's work when its checks pass and its cases agree. The signed
   revision appears at once in today's view of their group.

### W4 · A contract changes

1. **Node engineer:** asks, as a request in the node file.
2. **Subsystem engineer**, System workspace: changes the contract. Its version
   rises, and every node that reads it is named.
3. Re-issues the changed files. Each keeps its content and reads *behind* until
   re-signed.
4. A change to the group's own mount is asked of the system engineer (W1).

Message: *"The contract of <node> changed: <what>. Please check and re-sign."*

### W5 · A subsystem seals, and its work joins today's design

1. **Subsystem engineer**, System workspace: today's view of the group, built
   from every node engineer's latest signed revision. It checks that:
   - every node is signed by its node engineer, at its current contract;
   - nothing is behind;
   - there are no duplicate files;
   - the group's checks pass.
2. Sees it run inside today's design: the health map, every closure the group
   touches, and every value outside the group it moves.
3. Updates the base first if a newer design has been released (W12).
4. Writes the release notes as a de-risking record, naming any issues it
   resolves. The application proposes the version.
5. Seals. The release goes into `releases/`, and **joins today's design for
   everyone** the next time anyone opens it.

If the release moves values in other groups, the application names them, and
those subsystem engineers see it in their My work.

### W6 · Everyone looks at today's design, and discusses it

1. **Everyone**, Explore workspace or their own: today's design, and its health
   map at their level.
2. **Together,** every day or as everyone agrees: the health map on one screen,
   red marks first, each traced to its cause and owner. Each is either
   understood, or raised as an issue (W11).

### W7 · Values that move are answered

When a release moves another group's values:
1. **Each subsystem engineer named:** sees their group run with the change, and
   answers **fine** or **object, because…**, signed.
2. **System engineer**, on an objection, decides one of three:
   - ask the releasing group for its next version;
   - change an allocation in the frame;
   - release anyway, with the objection kept in the record and raised to the
     programme manager.

### W8 · The system engineer releases the design

1. **System engineer**, System workspace on the spacecraft: today's design, with
   the integration checks, refusing any release whole, with the lines that
   failed, if any of these fails:
   - the seal and the signature chain;
   - the base still fitting;
   - every node's method, cases and record;
   - no relation supplied by an assistant;
   - both ends of every declared range.
2. Before releasing:
   - every red mark is understood, and either fixed or raised as an issue;
   - every answer is in.
3. *Release design* signs it. The application writes:
   - the design into `design/` under its version, and as `design.vleo`;
   - the one before it into `design/archive/`;
   - `apps/NOTES.md`, `readable/` and the status.

This is the release every decision refers to. No one else can make it.

Message, to everyone: *"Design <version> is released: <what changed; what still
fails, and its issues>."*

### W9 · Each level reviews the release

| who | looks at | decides |
|---|---|---|
| node engineer | their node in the released design | whether it still holds; asks for a change (W4) |
| subsystem engineer | their system's health map and closures | their system's next changes (W2, W5); issues to raise |
| system engineer | every budget, closure and red mark, across releases | the next frame (W1); what to ask of which group |
| programme manager | the programme level (W10) | |

### W10 · The programme decides

1. **Programme manager**, Programme workspace: sees the released design at the
   programme level. That covers each customer's KPIs, cost, gate readiness and
   risks, with every failure traced to its group and node.
2. Decides: passes a gate or not; accepts a risk, or asks for it to be retired;
   changes a customer's KPI or the margin policy.
3. Each decision is signed and sealed in a release of the programme's branch. It
   joins today's design at once, and goes into the next release through the main
   valve.

Message: *"Programme decision: <what>, on design <version>."*

### W11 · An issue is raised and resolved

1. **Anyone**, from a red mark in any health map: *Raise issue*. It is written
   into `issues/`, addressed to a group and a node, with the evidence.
2. **Subsystem engineer:** sees it in My work and assigns it. It is resolved by a
   release whose notes name it.
3. It closes when that release is in today's design and the mark is no longer
   red.

### W12 · A group's base moves

When a design is released, each group's System workspace shows what changed
under it: inputs it reads from others, and its mount. *Update base* takes the new
mount and the neighbours' values. No node's content changes, and a node whose
inputs changed in meaning is named for its node engineer to check.

### W13 · A node is broken down further

1. **Node engineer, or anyone:** proposes it as a request in the node file.
2. **Subsystem engineer** decides one of two:
   - **Break down.** The node's behaviour becomes its children, and its old
     relation is kept as its estimate.
   - **Mount a new group,** when other engineers will own what is inside. The
     system engineer starts it (W1), and the subsystem engineer becomes its valve
     above, the system engineer of that new system.
3. The children are issued and written (W2, W3). The node's own cases run against
   them, beside the estimate.
4. Sealed as usual (W5): minor if no port changed, major if one did. Integration
   checks one thing more: the children reproduce the node's cases.

### W14 · Something needs the code

A new toolbox function, port type, picture or check, or a fault in the
application or the engine.

1. **Any engineer:** writes the request in My work's *Requests*, with an example.
2. **Developer:** builds it in a tool release (W15). Until then the node stays
   open, or keeps its estimate, and says what it waits for.

### W15 · The tool is released

**Developer only.**
1. The new application must run the **current released design** on every shared
   case and give the same answers. One that changes the design's answers does not
   ship.
2. A release from `main`, tagged `v<version>`.
3. The application goes into `apps/`, one installer for each computer, with its
   checksums, and `apps/NOTES.md` says what changed and whether anyone must act.

### W16 · Something goes wrong

| what | what happens |
|---|---|
| a released design is wrong | every earlier one is in `design/archive/`; the system engineer releases the last good one again under a new version, and the notes say why |
| a group release is wrong | a release is never edited; its subsystem engineer seals the next version, and today's design takes it at once |
| a group's latest release is refused | today's design keeps that group's last good release, marked red, until the next one passes |
| a node file was saved by download and left in Downloads | the subsystem engineer's view names the node as unchanged since its last signature |
| someone is away | their deputy signs |
| the drive cannot be reached | the application works on the files already on the computer; nothing is lost, only delayed |

## 12 · Status, and who hears what

There is no server, so status is part of what the application builds. Every
workspace shows, for each group:
- its working version, and its last sealed release;
- whether today's design took it or refused it;
- its issues open, and previews unanswered;
- which released design holds it.

The system engineer's application writes it to `readable/Status.csv` once a day,
with the snapshot.

Every hand-off is one file in an agreed place, and appears in the receiver's My
work. The same words also go by email or chat, so nobody has to be watching:

| step | from | to | the file | the message |
|---|---|---|---|---|
| W1 | system engineer | subsystem engineer | `groups/<group>/<group>.vgroup` | the group is set up |
| W2 | subsystem engineer | node engineer | `nodes/<node>.vnode` | your node, due by a gate |
| W4 | subsystem engineer | node engineers named | re-issued node files | the contract changed |
| W5 | subsystem engineer | everyone, in today's design | `releases/<group>-<version>.vleo` | sealed; and, to those whose values move, please answer |
| W7 | subsystem engineer | system engineer | the answer | fine, or object |
| W8 | system engineer | everyone | `design/design.vleo` | design released |
| W10 | programme manager | everyone it concerns | `groups/programme/releases/` | programme decision |
| W11 | anyone | subsystem engineer | `issues/<number>-<group>.vissue` | an issue on your group |
| W14 | any engineer | developer | the request | what is needed |
| W15 | developer | everyone | `apps/` | application released, and whether to act |

## 13 · Versions

| what | version | rule |
|---|---|---|
| node file | revision 1, 2, 3 … | every save; a signature names the revision it signed |
| node contract | contract version 1, 2 … | raised when the question, a port, a unit or a range changes; the node reads *behind* until re-signed |
| group release | major.minor, such as 1.2 | **major** when anything another group reads changes; **minor** otherwise |
| today's design | the date, and the releases in it | rebuilt on opening; kept daily as a snapshot |
| released design | year.month.sequence, such as 2026.11.2 | one per release by the system engineer |
| application | 1.0.0, 1.1.0 … | the code; a design names the oldest application that can run it |

**Every file names what it was based on:**
- a node revision, its contract version;
- a group file, the design it was based on;
- a release, its previous release and its base;
- a design, every release in it.

So "what changed since" always has an answer, and the application compares any
two.

**Where the history lives:**

| history of | kept in |
|---|---|
| each edit to a node or group file | the file itself |
| what a group sealed | `releases/`, never deleted |
| how the whole stood each day | `daily/` |
| what was integrated, checked and answered | each released design's integration record |
| every released design | `design/archive/` |

Google Drive keeps its own file history too; it is a backup, not the record.

## 14 · People and keys

Each person makes a key once, in the application, and keeps the private half as
`<person>.vkey` on their own computer, locked by a passphrase.

- **The anchor.** The programme manager's key fingerprint is written in START
  HERE, which only the programme manager can edit. The application shows it on
  first use and remembers it.
- **The chain.**
  - The programme file lists the system engineer's, every subsystem engineer's,
    and every deputy's public key.
  - Each group file lists its node engineers'.
  - A node signature checks against its group file, a group seal against the
    programme file, and a released design against the system engineer's key
    there.
- **A lost key** is replaced the way it was registered. Old signatures still
  check against the old public key, which stays in the file's history.
- **A key someone else may have** is revoked from a date by whoever registered
  it. Signatures made with it after that date stop checking, and the work they
  covered is signed afresh by its writer.
- **The programme manager's own key** is replaced by a new fingerprint in START
  HERE, with a note signed by the old key. If the old key is lost too, every
  application asks the person using it to accept the new anchor once, and says why.
- **Someone leaves.** Their nodes are reassigned and signed afresh.
- **Releases from before 1.0** carry a name and a fingerprint, not a key's
  signature. At the switch-over the programme manager anchors each one once, by
  its fingerprint; after that only key signatures count.

Signing works the same installed or from a page opened from a file, with no
network. It is one implementation, in the library that reads every file,
compiled for both, so a signature made in one always checks in the other.

## 15 · The shared drive

```text
Vleo Database/
  START HERE                     the drive, the daily rhythm, the programme manager's key
  apps/                          the application: installed for each computer, and as a page · CHECKSUMS · NOTES.md
  guides/                        one guide per role
  design/
    design.vleo                  the released design
    design-2026.11.2.vleo        the same, under its version
    NOTES.md                     what changed in each, newest first
    archive/                     every earlier release
  daily/                         a snapshot of today's design, one per day
  integration/                   previews, and the answers to them
  issues/                        every issue raised
  readable/                      Groups · Nodes · Ports · Wires · Closures · Health · Status, as CSV
  groups/
    programme/                   programme.vgroup · releases/
    systems/                     systems.vgroup · releases/
    l3_solar/                    l3_solar.vgroup · nodes/ · releases/
    …                            one folder per group
  cases/                         shared cases
```

**Who may change what.** The programme manager sets this once, by folder:

| folder | can edit | can read |
|---|---|---|
| `apps/`, `guides/` | the developer | everyone |
| `design/`, `daily/`, `readable/`, `integration/` | the system engineer | everyone |
| `integration/<group>-<version>/` | also the engineers whose values it moves, for their answers | everyone |
| `issues/` | everyone, each their own file | everyone |
| `groups/programme/` | the programme manager | everyone |
| `groups/systems/` | the system engineer | everyone |
| `groups/<group>/` | that group's subsystem engineer and node engineers | everyone |
| `cases/` | everyone | everyone |

**Rules the application holds to:**
- `releases/` holds sealed releases only; anything else there is ignored, and the
  application says so.
- Two files where there should be one, such as a name ending `(1)`, are named,
  and their owner keeps one.
- An application release never touches the design.
- **One person, two computers.** The application writes a file whole, never
  leaves a working copy on the drive, and refuses to save over a file that
  changed on the drive since it was opened. It shows both, and the person keeps
  one.

## 16 · The application and the files across versions

- **Every file states its format version, and the application the formats it
  reads.**
- **A newer application opens an older file,** upgrades it when it saves, and
  keeps the old one beside it.
- **An older application refuses a newer file by name,** and says which version
  it needs.
- **An application release is the installers in `apps/`.** Installing the new
  one is the update; it shows its version.
- **A design names the oldest application that can run it.**
- **An application release never changes anyone's files.**
- **Cases and results saved on a computer before 1.0** open and upgrade the same
  way, with the old copy kept.

## 17 · The developer's side

The repository holds the code and nothing of the design:
- the kernel and its toolbox;
- the method interpreter and the engine;
- the one library that reads, writes and checks every file;
- the application, one installed program with its screens;
- the tests.

The tests run on the example group, and on a copy of the current released design
taken when preparing an application release (W15). `AGENTS.md` governs the
code. The design's rules are enforced by the library's checks, which the
application runs, and described for people in these pages.

| branch or tag | holds |
|---|---|
| `developer` | code changes before release |
| `main` | released code |
| tag `v<version>` | an application release |

## 18 · Where this breaks

- **The main valve is one person.** Everyone sees today's design without waiting
  for them, but every decision waits for their release. A deputy is named, and
  every step is in the record.
- **Today's design is only as current as each computer's copy of the drive.** The
  application says which releases it used, and when.
- **Seeing is not deciding.** Today's design is for looking and discussing. A
  decision made on it instead of a release has nothing signed behind it, so every
  screen says which one it shows.
- **A message not sent is a hand-off not made.** My work shows it either way;
  sending it is still a person's job.
- **The drive enforces folders, not files.** One writer per file is held by the
  application and by integration's signature check.
- **There is no code review for the design.** In its place: comparisons, health
  maps, today's design in front of everyone every day, and signatures.
- **Trace to cause ranks what moves a margin.** It cannot say which node is
  wrong; that is the engineers' judgement.
- **A passphrase forgotten is a key lost.** It is replaced, never recovered.

## 19 · References

| for | see |
|---|---|
| usability: status, error prevention, recovery, recognition rather than recall | Nielsen's ten usability heuristics ([NN/g](https://www.nngroup.com/articles/ten-usability-heuristics/)) |
| the data is the organisation's, offline, lasting beyond the software | Kleppmann et al., *Local-first software*, Ink & Switch, 2019 ([inkandswitch.com](https://inkandswitch.com/local-first/)) |
| opening a whole folder from a page, to read and write | the File System Access API ([web.dev](https://web.dev/articles/files/open-a-directory)) |
| a change reviewed against the main file before it is merged | branching and review in design tools ([Figma](https://help.figma.com/hc/en-us/articles/360063144053)) |
| margins by maturity, checked at each review | ANSI/AIAA S-120A-2015 ([NASA NTRS](https://ntrs.nasa.gov/citations/20130014265)) |
| ranking what moves a result | Saltelli et al., *Global Sensitivity Analysis: The Primer*, 2008 ([overview](https://en.wikipedia.org/wiki/Variance-based_sensitivity_analysis)) |
