# Moving this repository to GitLab

Written while doing it, not from a specification. Every reachability claim
below was tested from the environment that would run the push.

---

## 1 · What moves, and what does not

The history is small and clean: **57 commits, two branches, no tags, no LFS**.
A mirror push moves all of it in one command and there are no migration
hazards in the git data itself.

What does not move is everything GitHub was doing *around* the git data.

| | moves on a mirror push | needs work |
|---|---|---|
| commits, branches, tags | yes | — |
| commit authorship | yes, unchanged | see §2 |
| the 12 CI jobs | no | `.gitlab-ci.yml`, written |
| CODEOWNERS enforcement | file moves, effect does not | **Premium** |
| the release approval | no | **Premium** |
| pull requests #1–#14 | no | importer only, see §3 |
| four hardcoded GitHub URLs | as text, now wrong | a decision first |

---

## 2 · Identity: the GitLab account can be anyone

Three identities are in play and none of them has to match:

1. the account used to sign in to the authoring tool,
2. the GitHub account the repository lives under today,
3. **the GitLab account that will own it.**

Only the third matters. A GitLab Personal Access Token from that account is
the whole authentication story — no linking, no shared sign-in, no
relationship to the other two.

Commit authorship travels unchanged, and this repository has three authors:

| commits | author |
|---|---|
| 39 | `Claude <noreply@anthropic.com>` |
| 14 | `AmanRai0603 <…@users.noreply.github.com>` |
| 4 | `Aman Rai <raiaman0603@gmail.com>` |

GitLab displays all three correctly but links a commit to a profile only when
the email matches an address on that account. Add the real address to the
GitLab account and those four link; the GitHub noreply address can never link
on GitLab, which is cosmetic.

**Do not rewrite history to tidy this.** It changes every commit hash, which
discards the provenance record the whole tool is built to keep.

---

## 3 · Two routes

**Mirror push.** Moves every commit, branch and tag. No issues, no merge
requests. Runs from a terminal; §5 is the procedure.

**GitLab's GitHub importer** (New project → Import project → GitHub). Moves the
repository *and* issues, merge requests, and the wiki. Runs in a browser and
needs a GitHub token. Choose this one if the fourteen pull requests are worth
keeping — a mirror push cannot bring them, ever, and re-running the import
later means starting the project again.

---

## 4 · The cost, which decides more than the tier does

Measured from a real run of the pipeline being ported — eight jobs, wall time
taken from the run itself:

| job | seconds |
|---|---|
| the profile that ships | 209 |
| the panels render, move and match | 205 |
| build, generate, gate, test | 96 |
| the faces outside the workspace | 40 |
| the tools check themselves | 19 |
| the kernel stays no_std | 18 |
| the changed nodes' tests are load-bearing | 10 |
| advisory review | 6 |
| **total** | **603 s ≈ 10 compute-minutes per push** |

GitLab Free includes **400 compute-minutes a month**. That is **forty pushes**,
and then the pipeline stops until the next month or the account pays.

The nightly audit is worse: roughly thirty-five minutes a night, which is
**1050 minutes a month — 2.6× the entire free allowance on its own**.

So the ported file is shaped around that rather than pretending otherwise:

- the nightly is **schedule-only** and has no cron in it, because GitLab
  schedules are a project setting. Nobody gets it by accident. Weekly is the
  honest compromise on Free; nightly needs paid minutes or a self-hosted runner.
- the two expensive per-push jobs — the panels and the shipping profile, 69% of
  the bill between them — are in their own stage behind a `HEAVY` variable. Set
  `HEAVY=false` in **Settings → CI/CD → Variables** and the per-push cost drops
  from 10 minutes to about 3.
- `workflow: rules` ensures one pipeline per push. Without it a branch with an
  open merge request builds twice and the bill doubles for nothing.

A self-hosted runner removes this entire section. On a machine you already own,
minutes are free and the nightly is affordable again.

---

## 5 · The mirror push

**The token.** GitLab → your avatar → **Edit profile** → **Access tokens** →
**Add new token**. Scope: `write_repository` is enough to push. Add `api` only
if the project should be created by the same command rather than in the UI.
Give it the shortest expiry that covers the work.

Keep it out of anything that persists. Do not put it in a remote URL that gets
written to `.git/config`, and do not paste it where a transcript keeps it — set
it as an environment variable instead, and revoke it when the move is done.

**Create the project** first in the UI (**New project → Create blank project**),
empty, with no README — an initialised project has a commit your history does
not contain, and the mirror push will refuse.

Then:

```
export GITLAB_TOKEN=…                    # not in a file, not in a URL
NS=your-namespace                        # user or group
PROJ=vleo-simulator

git clone --mirror https://github.com/AmanRai0603/VLEO_SIMULATOR.git vleo.git
cd vleo.git
git push --mirror "https://oauth2:${GITLAB_TOKEN}@gitlab.com/${NS}/${PROJ}.git"
```

`--mirror` on both sides is what makes this complete: every branch, every tag,
every note, exactly as it is. It is also why the target must be empty —
`--mirror` will delete refs on the destination that the source does not have.

**Then check it landed**, rather than trusting that it did:

```
git ls-remote "https://oauth2:${GITLAB_TOKEN}@gitlab.com/${NS}/${PROJ}.git" | wc -l
git rev-parse HEAD        # compare against the project's default branch in the UI
```

Finally, point a working clone at the new home:

```
git remote set-url origin https://gitlab.com/${NS}/${PROJ}.git
```

---

## 6 · After the push

1. **Set the default branch** to `main` (Settings → Repository → Branch
   defaults). A mirror push does not set it.
2. **Protect `main`** (Settings → Repository → Protected branches). Nothing is
   protected by default, so the branch rules GitHub had do not come across.
3. **Create the schedule** for the nightly audit, if the minutes are there
   (Settings → CI/CD → Schedules). Without it the audit never runs; there is
   no cron in the file.
4. **Decide about the four URLs** in `Cargo.toml` (×3), `README.md` and
   `docs/RELEASE_SETUP.md`. They are correct only while GitHub is the home.
5. **`docs/RELEASE_SETUP.md` is GitHub-specific** and needs rewriting for
   protected environments if GitLab becomes the home. The shape of the problem
   is the same; the tier is Premium instead of Pro.
6. **`CODEOWNERS` becomes inert.** The file moves and GitLab reads it, but
   code-owner approval is Premium. On Free, 1337 ownership rules stop routing
   anything, silently — nothing warns you.

---

## 7 · What is lost that no configuration restores

The authoring tool's GitHub integration — opening pull requests, watching
checks, waking on a failure and driving a branch back to green — is specific to
GitHub. On GitLab it can still commit and push with a token, but that loop goes
away.

That is worth weighing against whatever is driving the move, because it is the
part that cannot be ported.
