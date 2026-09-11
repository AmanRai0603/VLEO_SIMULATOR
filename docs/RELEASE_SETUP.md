# Setting up the release approval

The release pipeline is finished except for one thing, and that one thing
cannot live in this repository. This page is how to add it, what it costs, and
how to tell whether it worked.

Nothing in the repository can create it, and that is the property being bought
rather than a limitation to route around. The workflow names an environment;
the setting that makes that environment mean something is a repository setting
a person configures. A pipeline able to grant itself its own approval has no
approval.

---

## 1 · What the approval actually gates

`.github/workflows/release.yml` runs in three stages.

| stage | what it does | needs approval |
|---|---|---|
| `prove` | version matches the manifest, the gate, regeneration diff, bundle verify, release-profile tests, the notes | no |
| `build` | the three binaries for Linux, macOS and Windows | no |
| `publish` | creates the GitHub release and uploads the artefacts | **yes** |

Everything that can fail mechanically has already failed or passed by the time
a person is asked. That is deliberate: the question put to the approver is not
"is the build green" — a machine answered that — but "is this the state of the
tool that goes to somebody else". Those are different questions and only the
second needs a person.

Nothing is published before approval. A release that is never approved leaves
artefacts on the workflow run and no release.

---

## 2 · Before you start: this repository is private

`AmanRai0603/VLEO_SIMULATOR` is a private repository on a personal account.
That matters, because GitHub's own documentation
([`github/docs@main`](https://github.com/github/docs/blob/main/content/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments.md))
states:

> Creation of an environment in a private repository is available to
> organizations with Team and users with Pro.

> Users with free plans can only configure environments for public
> repositories.

So before anything below will work, one of these has to be true:

1. the account has **GitHub Pro**, or
2. the repository is moved to an **organization on Team**, or
3. the repository is **public** — which is not what this one is for.

If **Settings → Environments** is missing or refuses to save a protection rule,
that is the reason, and it is a billing decision rather than a configuration
mistake. Check it first; everything below assumes it is settled.

Until it is settled the release path is simply unavailable, and the pipeline
says so rather than publishing without an approver. That is the safe failure,
not a broken one.

---

## 3 · The steps

1. **Settings → Environments → New environment.**
2. Name it exactly `release`. The workflow matches on the name; `Release` or
   `releases` creates a second, empty environment and the guard will stop the
   run.
3. Tick **Required reviewers** and add the people who may release.
   - Up to **6 people or teams**.
   - **Only one of them needs to approve** for the job to proceed. Adding six
     does not mean six approvals; it means six people who are allowed to give
     the one.
4. **Prevent users from approving workflow runs that they triggered** is the
   option that stops the person who tagged a release from also approving it.
   On a one-person project it blocks every release, so leave it off — and know
   that you have, because with it off the approval is a second look by the same
   person rather than a second person.
5. **Save protection rules.**

Two options on the same page that you do not need:

- **Wait timer** delays the job by a fixed number of minutes. It does not
  substitute for a reviewer, and the guard does not accept it as one.
- **Deployment branches and tags** restricts which refs may deploy. The
  workflow already triggers only on `v*` tags, so this adds a second copy of a
  rule that already exists. A rule kept in two places is two rules that will
  disagree.

---

## 4 · Proving it worked

Three checks, cheapest first. Do at least the first and the third.

**a. Read it back from the API.** With the `gh` CLI authenticated:

```
gh api repos/AmanRai0603/VLEO_SIMULATOR/environments/release \
  --jq '[.protection_rules[].type]'
```

You want `required_reviewers` in the list. This is the exact call the pipeline
makes, so agreement here means agreement there.

**b. Look at the environment page.** It should list the reviewers by name. An
environment with a name and no rules is the failure this whole page exists to
prevent — it approves instantly and leaves the same green tick as a reviewed
release.

**c. Run a release and watch it stop for you.** The honest test, and there is
one trap in it: `prove` asserts that the version being released equals the
`version` in `Cargo.toml`, currently `0.1.0`. A made-up throwaway tag fails
there and never reaches the approval, which proves nothing about the approval.

So use the real version. **Actions → release → Run workflow**, enter `0.1.0`
without the leading `v`, let `prove` and `build` run, and confirm the run pauses
at **one human says yes** with a **Review deployments** button. Then **Reject**.
A rejected deployment fails the workflow and publishes nothing, so this costs
one workflow run and leaves no release behind.

Do not skip (c) because (a) passed. The two answer different questions: whether
the setting exists, and whether it fires.

---

## 5 · What the pipeline does if it is not configured

The first step of `publish` reads the environment's own protection rules before
anything is downloaded or uploaded. It has three outcomes:

| what it finds | what happens |
|---|---|
| `required_reviewers` present | the run proceeds to the approval gate |
| the environment exists, nobody required | the job fails, naming the setting and this page |
| the environment cannot be read at all | the job fails, printing GitHub's own response |

The third is separated from the second on purpose. "Nobody is required" and
"this token cannot see the setting" both stop a release, and both should — but
they send you to different places, and a guard that reports the wrong one wastes
the hour before a release. If you see the third outcome with the environment
visibly configured in the UI, the workflow token could not read it; that is a
permissions problem on the token, not on your setup.

In all three failing cases nothing has been published.

---

## 6 · What the approver should read before clicking

The `publish` job writes the release notes and the full artefact list to the
run summary **before** the approval prompt, so the thing being agreed to is on
the screen at the moment of agreeing. An approval given against a page that
does not say what is in the release is a button press, not a decision.

Worth checking in that summary:

- the version matches the tag, and the tag matches `Cargo.toml` — `prove`
  already asserts this, so a disagreement here means something changed after
  the check
- the notes describe this release and not the previous one
- all three targets are present: `x86_64-unknown-linux-gnu`,
  `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`. `publish` needs `build`, so
  a target that failed to compile never reaches you at all — what this catches
  is a leg that succeeded and uploaded less than it should have
- nothing in the artefact list is unexpected

To approve: open the run, click **Review deployments**, tick `release`, leave a
comment if the decision needs one, then **Approve and deploy**. **Reject** fails
the workflow and publishes nothing.

---

## 7 · What is still absent after this

Configuring the environment closes the approval gap and no other.

**Signing and notarisation are absent, not stubbed.** They need a certificate
in an organisation secret that a release engineer controls, and no certificate
exists yet. The signing job is deliberately not written: a workflow that
pretends to sign is worse than one that admits it does not. Binaries from this
pipeline are unsigned, and macOS and Windows will say so to anybody who
downloads them.

**Branch protection is not set by this pipeline either**, and for the same
reason. What may merge is a repository setting; a pipeline that can widen its
own gate has no gate.

---

## 8 · Releasing, once this is done

```
# the version in Cargo.toml is the version being released
git tag v0.1.0 && git push origin v0.1.0
```

Or **Actions → release → Run workflow**, giving the version without the leading
`v`.

Then wait for `prove` and `build`, approve at `publish`, and the release
appears with the three binaries and the generated notes attached.
