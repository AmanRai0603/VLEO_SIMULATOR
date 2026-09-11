# areas/release.md

Packaging, the installer, the wheel, the release workflow and the daemon
packaging.

Applies to `.github/workflows/release.yml`, packaging manifests.
Agent L's area — a pipeline now, not an agent.

## Two things this area may never do

**Change branch protection.** The rules that decide what may merge are set by a
person in repository settings. A pipeline that can widen its own gate has no
gate.

**Hold a signing key.** Signing and notarisation take a certificate that lives
in an organisation secret a release engineer controls, requested at the
approval step and never read by a workflow file. The signing job is
deliberately absent until that certificate exists: a workflow that pretends to
sign is worse than one that admits it does not.

## Prove before you build

A release that discovers its own failure after the artefacts are uploaded has
already told somebody the wrong thing. The order is fixed:

1. the tag agrees with the version in `Cargo.toml`
2. the gate
3. the regeneration diff is clean
4. every bundle hash verifies
5. golden vectors against the **release** profile — link-time optimisation
   changes inlining, so a release proved only in debug is not proved
6. the notes, read off the log
7. only then, the binaries

## The one human approval

A GitHub environment named `release`, with required reviewers configured in
Settings → Environments. The workflow only names it. That separation is the
whole safety property: the workflow cannot grant itself the approval by editing
itself.

**Until that environment exists with reviewers on it, the gate approves
instantly.** Creating it is a one-time human action and nothing in this
repository can do it.

The approver sees the release notes and the artefact list in the job summary at
the moment they approve. An approval given against a page that does not say
what is in the release is a button press, not a decision.

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

## Release notes

Read off the log by `tools/release_notes.py`, which is why every commit subject
carries a machine-readable prefix. It never drops a commit: a subject it does
not recognise is listed under "uncategorised" where somebody will see it.
Breaking changes are first, always.
