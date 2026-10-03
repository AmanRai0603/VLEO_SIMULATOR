# Setting up the shared drive

> **Answer first.** The team's shared drive is written by the pipeline: on every release tag, and whenever you run it by hand, the `drive` workflow builds each group's files from the repository and mirrors them into one Drive folder. It signs in as the folder's owner, so it needs a refresh token from a one-time sign-in, kept as three repository secrets beside one variable naming the folder.
>
> **Kind:** how-to · **For:** the drive's owner, once

---

## What ends up in the drive

`tools/drive.py pack` builds this folder from the repository, and `tools/drive.py
upload` mirrors it into the Drive folder (docs/GROUP_APPS.md says how each part is
used):

```text
<the drive folder>/
  apps/group.html, apps/node.html     open a group's files; nothing to install
  guides/user.html, maintainer.html, developer.html
  design/design.vleo                  the whole design, as one file
  groups/READY.csv                    what each group's own checks still ask of it
  groups/<group>/                     one folder per group that owns a node
    <group>.vgroup                    the structure — the lead's
    nodes/<node>.vnode                one per node — each its author's
    releases/<group>-<version>.vleo
```

`groups/l3_solar/` is the solar worked example (`groups/solar`), not solar's plain
export. *Said simply:* every group starts from what the design already holds, and
solar starts from the most.

**What the upload does.** A file missing from the drive is created; a file whose
bytes changed is replaced in place, so every link to it keeps working; a file
whose bytes are the same is left alone. **It never deletes.** A file the pack no
longer writes is named in the job's log and kept.

**What it does not do.** It does not read the drive back. A lead's saved
structure or an author's filled node file is theirs; the next release that
carries the same file name replaces it. Move work in progress out of the
mirrored folders — the group application saves where you tell it — or take it
in first (docs/GROUP_APPS.md, *The developer, taking a sealed release in*).

---

## Why it signs in as you

A Google service account has no storage of its own, and a personal Drive
refuses the files one would create there. So the workflow signs in as the
folder's owner, with a refresh token from one sign-in on your own computer, and
every file it writes is yours and counts against your storage.

The token can read and write your whole Drive (the `drive` scope: a narrower
scope cannot write into a folder the tool did not create). Keep it like a
password: in the repository's secrets, never in a file. If it leaks, remove the
app's access at <https://myaccount.google.com/permissions>, which ends it at once.

---

## The one-time setup

1. **A Google Cloud project.** At <https://console.cloud.google.com/>, create a
   project (any name), and under *APIs & Services → Library* enable the
   **Google Drive API**.
2. **The consent screen.** *APIs & Services → OAuth consent screen*: user type
   **External**, an app name, your address as support and developer contact.
   Then **Publish app** (status *In production*). An app left *in testing* has
   its refresh tokens expire after seven days, and the upload would stop working
   a week later. Unverified is fine for your own account: at sign-in Google says
   so, and you continue under *Advanced*.
3. **A client.** *APIs & Services → Credentials → Create credentials → OAuth
   client ID*, type **Desktop app**. Note its client ID and client secret.
4. **Sign in once, on your computer**, from a checkout of the repository:

       python3 tools/drive.py auth --client-id <ID> --client-secret <SECRET>

   Open the address it prints, signed in as the folder's owner, and allow it.
   It prints `GDRIVE_REFRESH_TOKEN`.
5. **The repository's settings** → *Secrets and variables → Actions*:
   - secrets `GDRIVE_CLIENT_ID`, `GDRIVE_CLIENT_SECRET`, `GDRIVE_REFRESH_TOKEN`;
   - variable `GDRIVE_FOLDER_ID`: the folder's id — the last part of its link,
     `https://drive.google.com/drive/folders/<id>`.

   The folder id is a variable, not a line in the repository, because the
   repository is public and the folder is shared by link.
6. **Run it.** *Actions → drive → Run workflow*. The log ends with
   `drive: N created, N replaced in place, N unchanged, N folders made`. From
   then on every release tag runs it again.

## Where the simple version breaks

- **The first run is slow.** About 300 MB and 1,600 files go up the first time;
  after that only the files whose bytes changed.
- **Sharing is yours.** The workflow never changes who can see the folder. A
  folder shared *anyone with the link* is readable by anyone the link reaches —
  every group's design included.
- **A file of the same name is replaced.** See *What it does not do* above.
- **Without the secrets the job stops green**, with a notice naming what is
  missing: a drive not set up yet is not a broken build.
