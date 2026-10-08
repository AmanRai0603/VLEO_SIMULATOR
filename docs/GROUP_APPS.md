# The group and node applications

> **Answer first.** A group keeps its part of the design in database files on
> its shared drive. The subsystem engineer opens the group's file in **web/group.html**,
> decides the nodes and how they connect, and issues each node engineer a node file.
> Each node engineer fills their file in **web/node.html** and puts it back. The subsystem engineer
> assembles the files into a release, has it signed and seals it. Both pages
> run offline from a double-click; nothing is installed and nothing leaves the
> computer.
>
> **Kind:** how-to · **For:** subsystem engineers, node engineers, and the developer who supports them

## Said simply

A node is one small question with one answer — *how fast does the spacecraft
move?* — and it is the unit everything is built from. Each node has one
node engineer, and one file that only they write. The subsystem engineer owns the
**structure**: which nodes exist, what each one answers, in what unit, and
which node feeds which. That is the **contract** between neighbours, and it is
the only thing the subsystem engineer decides about a node; everything else in the node —
the explanation, the theory, the pseudocode, the node engineer's own results, the
evidence, the pictures, the code — is the node engineer's.

Because each file has one writer, nobody overwrites anybody. The release is
never edited by hand: it is rebuilt from the node files, every time.

## The three files

| File | Written by | Holds |
| --- | --- | --- |
| `<group>.vgroup` — the **structure** | the subsystem engineer, in the group application | every node's contract, the arrows between them, the people, the group's own text and tables |
| `<node>.vnode` — a **node file** | its node engineer, in the node application | every node's contract (so the node engineer sees their neighbours), the group's tables, and this node's content |
| `<group>-<version>.vleo` — a **release** | the subsystem engineer, by assembling | the structure with every node's content put in, every sign-off, and — once sealed — who sealed it and the fingerprint they sealed |

Every one is an ordinary SQLite database in the format of
[`groups/schema.sql`](../groups/schema.sql). The applications, Python's
`sqlite3` and the developer's tools all open it as it is. A file says which of
the three it is inside, as well as by its name.

## On the shared drive

```text
Solar group (shared drive)/
  apps/                       group.html and node.html, from the kit
  solar.vgroup                the structure — the subsystem engineer's
  nodes/
    sw_f107_design.vnode      one per node — each its node engineer's
    sw_ap_design.vnode
    …
  releases/
    solar-1.0.vleo            sealed, never changed
    solar-1.1.vleo
```

Google Drive for desktop makes the shared drive an ordinary folder on each
computer; open the pages and the files from there. A file opened in a browser
that cannot write back (anything but Chrome or Edge) is saved by downloading:
put the downloaded file back where it came from, replacing the old one.

## Every group, set up for its subsystem engineer

Every group that owns a node in the design starts from what the design already
holds. Two commands make all of them at once:

    cargo run -p xtask -- group-export --all --out ~/groups
    node tools/group_db.mjs --all ~/groups --out ~/groups-db

| What | Where |
| --- | --- |
| A folder of the pattern for each group, written from its sheets | `~/groups/<group>/` |
| `GROUPS.csv` — each group, its layer, its owner team, how many nodes, how many compute, how many already have a method | `~/groups/` |
| Each group's structure, one node file per node, and an unsealed release | `~/groups-db/<group>/` |
| `READY.csv` — what each group's own checks still ask of it: empty sections, pseudocode to write, results to supply, declared values to decide, defaults to give | `~/groups-db/` |

Copy `~/groups-db/<group>/` onto that group's shared drive, beside `apps/`, as
laid out above. The `drive` workflow does all of this for every group at once —
`tools/drive.py pack`, then a mirror into one Drive folder, on every release tag
— or, without the sign-in, `pack --zip` and the folder dragged into Drive by
hand (docs/DRIVE_SETUP.md). The subsystem engineer opens the structure in the group application and
starts from there; the node engineers open their node files. Nothing is invented for
them: what the design lacks stays empty, and `READY.csv` counts it, so it is
also the first plan of each group's work.

## The subsystem engineer, in the group application

1. **Start the group** — open `web/group.html`, *Start a new group*: an id
   (`solar`), a name, and your own name as its owner. Save the file as
   `solar.vgroup` on the drive.
2. **Set out the structure** — *Structure*. Add each node with its id and kind
   (computed, declared, required, achieved); give it the question it answers,
   its output symbol and unit, its bounds; give each computed node its inputs
   and say where each comes from — another node of the group, `case` for a
   value set in the case, or `group.node` for another group's output. The map
   draws itself as you go. Add the people and give each node its node engineer.
   *Write the flow from the inputs* fills the group's flow.
3. **Issue the node files** — *Node files & release*. *Issue* one, or every
   node file into a folder at once (a zip where the browser cannot write a
   folder). Put each in `nodes/` and tell its node engineer. Save the structure: it
   records what was issued, at which contract version.
4. **Change a contract later** — edit it in *Structure*. The page names every
   node that reads the one you changed, and marks the changed node's file as
   *behind*: re-issue it, and tell those node engineers.
5. **Assemble the release** — *Node files & release*, *Choose their folder…*
   (or the files). Each node's content is taken from its file; a file written
   to an older contract, a file for a node that is not in the structure, and a
   node without a file are each named. The result is the release, not yet
   sealed: save it as `releases/solar-1.0.vleo`.
6. **Sign and seal** — *Sign & seal*. Each node is signed by its node engineer (in
   the node application, or here), and the whole group by its owner. A
   signature is for the content as it is: change it and the signature goes
   stale. With no errors and every signature current, *Seal* writes the sealed
   release. Save it in the group's `releases/` on the drive; nobody takes it in
   by hand (*A sealed release, into today's design*, below).
7. **The next version** — open the sealed release and issue node files from
   it: each node engineer starts from what was sealed. A sealed release is never
   edited; *start the next version from this one* makes an unsealed copy.

A **folder** of the pattern ([`docs/GROUP_FOLDER.md`](GROUP_FOLDER.md)) — the
worked example, or one written from the design by `xtask group-export` — opens
too, and *Keep as a database* turns it into a release file in one step.

## The solar group, worked through

[`groups/solar/`](../groups/solar/) is the solar group's whole design as a
group would hold it: 58 nodes, every computed node's pseudocode transcribed from
the code that runs it and checked by the method checker against all of its
published fixtures, its results, evidence and sources, and pictures drawn from
the 1997–2025 record. Its `versions.csv` says what is drafted and what is still
open. To have it as the files a group keeps on its drive:

```text
node tools/group_db.mjs groups/solar          → target/groups/l3_solar-db/
  l3_solar.vgroup                                 the structure
  nodes/<node>.vnode                              58 node files
  releases/l3_solar-1.1.vleo                      assembled, not sealed
```

How it was made, which is how any group's design can be brought in: `xtask
group-export l3_solar` wrote the folder from the design — questions, words,
theory, inputs, and each node's fixtures as its isolation results and evidence.
Each computed node's pseudocode was then transcribed from the code that runs it
and run by the method checker (web/method.wasm.gz) against every one of its
fixtures, all of which agree. The pictures are the solar-weather record's own
figures (`vleo figure …`). The few sections the design never had — mostly
*Picture it* and *Guess first* — were drafted from each node's own text and are
marked in `versions.csv` for the node engineers to confirm or replace.

Its owner, Aman Rai, signed each computed method as transcribed from its node's
own `model.rs`, judged by its results, and declared the two values the design
had left for a person: `sw_band_confidence` 0.90, and `sw_kp_driving_slot` the
day's mean. Open the release in the group application and *Sign & seal* names
nothing left in the checks, only the sign-offs. Solar 1.0 was sealed and taken
in, and intake refused it whole for two things the application did not then
ask — a refusal on six nodes, and every value of a node that publishes a set —
so 1.1 answers both, and that one was built from its methods and held its own
results: `versions.csv` says what each version learned.

## The node engineer, in the node application

Open `web/node.html` and your node file. The page walks you through, step by
step, and every step shows what it will look like as you type:

| Step | What you give |
| --- | --- |
| Your node | read the contract — what it answers, what feeds it, who reads it; ask the subsystem engineer for a change here, never work around it |
| Explanation | six short sections: in one line, said simply, picture it, guess first, where it breaks, the common misreading |
| Theory & maths | the equations, the derivation, the assumptions, where it holds; each equation once, in LaTeX, with the equation helper beside it |
| Pseudocode | the algorithm, line by line — required for every computed node; typeset as equations and read by the method checker as you type — every line, every unit — never run |
| Inputs | how each input is written, its default and the range the node holds for |
| Results | your own answers, from your own code, a hand calculation, a spreadsheet or a paper — paste them from your spreadsheet |
| Evidence & sources | values from outside any code, each citing a source; add your own sources here |
| Pictures | charts, flows, step-by-step walk-throughs, heatmaps, 3D views from CSV data; images and short videos |
| Code & files | your own code, kept for the record, and a source PDF you may keep |
| Preview | the node exactly as the group and the application will show it |
| Check & sign | what is still wrong, by file and line; then your signature |

**Save** often; each save is a new revision. Then put the file back in
`nodes/` on the drive.

## A sealed release, into today's design

A sealed release is the group's signed word, and nobody takes it in by hand.
It goes in the group's `releases/` folder on the shared drive. When the tool
opens on the drive (`VLEO_DRIVE`), it builds today's design from every group's
latest sealed release that passes its checks, in memory, and writes nothing on
the drive. A group whose latest release is refused is built from its last good
release, and the tool says which release each group's part is and why one was
refused. The library that does the checking is `vleo-files`
(`vleo_files::intake`, `vleo_files::checks`, `vleo_files::seal`).

| Check | What happens |
| --- | --- |
| It opens | the file is read as a sealed group release, and its group is the one whose folder it is in |
| Seal | every file's SHA-256 is recomputed; a release whose files no longer give the sealed fingerprint is refused before anything is read |
| Content | every check of a release's content, each error named: a method or results an assistant supplied (`declaration.csv` says `relation`, or says nothing, or says `transcribed` without the `source` it was copied from and the person who checked the copy, `checked_by`, or with an assistant there); an input that connects to nothing; a signature under an assistant's name |
| Each node against the design | each computed node is planned field by field against the design: the pseudocode as its method; its results, brought back to SI, as its test cases; its derivation (`theory.md`) as its theory, without which a row never answers; its node engineer and the declaration of any assistant's help; and the newest row of `versions.csv` as the de-risking record. A conflict with a change the design made since, an incomplete de-risking record (`rests_on` and `breaks_if` included), or a case not in SI refuses the release |

To read a sealed release as the folder it was sealed from, with `RELEASE.toml`
— the group, the version, who sealed it, the fingerprint, and each node's
revision and author:

    node tools/group_db.mjs --unpack example_orbit-1.0.vleo --out <dir>

`xtask impact <node|group>` lists every row of another group downstream of a
node — the rows that read it, theirs, and so on, by group and nearest first;
`xtask catalogue` lists what each group publishes and who reads it, and the
design file carries that catalogue as its `published` table.

A change the group asks for goes back into the group folder and a new sealed
release, saved beside the old one; the developer never edits the release, and
never moves a group's results to make a test pass.

What the release holds beyond its methods, derivations, cases and code — its
plain words, pictures and evidence — stays in the release, which is what the
group signed.

**Today's design is tested on the solar group's own releases**
(`crates/vleo-server/tests/today_s_design_is_built_from_the_releases.rs`): the
two it sealed in the group application are kept as fixtures, 1.1 is taken, a
refused release is replaced by its group's last good one and marked by its
reason, and a release whose relation an assistant supplied is refused.

## What the applications do not do

- **They run nothing.** The pseudocode is read and typeset, never executed;
  the results are the node engineer's. The developer's engine is generated from the
  pseudocode after the seal and tested against these results.
- **They do not merge two people's edits to one file.** Each file has one
  writer by design. If two copies of one node file come back, the subsystem engineer keeps
  one and the assembly names the other.
- **The pseudocode is read, never run.** Both pages carry the repository's
  own method checker (`web/method.wasm.gz`), the one the developer's tools use:
  every line must parse, every unit must agree, and the answer must be in the
  unit the contract names. No case is handed to it, so nothing is computed.

## Where it breaks

A browser that cannot open a file for writing (Firefox, Safari) saves by
downloading, and a downloaded file left in *Downloads* is a change nobody
assembled. The page says which way it saves, every time.

## Common misreading

*"The release is the file we edit."* It is not. The release is rebuilt from
the node files; the node files are where content lives, and the structure is
where contracts live. Edit those, and assemble again.
