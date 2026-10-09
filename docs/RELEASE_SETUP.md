# How a release is decided

> **Answer first.** A release is decided by merging a pull request into `main`; the release workflow builds and publishes only a commit that a merged pull request put on `main`, and refuses everything else before a single binary is built.
>
> **Kind:** how-to · **For:** the developer and their deputy

This page says what the rule is, why it replaced the approval button the
pipeline used to have, what it cannot stop on its own, and how to tell that it
works.

---

## 1 · The three branches, and where a release comes from

| branch | what moves it | what it holds |
|---|---|---|
| `developer` | pull requests from working branches | the software — generators, daemon, faces, tests |
| `maintainer` | nothing, now: a group's sealed release reaches today's design from the shared drive, not through the repository | the design as the repository holds it until the switch-over, when the branch is deleted |
| `main` | pull requests from `developer`, `maintainer` and `release/<version>` | what ships; every release tag is on it |

`CONTRIBUTING.md` says who reviews each. The only thing this page adds is the
last step: **a release is a commit on `main` that a merged pull request put
there.** Merging that pull request is the decision. The review happened on the
pull request, where the change could be read line by line — not on a button
pressed after the artefacts exist, against a page of file names.

---

## 2 · What the workflow checks

`.github/workflows/release.yml` runs in four stages. The rule is the first step
of the first stage.

| stage | what it does |
|---|---|
| `prove` | **the commit came to `main` through a merged pull request**, the tag matches `Cargo.toml`, every node version names this release, the gate, the regeneration diff, bundle verify, release-profile tests, the notes |
| `build` | the three kits and the Python engines for Linux, macOS and Windows |
| `wheel`, `wheel-check` | the one-file package, installed and started on all three |
| `publish` | creates the GitHub release and uploads the four files |

The rule, in order:

1. **Started by hand** (*Actions → release → Run workflow*): it must be run from
   `main`. A run started from any other branch stops.
2. **Started by a tag**: the tagged commit must be on `main`. A tag on a side
   branch stops.
3. Either way, GitHub is asked which pull requests the commit belongs to. At
   least one must be **merged, into `main`**. A commit pushed straight to
   `main` belongs to none, and stops.
4. **Optionally**, that pull request must carry a number of approving reviews,
   set by the repository variable `RELEASE_APPROVALS` (section 4).

The pull requests it found, and how many approvals each had, are written to the
run's summary page — so every release says which merge decided it.

When any of these fails, the message names what was wrong and says that nothing
has been built. Nothing is built, uploaded or published after a refusal.

---

## 3 · Why not the approval button

The pipeline used to end in a GitHub environment named `release` with required
reviewers: the build ran, then paused until a person clicked **Approve**. Three
things were wrong with it here.

- **This repository cannot have one.** Environments with required reviewers
  are not available to a private repository on a free personal account.
  GitHub's documentation says so
  ([`manage-environments.md`](https://github.com/github/docs/blob/main/content/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments.md)):
  *“Users with free plans can only configure environments for public
  repositories.”* So the job failed every time by design, and every release was
  published by hand from the run's artefacts.
- **It asked the wrong question at the wrong time.** By the time the button
  appeared, everything was built; the approver saw a list of files and release
  notes. The change itself had been readable on the pull request that merged it.
- **It was a second copy of a decision already made.** Merging into `main` is
  already the moment somebody says "this is what goes out". Two places where one
  decision lives are two places that disagree.

The rule that replaced it lives in the workflow file, costs nothing on any plan,
and refuses before it builds instead of after.

---

## 4 · Requiring approvals: `RELEASE_APPROVALS`

**Settings → Secrets and variables → Actions → Variables → New repository
variable**, name `RELEASE_APPROVALS`, value a whole number.

| value | what a release needs |
|---|---|
| unset or `0` | a merged pull request into `main` |
| `1` | …that at least one person approved |
| `2` | …that at least two different people approved |

It is unset on this repository, on purpose. GitHub does not let anybody approve
their own pull request, and today one person opens and merges them all; `1`
would stop every release. Set it to `1` the day a second person reviews.

It is a variable in settings, not a number in the workflow file, for the same
reason branch protection is: a pipeline that can lower its own bar has no bar.

---

## 5 · What the rule cannot stop on its own

The workflow checks where a commit came from. It cannot stop somebody from
pushing to `main` in the first place — only branch protection can, and that is
a repository setting.

- **On a paid plan, or a public repository:** *Settings → Branches → Add rule*
  (or *Rules → Rulesets*) for `main`: require a pull request before merging, and
  do not allow bypass. With that, "on `main`" and "merged by a pull request"
  are the same thing, and the check above is a second lock.
- **On this repository (private, free):** branch protection is not available,
  so a direct push to `main` is possible. It is not releasable — step 3 above
  refuses it — but it is on `main`. `xtask ship` never pushes to
  `main`; if one appears there, revert it with a pull request.

**Signing and notarisation are absent, not stubbed.** They need a certificate a
release engineer controls, and none exists yet. The binaries are unsigned, and
macOS and Windows say so to whoever downloads them.

---

## 6 · Releasing

```
cargo run -p xtask -- ship 0.4.0      # cuts release/0.4.0: versioned, regenerated, gated, tested, pushed
```

Open the pull request `ship` prints, from `release/0.4.0` into `main`. When it
is green and merged:

```
git switch main && git pull origin main
git tag v0.4.0 && git push origin v0.4.0
```

— the tag goes on the merge commit, the commit the pull request put on `main`.
Or *Actions → release → Run workflow* on `main`, giving `0.4.0`.

The workflow proves, builds, installs the package on three systems, and
publishes the release with the three kits and the one `.whl` attached.

---

## 7 · Proving it works

Two checks, both cheap. Do both once.

**a. It refuses a commit that is not on `main`.** Run the workflow by hand from
`developer`. The first step of `prove` stops with *A release is run from main*,
and the run builds nothing.

**b. It says which merge decided a real release.** After the next release, open
the run's summary: under **Merged into main by** it names the pull request and
its approvals. If that section is missing, the release did not go through this
rule.
