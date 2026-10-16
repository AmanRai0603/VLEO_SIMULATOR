# Lessons

> **Answer first.** A lesson is a row's `lesson.toml`: content only — stations of text with a claim each, equations with their source, "try it" widgets that name input and output rows, check-yourself questions, references. The page draws every part from one component library, and a widget computes nothing itself: the engine answers it. The gate refuses a lesson with markup, an untagged claim, or a widget naming a row that is not there. Its place is the row's node file, where it is signed with the node; no row has one yet.
>
> **Kind:** reference · **For:** the node engineers who write lessons, and the developer who keeps the form and its check

## Said simply

A node page says what a row *is*. A lesson says how to *understand* it: the
plain words, the real relation and where it came from, where it stops being
true, a slider to try it with, and a question to check yourself. The person who
knows the row writes the words; nobody writes HTML, and nobody writes a formula
for the page to compute — a widget names rows, and the tool computes them, so a
lesson and the tool cannot disagree.

## The file

`docs/examples/orbit_velocity.lesson.toml` is a complete example. Every key is
one the reader knows; a misspelt key is refused, not ignored.

| Table | Keys | Held to |
|---|---|---|
| `[lesson]` | `title`, `by`, `answer`, `kind` | none empty; `kind` is `tutorial`, `how-to`, `explanation` or `reference` (docs/EXPLAINING.md) |
| `[[station]]` | `kind`, `title`, `text`, `claim`, `source` | `kind` is `simply`, `real`, `breaks` or `story`; `claim` is `sourced`, `derived`, `declared` or `illustrative`; a sourced claim names its source; at least one station |
| `[[equation]]` | `text`, `says`, `claim`, `source` | plain text, shown after the *real* station; claimed like a station |
| `[[widget]]` | `title`, `inputs`, `outputs`, `sweep` | every input a **declared** row (a value a person picks); every output a row that is not seeded; `sweep`, if given, one of the inputs |
| `[[check]]` | `question`, `options`, `answer`, `why` | at least two options; `answer` counts from 1 and is one of them |
| `[[reference]]` | `text` | — |

No text may contain markup: `<b>` in a station is a component built by hand,
and the page builds every part from its library (`web/js/components.js`).

## Where it shows

On the row's page, as segment *the lesson — how this row is taught*, drawn by
`renderLesson`. **Learn · Read · Expert** apply: the plain-words and story
stations and the checks are dropped for an expert. A widget sends the reader's
value to the engine as a supplied input (`/v1/run`) and shows what comes back,
including a refusal with the engine's reason; with `sweep` it draws the first
output across the input's range from the figure the engine describes
(`/v1/sweep`, `figure`). Served by `GET /v1/lesson/<id>`, checked again on every
request; a lesson that fails is refused with its reasons, never drawn half-right.

## Where it is read

In the tool, on the row's page, and nowhere else. The readers' folder, which
carried every row's page and lesson as plain pages with the engine running in
the page, was removed in phase F2 (docs/PLAN_F.md, decision 1).

## How one arrives

Like a change to a node: written by the node engineer who knows the row, in
the row's node file, and into the design with its group's release. The
developer applies none.

1. **The form.** `cargo run -p xtask -- lesson form <node> --out <file>`, or
   *write a lesson — its form* on the row's page. One HTML file that works
   with no server and no network: the lesson as fields, the row's question
   beside it, and **the check the gate runs, run in the page** as the node engineer
   types. The form carries the method checker the pages carry
   (`web/method.wasm.gz`, which holds `vleo_sheet::lesson::report`) and a
   table of the tree's rows, so a widget naming a row that is not there — or
   asking a reader to move a computed one — is refused before it is sent.
   *Save a filled copy* writes the file to send back.
2. **The check.** `cargo run -p xtask -- lesson check <file>` says what the
   filled form holds and every reason it would be refused. It reads a filled
   form or a bare `lesson.toml` (with `--for <node>`), and writes nothing.
3. **Into the node's file.** The node engineer saves the lesson in the
   row's node file in their working copy (`POST /v1/node/<id>/lesson`, with
   the filled form or the bare `lesson.toml`). It is checked first as the gate
   checks it, against the design the program reads, and refused with every
   reason; otherwise it is saved as the node's next revision, like any save.
   It lives there as a `text` of kind `lesson` on the node's block, word for
   word as it was written (docs/briefs/F3.md, F3-e). No screen sends it yet:
   the node page's own *write a lesson* is the Node workspace's. A lesson is
   taken out by saving the node's file without it; the route only writes one.
4. **Signed with the node, and into the design with its group's release.**
   The node's signature covers its texts, so the lesson is signed with the
   node as the day's work, and a lesson changed after is not signed. The
   node's own checks hold it to everything the gate checks but the rows it
   names. Read from the design's files, the lesson is served to the loader as
   the `lesson.toml` its folder held (`vleo_files::convert::Served`), so the
   row's page, the gate and the form read it as they read one in a checkout.
   Nothing in this repository writes one into `design/`. A plain save and a
   signing hold it to the design's rows too, as the lesson route does.
5. **Not yet into today's design.** Today's design takes from a group's
   release only its rows' sheets (a release from before 1.0), so a lesson in
   a release does not reach it yet: that is F5's intake. A lesson that does
   not pass its check refuses its node, and so its release, which then keeps
   its group's last good one: a lesson never changes an answer, but a broken
   one can hold a release back.

The lesson travels in the form escaped, so nothing it says can end or open an
element of the page.

A lesson sits **outside the sheet hash**, like `[explain]` and `[theory]`: it is
how a row is taught, not what it computes, so a lesson never changes an answer
or a saved result's key.

## Where this breaks

- **A lesson cannot be read without the tool.** The readers' folder that
  carried one off the disk was removed in phase F2 (docs/SHARING.md, section
  4), and the engine compiled for the browser went with it.
- **No row has a lesson yet.** The example is an example: its text is taken
  from what `orbit_velocity`'s own sheet says, and its `by` names nobody. A
  row's file can hold one now, and none does. And that row does not answer
  today (its relation is stated, not derived), so a widget on it shows the
  engine's refusal — which is the right thing for it to show.
- **A group cannot carry a lesson.** A lesson belongs to a row, and a group —
  a subsystem such as `l3_prop` — is not one. A lesson about a
  whole subsystem hangs on its interface row: the first one planned, on the
  propulsion subsystem, has its form made for `l3_prop_interface` and waits
  on the propulsion owner to write it.
