# Glossary

> **Answer first.** The words this repository uses in a meaning of its own, each in a sentence, with where to read more.
>
> **Kind:** reference · **For:** users, maintainers, developers

Terms are in alphabetical order. A word in *italics* is defined here too.

| term | what it means here |
|---|---|
| **achieved** | A *kind* of row: the value the design reaches, set against a *required* row. |
| **blocked** | A row a run could not compute, always named with the reason. A run says how many ran and how many were blocked; it never leaves one out. |
| **branch** | The *run mode* that computes a row and everything it reads. The mode a defensible number comes from. Also `alone` (only the row) and `all` (everything buildable). |
| **bundle** | A named, versioned, content-hashed set of *reference data* in `bundles/`. A run either has verified data or refuses to start. |
| **case** | The inputs a run uses: every input's default, with the values a person saved on top. Kept outside the repository, in `~/.vleo/`. |
| **chain** | The hash that says exactly which kernel, tree, inputs and data produced a result. Two results with the same chain are the same run. |
| **computed** | A *kind* of row whose value comes from a relation over the rows it reads. |
| **credibility** | A score from 0 to 4 on each value, the lowest of eight factors. The factor that sets it is named as what *governs* the value. |
| **criticality** | `minor` or `significant`, set on a sheet. It decides how many reviewers a change needs and whether its holes are filled twice by different models. |
| **declared** | A *kind* of row that states one measured number, with its source and who confirmed it. |
| **evidence debt** | The first line of `xtask status`: computed rows with no *fixture*, and relations with nobody's name against them. It should only go down. |
| **face** | A way into the one kernel: the browser (through the daemon), the `vleo` command line, the Python package, the desktop app. |
| **fixture** | A known-good value for a row, with where it came from. It may never come from the code under test. `fixtures.toml` beside the sheet. |
| **form** | A *node form*: one self-contained HTML file that asks every question a sheet needs, filled offline and sent back. `xtask take` brings it in. |
| **gate** | `cargo run -p xtask -- gate`: the checks every node and the whole tree must pass. A refusal is a finding, never a tolerance to widen. |
| **governing** | The credibility factor holding a value down. |
| **hole** | A numbered `// ---- HOLE n` block in a generated `model.rs`, the only part a person writes. Everything outside it is regenerated. |
| **kit** | The tool as a team member gets it: the programs and the files they read, in one folder. `xtask kit`; see [SHARING.md](SHARING.md). |
| **kpi** | A *kind* of row the design is judged by. |
| **layer** | Management, the system, subsystem: the three levels a row sits at. |
| **node**, **row** | One quantity of the design, with its sheet, its generated code and its evidence. Used interchangeably. |
| **pinned** | A saved result kept whole whatever its age. |
| **provenance** | Where a fixture's value came from: an independent derivation, a published source, an independent tool, or a physical bound. |
| **published** | A row whose sheet is filled in and whose code is generated. The opposite of *seeded*. |
| **reference data** | Slow-changing inputs every run needs (solar drivers, coefficients), shipped as *bundles*, never fetched during a run. |
| **required** | A *kind* of row that states a bound, with a *sense*: `<=` if the achieved value must stay under it, `>=` if it must reach it. |
| **result** | One run kept as a CSV: every input, every value, every blocked row, and the *chain*. Opening one runs nothing. |
| **ring** | The dependency levels of the crates: units, core, bus, the node crates, the faces. A crate depends only inward. |
| **seeded** | A row that exists in the tree with an id, a label, a parent and an owner, and nothing yet specified. |
| **sheet** | A row's `node.toml`: the only source. Everything else about the row is generated from it. |
| **thinned** | A saved result past `VLEO_KEEP_DAYS` that keeps only its answer, the inputs it changed and its chain. Running it again brings every value back beside the record. |
| **`.vleo`** | A result as one file to send: its CSV, its report page and a manifest, zipped. |
| **version** | A recorded change to what a row believes, with the release that carried it. A saved result says which versions it rested on and which have moved since. |

The order the pipeline does things in is [PIPELINE.md](PIPELINE.md); how to
change the tool itself is [CHANGING.md](CHANGING.md).
