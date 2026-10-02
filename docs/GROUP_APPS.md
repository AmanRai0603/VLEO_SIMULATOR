# The group and node applications

> **Answer first.** A group keeps its part of the design in database files on
> its shared drive. The lead opens the group's file in **web/group.html**,
> decides the nodes and how they connect, and issues each author a node file.
> Each author fills their file in **web/node.html** and puts it back. The lead
> assembles the files into a release, has it signed and seals it. Both pages
> run offline from a double-click; nothing is installed and nothing leaves the
> computer.
>
> **Kind:** how-to · **For:** group leads, node authors, and the developer who supports them

## Said simply

A node is one small question with one answer — *how fast does the spacecraft
move?* — and it is the unit everything is built from. Each node has one
author, and one file that only that author writes. The group lead owns the
**structure**: which nodes exist, what each one answers, in what unit, and
which node feeds which. That is the **contract** between neighbours, and it is
the only thing the lead decides about a node; everything else in the node —
the explanation, the theory, the pseudocode, the author's own results, the
evidence, the pictures, the code — is the author's.

Because each file has one writer, nobody overwrites anybody. The release is
never edited by hand: it is rebuilt from the node files, every time.

## The three files

| File | Written by | Holds |
| --- | --- | --- |
| `<group>.vgroup` — the **structure** | the lead, in the group application | every node's contract, the arrows between them, the people, the group's own text and tables |
| `<node>.vnode` — a **node file** | its author, in the node application | every node's contract (so the author sees their neighbours), the group's tables, and this node's content |
| `<group>-<version>.vleo` — a **release** | the lead, by assembling | the structure with every node's content put in, every sign-off, and — once sealed — who sealed it and the fingerprint they sealed |

Every one is an ordinary SQLite database in the format of
[`groups/schema.sql`](../groups/schema.sql). The applications, Python's
`sqlite3` and the developer's tools all open it as it is. A file says which of
the three it is inside, as well as by its name.

## On the shared drive

```text
Solar group (shared drive)/
  apps/                       group.html and node.html, from the kit
  solar.vgroup                the structure — the lead's
  nodes/
    sw_f107_design.vnode      one per node — each its author's
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

## The lead, in the group application

1. **Start the group** — open `web/group.html`, *Start a new group*: an id
   (`solar`), a name, and your own name as its owner. Save the file as
   `solar.vgroup` on the drive.
2. **Set out the structure** — *Structure*. Add each node with its id and kind
   (computed, declared, required, achieved); give it the question it answers,
   its output symbol and unit, its bounds; give each computed node its inputs
   and say where each comes from — another node of the group, `case` for a
   value the user sets, or `group.node` for another group's output. The map
   draws itself as you go. Add the people and give each node its author.
   *Write the flow from the inputs* fills the group's flow.
3. **Issue the node files** — *Node files & release*. *Issue* one, or every
   node file into a folder at once (a zip where the browser cannot write a
   folder). Put each in `nodes/` and tell its author. Save the structure: it
   records what was issued, at which contract version.
4. **Change a contract later** — edit it in *Structure*. The page names every
   node that reads the one you changed, and marks the changed node's file as
   *behind*: re-issue it, and tell those authors.
5. **Assemble the release** — *Node files & release*, *Choose their folder…*
   (or the files). Each node's content is taken from its file; a file written
   to an older contract, a file for a node that is not in the structure, and a
   node without a file are each named. The result is the release, not yet
   sealed: save it as `releases/solar-1.0.vleo`.
6. **Sign and seal** — *Sign & seal*. Each node is signed by its author (in
   the node application, or here), and the whole group by its owner. A
   signature is for the content as it is: change it and the signature goes
   stale. With no errors and every signature current, *Seal* writes the sealed
   release. Tell the developer.
7. **The next version** — open the sealed release and issue node files from
   it: each author starts from what was sealed. A sealed release is never
   edited; *start the next version from this one* makes an unsealed copy.

A **folder** of the pattern ([`docs/GROUP_FOLDER.md`](GROUP_FOLDER.md)) — the
worked example, or one written from the design by `xtask group-export` — opens
too, and *Keep as a database* turns it into a release file in one step.

## The author, in the node application

Open `web/node.html` and your node file. The page walks you through, step by
step, and every step shows what it will look like as you type:

| Step | What you give |
| --- | --- |
| Your node | read the contract — what it answers, what feeds it, who reads it; ask the lead for a change here, never work around it |
| Explanation | six short sections: in one line, said simply, picture it, guess first, where it breaks, the common misreading |
| Theory & maths | the equations, the derivation, the assumptions, where it holds; each equation once, in LaTeX, with the equation helper beside it |
| Pseudocode | the algorithm, line by line — required for every computed node; typeset as equations and read for mistakes as you type, never run |
| Inputs | how each input is written, its default and the range the node holds for |
| Results | your own answers, from your own code, a hand calculation, a spreadsheet or a paper — paste them from your spreadsheet |
| Evidence & sources | values from outside any code, each citing a source; add your own sources here |
| Pictures | charts, flows, step-by-step walk-throughs, heatmaps, 3D views from CSV data; images and short videos |
| Code & files | your own code, kept for the record, and a source PDF you may keep |
| Preview | the node exactly as the group and the application will show it |
| Check & sign | what is still wrong, by file and line; then your signature |

**Save** often; each save is a new revision. Then put the file back in
`nodes/` on the drive.

## What the applications do not do

- **They run nothing.** The pseudocode is read and typeset, never executed;
  the results are the author's. The developer's engine is generated from the
  pseudocode after the seal and tested against these results.
- **They do not merge two people's edits to one file.** Each file has one
  writer by design. If two copies of one node file come back, the lead keeps
  one and the assembly names the other.
- **The pseudocode check here is a reading, not the language checker.** It
  finds an unused input, an unclosed `if`, unpaired brackets, a missing
  `return`. The full unit check runs when the developer takes the release.

## Where it breaks

A browser that cannot open a file for writing (Firefox, Safari) saves by
downloading, and a downloaded file left in *Downloads* is a change nobody
assembled. The page says which way it saves, every time.

## Common misreading

*"The release is the file we edit."* It is not. The release is rebuilt from
the node files; the node files are where content lives, and the structure is
where contracts live. Edit those, and assemble again.
