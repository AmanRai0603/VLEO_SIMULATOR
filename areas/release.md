# areas/release.md

> **Answer first.** The release pipeline builds, gates, packages and publishes only a commit a merged pull request put on main; it never changes branch protection or signs without a certificate.
>
> **Kind:** reference + explanation · **For:** developers

Packaging, the installer, the wheel, the release workflow and the daemon
packaging.

Applies to `.github/workflows/release.yml`, packaging manifests.
A pipeline, maintained by the developers. A release is how a change to the
code reaches everyone. It never changes anyone's design files.

## Two things this area may never do

**Change branch protection.** The rules that decide what may merge are set by a
person in repository settings. A pipeline that can widen its own gate has no
gate.

**Hold a signing key.** Signing and notarisation take a certificate that lives
in an organisation secret a release engineer controls, never read by a workflow
file. The signing job is deliberately absent until that certificate exists: a
workflow that pretends to sign is worse than one that admits it does not.

## Prove before you build

A release that discovers its own failure after the artefacts are uploaded has
already told somebody the wrong thing. The order is fixed:

1. the commit came to `main` through a merged pull request
2. the tag agrees with the version in `Cargo.toml`
3. the gate
4. the regeneration diff is clean
5. every bundle hash verifies
6. golden vectors against the **release** profile — link-time optimisation
   changes inlining, so a release proved only in debug is not proved
7. the notes, read off the log
8. only then, the binaries

## The decision is the merge

A release is a commit on `main` that a merged pull request put there. The
first step of `prove` refuses anything else — a run started off `main`, a tag
on a side branch, a commit pushed to `main` around review — before a single
binary is built, and writes the pull request that decided it to the run's
summary. How many approvals that pull request needs is the repository variable
`RELEASE_APPROVALS`, set by a person, 0 when unset; a pipeline that could lower
its own bar would have none.

It replaced a GitHub environment with required reviewers, which a private
repository on a free plan cannot have, and which asked for approval after
everything was built, against a list of file names. `docs/RELEASE_SETUP.md`
has the whole rule and how to prove it works.

## The three doors

One kernel, three ways in, built from one merge:

| door | crate | for |
|---|---|---|
| the command line | `vleo-cli` | scripts, campaigns, CI |
| the local server | `vleo-daemon` | the browser face |
| the shared library | `vleo-ffi` | the Python wheel, and MATLAB through it |

## What a release may not change

A published version's numbers. If a release would move a result, that is a
finding for the integrator before it is a release note — the chain hash of an
old ledger row must still reproduce.

**The released design's answers.** `baseline/today.csv` holds what the engine
answers, and `cargo test` fails when an answer moves. From the switch-over on,
the record is taken from the current released design when a release is prepared
(W15), and a release that moves an answer is not made.

## Release notes

Read off the log by `tools/release_notes.py`, which is why every commit subject
carries a machine-readable prefix. It never drops a commit: a subject it does
not recognise is listed under "uncategorised" where somebody will see it.
Breaking changes are first, always.
