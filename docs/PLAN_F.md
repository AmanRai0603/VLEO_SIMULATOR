# Phase F, stage by stage

> **Answer first.** Phase F builds the one application in nine stages, in the order the plan fixes: the shell and the key, Explore, Node, the breakdown, System at a subsystem's valve, System at the system engineer's, Programme, the ways of bringing data in, and the whole cycle twice over. Each stage is a few pull requests into `developer`, each proved by a browser scenario in CI that runs the same page installed and opened from the drive. It starts lower than the plan assumed: no front end can make or use a key today, no design file registers a person, and today's design, the health map and comparison have no route a page can call. Eight decisions below need your word before the first line of F1. About twelve working days of building, not the five the plan gave.
>
> **Kind:** explanation + reference · **For:** the developer, and you, who approve each stage

What phase F must deliver is `docs/PLAN_1_0.md`, phase F, and what the
application is, screen by screen and step by step, is `docs/OPERATING_1_0.md`,
sections 4 to 8 and 11. This page is only the order it is built in, what each
stage stands on, and how each is proved.

## Said simply

The application is built the way a person will meet it. First it must know who
they are and open the drive (F1). Then everyone can look at today's design
(F2). Then a node engineer can write a node (F3) and grow the breakdown (F4). Then a
subsystem engineer can frame, seal and answer (F5), the system engineer can set
the frame and release the design (F6), and the programme manager can decide
(F7). Then the easier ways of bringing data in (F8), and last the whole cycle,
every workflow, run end to end with several people (F9).

Every stage leaves a working application that does a little more, and every
stage is proved by a browser in CI doing what a person would do, with more than
one person where the workflow has more than one. Nobody tries it before 1.0.0
(`docs/PLAN_1_0.md`, *While it is built*), so those scenarios are the only trial
it gets.

## What there is to build from

*Derived, from a survey of the code at `developer`, 10 October 2026.*

| what | where | state for phase F |
|---|---|---|
| keys: make, lock by passphrase, sign, check | `vleo_files::keys` | in the library; **not exported to any page** |
| the signature chain | `vleo_files::chain` (`open_programme`, `open_group`, `check_node`, `check_release`, `check_design`) | in the library, used only by its tests |
| people, keys, assignments, requests, issues, comments, changes | format-2 schema, `vleo_files::model` | modelled; **no design file holds a row of any**; nothing user-facing reads or writes them |
| today's design from the drive's releases | `vleo-server/src/today.rs` (`build`) | built in the server; **reported only on the console**; does not check the signature chain yet |
| the health map and trace to cause | `vleo_modules::health` | reached only by `vleo health`; **no route, no wasm export** |
| compare any two files | `vleo_files::compare` | in the library and in `files.wasm`; **no page loads it** |
| the engine in the page | `vleo-kernel-wasm` (`vleo_open`, `vleo_run`, `vleo_sweep`) | used by the readers' folder only; no health, trace, probe or levers |
| the page library | `vleo-files-wasm` (`check_release`, `check_content`, `check_folder`, `seal_state`, `compare`) | built and checked; **inlined in no page**; no key, sign or write export |
| the method checker in the page | `vleo-method-wasm` | used by the group and node pages |
| the tool's page | `web/index.html`, `web/js/app.js`, 40 modules | talks only to the local server's routes; layer, node, run, inputs, results, manual and architecture views |
| the group and node pages | `web/group.html`, `web/node.html`, from `xtask group-app` | **format 1**; a typed name for a signature; folder picker, saving in place, SQLite in the page, pasting from a spreadsheet |
| browser scenarios in CI | `tools/mock_check.py`, `group_check.py`, `group_db_check.py`, `readers_check.py`, `panel_check.py` | none drives a key, today's design, the health map, comparison or a release through review |
| the installed application | `vleo-daemon` (the local server), `python -m vleo`, `xtask kit` | serves the tool's page and its routes; the kit carries no page that runs the engine in itself |

Two consequences shape every stage. **The page's engine and library must grow
first:** health, trace, today's design and every check have to be callable from
the page, because both ways of opening the application run the engine in the
page (*Decisions*, 2). And **the group and node pages are not ported:** they are
format 1 and sign with a typed name. The Node and System workspaces are written
on format 2 beside them, reusing their pieces (the folder picker, saving in
place, the spreadsheet paste, the drawing), and the old pages retire on the
switch-over day.

## Decisions this rests on

Each is the developer's proposal, waiting for your word. The first two follow
`docs/PLAN_1_0.md`; the other six are new.

1. **A new page, beside the old.** The application is `web/app/`, ES modules
   with no framework like the rest of `web/`, bundled into one file for the
   drive by the bundler the readers' folder already uses. Today's tool page,
   the group and node pages and the single-node form stay as they are until the
   switch-over.
2. **One engine, in the page, in both ways of opening.** The kernel
   (`kernel.wasm`), the library (`files.wasm`) and the method checker
   (`method.wasm`) run in the page. Installed, `vleo-daemon` serves that same
   page from the computer, so the screens cannot differ. The server's routes
   stay for Python and the command line, and the contract and mock engine
   follow them.
3. **How the page reaches the drive.** Opened from the drive, through the
   browser's File System Access API (Chrome and Edge), which is what saving in
   place needs (`docs/OPERATING_1_0.md`, section 4). Installed, through a small
   set of file routes the local server adds over the drive folder it was started
   on, so any browser works. One drive interface in the page, with those two
   behind it.
4. **The page writes a format-2 file with the vendored SQLite, and the library
   checks it before it is written.** The library's row encoding
   (`vleo_files::rows`) crosses between them, as it already does for the
   readers' folder. No second schema is written in JavaScript.
5. **A test drive for CI, built by one command.** `xtask test-drive` writes an
   empty drive with a programme of named test people, each with a key made from
   a fixed seed, the example group, and three groups from `design/`. Every
   scenario starts from a fresh copy. The people are invented and say so; no
   real person's name or key appears in CI.
6. **Opened from the drive, CI hands the page its folder from the browser's
   private storage.** A browser in CI cannot click the system's folder picker.
   The scenario copies the test drive into the browser's private file system,
   whose folder handle is the same interface the picker returns, and passes it
   to the page through a hook that exists only in the test build. That hook is
   the one difference from a person's run, and it is stated in every report.
7. **Registering keys comes in F1, not F7.** Nothing can be signed until
   someone is registered, so the People screen (register, replace and revoke a
   key, at the valve that owns it) is built with the shell. The programme
   manager's is the first, on the empty test drive.
8. **No Excel reader in 1.0.** A table comes in by pasting from any spreadsheet,
   and results as CSV. A reader for `.xlsx` files is a later request, vetted
   the way SQLite was (`docs/OPERATING_1_0.md`, section 7).

## The stages

Each stage lists what the library and the page's engine gain, what appears on
screen, which workflows it carries, and the scenarios that prove it. A scenario
runs twice, installed and opened from the drive, from F1 on.

### F1 · The shell, the drive and the key

- **Library and engine:** keys exported to the page (make, lock, unlock, sign);
  *who is this key*: the person registered with it in the programme's and the
  groups' files, their roles, the groups they own and the nodes they are
  assigned; writing a format-2 file (decision 4); the drive interface (3).
- **On screen:** the shell with its five workspaces named and four of them
  empty; opening the drive; the anchor shown on first use and remembered; make a
  key, open it with its passphrase; People (decision 7); My work, with the first
  items it can know: nodes assigned and unsigned, a group with nothing sealed.
- **Workflows:** the start of W1 (the programme registers its people), section 14.
- **Proved by:** the programme manager makes a key and anchors the empty test
  drive; registers the system engineer, who opens the application and finds
  their role in My work; a wrong passphrase refused in plain words; a key
  nobody registered opens nothing and says why; the server and the page agree on
  every signature made.
- **Size:** three pull requests.

### F2 · Explore

- **Library and engine:** today's design moved from the server into the
  library, so the page builds it too, now checking the signature chain;
  the health map, trace to cause, probe and levers exported from the kernel;
  results kept from the page.
- **On screen:** today's design or any released one, or two side by side with
  the comparison; run and sweep; the health map at every level with trace to
  cause; *Raise issue* from a red mark; every screen saying which design it
  shows and which releases built it (section 5).
- **Workflows:** W6, the start of W11.
- **Proved by:** a refused release replaced by its last good one, and marked;
  a failing closure traced to its node and raised as an issue; two computers
  building the same today's design, by fingerprint.
- **Also:** the readers' folder becomes Explore's read-only export.
- **Size:** three pull requests.

### F3 · Node

- **Library and engine:** a node's six parts and whether each is done, missing
  or failing; a revision written in place, refused if the file moved on since
  it was opened; a node signature through the chain.
- **On screen:** the Node workspace (section 4): the parts on the left, the live
  node page in the centre (answer, curve, cases on the curve, the method running
  line by line), the source on the right; signing the day's work; asking for a
  contract change or a breakdown.
- **Workflows:** W3, and the node engineer's half of W4 and W13.
- **Proved by:** a node engineer writes a node and signs it; a second writer
  stopped; the signed revision in today's view of the group for its subsystem
  engineer.
- **Size:** three pull requests.

### F4 · The breakdown

- **Library and engine:** adding, breaking down, moving, renaming, splitting,
  folding back and removing a block, each with its history saying why; an open
  block filled a part at a time; whether children reproduce their parent's
  cases.
- **On screen:** the branch as a tree in the System workspace, editable by its
  owner; what is still open, on every branch.
- **Workflows:** W13.
- **Proved by:** a node broken down whose children reproduce its cases, and one
  whose children do not, refused by name.
- **Size:** two pull requests.

### F5 · System, at a subsystem's valve

- **Library and engine:** today's view of a branch; impact on every value
  outside it; the seal (the rules `vleo_files::seal` holds, now with keys);
  answers to moved values; the base update.
- **On screen:** the System workspace on a group: frame (wire, assign, issue
  node files, the sheet that sets up many nodes at once), today's view, impact,
  seal, people, compare; the valve above and below.
- **Workflows:** W2, W4, W5, W7 (the subsystem engineer's answer), W11 (assign),
  W12.
- **Proved by:** a sealed release appearing in today's design for a second
  person; a contract change leaving the nodes that read it behind until
  re-signed; an objection answered.
- **Size:** three pull requests.

### F6 · System, at the system engineer's valve

- **Library and engine:** mounts and allocations; *start group*; the
  integration checks of W8, refusing a release whole with the lines that
  failed; writing the released design, its archive, notes, `readable/` and the
  status; budgets across releases.
- **On screen:** the System workspace on the spacecraft: the frame, *start
  group*, previews and their answers, the decision on an objection, *Release
  design*, budgets, requests for the developer (W14).
- **Workflows:** W1, W7 (the system engineer's decision), W8, W14, W16.
- **Proved by:** a new group mounted on a node, its owner becoming the valve
  above it; a design released.
- **Size:** three pull requests.

### F7 · Programme

- **Library and engine:** a decision signed into a release of the programme's
  branch and taken into today's design; mission health; gates; risks from the
  de-risking record.
- **On screen:** the Programme workspace (section 4): mission health, gates,
  risks, the organisation, decisions; the released design beside today's in
  every workspace, for W9.
- **Workflows:** W9, W10.
- **Proved by:** a design released, reviewed at each level, and a programme
  decision taken back into today's design.
- **Size:** two pull requests.

### F8 · Bringing data in

- **Library and engine:** a formula typed as written, made a method, its
  inputs found and its units asked; units as people write them, through the
  engine's own unit parser; a range or a spread as typed; a pasted table as a
  lookup or as cases; CSV results as cases; *start from a similar node*.
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
- **Proved by:** every scenario of *Done when* in one run, from an empty drive
  to a released design and a programme decision, with five people, installed
  and opened from the drive.
- **Size:** two pull requests.

## Where each workflow lands

| workflow | stage | proved by |
|---|---|---|
| W1 · the system sets the frame | F1 (people), F6 | a new group mounted on a node |
| W2 · a subsystem sets its frame | F5 | in the seal scenario |
| W3 · a node is written | F3 | a second writer stopped |
| W4 · a contract changes | F3, F5 | nodes behind until re-signed |
| W5 · a subsystem seals | F5 | a sealed release in today's design for a second person |
| W6 · everyone looks at today's design | F2 | two computers, one fingerprint |
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

What `docs/PLAN_1_0.md` sets for phase F, unchanged: CI drives every workflow
on the example group, installed and opened from the drive, with no developer
step in W1 to W13. Each of its nine named scenarios is in the table above, and
F9 runs them in one cycle.

## What is not in phase F

- **The old pages are not changed.** They stay in use until the switch-over
  and retire then (phase H).
- **No real person is registered.** Real keys are made and registered on the
  switch-over day (phase H); phase F proves it on invented people.
- **The guides are phase G.** The screens carry their own help; the role guides
  are written from them after.
- **A lesson's place in a node's file** is added in F3 with the node's other
  parts, since the design's files have none yet (`docs/LESSONS.md`).

## Where this breaks

- **Twelve days is an estimate from this survey, not a measurement.** The plan
  gave phase F five; the foundations it assumed (keys in the page, today's
  design and the health map reachable from it) are not there yet, and F1 and F2
  build them.
- **The folder picker is not tested.** CI hands the page its folder another way
  (decision 6), so the system's own dialog is first clicked by a person after
  1.0.0.
- **The File System Access API is Chrome and Edge only.** Opened from the drive
  in another browser, the application saves by download and says so
  (section 8, point 8); installed, any browser works.
- **A browser is slower than the installed engine on large runs.** Both run the
  engine in the page (decision 2). If a sweep the design needs proves too slow
  in a page, the installed application may run it through its server instead;
  that would be measured in F2, not assumed.
