# The plan to 1.0.0

> **Answer first.** One release, 1.0.0, after eight phases. The code becomes the engine and the apps; the shared drive becomes the design, end to end. The team works in three apps, and every change to the design travels one way: a group's sealed release.
>
> **Kind:** explanation + reference · **For:** everyone

The model this plan builds is `docs/SYSTEM_MODEL.md`. The rules every phase
works to are `AGENTS.md`, until phase B rewrites them.

## Why 1.0.0, and why one release

0.5.0 was never shipped, and what changes is larger than a minor version: the
file formats, the rules, the apps and where the design lives. So the next
release is **1.0.0**, and it ships once, when every phase below is done. Each
phase merges into `developer` when its checks are green; nothing reaches the
team until 1.0.0.

After 1.0.0 there are two kinds of release:

| release | what changes | how often |
|---|---|---|
| **tool release**, such as 1.1.0 | the code: engine, toolbox, apps | rarely |
| **design release**, named by date | the data: a new `design.vleo` with a group's accepted work | whenever a group is accepted |

A design release needs no new tool, as long as it asks for nothing the tool
lacks; the tool checks that before it runs anything.

## Who holds what

| role | holds | works in |
|---|---|---|
| **programme lead** | the programme's branch: customers, KPIs, gates, risks; approves rules; says when to ship | group app, on the programme's branch |
| **system engineer** | the systems branch: where each group mounts, what is allocated to it, the system closures | group app, on the systems branch |
| **group lead** | one group's branch: breaks it down, assigns authors, seals releases, accepts deliveries | group app |
| **author** | one node at a time | node app |
| **team** | cases and results | the tool |
| **developer** | the code; takes sealed releases in, tests and delivers them, compiles and signs the design, releases the tool | the repository, `xtask`, CI |
| **reviewer** | the second person on a rule, a change to the tree, or a significant relation | the pull request |

One person may hold several roles. Today Aman Rai is programme lead and Solar's
lead.

## Where everything sits

**The repository is the code.** The kernel and its toolbox, the method
interpreter, the engine, the one library that reads and writes every file, the
apps' source, the checks, `xtask`, the rules and these documents. It also keeps
a readable copy of every sealed release, written by `xtask` and never by hand,
so a change to the design can be reviewed line by line and its history kept.

**The shared drive, "Vleo Database", is the design.**

| folder | holds | written by |
|---|---|---|
| START HERE | what the folder is and the loop | the developer |
| `apps/` | node app, group app, the tool | the developer |
| `guides/` | the role guides | the developer |
| `design/` | the current `design.vleo`, signed, and the earlier ones | the developer |
| `readable/` | the design as CSV: groups, nodes, ports, wires, closures | the developer |
| `groups/<group>/` | the group's file, `nodes/` one file per node, `releases/` its sealed releases only | the group |
| `groups/programme/`, `groups/systems/` | the top two branches, held like any group | programme lead, system engineer |
| `deliveries/<group>-<version>/` | a test application's record, and the lead's answer beside it | the developer, then the lead |
| `cases/` | cases the team shares | the team |

**A person's machine** holds the tool, and their own cases and results under
`~/.vleo/`. A file opened from a folder Google Drive for desktop syncs is saved
straight back into it by the node and group apps, in Chrome and Edge.

## The apps the team works in

The team works only in these. The browser cannot change the design except
through them, and they change it only through files that are signed and sealed.

| app | who | opens | does | saves |
|---|---|---|---|---|
| **node app** | author | one node file | the live node page in edit mode: question, ports with type, unit, range, state and maturity; the behaviour, typed from the source as read, or a table pasted in; cases from the source's worked examples; explanation; sign with the author's own key. It runs while it is written: answer, curve, cases on the curve, the method line by line | `.vnode` |
| **group app** | group lead, programme lead, system engineer | the group's file | the tree: break down, assign authors, issue node files. The N2 and the map, opened to any depth. The mount: what is asked of the group and what it is given. The status of every node. The group run live, with other groups' values held. Range verdicts and tornadoes for its closures. Assemble and seal. Delivery and acceptance | `.vgroup`, sealed `.vleo`, `.accept.toml` |
| **the tool** | the team | `design.vleo` and a case | the design map and N2 at any depth; run a case, sweep, compare; every closure with its margin, range verdict and tornado. Results name the design and engine that made them. It changes nothing in the design | `.vcase`, `.vleor` |
| **readers' pages** | anyone | the released design | read and play with every node; change nothing | nothing |

**Retired at 1.0.0:** the single-node form and its loop (`xtask form`,
`intake`, `take`, `approve`, `preview`, `queue`). Every change arrives as a
sealed group release, so there is one way in, not two. A change to one node is
a release with one node changed.

## How one change travels

1. **Anyone sees a need.** It goes to the node's owner, as a comment in the file
   or to the group's lead.
2. **The author** opens the node in the node app, changes it, watches it run and
   prove itself on its cases, signs it, and saves it in the group's folder.
3. **The group lead** sees it in the group app, runs the group with it, seals a
   release into `releases/`, and tells the developer.
4. **The developer** takes the release in on its own branch: the seal checked
   first, then every node, then the group built and tested against its own
   results, then the whole design run to list what the change reaches in other
   groups. The test application goes into `deliveries/`.
5. **The lead** tries it and answers Accepted or Changes, saved beside the
   delivery.
6. **The developer** records the answer, merges, compiles and signs the new
   `design.vleo`, and publishes it as a design release.
7. **The team** is offered the new design the next time the tool opens. Results
   made with the earlier design say so.

A failure at step 4 goes back to the group with the lines that failed. The
group's results and tolerances are never the thing changed to make it pass.

## The phases

Each phase ends in a pull request into `developer`, green in CI, merged on your
word. The order follows what each needs: A and B first; C before D and F; D
before E; G after E and F; H last.

### A · Safe ground

- The tool checks `design.vleo` against its engine and toolbox, and refuses a
  mismatch by name.
- Every result carries the fingerprint of the design and the engine that made
  it, and is marked stale when either changes.
- The method checker the forms carry is rebuilt whenever the kernel it runs
  changes: `crates/vleo-sheet/src/method/cases.rs` lists the kernel's own files
  among its sources.
- `docs/SOLAR_INVENTORY.md` and `docs/SOLAR_ROWS.md` are either checked against
  the tree or removed, and the repository's copy of the Solar group folder is
  either checked or removed.
- `docs/DRIVE_START_HERE.md` says how to save in place.

**Done when** a design built for another engine is refused in a test, and gate
and tests are green.

### B · The model and the rules

- `docs/SYSTEM_MODEL.md` and this plan.
- `AGENTS.md`, `CONTRIBUTING.md`, the area files and `docs/WORK_MODEL.md`
  rewritten: one recursive block; a block reads its children only through
  their ports; sealed group releases are the source, and the repository keeps
  their readable copy; one way in. The five rules that do not bend stay, with
  rule 1 restated.

**Done when** you approve the rules, with a second reviewer.

### C · One schema, one library

- One schema for every file: block, with its parent to any depth; port, with
  type, unit, range and its reason, state, maturity and direction; wire;
  mount; closure; loop; case; text; table; media; sign-off; change. A file's
  kind is only the size of its branch: one node, one group, the whole design.
- One library reads and writes it, in the tool and compiled for the browser,
  so the apps stop carrying their own copy of the rules.
- A file in the format before it upgrades when opened and keeps a copy. Solar
  1.1 converts without anyone in Solar doing anything.
- Intake keeps everything a group wrote. Nothing a group declares is dropped on
  the way in.
- The readable copy: every sealed release written as text into the repository,
  with a test that text and database round-trip identically.

**Done when** Solar 1.1 converts, round-trips identically, and opens in the
tool, in Python and in both apps.

### D · The engine runs the database

- The engine builds its graph from `design.vleo` when it starts, not from
  tables compiled into it.
- Behaviours: method, run by the interpreter; children; stated; lookup; open,
  refused by name; and **built-in**, a relation that is still compiled code,
  run by its id until its group migrates it to a method.
- Loops declared on the block that holds them. Ports typed: number, choice,
  list and parameter first.
- State, maturity and range carried on every value; every closure gives its
  range verdict and its tornado.
- The design names the engine version and the toolbox functions it needs.

**The parity gate.** For every row in every case the new engine gives the same
answer as the 0.4 engine, within each row's own tolerance, and refuses where
it refused. There is no switch without it.

**Done when** parity holds for the whole design on every case shipped.

### E · The tree, converted and mounted

- The layer files and all 1,396 rows become the programme branch, the systems
  branch and the 18 group branches, in the one schema. Each row's layer becomes
  its perspective tag.
- Each group mounts on its system block (`docs/SYSTEM_MODEL.md`, section 8).
- The proposed next level is added as open blocks, owned by each group's lead.
- Each group's starting folder is written from the tree for the drive.

**Done when** the design built from these files passes the parity gate, and its
N2 shows the one declared loop.

### F · The apps

- One node page shared by the node app, the group app, the tool and the readers'
  pages.
- The node app, the group app and the tool, as described above.
- Signing with a person's own key: Ed25519, built into every current browser.
- The retired form loop and its documents removed.

**Done when** CI drives each app end to end in a browser on the example group.

### G · The developer's loop and the drive

- `xtask` takes, builds, tests, delivers and accepts schema-2 releases;
  compiles and signs the design; and makes design releases apart from tool
  releases.
- `tools/drive.py` packs the drive's new layout.
- The manual, the role guides, START HERE and `docs/PIPELINE.md` regenerated.

**Done when** CI runs the whole loop on the example group: sealed release in,
accepted delivery, signed design out.

### H · Proof with real people, then 1.0.0

- **Solar 1.2**, made by Solar in the new apps: `sw_band_confidence` becomes the
  one parameter its four methods read, in place of the literal 1.28, and
  `sw_kp_driving_slot` is declared. Sealed, taken in, delivered, accepted.
- The programme and systems branches sealed by their owners.
- Gate and tests green. You say **"Ship 1.0.0"**: the tool and the first design
  release are tagged and published, and the drive is updated.

**Done when** Solar 1.2 is accepted on the release candidate.

## After 1.0.0

- Each group moves its built-in relations to methods, transcribed from the code
  and signed by the group's own people. 158 of the 190 computing nodes start
  there.
- Groups break their branches down further.
- Variance-based sensitivity and the probability each closure holds; an
  optimiser over the open ranges; each closure's verification method; nodes
  that keep state from one time step to the next; SysML v2 and FMI exchange.

## What needs your word

- The rules in phase B: the block as the source, one way in, the recursive
  block. Rule changes take your approval and a second reviewer.
- Each phase's merge.
- "Ship 1.0.0".

The developer does not merge, release, change a rule or migrate a group's
relations on its own initiative, and no assistant supplies a relation.

## Where this plan breaks

- **The parity gate is the hard part.** If the engine cannot reproduce today's
  answers exactly, phase D stops until it does, and everything after it waits.
- **The apps are the largest phase.** F can start once C is done, alongside D
  and E.
- **People learning new apps.** Phase H is the test of that, on one group, before
  anyone else depends on it.
- **Built-in relations are a debt.** On 1.0.0 most nodes still compute in code.
  They are marked, listed by owner, and migrated after the release, not hidden.
