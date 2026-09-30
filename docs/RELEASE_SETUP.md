# Setting up a release

> **Answer first.** Pushing a `vMAJOR.MINOR.PATCH` tag is the decision to release, and the pipeline does the rest; this page says what it checks, how to add a reviewer's approval when the account allows one, and how signing will be switched on.
>
> **Kind:** how-to · **For:** developers

---

## 1 · What a release is

`.github/workflows/release.yml` runs on a tag `vMAJOR.MINOR.PATCH` (and nothing
else: `v1`, `v0.3.0-rc` or `vtest` start no release), or by hand from
**Actions → release → Run workflow** with the version typed without its `v`.

| stage | what it does |
|---|---|
| `prove` | the version is MAJOR.MINOR.PATCH and matches `Cargo.toml`; every node version is stamped; the gate; the regeneration diff; bundle verify; the release-profile tests; the notes |
| `build` | on Linux, macOS and Windows: the kit, the desktop app, the engine for the Python package |
| `wheel`, `wheel-check` | the one-file Python package, installed and started on every system |
| `publish` | the GitHub release, with `SHA256SUMS.txt` beside the files |

Nothing is built before the tool is proven, and nothing is published before
every build and install has passed.

---

## 2 · Who decides

**By default, the person who pushes the tag.** Only somebody with write access
can push one, and by the time anything is published every mechanical check has
passed. This works on any GitHub plan.

It is the default because this repository is private on a personal account,
and GitHub's own documentation
([`github/docs@main`](https://github.com/github/docs/blob/main/content/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments.md))
says:

> Users with free plans can only configure environments for public
> repositories.

A publish step that waited for an environment's reviewer could therefore never
run here, and no release could be made at all. That is what the pipeline used
to do: its last job ended red by design and the files were uploaded by hand.

### When the account can have a reviewer

With GitHub Pro, or with the repository in an organisation on Team:

1. **Settings → Environments → New environment**, named `release`.
2. Tick **Required reviewers** and add who may release. One approval is enough.
3. **Settings → Secrets and variables → Actions → Variables**: add
   `RELEASE_ENVIRONMENT` = `release`.

The pipeline then publishes through `publish-reviewed` instead of `publish`,
which waits for a reviewer. It **refuses** to publish if the environment has no
required reviewers, because an unconfigured environment approves instantly and
leaves the same green tick as a reviewed one. Unset the variable to go back to
releasing on the tag alone.

To prove it fires: run the workflow by hand with the real version from
`Cargo.toml`, confirm it stops at **one reviewer says yes**, and **Reject**. A
made-up version fails in `prove` and proves nothing about the approval.

---

## 3 · Signing

The programs are **not signed yet**. Windows SmartScreen and macOS Gatekeeper
ask once before opening them; [FIRST_RUN.md](FIRST_RUN.md) is what to tell
people, and `SHA256SUMS.txt` lets anybody check that a download is the file
that was built.

The signing steps are written and switched off. Each runs only when its
secrets exist, so adding them is the whole of switching signing on:

| system | secrets | what then happens |
|---|---|---|
| Windows | `WINDOWS_CERT_PFX` (base64 of the .pfx), `WINDOWS_CERT_PASSWORD` | every `.exe` in the kit and the app is signed with `signtool` and timestamped |
| macOS | `APPLE_CERT_P12` (base64), `APPLE_CERT_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_TEAM_ID`, `APPLE_APP_PASSWORD` | the app is signed with the Developer ID, hardened runtime on, notarised and stapled |

Without them the macOS app is still signed *ad hoc*. That carries no identity,
but it is what lets an Apple-silicon Mac open the app after one confirmation
instead of calling it damaged.

Branch protection is not set by this pipeline. What may merge is a repository
setting; a pipeline that can widen its own gate has no gate.

---

## 4 · Releasing

```
# the version in Cargo.toml is the version being released
git tag v0.3.0 && git push origin v0.3.0
```

Then watch the run. When it is green, the release is on GitHub with its notes,
its files and their checksums.
