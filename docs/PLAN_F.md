# Phase F, stage by stage

> **Answer first.** Phase F builds the one application in nine stages, in the order the plan fixes: the shell and the key, Explore, Node, the breakdown, System at a subsystem's valve, System at the system engineer's, Programme, the ways of bringing data in, and the whole cycle. The application is one installed program: it opens the database folder (beside it by default, or any folder it is pointed at, such as the shared drive), runs the engine, makes every check and signature, and shows its screens in a browser window on the same computer. It replaces today's pages as each workspace supersedes them. Each stage is a few pull requests into `developer`, each proved by a browser scenario in CI driving the installed program. It starts lower than the plan assumed: no front end can make or use a key today, no design file registers a person, and today's design, the health map and comparison have no route a screen can call. About ten working days of building, not the five the plan gave.
>
> **Kind:** explanation + reference · **For:** the developer, and you, who approve each stage

What phase F must deliver is `docs/PLAN_1_0.md`, phase F, and what the
application is, screen by screen and step by step, is `docs/OPERATING_1_0.md`,
sections 4 to 8 and 11. This page is only the order it is built in, what each
stage stands on, and how each is proved.

## Said simply

The application is built the way a person will meet it. First it must know who
they are and open the database (F1). Then everyone can look at today's design
(F2). Then a node engineer can write a node (F3) and grow the breakdown (F4).
Then a subsystem engineer can frame, seal and answer (F5), the system engineer
can set the frame and release the design (F6), and the programme manager can
decide (F7). Then the easier ways of bringing data in (F8), and last the whole
cycle, every workflow, run end to end with several people (F9).

Every stage leaves a working application that does a little more, and every
stage is proved by a browser in CI doing what a person would do, with more than
one person where the workflow has more than one. Nobody tries it before 1.0.0
(`docs/PLAN_1_0.md`, *While it is built*), so those scenarios are the only trial
it gets.

## How the application works

*Declared: your word, 10 October 2026.*

**One program does everything.** The installed application (`Start VLEO.exe`
on Windows, `vleo-daemon` elsewhere) is the engine. When it starts, it:

1. opens the database folder: the folder beside the program by default, or any
   folder it has been pointed at, such as the shared drive (a network drive, or
   a Google Drive folder kept on the computer);
2. reads every file there: the programme's, the systems' and every group's,
   their nodes, methods, cases and releases;
3. builds today's design from them, runs the physics engine, and makes every
   check and every signature, natively;
4. shows its screens in a browser window on the same computer, at
   `127.0.0.1`. The browser is only the screen: it draws what the program
   answers, the numbers of every figure included, and computes nothing itself.

A save is the program writing the file into the folder, whole, through the one
library that reads and checks every file; it refuses to save over a file that
changed in the folder since it was opened (`docs/OPERATING_1_0.md`, section
15). Everyone on the same shared folder therefore builds the same today's
design from the same files.

This is how the tool already runs today (`vleo-daemon` serving `web/`), so
phase F grows that program and its screens rather than building a second one.

## What there is to build from

*Derived, from a survey of the code at `developer`, 10 October 2026.*

| what | where | state for phase F |
|---|---|---|
| keys: make, lock by passphrase, sign, check | `vleo_files::keys` | in the library; **nothing calls it but its tests** |
| the signature chain | `vleo_files::chain` (`open_programme`, `open_group`, `check_node`, `check_release`, `check_design`) | in the library, used only by its tests |
| people, keys, assignments, requests, issues, comments, changes | format-2 schema, `vleo_files::model` | modelled; **no design file holds a row of any**; nothing user-facing reads or writes them |
| today's design from the drive's releases | `vleo-server/src/today.rs` (`build`) | built in the server; **reported only on the console**; does not check the signature chain yet |
| the health map and trace to cause | `vleo_modules::health` | reached only by `vleo health`; **no route** |
| compare any two files | `vleo_files::compare` | in the library; **no route** |
| the local server | `vleo-server` (`/v1/run`, `/sweep`, `/probe`, `/levers`, `/node`, `/results`, `/figures`, …) | serves today's tool page and its routes, loopback only; finds the design by `VLEO_DESIGN` or `design/` beside it, and today's design by `VLEO_DRIVE` |
| the tool's page | `web/index.html`, `web/js/app.js`, 40 modules | layer, node, run, inputs, results, manual and architecture views; the drawing (`figure.js`, `chart.js`, `figures.js`) is reused |
| the group and node pages | `web/group.html`, `web/node.html`, from `xtask group-app` | **format 1**; a typed name for a signature; reusable pieces: the structure editor, the spreadsheet paste, the drawing |
| browser scenarios in CI | `tools/mock_check.py`, `group_check.py`, `group_db_check.py`, `readers_check.py`, `panel_check.py` | none drives a key, today's design, the health map, comparison or a release through review |

Two consequences shape every stage. **The library's work must be reachable
from the screens:** keys, the chain, today's design, the health map, trace and
comparison all exist but have no route, so F1 and F2 open them. And **the
group and node pages are not ported:** they are format 1 and sign with a typed
name. The Node and System workspaces are written on format 2, reusing their
pieces, and the old pages are removed as the new workspaces supersede them.

## Decisions this rests on

*Declared: your word, 10 October 2026, on the proposals of PR 162.*

1. **The application replaces today's pages; nothing is kept beside it.** Each
   old page is removed in the stage that supersedes it: today's tool page when
   Explore and Node cover it (F2, F3), the group and node pages when System
   covers them (F5), the single-node form and the readers' folder with them.
   The three role guides go in phase G as planned.
2. **One installed program is the engine,** as *How the application works*
   says. The engine, the library and the checks run in it, natively; its
   screens are served to a browser window on the same computer. There is no
   page that runs on its own from the drive, so nothing is compiled for the
   browser for the application.
3. **The database is a folder.** Beside the program by default; any folder the
   person points it at otherwise, chosen in the application and remembered on
   that computer. Read and written directly; no browser file interface.
4. **Files are written by the program through the library,** whole, checked
   before they are written, and never over a file that changed in the folder
   since it was opened.
5. **A test drive for CI, built by one command.** `xtask test-drive` writes an
   empty database folder with a programme of named test people, each with a key
   made from a fixed seed, the example group, and three groups from `design/`.
   Every scenario starts from a fresh copy. The people are invented and say so;
   no real person's name or key appears in CI.
6. **CI drives the installed program.** Each scenario starts the program on its
   copy of the test drive, opens its screens in a browser, and does what the
   person would do. Two people are two program instances on the same folder,
   each with its own key, as two computers on one shared drive would be.
7. **Registering keys comes in F1, not F7.** Nothing can be signed until someone
   is registered, so the People screen (register, replace and revoke a key, at
   the valve that owns it) is built with the shell. The programme manager's is
   the first, on the empty test drive.
8. **No Excel reader in 1.0.** A table comes in by pasting from any spreadsheet,
   and results as CSV. A reader for `.xlsx` files is a later request, vetted
   the way SQLite was (`docs/OPERATING_1_0.md`, section 7).

## The stages

Each stage lists what the program gains, what appears on screen, which
workflows it carries, and the scenarios that prove it.

### F1 · The shell, the database folder and the key

- **The program:** the database folder (beside it, or chosen and remembered);
  keys made, locked by passphrase, opened and used to sign; *who is this key*:
  the person registered with it in the programme's and the groups' files,
  their roles, the groups they own and the nodes they are assigned; files
  written through the library (decision 4).
- **On screen:** the shell with its five workspaces named and four of them
  empty; choosing the database folder; the anchor shown on first use and
  remembered; make a key, open it with its passphrase; People (decision 7); My
  work, with the first items it can know: nodes assigned and unsigned, a group
  with nothing sealed.
- **Workflows:** the start of W1 (the programme registers its people), section 14.
- **Proved by:** the programme manager makes a key and anchors the empty test
  drive; registers the system engineer, who opens the application on the same
  folder and finds their role in My work; a wrong passphrase refused in plain
  words; a key nobody registered opens nothing and says why; the folder beside
  the program and a folder chosen elsewhere both open.
- **Size:** three pull requests.

### F2 · Explore

- **The program:** today's design built on opening, now checking the signature
  chain, and reported on screen, not only on the console; the health map, trace
  to cause and comparison given routes; results kept.
- **On screen:** today's design or any released one, or two side by side with
  the comparison; run and sweep; the health map at every level with trace to
  cause; *Raise issue* from a red mark; every screen saying which design it
  shows and which releases built it (section 5).
- **Workflows:** W6, the start of W11.
- **Proved by:** a refused release replaced by its last good one, and marked;
  a failing closure traced to its node and raised as an issue; two program
  instances on one folder building the same today's design, by fingerprint.
- **Removed:** today's tool page's layer, run, inputs and results views, which
  Explore now holds; the readers' folder.
- **Size:** three pull requests.

### F3 · Node

- **The program:** a node's six parts and whether each is done, missing or
  failing; a revision written in place, refused if the file moved on since it
  was opened; a node signature through the chain; a lesson's place in the
  node's file (`docs/LESSONS.md`).
- **On screen:** the Node workspace (section 4): the parts on the left, the live
  node page in the centre (answer, curve, cases on the curve, the method running
  line by line), the source on the right; signing the day's work; asking for a
  contract change or a breakdown.
- **Workflows:** W3, and the node engineer's half of W4 and W13.
- **Proved by:** a node engineer writes a node and signs it; a second writer
  stopped; the signed revision in today's view of the group for its subsystem
  engineer.
- **Removed:** the rest of today's tool page (its node view); the single-node
  form.
- **Size:** three pull requests.

### F4 · The breakdown

- **The program:** adding, breaking down, moving, renaming, splitting, folding
  back and removing a block, each with its history saying why; an open block
  filled a part at a time; whether children reproduce their parent's cases.
- **On screen:** the branch as a tree in the System workspace, editable by its
  owner; what is still open, on every branch.
- **Workflows:** W13.
- **Proved by:** a node broken down whose children reproduce its cases, and one
  whose children do not, refused by name.
- **Size:** two pull requests.

### F5 · System, at a subsystem's valve

- **The program:** today's view of a branch; impact on every value outside it;
  the seal (the rules `vleo_files::seal` holds, now with keys); answers to moved
  values; the base update.
- **On screen:** the System workspace on a group: frame (wire, assign, issue
  node files, the sheet that sets up many nodes at once), today's view, impact,
  seal, people, compare; the valve above and below.
- **Workflows:** W2, W4, W5, W7 (the subsystem engineer's answer), W11 (assign),
  W12.
- **Proved by:** a sealed release appearing in today's design for a second
  person; a contract change leaving the nodes that read it behind until
  re-signed; an objection answered.
- **Removed:** the group and node pages, and `xtask group-app`.
- **Size:** three pull requests.

### F6 · System, at the system engineer's valve

- **The program:** mounts and allocations; *start group*; the integration
  checks of W8, refusing a release whole with the lines that failed; writing
  the released design, its archive, notes, `readable/` and the status; budgets
  across releases.
- **On screen:** the System workspace on the spacecraft: the frame, *start
  group*, previews and their answers, the decision on an objection, *Release
  design*, budgets, requests for the developer (W14).
- **Workflows:** W1, W7 (the system engineer's decision), W8, W14, W16.
- **Proved by:** a new group mounted on a node, its owner becoming the valve
  above it; a design released.
- **Size:** three pull requests.

### F7 · Programme

- **The program:** a decision signed into a release of the programme's branch
  and taken into today's design; mission health; gates; risks from the
  de-risking record.
- **On screen:** the Programme workspace (section 4): mission health, gates,
  risks, the organisation, decisions; the released design beside today's in
  every workspace, for W9.
- **Workflows:** W9, W10.
- **Proved by:** a design released, reviewed at each level, and a programme
  decision taken back into today's design.
- **Size:** two pull requests.

### F8 · Bringing data in

- **The program:** a formula typed as written, made a method, its inputs found
  and its units asked; units as people write them, through the engine's own
  unit parser; a range or a spread as typed; a pasted table as a lookup or as
  cases; CSV results as cases; *start from a similar node*.
- **On screen:** each way in, in the Node workspace and the System workspace's
  sheet, each with an example beside it and its error in plain words as it is
  typed (section 8, point 3); a PDF dropped beside a node as its source.
- **Workflows:** section 7, every row of its table.
- **Proved by:** each way of bringing data in, one scenario each.
- **Size:** three pull requests.

### F9 · The whole cycle

- **On screen:** what the earlier stages left for last: My work complete for
  every role; status (section 12) in every workspace; the daily snapshot; the
  messages of section 12, written as each hand-off happens.
- **Proved by:** every scenario of *Done when* in one run, from an empty
  database folder to a released design and a programme decision, with five
  people on one shared folder.
- **Size:** two pull requests.

## Where each workflow lands

| workflow | stage | proved by |
|---|---|---|
| W1 · the system sets the frame | F1 (people), F6 | a new group mounted on a node |
| W2 · a subsystem sets its frame | F5 | in the seal scenario |
| W3 · a node is written | F3 | a second writer stopped |
| W4 · a contract changes | F3, F5 | nodes behind until re-signed |
| W5 · a subsystem seals | F5 | a sealed release in today's design for a second person |
| W6 · everyone looks at today's design | F2 | two instances, one fingerprint |
| W7 · values that move are answered | F5, F6 | an objection answered |
| W8 · the design is released | F6 | a design released |
| W9 · each level reviews | F7 | reviewed at each level |
| W10 · the programme decides | F7 | a decision taken back into today's design |
| W11 · an issue | F2, F5 | traced, raised, closed by the release that fixes it |
| W12 · a group's base moves | F5 | in the release scenario |
| W13 · a node is broken down | F3, F4 | children that reproduce the cases, and children that do not |
| W14 · something needs the code | F6 | a request written and listed |
| W15 · the tool is released | not in F | `xtask ship` and phase G |
| W16 · something goes wrong | F2, F6 | the refused release and its last good one; a design released again |

## Done when

What `docs/PLAN_1_0.md` sets for phase F: CI drives every workflow on the
example group with the installed program, with no developer step in W1 to W13.
Each of its nine named scenarios is in the table above, and F9 runs them in one
cycle.

## What is not in phase F

- **No real person is registered.** Real keys are made and registered on the
  switch-over day (phase H); phase F proves it on invented people.
- **The guides are phase G.** The screens carry their own help; the role guides
  are written from them after.
- **Opening the application without installing it.** Not planned (decision 2).
  If someone must, the engine can be compiled for a page later, as the readers'
  folder did; it would be a request (W14).

## Where this breaks

- **Ten days is an estimate from this survey, not a measurement.** The plan
  gave phase F five; the foundations it assumed (keys, today's design and the
  health map reachable from the screens) are not there yet, and F1 and F2 build
  them.
- **The folder dialog is not tested by CI.** CI starts the program on a folder
  it names; the dialog that chooses one is first clicked by a person after
  1.0.0.
- **A shared folder is only as current as its copy on the computer.** On a
  synced Google Drive folder, a file another person saved a minute ago may not
  have arrived yet. The application says which releases it used and when
  (section 5), and refuses to save over a file that changed, but it cannot see
  a file that has not arrived.
- **Everyone must be able to install it.** With no page that runs on its own,
  a computer that cannot run the program cannot open the application.
