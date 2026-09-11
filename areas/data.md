# areas/data.md

Reference data: bundles, hashes, the lockfile, verification and expiry.

Applies to `crates/vleo-data/**`, `bundles/**`.
Agent K's area — which is a script now, not an agent, because publishing
without a hash should be impossible rather than forbidden.

## Why there is no database on the physics path

Reference data and operational data are one hard problem only while they are
treated as one problem. Separated, each has an obvious answer:

| | reference data | operational data |
|---|---|---|
| is | solar drivers, coefficients, fitted biases, validated cases | identity, entitlement, the run ledger |
| changes | slowly, by publishing a new version | constantly |
| needed | by every evaluation | occasionally, never during one |
| lives in | immutable versioned bundles, synced to a local store | a database behind an interface |
| network down | everything still works on what is local | history pauses; nothing else |

Only one of the two is on the physics path, and it never crosses the network at
run time. A campaign of a million cases makes zero network calls, and a run in
an air-gapped facility is the same run made anywhere else.

## The bundle format

A manifest plus payload files, content-addressed with the same FNV-1a the
kernel uses for its chain hash.

A manifest that is missing any of `name`, `version`, `provenance`,
`licence_until`, `stale_after_days` or a non-empty `files` list **does not
load**, and the refusal names the field. `licence_until` is checked for shape,
not trusted: `licence_until = "soon"` parses as TOML and expires on no day at
all.

## Publishing is irreversible

    cargo run -p xtask -- bundle publish bundles/<name>/<version>

A published version is never edited. A correction is a new version. That is
what lets a ledger row from eighteen months ago be reproduced exactly.

Publishing twice from the same input gives the same hash, which is the only
thing that makes verification mean anything.

## Synchronisation and evaluation never overlap

A run either has verified data on disk or refuses to start. It does not fetch,
wait, retry, or fall back silently. A result computed from unverifiable data is
not a degraded result; it is not a result.

## Expiry is read locally

`licence_until` is a date in the manifest, so a time limit works with no
network. `stale_after_days` decides when the input-pedigree credibility factor
drops. Neither may default to a number nobody chose — a missing
`stale_after_days` is a refusal, not a zero.

## The format that is deliberately not built yet

The intended production format is columnar — Parquet, read in process, no
server to install, run, back up or version. The trigger for building it is the
first bundle that does not fit comfortably in memory as text. Until then, a
format anybody can read in a text editor is worth more than one that needs a
library to inspect.

## The parity grid

Not reference data, but it lives by the same rule and is easy to confuse with a
fixture, so it is written down here.

A node translated from the existing MATLAB tool sets `migrated_from` on its
sheet, naming the function and line. The old implementation's numbers go in
`parity.csv` beside the node:

    a_in,a_out,eta_geo,beta,t_c,matlab_eta_c
    0.2,0.01,0.9,0.06,600.0,0.40909

Input columns are keyed by symbol, as fixtures are. The last column is what the
prior implementation returned.

**These are never fixtures.** An implementation cannot supply its own expected
values, and the MATLAB is an implementation. A disagreement between the grid
and this engine is a finding about one of the two — and both classes have been
found before — not a check either of them has passed. The check numbers still
come from the paper, textbook or measurement the MATLAB was built from, which
for most of this chain still exists.

That is what turns eighteen months of prior work from a liability into the
strongest verification asset the programme has. The gate says so while the grid
is missing: `migrated_from` with no `parity.csv` is a note on every run.
