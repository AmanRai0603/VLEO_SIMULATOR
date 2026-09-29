# Lessons

> **Answer first.** A lesson is `lesson.toml` beside a row's `node.toml`: content only — stations of text with a claim each, equations with their source, "try it" widgets that name input and output rows, check-yourself questions, references. The page draws every part from one component library, and a widget computes nothing itself: the engine answers it. The gate refuses a lesson with markup, an untagged claim, or a widget naming a row that is not there.
>
> **Kind:** reference · **For:** the experts who write lessons, and the developers who place them

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

## How one arrives

The expert writes it and sends it; a developer places it beside the row's
`node.toml` and runs the gate — `cargo run -p xtask -- gate <node>` — which
reads it (check `lesson`) and refuses it with every reason. It goes through
review like a node change, and ships in the kit with the row's folder.

A lesson sits **outside the sheet hash**, like `[explain]` and `[theory]`: it is
how a row is taught, not what it computes, so a lesson never changes an answer
or a saved result's key.

## Where this breaks

- **There is no lesson form yet.** Node forms are a page anyone fills and a
  developer takes in with one command; a lesson is still a TOML file sent by
  hand. The form is the next piece of this work.
- **A widget needs the engine running.** The page asks the local engine;
  a lesson read from a static copy of the docs has no widgets that answer.
- **No row has a lesson yet.** The example is an example: its text is taken
  from what `orbit_velocity`'s own sheet says, its `by` names nobody, and it is
  not placed beside the row. And that row does not answer today (its relation
  is stated, not derived), so a widget on it shows the engine's refusal — which
  is the right thing for it to show.
