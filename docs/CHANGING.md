# Changing the code

> **Answer first.** Four kinds of change reach this code, and each has one path: a **node** arrives in its group's sealed release, on the shared drive, and the engine runs its method; a **route** is a handler, a contract entry and a manual entry; a **component** is drawn once in `web/js/components.js` or declared as a panel in `panels/`; an **output kind** is a variant of `vleo_modules::figure::Kind`, a player for it and a sample. Every path ends at the same two commands: `cargo run -p xtask -- gate && cargo test`.
>
> **Kind:** how-to + reference · **For:** developers changing the tool, and any assistant they run

This page is the map. The rules the map obeys are in `AGENTS.md` and are not
repeated here; the five that do not bend are worth reading first. When this page
and `AGENTS.md` disagree, `AGENTS.md` is right and this page is the bug.

## Said simply

The tool is one engine with several faces. The engine computes; the faces show.
A change belongs to exactly one of four places, and the place decides what else
has to change with it — the generators, the contract, the manual, a picture a
person approved. Find the place first; the list of what comes with it follows.

| You are changing | It lives in | What comes with it |
|---|---|---|
| what a row computes or declares | its node file, `design/groups/<group>/nodes/<id>.vnode`, written by its node engineer in the application | its group's sealed release, the generators, fixtures, a de-risking record |
| what the engine answers over HTTP | `crates/vleo-server/src/`, one file per concern | `contract/`, the manual, the mock engine's examples |
| how something is drawn | `web/js/components.js`, or a panel in `panels/` and `web/js/solar.js` | the panel check, sometimes a reference picture |
| a new kind of figure | `crates/vleo-modules/src/figure.rs` | a player in `web/js/figures.js`, a sample, the schema |

## 1 · A node

A node changes in its own file, written by its node engineer, and reaches the
design with its group's sealed release — never by editing the sheet in a browser
and never by an assistant supplying a relation. The release goes on the shared
drive, and the tool builds today's design from it when it opens there, checked,
with no command in this repository (`docs/GROUP_APPS.md`). The node arrives
here in `design/`. What the developer does with it is check it, each command
explaining itself (`cargo run -p xtask -- explain <command>`):

    cargo run -p xtask -- method <node>               its method, on its node engineer's cases
    cargo run -p xtask -- ready <node>                how far it is past every machine stage
    cargo run -p xtask -- gate <node> && cargo test

A need the code cannot meet comes back as a request (W14), answered with an
application release; the developer edits no node. A node is its file in
`design/`, written by its node engineer in the group's application; the tools
read it as its sheet, `node.toml` and `fixtures.toml` (values from outside this
code, never `self-snapshot`), and nothing in this repository writes it. What a
sheet's fields mean is
`docs/NODE_AUTHORING.md`; the method language is `docs/PSEUDOCODE.md`.

A relation is a formula, and a formula lives in `vleo-core::physics` (rule 3).
If a node needs one that is not there, adding it is a reviewed change to the
kernel, made before the node that reads it.

## 2 · A route

1. **The handler.** In the file of `crates/vleo-server/src/` that owns the
   concern — `results.rs` for saved results and the record's figures, `pages.rs`
   for pages, `lib.rs` for runs, sweeps and levers. Add the match arm in
   `route()` in `crates/vleo-server/src/lib.rs`. A refusal is an answer
   (`{"ok":false,"message":…}`), never a panic and never a substitute value.
2. **The contract.** A `[[route]]` in `contract/routes.toml` — method, path,
   answer kind, schema — and at least one `[[route.example]]`. A JSON answer
   needs a schema in `contract/schemas/`; a field the engine sends and the
   schema does not declare fails the contract test. Record the examples from the
   real engine:

       VLEO_CONTRACT_RECORD=1 cargo test -p vleo-server --test the_contract_holds

   Within `/v1` a change may only add (`contract/README.md`).
3. **The manual.** A `[[route]]` in `docs/manual.toml` saying what it is for.
   The contract test refuses a route the manual documents and the contract does
   not, and the other way round. Regenerate the role guides from the manual:
   `cargo run -p xtask -- guides`.
4. **The page**, if it reads the route: through the helpers in
   `web/js/record.js`, never a bare `fetch` scattered through a panel. The mock
   engine (`tools/mock_engine.py`) serves the recorded examples, so
   `python3 tools/mock_check.py` proves the page works with no Rust running.

## 3 · A component

**A part every page can use** — a claim tag, an equation, a "try it" widget —
is drawn once in `web/js/components.js` and used by name. Nothing reaches a
page as hand-written markup; the gate refuses a lesson that carries any. A
widget computes nothing itself: it sends the reader's value to the engine and
shows what comes back.

**A panel on a row's page** is two things that must agree:

- its spec, `panels/<id>.toml`: which rows it argues about, which inputs move
  it, which engine figures and rows it reads (`[[figures]]`, `[[engine]]`), and
  whether it has an approved reference picture;
- its entry in `PANELS` in `web/js/solar.js`: `data()` asks the engine,
  `build()` draws. A panel computes no statistic of its own — that moved into
  the engine in phase 10, and `vleo figure <id>` shows the same numbers.

Then `python3 tools/panel_check.py --panel <id>` (it renders, it moves, it
reads what it declares, it matches its picture, an interaction undoes to the
pixel) and `python3 tools/panel_review.py` to regenerate `panels/REVIEW.md`.
A reference picture is confirmed by a person who looked at it; recording one
(`--record`) is never the same as approving it.

## 4 · An output kind

The engine describes every figure as data (`crates/vleo-modules/src/figure.rs`)
and the page plays it (`web/js/figures.js`). The kinds today, by the name the
wire carries:

| `Kind` | drawn as |
|---|---|
| `line` | curves against one axis |
| `scatter` | points against one axis |
| `bar` | values by category |
| `heatmap` | a value over two inputs, in the ordinal ramp |
| `animation` | a line figure in frames: play, pause, scrub |
| `scene3d` | bodies and paths, turned by dragging |

A new kind is, in order: the variant in `Kind` and in `Kind::ALL`, with its
name; what `figure::check` requires of it; a sample in `figure::samples`, which
`/v1/figures/samples` serves and the gallery draws; its shape in
`contract/schemas/figures.json`; and its player in `web/js/figures.js`. The
documentation check fails until this table names it.

## 5 · The generators, with an example

A sheet is the only source (rule 1). From it the generators write the rest, and
the pipeline's regeneration diff fails any generated file that does not match what
its generator would write now.

| Command | Reads | Writes |
|---|---|---|
| `xtask docs` | the method language's checker, `vleo_sheet::method` | `docs/PSEUDOCODE.md`, the method language's reference — a node's page is rendered when it is opened, never written |
| `xtask assemble` | every sheet | the index the faces read and every page fragment, in `generated/` |
| `xtask variables` | every sheet | `docs/VARIABLES.md` |
| `xtask codeowners` | `areas/teams.toml` | `CODEOWNERS`, for code paths only |
| `xtask derisk` | every `[[version]]` | `docs/DERISK_NARRATIVE.md` |
| `xtask guides` | `docs/manual.toml` | `docs/roles/*.html` |
| `xtask pipeline` | the command table in `xtask/src/pipeline.rs` | `docs/PIPELINE.md` |
| `xtask method-wasm` | `vleo_sheet::method::CHECKER_SOURCES` | `web/method.wasm.gz`, the checker the pages carry |
| `xtask readers` | the tree and the lessons | the readers' folder, opened from a file |

**One field, followed through.** `sw_ap_design` declares the lowest answer it
will give, and why:

    # design/groups/l3_solar/nodes/sw_ap_design.vnode, its output port
    lower        = 40.0
    range_reason = "lower: the lowest value this relation can return is 48, at G1. …"

The engine turns those two lines into a guard that refuses a smaller answer by
name, with the reason as the refusal's text; the row's page says "Outside
`40 … 140` the row refuses rather than answers" followed by the same reason.
Change the bound in the node file and both move together. That is the whole design
in one field: the number and its reason are written once, by a person, and everything
that repeats them is printed.

## 6 · The docs a change needs

`python3 tools/docs_lint.py` holds the documentation to the code, and it
refuses:

- a Rust source file that does not open with a `//!` line saying what it is —
  a file nobody can place is a file nobody reviews;
- a figure kind this page's table in §4 does not name;
- an `xtask` command no document names, and a sheet field
  `docs/NODE_AUTHORING.md` does not explain;
- an instruction file that names a path that no longer exists.

Beyond what is checked: a change that moves a decision appends its
`[[version]]` (`docs/DERISKING.md`); a change people will notice says so in the
manual (`docs/manual.toml`); and everything that teaches follows
`docs/EXPLAINING.md` — answer first, then said simply, then the real thing, then
where it breaks.

## Where this page breaks

It maps the four kinds of change the tool has had. A change that is none of them
— a new face, a new ring, a dependency — is an architecture decision, and starts
in `docs/ARCHITECTURE.html` §11, not here.
