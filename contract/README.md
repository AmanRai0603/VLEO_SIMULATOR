# The contract

> **Answer first.** The frontend and the backend meet here and nowhere else: every route the engine serves, a schema for every JSON answer, the file formats, and example answers recorded from the real engine. The backend proves it keeps the contract with `cargo test`; the frontend builds against the examples on the mock engine, with no Rust. Inside `/v1` a change may only add.
>
> **Kind:** reference · **For:** frontend, backend and data developers

## What is here

| | |
|---|---|
| `VERSION` | The contract's version. `/v1/version` says it as `contract`, and the page carries the one it was built for; when they differ the page shows a banner rather than misreading an answer. |
| `routes.toml` | Every route: its method and path, as the manual lists them; what kind of answer it gives; its schema or file format; and the example requests the tests make. |
| `schemas/*.json` | One JSON Schema per kind of JSON answer. A field the engine sends and a schema does not declare is a failure — so the schemas are the complete list of what the page may read. |
| `formats/*.toml` | The CSV files people keep and send: a saved result, a sweep, the case. Their marker line, their `#!` meta lines and their columns. |
| `examples/*.json` | Answers recorded from the real engine for the requests in `routes.toml`. The mock engine serves them. |

## The rule

**Within `/v1`, add only.** A new route, or a new field in an answer, breaks nobody: an older page ignores it, and a newer page can show it as soon as the engine sends it. **Removing a field, renaming one, or changing what one means is `/v2`**, served beside `/v1` for one release while the page moves over; `/v1` goes one release later. Nobody has to merge on the same day.

**A contract change is reviewed by both sides.** CODEOWNERS names the frontend and the backend teams on `contract/` (and the data team on `formats/`). GitHub asks any one owner of a path, so reaching both is a rule of review, written in `CONTRIBUTING.md`: a pull request that changes `contract/` waits for a reviewer from each side.

## Changing it

**A new field.** First a pull request that adds it to the schema (both sides review); then the backend sends it and the frontend shows it, in either order. Until the schema declares it, the contract tests fail on the engine that sends it — by design: the page may read only what the contract lists.

**Re-record the examples** when an answer's shape changes:

    VLEO_CONTRACT_RECORD=1 cargo test -p vleo-server --test the_contract_holds

This rewrites `examples/` from the real engine and drafts a schema for a route that has none. It never rewrites a schema: a schema is the contract, and changing it is done by hand.

**A new route.** Add it to the manual (`docs/manual.toml`), to `routes.toml` with an example request, and record. The tests refuse a route in one and not the other.

## What the tests hold

`crates/vleo-server/tests/the_contract_holds.rs` starts the real server on a scratch case and results folder, makes every request `routes.toml` lists, in order, and checks:

- every JSON answer validates against its schema, with no undeclared field;
- every recorded example validates too, so the mock never serves an answer the engine no longer gives;
- every CSV in a file format has its marker, meta lines and columns;
- every route the manual documents is here, and nothing else; every schema, example and format on disk is used;
- the contract version is the same in `VERSION`, `/v1/version` and `web/js/state.js`.

The schemas use a subset of JSON Schema — `type`, `properties`, `required`, `additionalProperties`, `items`, `anyOf`, `enum`, and the notes `title`, `description`, `$comment`, `$schema`. A keyword outside it is refused, not ignored.

## Working on the page without Rust

    python3 tools/mock_engine.py            the page, on http://127.0.0.1:7780
    python3 tools/mock_check.py             the page on it, in a browser

The mock engine serves `web/` as it is on disk and every `/v1` answer from `examples/`, picked by the request's parameters; node pages, reference data and parity files come from the repository, where the engine reads them too. It computes nothing — a run of a row with no recorded example answers with another row's numbers — so every answer carries an `X-VLEO-Mock` header naming its example, and the page shows **MOCK ENGINE** at the top. A request with no recorded answer is a 404 that says how to record one.
