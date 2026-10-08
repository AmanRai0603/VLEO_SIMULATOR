<!-- GENERATED from groups/SPEC.toml by `cargo run -p xtask -- group-app`. Do not edit. -->
# The group folder, version 1

> **Answer first.** A group keeps everything it knows in one folder in this fixed pattern: tables as CSV, words as Markdown, the algorithm as pseudocode, and its own test results. The group application (`web/group.html`) opens the folder offline, shows it the way the VLEO application will, checks it against this pattern, records each member's sign-off and seals it for the developer.
>
> **Kind:** reference · **For:** group members, and any assistant that helps them

## Said simply

The folder is the design. Nobody types the design anywhere else: the developer compiles this folder into the database the application reads, and generates the engine from the pseudocode in it. So what the folder says is what everyone gets — and the group application lets a group see exactly that before anything is built.

## The conventions

- **csv.** UTF-8, comma-separated, one header row, one table per file, no merged cells and no formulas. A column that carries a quantity says its unit in the header: `altitude [km]`. A cell that holds several ids separates them with spaces. Save a spreadsheet as CSV; never send the spreadsheet.
- **equations.** Written in LaTeX, stored once in equations.csv. Nobody has to type LaTeX by hand: the group application's equation helper turns plain shorthand such as `v = sqrt(mu/r)` into it, and an assistant can write it from a photo of handwriting, a screenshot of a paper or a Word equation. Every `let` and `return` line of a pseudocode is shown as an equation automatically, so an equation the pseudocode already states is never written twice.
- **ids.** An id is lower case letters, digits and underscores, starting with a letter: `sw_f107_design`. A node's folder is named by its id and never renamed.
- **markdown.** Markdown text with the headings this pattern requires, in order, each as `## Heading`. Maths inline as $...$ and on its own line as $$...$$, both in LaTeX. An equation used more than once lives in equations.csv and is placed with {{eq E1}}. A picture is placed with {{fig id}}, where id is a row of figures.csv.

## The two kinds of text

### `explanation.md` — the explanation

How to UNDERSTAND it, for anyone. The answer first, then the simple version, then a picture, then a question to guess, then where the simple version breaks, then the idea people most often get wrong. Built to be understood, not to be derived: no derivations here. It may place an equation with {{eq}} to point at it, but it never restates the mathematics.

Its sections, in this order: `## In one line`, `## Said simply`, `## Picture it`, `## Guess first`, `## Where it breaks`, `## Common misreading`.

### `theory.md` — the theory

Only the mathematics and its reasoning: the governing equations, the derivation step by step, the assumptions stated as conditions, and where the result is valid. No analogies and no intuition — those are the explanation's, and saying them twice is how the two drift apart.

Its sections, in this order: `## Equations`, `## Derivation`, `## Assumptions`, `## Validity`.

## What a text can hold

- `{{eq E1}}` — an equation from equations.csv (the node's own, then the group's), typeset, with its symbols explained on hover
- `{{fig flow}}` — a picture from figures.csv: a chart, a flow diagram, a step-by-step animation, a heatmap, a 3D view, an image or a video
- `{{guess Which is larger, the day or the slot? || The slot: a day averages eight of them.}}` — a question the reader answers in their head before the answer is shown — explanation only
- `{{node sw_ap_design}}` — a link to another node in this group

## The group's files

- `group.csv` — **required**. who the group is and which version of its folder this is
  - `id` — the group's id, e.g. solar.
  - `name` — its name for people, e.g. Solar weather.
  - `owner` — the person who signs the whole group.
  - `version` — this folder's version, e.g. 1.3; raise it whenever anything changes.
  - `summary` — the group's answer in one sentence.
- `members.csv` — **required**. everyone who signs, and what they sign
  - `name` — as they sign.
  - `role` — owner, author or reviewer. One of: owner, author, reviewer.
  - `nodes` (optional) — the node ids they are the node engineer of, space-separated; * for every node.
- `explanation.md` — **required**. the whole group, understood: what it answers, how its nodes work together, with the flow as a picture
- `theory.md` — **required**. only the maths that spans nodes: coupled equations, the logic of a loop, shared assumptions. Write 'None beyond the nodes' own.' under each heading when there is none.
- `flow.txt` — **required**. the group's pseudocode: which node feeds which, in order. One line per step: `node_id <- input_a, input_b`.
- `nodes.csv` — **required**. every node in the group, one row each
  - `id` — the node's id; its folder is nodes/<id>/.
  - `question` — the one question the node answers, as a question.
  - `kind` — computed (has pseudocode), declared (a value someone states), required or achieved (the two sides of a requirement). One of: computed, declared, required, achieved.
  - `output` — the name of what it answers, e.g. f107_design.
  - `unit` — the output's unit, e.g. sfu, km, W, 1 for a pure number.
  - `lower` (optional) — the lowest value it can sensibly give.
  - `upper` (optional) — the highest value it can sensibly give.
  - `value` (optional) — a declared node's value.
- `publishes.csv` — optional. what the group sends upward to the rest of the design
  - `node` — the node whose output is sent.
  - `to` — the group or system row that reads it.
  - `says` — what the reader gets, in a sentence.
- `requirements.csv` — optional. every requirement, and which way it binds — never left to prose
  - `id` — the requirement's id.
  - `required` — the node holding the bound.
  - `achieved` — the node holding what the design achieves.
  - `sense` — <= when the achieved value must stay under the bound, >= when it must reach it. One of: <=, >=.
  - `says` — the requirement in a sentence.
- `loops.csv` — optional. every loop the group's nodes form on purpose, declared so it is iterated rather than refused
  - `nodes` — the node ids in the loop, space-separated.
  - `converge_on` — the node whose output must settle.
  - `tolerance` — how close two passes must be to stop.
  - `max_iter` — the most passes before it gives up and says so.
  - `seed_node` — the node given a starting value.
  - `seed_value` — that starting value, in the node's unit.
  - `source` — where the starting value comes from.
- `constants.csv` — optional. numbers several nodes share, stated once with their source
  - `id` — the constant's id.
  - `value` — its value.
  - `unit` — its unit.
  - `source` — a sources.csv id.
  - `says` — what it is.
- `sources.csv` — **required**. every source the group rests on; a cited PDF sits in sources/. A node may add its own in nodes/<id>/sources.csv, so its node engineer never waits on the subsystem engineer to cite a paper
  - `id` — the source's id, used wherever it is cited.
  - `cite` — author, year, title, where.
  - `file` (optional) — the file under sources/, if there is one.
  - `url` (optional) — where it can be found.
  - `licence` (optional) — whether we may keep and share it.
- `versions.csv` — **required**. the group's history: what each version believed, tested and learned, and what changed
  - `version` — the version, matching group.csv.
  - `date` — YYYY-MM-DD.
  - `by` — who made it.
  - `believed` — what we believed before.
  - `tested` — what tested it.
  - `learned` — what we now know.
  - `changed` — what changed in the folder because of it.
  - `risks` (optional) — the risks it opened or closed.
  - `cost` (optional) — what it cost — time, mass, margin — if anything.
  - `rests_on` (optional) — what this version rests on: the belief that, if it fails, takes it down.
  - `breaks_if` (optional) — what would break it: the observation that would send the group back.
- `equations.csv` — optional. every equation, once, in LaTeX
  - `id` — E1, E2… — placed in text with {{eq E1}}.
  - `latex` — the equation in LaTeX; the equation helper writes it from shorthand.
  - `says` — the equation in words.
  - `source` (optional) — a sources.csv id.
- `symbols.csv` — optional. what each symbol means, shown when a reader points at it
  - `symbol` — as it appears in the LaTeX, e.g. \mu or F_{10.7}.
  - `name` — its name.
  - `unit` — its unit.
  - `means` — what it means, in a sentence.
- `figures.csv` — optional. every picture, drawn from data; images only for pictures that are not data
  - `id` — placed in text with {{fig id}}.
  - `kind` — line, scatter, bar, heatmap, animation, scene3d, flow, steps, image or video. One of: line, scatter, bar, heatmap, animation, scene3d, flow, steps, image, video.
  - `file` — the data file (figures/…csv) or the image or video (images/…).
  - `title` — the caption: what the picture shows, in a sentence.
  - `x` (optional) — the column across (line, scatter, bar, heatmap, animation).
  - `y` (optional) — the column(s) up, space-separated for several series.
  - `z` (optional) — heatmap: the value column; animation: the frame column.
  - `on` (optional) — steps: the flow figure the steps walk through.
- `results/group.csv` — optional. The group as a whole: group inputs in, group outputs out, at the defaults and at the edges — the group's own test of itself. Columns: one per declared node the test sets, named by its node id with its unit (`orbit_altitude [km]`); then one `answer.<node id> [unit]` per node the test reads; then `tolerance`, `refuses` and `origin`, as in a node's isolation results.
  - `answer` — answer.<node id>, with its unit in the header; one column per node the test reads.
  - `tolerance` — relative tolerance.
  - `refuses` — yes or no. One of: yes, no.
  - `origin` — code, hand, spreadsheet or paper. One of: code, hand, spreadsheet, paper.
  - `says` (optional) — what the case is, in words.
- `reviews.csv` — written by the group application. every sign-off, written by the group application: who signed what, at which content fingerprint
  - `name` — who signed.
  - `scope` — group, or a node id.
  - `version` — the folder version signed.
  - `fingerprint` — the content fingerprint signed; a change to the content makes it stale.
  - `date` — when.
  - `verdict` — ok, or changes. One of: ok, changes.
  - `note` (optional) — what they want changed.

## Each node's files

- `nodes/<id>/explanation.md` — **required**. the node, understood
- `nodes/<id>/theory.md` — **required**. the node's mathematics and nothing else
- `nodes/<id>/pseudocode.txt` — **required** for computed nodes. The algorithm, in the method language (docs/PSEUDOCODE.md): `let`, `if`, `for`, `refuse`, `return`, every number with its unit. Required on every computed node. The developer's code is generated from it, so it must say exactly what the node does. Nothing in the group application runs it.
- `nodes/<id>/inputs.csv` — **required** for computed nodes. every input: where it comes from and the value it takes by default
  - `name` — the name the pseudocode uses.
  - `from` — the node it comes from (id in this group, or group.node in another), or case when the case sets it.
  - `unit` — its unit.
  - `default` (optional) — the value used when nothing else is given.
  - `min` (optional) — the lowest valid value.
  - `max` (optional) — the highest valid value.
  - `says` (optional) — what it is.
- `nodes/<id>/results/isolation.csv` — **required** for computed nodes. The node on its own: one row per test, the inputs in, the answer out. This is the reference the developer's code must match, so it comes from outside that code — your own code, a hand calculation, a spreadsheet or a paper. Columns: one per input, named as in inputs.csv with its unit (`r [m]`); then `answer [unit]`, and for a node that publishes several values one `answer.<member> [unit]` beside it per member it publishes; then `tolerance` (relative, e.g. 1e-9, held by every answer in the row); `refuses` (yes when the node must refuse, with answer left blank); and `origin` (code, hand, spreadsheet or paper). At least: the defaults, three ordinary cases, both ends of every input's range, and one refusal.
  - `answer` — the expected answer, with its unit in the header; blank on a refusal.
  - `tolerance` — relative tolerance.
  - `refuses` — yes or no. One of: yes, no.
  - `origin` — code, hand, spreadsheet or paper. One of: code, hand, spreadsheet, paper.
  - `says` (optional) — what the case is, in words.
- `nodes/<id>/results/how-run.md` — optional. how the code that produced isolation.csv was run: language, tool and version, machine, date, the command — and, so the developer can run it again on each case, `**Entry:** name`, the function that takes this node's inputs by name, in SI, and returns its answer
- `nodes/<id>/evidence.csv` — optional. values from OUTSIDE any code — a paper, a handbook, another tool — that check the physics, not the code
  - `inputs` — the inputs it holds for, as name=value pairs separated by spaces.
  - `value` — the value.
  - `unit` — its unit.
  - `source` — a sources.csv id.
  - `says` (optional) — what it is.
- `nodes/<id>/declaration.csv` — optional. Who made this node's method and its numbers, and whether an assistant helped: `none`, `wording` (the words only), `relation` (the pseudocode, the equations, the results or the evidence), or `transcribed` (an assistant copied into pseudocode a relation a person had already written: their code, their paper, the design as it stands). Written by the node application when its node engineer signs. An assistant may never supply mathematics: the library refuses a method or results an assistant supplied, and takes a node with no declaration as one an assistant helped with. A `transcribed` node names the `source` it was copied from and the person who read the copy against that source (`checked_by`). Without both, or with an assistant's name as the checker, the library takes it as one an assistant supplied.
  - `author` — the person who owns the method and its numbers.
  - `ai` — none, wording, relation or transcribed. One of: none, wording, relation, transcribed.
  - `date` — when it was declared.
  - `source` (optional) — transcribed only: what the relation was copied from — a file and line, a paper and equation, a node.
  - `checked_by` (optional) — transcribed only: the person who read the copy against that source and signs it as theirs.
- `nodes/<id>/code/` — optional. your own code, if you have it, kept for the record. Nothing runs it; its results are in results/isolation.csv

## Where the simple version breaks

- **Nothing in the group application computes.** The pseudocode is shown, and shown as equations, never run. Whether it gives the group's results is decided by the developer's engine, after the seal.
- **The checks are of the folder, not of the physics.** A folder can pass every check and still be wrong; the results and the evidence are what test it, and the developer's tests use both.
- **A browser that cannot write into a folder downloads instead.** Sign-offs, packages and issues are then saved as downloads named for where they belong, and the member puts them in the folder.
