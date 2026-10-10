# The plan to 1.0.0

> **Answer first.** One release, 1.0.0, built in eight phases and switched over on one planned day. After it everyone has one application and one shared drive; each person looks after their own part of the design database every day, sees today's design built from everyone's latest work, and the system engineer releases the design decisions are made on. The developer maintains only the code. Each phase proves what it builds in CI and merges into `developer` on your word; nothing changes for anyone until the switch-over. There is no trial before the release: 1.0.0 ships once everything is built and the design is switched over, and you use it after, bringing what you find as the next update. Audited against the code on 5 October 2026, it is about twenty working days of building, at the pace this repository has kept: 1.0.0 around 2 November 2026 if parity holds first time and each phase is reviewed the day it is ready, a week later if parity does not.
>
> **Kind:** explanation + reference · **For:** everyone

What 1.0.0 is: the design model, `docs/SYSTEM_MODEL.md`, and how it is
operated, `docs/OPERATING_1_0.md`. That covers the valves and the daily rhythm,
the roles, the application and its workspaces, today's design, the health map,
how data comes in, every file, version and folder, each step from W1 to W16,
and who hears what. This page is only how we get there.

## Why 1.0.0, and why one release

0.5.0 was never shipped, and what changes is larger than a minor version: the
file formats, the rules, the application, the roles, and where the design lives.
So the next release is **1.0.0**, and it ships once. After it:

| what | made by | when |
|---|---|---|
| an application release, the code | the developer | rarely |
| today's design | the application, from the drive, for everyone | whenever anyone opens it |
| a released design | the system engineer, and no one else | when today's design is right |
| a programme decision | the programme manager, as a release of the programme's branch | after a released design |

## The one idea every phase serves

**The design never passes through the code again.** The developer moves it out
of the repository once, at the switch-over. From then on it is written, checked,
combined, run, signed and released in the application, by the people who own it.
Everything the developer does today on a group's release (intake, build, test,
deliver, accept, compile) becomes checks in one library, which the application
runs at every valve.

## While it is built

- **Nothing changes for anyone until the switch-over.** Today's apps, tool and
  loop stay in use as documented now (`docs/GROUP_APPS.md`). A group may keep
  sealing on them. What it seals is upgraded on the day, and checked against
  today's answers.
- **The new application is built beside the old, not patched into it.** The old
  ones retire on the switch-over day, so nobody has two of anything to choose
  between.
- **No trial before 1.0.0, by your choice.** Everything planned is built and
  proved in CI, the design is switched over, and 1.0.0 ships. You use it after
  the release, in every role, round the whole cycle, and what you find comes
  back as requests (W14) for the next update. So every workflow, W1 to W16, is
  proved end to end in CI before it ships, because nobody tries it first.

## The phases

| phase | needs | delivers | who | size | days of building |
|---|---|---|---|---|---|
| A · Safe ground | nothing | the application refuses a design it cannot run; results name their design | developer | small | done |
| B · Rules and roles | A | the code's rules, the design's rules, the valves and the role names; your approval | developer, you | medium | 1 |
| C · Files, versions and keys | B | one schema, every file kind, versions, signatures, the one library with every check | developer | large | 3 |
| D · The engine runs the design | C | today's answers recorded; the graph read from the design file; today's design; the health map; the parity gate | developer | large | 4 |
| E · The design leaves the repository | D | the programme, systems and 18 groups as files; every relation in code moved into its node's method | developer | medium-large | 2 |
| F · The application | C, D, then E | one application, installed and as a page, with its five workspaces, doing W1 to W16, each proved end to end in CI | developer | the largest | 5 |
| G · The drive and the people | E, F | the drive's layout and sharing, keys, one guide per role, START HERE | developer, programme manager | small-medium | 1 |
| H · Switch-over and release | G | the design converted for the last time, the new drive, the first released design; then 1.0.0 | developer, you | one day |

### A · Safe ground

- The application checks `design.vleo` against its engine and toolbox, and
  refuses a mismatch by name.
- Every result carries the fingerprint of the design and the engine that made
  it, and is marked stale when either changes.
- The method checker is rebuilt whenever the kernel it runs changes:
  `crates/vleo-sheet/src/method/cases.rs` lists the kernel's own files among its
  sources.
- `docs/SOLAR_INVENTORY.md`, `docs/SOLAR_ROWS.md` and the repository's copy of
  the Solar group folder are each checked against the tree or removed.

**Done when** a design built for another engine is refused in a test.

### B · Rules and roles

- **The code's rules:** `AGENTS.md`, `CONTRIBUTING.md` and the area files,
  rewritten for a repository that holds only code.
  - The four rings, portable maths, refusal never substitution, and every
    formula in the kernel all stay.
  - An application release must give the current released design's answers
    unchanged (W15).
- **The design's rules,** written as checks in the library and described for
  people in the operating model:
  - one writer per file, and the signature chain;
  - an expected value never comes from the code under test;
  - no relation supplied by an assistant, and a transcription signed by the
    person who checked it;
  - every requirement says which way it binds;
  - a node reads its children only through their ports;
  - a release is never edited;
  - a parameter is changed only by the level that owns it;
  - every valve's owner controls what passes it;
  - the design is released by the system engineer alone, and today's design is
    never taken for a released one.
- **The roles and their names:** programme manager, system engineer, subsystem
  engineer, node engineer, developer, each with a deputy. Every owner of a branch
  is the system engineer of that branch. Every page, screen and guide uses these
  and no other.
- `docs/WORK_MODEL.md` is replaced by the operating model, not kept beside it.

**From the audit:**
- **Words, not stored names.** B renames what people read: the rules, the
  documents, the screens and the guides. The role names stored in files (the
  group schema's `owner`, `author` and `reviewer` in `groups/schema.sql`, and the
  sheet's `[author]` section) change with C's one schema, or saved files would
  break twice.
- **"Lead" is also a forecast's lead time.** About 1,400 uses in the solar rows
  mean that, and stay. Only the role is renamed, so the sweep is read, not
  replaced mechanically.
- **Today's loop keeps working.** The form loop, its approval check and its
  preview builds (`.github/workflows/preview.yml`) stay, under a section headed
  *Until the switch-over*, which phase H deletes.
- **`areas/teams.toml` stays a code concept.** It says who reviews which crate,
  not who owns which part of the design.
- **A second reviewer** must be named. Today every review team is one person.

**Done when** you approve, with a second reviewer.

### C · Files, versions and keys

- **One schema** for every file kind in section 10 of the operating model:
  - a block, whose parent goes to any depth;
  - a port, with type, unit, range and its reason, state, maturity and
    direction;
  - wire, mount, closure, loop, case, text, table, media, signature, change,
    request and issue.
- **Identity and versions** of every file, by the rules in section 13.
- **Keys:** make a key, lock it by passphrase, sign, and check a signature
  through the chain in section 14.
- **One library** reads, writes and checks every kind, compiled for the installed
  application and for the page.
  - Every check the developer's `xtask` runs today on a group's release moves
    into it: the seal, the signatures, the method and its units, the cases, both
    ends of every range, the de-risking record, and the assistant rules.
- **Upgrade:** a file in today's format upgrades when opened, and keeps a copy.
- **Nothing a group writes is dropped.**
- **Comparison:** any two revisions, releases or designs, node by node.

**From the audit:**
- **Signing starts from nothing.** Today a sign-off is a name and a fingerprint;
  there are no keys and no cryptography in the repository. Signing and checking
  are written once, in the library, and compiled for both the installed
  application and the page. That needs Ed25519, SHA-256, PBKDF2 and AES-GCM
  adopted through `ADOPTION.lock`, which is your word.
- **About 70 checks exist only in the page.** `web/js/gcheck.js` and the seal
  rules in `web/js/gseal.js` move into the library. The fingerprint is written
  twice today, in the page and in `xtask`, and the two have already drifted.
- **SQLite in the page.** The installed library uses SQLite directly; in the
  page, it checks the rows the page's own SQLite hands it.
- **Signatures from before 1.0** are names, so they cannot check through a
  chain. Each pre-1.0 release is anchored once, by its fingerprint, by the
  programme manager at the switch-over; from then on only key signatures count.
- **Nothing dropped** includes the comments and issues a group writes today,
  which the list of kinds above did not name.
- **Every reader follows the schema:** `vleo-py`, `vleo-cli`, `vleo-server` and
  `vleo-ffi`, and the cases and results saved under `~/.vleo`.
- **Checks that need the engine** (building a node, both ends of a range, the
  group's own results) move in D, not C.

**Done when**:
- Solar 1.1 upgrades with nothing dropped, from a copy committed as a test
  fixture, and its pre-1.0 sign-offs are anchored by fingerprint;
- a release signed afresh checks through the chain, installed and in the page;
- the library refuses everything today's intake refuses, in a test for each.

### D · The engine runs the design

- The graph is built from the design's files when the application opens, not
  compiled into it, both installed and in the page.
- **Today's design:** built from the drive on opening, from every group's latest
  sealed release that passes its checks. A refused release is replaced by its
  group's last good one and marked. The application says which releases it used.
  The same at every valve: a group's today is built from its nodes' latest signed
  revisions.
- Behaviours:
  - method, run by the interpreter;
  - children;
  - stated;
  - lookup;
  - open, refused by name;
  - built-in: a relation still in compiled code, found by its node's id until
    its group writes a method for it.
- Loops are declared on the block that holds them. Ports are typed: number,
  choice, list and parameter first.
- Every value carries its state, maturity and range. Every closure gives its
  range verdict and tornado.
- Every node gets its health-map state, with the roll-up valve by valve and the
  trace to cause of section 6.
- A design names the application version and toolbox it needs.

**The parity gate.** For every row in every case, the new engine gives today's
answer within the row's own tolerance, and refuses where today refuses. There is
no switch without it.

**From the audit:**
- **Today's answers are recorded first, and are.** No baseline existed:
  tolerances were per fixture, and only one case is shared. `baseline/today.csv`
  now holds what today's engine answers for every row in every shared case, with
  the reference data and without it, at both ends of every input's range, and on
  every fixture, with every refusal and its reason. `cargo test` holds the engine
  to it exactly (`baseline/README.md`).
- **Built-in relations keep their compiled code, so their parity is exact.**
  Methods run in the interpreter share the same portable maths, so they are
  held exact too, and any difference is a defect.
- **The graph becomes data through the resolver's existing seam**, the node
  table in `crates/vleo-core/src/graph.rs`. The guards generated from each sheet
  today (units, limits, not-finite) move to run time in the same order with the
  same reasons. The one loop, in `layers/cycles.toml` until the sheets left and
  in the systems' file since, is reproduced with its order, seed and residual.
- **The interpreter is measured.** If a sweep is too slow, translated code stays
  as the fast path, held equal to the interpreter. *Measured in E (stage 4), on
  a release build:* the solar design and closure figures take the same time
  both ways, and the heaviest case, `l3_solar_interface` swept over 100,001
  points, takes 1.6 times as long (0.27 ms a point against 0.17). The fast path
  is off; every method runs in the interpreter.
- **Ports in 1.0 are number and parameter.** The bus carries only numbers; choice
  and list follow after 1.0, unless a node needs one first.
- **Credibility is part of parity**, as it is computed today.
- **Every face is re-pointed:** the command line, Python, the C interface, the
  server, and the engine in the page. The check made in A changes meaning, from
  "the same compiled tree" to "a toolbox this design can run on".

**Done when**:
- parity holds for the whole design on every shared case, installed and in the
  page;
- today's design is built identically on two computers from the same drive;
- a deliberately broken node is traced to by name from the KPI it breaks.

### E · The design leaves the repository

This is the last time the design passes through the code.

- The layer files and all 1,396 rows become files: the programme's branch, the
  systems branch and the 18 group branches.
  - Each row's layer becomes its perspective tag.
  - Each group mounts on its system block, with its mount drawn from today's
    interface rows (`docs/SYSTEM_MODEL.md`, section 8).
  - The breakdown is carried as its owners wrote it. The 1,076 rows not filled
    yet are part of it: each has its name, place, owner and role, and becomes an
    open block, refused by name if a run reaches it, never given a value.
  - Only the blocks the breakdown does not hold yet are added, as open blocks
    each saying why, for its owner to confirm or remove: five of the next level
    `docs/SYSTEM_MODEL.md` proposes. A regrouping of the rows already there is
    proposed separately, group by group, and changes nothing until you accept
    it.
  - Today's parameters are placed at the level that owns them.
- **Nothing of the design stays in code (your word).** The 158 relations still in
  code move into the design as methods:
  - the toolbox functions they call are opened to the method language: the
    formulas stay in the kernel, and which one a node uses, with which inputs
    and constants, becomes the node's method;
  - each is transcribed, declared `transcribed` with its source, and held by the
    parity gate to the code it replaces, to the bit, at every case, both ends of
    every range and on bad inputs; it waits for your signature, given in the
    application before "Ship 1.0.0";
  - a solar-weather relation moves exactly, whatever the language needs. Any
    other that cannot be transcribed exactly loses its relation and keeps its
    block, open, with the expression and source it had, for its group to write
    again;
  - then the built-in code is deleted.
- **Freeze and switch (your word), at the end of E.** The converted files become
  the design's only source, and the design is frozen until 1.0.0. The node
  sheets, the layer files, the generators, the `vleo-mod-*` crates, today's
  loop and its commands, and today's pages leave the repository then, not at H.
  The application in F is built on the real converted design.
- The repository's tests keep the example group, and a copy of the converted
  design as the regression for W15.

**From the audit:** the conversion itself is mechanical; the work is cutting off
everything that reads the sheets when the code is built. That is the generators,
the gate, the readers' folder, the de-risking narrative, the manual and the
catalogue, the contract's recorded examples, CODEOWNERS, the panels and bundles
that name rows, about 36 row ids written into the code, and the translated
methods.

**Done when**:
- the converted design passes the parity gate;
- its N2 shows the one declared loop;
- no relation of the design is in code: each is a method waiting for its
  signature, or an open block saying why;
- the repository builds and tests with no node sheet, layer file or node crate
  in it.

### F · The application

One application, installed and as a page from the drive, with the same screens.
It knows each person by their key, and opens on My work.

Staged in `docs/PLAN_F.md`: nine stages, what each builds on, and the scenario
that proves each.

**Built in this order (your word),** each workflow proved end to end in CI as it
lands, and all of it in 1.0.0: the shell and the key; Explore; Node, with the
breakdown itself edited there; System at the subsystem's valve, then the
system engineer's; Programme; then the easier ways of bringing data in.

**Growing the breakdown is the core of it.** The design is broken down as far as
it is understood, and goes deeper as understanding grows. So a block is added,
broken down, moved, renamed, split, folded back or removed in the application,
by its owner, with its history saying why. An open block is filled a part at a
time, and every branch shows what is still open.

- **My work:** everything that needs the person now, at their valve.
- **Node workspace:** W3, the node engineer's side of W4 and W13.
- **System workspace:**
  - one workspace at every valve: W1 and W8 for the system engineer, W2, W4, W5,
    W7, W12 and W13 for a subsystem engineer;
  - today's view of the branch, impact, the sheet view, people, comparisons;
  - for the system engineer, budgets across releases, previews and requests.
- **Programme workspace:** W10; mission health, gates, risks, the organisation,
  measures across releases.
- **Explore workspace:** today's design and any released one, side by side; runs,
  sweeps, the health map, trace to cause; W6's daily discussion on one screen.
- **Bringing data in,** every way in section 7 of the operating model. A formula
  is typed as written. A table is pasted from any spreadsheet. A worked example
  becomes a case, and CSV results from MATLAB, Python or a spreadsheet are taken
  as they are. Units are written naturally, and a range or a spread as typed. A
  PDF beside the node, and *start from a similar node*.
- **Made easy to use** by the eight points in section 8 of the operating model.
- Today's group and node apps, the single-node form and today's intake and
  delivery commands are removed at the switch-over.

**From the audit:**
- **There are four fronts today, not one:** the tool's page, the group and node
  pages, the single-node form and the readers' folder. The readers' folder
  becomes the Explore workspace's read-only export.
- **One engine for both.** The engine runs in the page in both, and the installed
  application is a local shell serving the same page, so the screens cannot
  differ. The local server's routes stay for Python and the command line, and
  the contract tests and mock engine follow them.
- **New ways in are new code:** a table pasted from a spreadsheet (tab-separated
  or as the spreadsheet copies it), a typed range or ±, natural units checked by
  the engine's own unit parser, and *start from a similar node*. None exists
  today.
- **No workflow is driven end to end in a browser today.** The CI below is about
  thirteen scenarios, each with more than one person.

**Done when** CI drives every workflow on the example group, installed and as a
page, with no developer step in W1 to W13. That includes:
- a second writer stopped in W3;
- a sealed release appearing in today's design for a second person;
- a refused release replaced by its last good one, and marked;
- an objection answered in W7;
- a failing closure traced to its node, raised as an issue, and closed by the
  release that fixes it;
- a node broken down whose children reproduce its cases, and one whose children
  do not;
- a new group mounted on a node, its owner becoming the valve above it;
- a design released, reviewed at each level, and a programme decision taken back
  into today's design;
- each way of bringing data in.

### G · The drive and the people

- `tools/drive.py` packs only what the developer owns: `apps/` with the
  application, installed and as a page, its checksums and notes, and `guides/`.
  Everything else is written by the application.
- START HERE, with the daily rhythm and the programme manager's key fingerprint.
- The sharing table, as a checklist for the programme manager.
- One guide per role: node engineer, subsystem engineer, system engineer,
  programme manager, developer. Each is written from the operating model, with
  the role's day and its workflows step by step.
- `docs/GROUP_APPS.md`, `docs/DRIVE_SETUP.md` and `docs/PIPELINE.md` replaced,
  not kept beside the new ones.

**From the audit:**
- `docs/PIPELINE.md` and the role guides are generated: the first from the table
  in `xtask/src/pipeline.rs`, the others from `docs/manual.toml`. Their sources
  change, and the guides go from three to five.
- Up to 37 files link to the four that go, and each link is fixed.
- The release, kit, wheel, preview and drive workflows ship the new application.

**Done when** CI follows each role's guide, step by step as it is written, from
an empty drive to a released design, so a guide that says one thing while the
application does another fails before anyone reads it.

### H · Switch-over and release

One planned day. Nothing is overwritten.

1. **Freeze.** The design has been frozen since its conversion at the end of E.
2. **Parity, once more.** The converted design gives today's answers on the day.
3. **A new drive beside the old one.** The new layout is built as *Vleo
   Database*, beside today's, which becomes *Vleo Database (0.4, read only)* and
   is kept for a month.
4. **Share and register.**
   - The programme manager shares each folder by the table, writes their key
     fingerprint into START HERE, and registers the system engineer's, each
     subsystem engineer's and each deputy's key.
   - Each subsystem engineer registers their node engineers'.
5. **First release.** The system engineer releases the first design. The
   developer puts the 1.0.0 release candidate of the application into `apps/`.
6. **Ship.** You say **"Ship 1.0.0"**. The application is tagged `v1.0.0`, and
   START HERE goes to every subsystem engineer.

**Done when** the first design is released from the new drive with parity
holding on the day, and 1.0.0 is tagged.

There are no days of real use before the release. The first use is yours,
after it (*After 1.0.0*).

## Timeline

Days of building, one phase after another, at the pace this repository has kept
(about 115 pull requests in its first four weeks). Each phase is ready for your
merge on the day shown; the dates assume you review it that day. What cannot be
made faster is people: your reviews, the second reviewer, and the switch-over
day itself.

| phase | building | ready for your merge | what can move it |
|---|---|---|---|
| A · Safe ground | done | now, pull request 117 | |
| B · Rules and roles | 6 Oct 2026 | 6 Oct | the second reviewer |
| today's answers recorded (D) | done | now, pull request 117 | |
| C · Files, versions and keys | 7–9 Oct | 9 Oct | the cryptography crates' adoption |
| D · The engine runs the design | 12–15 Oct | 15 Oct | parity: up to five more days if it does not hold first time |
| E · The design leaves the repository | 16–19 Oct | 19 Oct | |
| F · The application | 20–26 Oct | 26 Oct | |
| G · The drive and the people | 30 Oct | 30 Oct | |
| H · Switch-over and release | 2 Nov | "Ship 1.0.0" around 2 Nov | the day you set |

- **The one lever.** F's shell and Node workspace need only C. Built beside D,
  they bring 1.0.0 about two days earlier, at the cost of two pull requests open
  for your review at once.
- **The one risk to the date** is parity, in D. Everything after it waits.
- **Every phase keeps the same bar:** the gate and every test green before a
  push, and every new test shown red against a broken implementation first. A
  day is not saved by lowering it.

## After 1.0.0

- **You use it, and what you find is the next update.** In every role, round
  the whole cycle, W1 to W10. Each thing you find comes back as a request (W14)
  and is answered by an application release, which gives the released design's
  answers unchanged (`AGENTS.md`, rule 5).
- **Solar 1.2 goes round the whole cycle,** signed, sealed, seen in today's
  design, released, reviewed at each level and decided on, without the
  developer touching it:
  - `sw_band_confidence` becomes the one parameter its four methods read, in
    place of the literal 1.28;
  - `sw_kp_driving_slot` is declared.
- Each group owns the methods E transcribed from the code, in the Node workspace.
  - 158 of the 190 computing nodes start as transcriptions: the relation the
    code ran, copied line for line, waiting for its owner's signature, and
    held to what the code answered (`baseline/transcribed.csv`).
  - A group rewrites one as its own understanding grows; the application
    shows any difference from the transcription before it is released.
- Groups break their branches down further, as data, with no developer.
- New kinds of maths and features arrive as requests (W14) and application
  releases.
- Later: variance-based sensitivity and the probability each closure holds; an
  optimiser over the open ranges; each closure's verification method; nodes that
  keep state from one time step to the next; SysML v2 and FMI exchange.

## What needs your word

- The rules and roles in phase B, and the second reviewer.
- Adopting the cryptography crates in phase C.
- Anchoring the pre-1.0 releases by fingerprint at the switch-over.
- Number and parameter ports only in 1.0.
- One engine in the page, for the installed application too.
- Each phase's merge.
- Your signature on each relation transcribed into a method, in the application.
- The regrouping proposed for the breakdown, item by item.
- The switch-over date.
- "Ship 1.0.0".

The developer does not merge, release, change a rule or touch the design on its
own initiative, and no assistant supplies a relation.

## Where this plan breaks

- **There is no baseline today.** Parity needs today's answers on record before
  anything changes, which is why they are recorded first.
- **Signing is built from nothing.** Keys, the chain and the passphrase lock are
  new code in C, held equal installed and in the page.
- **The parity gate is the hard part.** If the engine cannot reproduce today's
  answers, phase D stops until it does, and everything after it waits.
- **The application carries everything now.** With no developer in the loop,
  every check the developer made by hand must be in the library, and the
  application must be easy enough that nobody needs one. Phases C and F are the
  largest, and with no trial before the release, CI is the only test before you
  use it: every workflow, W1 to W16, is run end to end there.
- **Today's design depends on the drive being in step.** Phase D proves two
  computers build the same one, and the application always says which releases
  it used.
- **People learning a new application, roles, keys and a daily rhythm,** with no
  days of real use before the release. The first real use is yours, after
  1.0.0; what it finds is the next update, and the old drive is kept read only
  for a month in case the new one has to wait.
- **Built-in relations are a debt.** On 1.0.0 most nodes still compute in code.
  They are marked, listed by owner, and replaced by methods after, not hidden.
