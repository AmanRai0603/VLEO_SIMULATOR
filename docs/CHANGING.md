# Changing the tool

> **Answer first.** Where each kind of change goes, which file it starts in and what else must move with it, so that the gate, the tests and the manual all agree once you commit.
>
> **Kind:** how-to · **For:** developers

This is about the tool itself: the server, the task runner, the generators.
A change to the design (a node, a relation, a fixture) goes through a node form
instead; see [USING_IT.md](USING_IT.md) section 3 and [NODE_AUTHORING.md](NODE_AUTHORING.md).

Before any of it: `cargo run -p xtask -- explain <command>` says what a command
will do, and `cargo run -p xtask -- why <path>` says whether a file is generated.
Never edit a generated file. Change what it comes from.

## Where things live

| to change | start in |
|---|---|
| a route the daemon answers | `crates/vleo-server/src/lib.rs` (the route table); the handler in `http.rs`, `case.rs`, `runs.rs`, `saved.rs` or `pages.rs` |
| request limits, the site check | `crates/vleo-server/src/http.rs` |
| how saved results are kept, thinned, pinned, shared | `crates/vleo-modules/src/results.rs` (`store`, `share`) |
| a write of a person's file | `crates/vleo-modules/src/files.rs` (`write_whole`); never `fs::write` directly |
| an `xtask` command | `xtask/src/main.rs` (dispatch and help); the command in `forms.rs`, `authoring.rs`, `generate.rs`, `report.rs`, `release.rs`, `flow.rs`, `method.rs` or `pipeline.rs` |
| a gate check | `crates/vleo-sheet/src/gate.rs` |
| what a sheet field is, how it is written back | `crates/vleo-sheet/src/form/` (`fields.rs`, `text.rs`, `arrays.rs`, `save.rs`) |
| the node form page | `crates/vleo-sheet/src/template.rs`; its script and style in `crates/vleo-sheet/assets/` |
| HTML or JSON escaping in the sheet crate | `crates/vleo-sheet/src/escape.rs` |
| a date or a timestamp | `crates/vleo-units/src/clock.rs`, the one clock |
| the build script of a node crate | `tools/node_crate_build.rs`; every node crate includes it |
| the manual, the role guides, the pipeline page | `docs/manual.toml`; the guides and `docs/PIPELINE.md` are generated from it |

The rings point inward (`vleo-units` → `vleo-core` → `vleo-bus` →
`vleo-mod-*` → faces). A helper a lower ring needs goes down, never up;
`cargo test` refuses an outward dependency.

## Adding a route

1. Add the arm to `route` in `crates/vleo-server/src/lib.rs`, and the handler
   beside the ones it resembles.
2. Add a `[[route]]` to `docs/manual.toml`. The manual test refuses a route the
   code answers and the manual does not name.
3. If it reads an environment variable, add an `[[env]]` too.
4. Run `cargo run -p xtask -- guides`, then commit the guides with the change.

A route may never write the repository. It writes only under `~/.vleo/`
(the case, results, logs), and through `write_whole`.

## Adding an xtask command

1. Add the arm to `let r = match cmd {` in `xtask/src/main.rs`, and its lines to
   `help()`. The manual test holds each usage to the help text.
2. Add a `[[command]]` to `docs/manual.toml` with `usage`, `what`, `who`,
   `effect`, and `steps`, `writes` and `runs`. A command that changes the
   repository and does not say how will not load.
3. Name it in [USING_IT.md](USING_IT.md). The docs lint refuses a command no
   document names.
4. Run `cargo run -p xtask -- pipeline` and `cargo run -p xtask -- guides`,
   then commit what they write.

## Adding or changing a gate check

1. Write it in `gate_node` or `validate_tree` in `crates/vleo-sheet/src/gate.rs`,
   with a named `Check`.
2. Add a case to `crates/vleo-sheet/tests/every_gate_check_refuses.rs`: take a node the gate
   passes, break it in the one way the check is for, and require that check,
   by name, to refuse it.
3. Run the new test once with the check removed, and see it fail. A check that
   passes whatever it is given is not checking anything.
4. `cargo run -p xtask -- gate` must still pass on the tree. If it does not, the check
   has found something. Take it to the node owner. Never loosen the check to
   get green.

Code a check reads is read with `gate::code_only`, so a comment or a string
is never mistaken for code.

## Before you commit

    cargo fmt --all
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo run -p xtask -- gate
    cargo run -p xtask -- docs && cargo run -p xtask -- variables && cargo run -p xtask -- derisk && cargo run -p xtask -- guides && cargo run -p xtask -- pipeline
    git status        # nothing the regeneration changed may be left out
    python3 tools/docs_lint.py

The commit subject is `type(scope): a sentence`, at most 72 characters. The
hook set up by `cargo run -p xtask -- setup` checks it.
