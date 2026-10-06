# Lessons

> **Answer first.** A lesson is `lesson.toml` beside a row's `node.toml`: content only — stations of text with a claim each, equations with their source, "try it" widgets that name input and output rows, check-yourself questions, references. The page draws every part from one component library, and a widget computes nothing itself: the engine answers it. The gate refuses a lesson with markup, an untagged claim, or a widget naming a row that is not there.
>
> **Kind:** reference · **For:** the node engineers who write lessons, and the developer who places them

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

In the tool, on the row's page. And in **the readers' folder** —
`cargo run -p xtask -- readers` — every row's page and every lesson as plain
pages for a shared drive or an internal web server, drawn by the same
components, with the engine running in the page to answer the widgets —
the same numbers as the tool's, which the pipeline proves on every push
(`tools/readers_check.py`, then `cargo test` in `crates/vleo-kernel-wasm`).

## How one arrives

Like a change to a node: through a form, filled by the person who knows the
row, and applied by a developer.

1. **The form.** `cargo run -p xtask -- lesson form <node> --out <file>`, or
   *write a lesson — its form* on the row's page. One HTML file that works
   with no server and no network: the lesson as fields, the row's question
   beside it, and **the check the gate runs, run in the page** as the node engineer
   types. The form carries the checker every node form carries
   (`web/method.wasm.gz`, which holds `vleo_sheet::lesson::report`) and a
   table of the tree's rows, so a widget naming a row that is not there — or
   asking a reader to move a computed one — is refused before it is sent.
   *Save a filled copy* writes the file to send back.
2. **The check.** `cargo run -p xtask -- lesson check <file>` says what the
   filled form holds and every reason it would be refused. It reads a filled
   form or a bare `lesson.toml` (with `--for <node>`), and writes nothing.
3. **The apply.** `cargo run -p xtask -- lesson apply <file>` checks it again,
   writes it as `lesson.toml` beside the row's `node.toml`, and gates the row —
   or puts the row back as it was. It goes through review like a node change,
   and ships in the kit with the row's folder.

The lesson travels in the form escaped, so nothing it says can end or open an
element of the page.

A lesson sits **outside the sheet hash**, like `[explain]` and `[theory]`: it is
how a row is taught, not what it computes, so a lesson never changes an answer
or a saved result's key.

## Where this breaks

- **Read without the tool, a widget runs on the declared values only.** The
  readers' folder (`xtask readers`, docs/SHARING.md) carries the engine
  compiled for the browser, so its widgets answer — on the declared values,
  not a saved case, and with no reference data, so a row that reads a data
  bundle refuses there by name.
- **No row has a lesson yet.** The example is an example: its text is taken
  from what `orbit_velocity`'s own sheet says, its `by` names nobody, and it is
  not placed beside the row. And that row does not answer today (its relation
  is stated, not derived), so a widget on it shows the engine's refusal — which
  is the right thing for it to show.
- **A group cannot carry a lesson.** A lesson sits beside a row's `node.toml`,
  and a group — a subsystem such as `l3_prop` — has none. A lesson about a
  whole subsystem hangs on its interface row: the first one planned, on the
  propulsion subsystem, has its form made for `l3_prop_interface` and waits
  on the propulsion owner to write it.
