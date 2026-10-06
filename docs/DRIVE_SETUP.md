# Setting up the shared drive

> **Answer first.** The shared drive is written from the repository. By hand: `tools/drive.py pack --zip` builds the whole folder as one zip, and its owner unzips it and drags the folder into Drive — no secrets, no sign-in. By the pipeline: on every release tag the `drive` workflow builds the same folder and mirrors it into Drive, signed in as the folder's owner with a refresh token from a one-time sign-in, kept as three repository secrets beside one variable naming the folder.
>
> **Kind:** how-to · **For:** the drive's owner, and the developer who builds it

---

## By hand: one zip, no sign-in

The first time, the developer builds the whole folder, with every group's
sealed release and test application they hold:

    python3 tools/drive.py pack --out "target/VLEO Drive" --zip \
        --sealed <group>-<version>.vleo … --delivery <folder group-deliver wrote> …

The drive's owner unzips `target/VLEO Drive.zip` and drags its six folders —
`apps/`, `guides/`, `design/`, `readable/`, `groups/`, `deliveries/` — into the
drive's folder, beside its START HERE page (the Google Doc made from
[`DRIVE_START_HERE.md`](DRIVE_START_HERE.md)).

**On a release**, `pack --update --zip` leaves `groups/` out. The owner deletes
`apps/`, `guides/`, `design/` and `readable/` from the drive and drags in the
new ones; a new delivery's folder goes into `deliveries/`. `groups/` is never
replaced, because once it is in the drive it is the groups' own work.

*Said simply:* the pipeline below is the first-time zip, mirrored for you on
every release. Until the secrets exist, this is how the drive is filled.

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
  readable/Groups.csv, Nodes.csv, Interfaces.csv
                                      the released design, for any spreadsheet
  groups/<group>/                     one folder per group that owns a node
    <group>.vgroup                    the structure — the subsystem engineer's
    nodes/<node>.vnode                one per node — each its node engineer's
    releases/<group>-<version>.vleo   SEALED releases only (--sealed)
  deliveries/<group>-<version>/       a test application's record (--delivery):
                                      DELIVERY.toml, DELIVERY.md, group-test.csv
```

`releases/` holds sealed releases and nothing else. The export assembles an
unsealed release for every group; the pack leaves it out, because a file of
that name reads as the group's release and would sit beside — or, mirrored,
replace — the one the subsystem engineer sealed. A delivery is refused unless the folder also
holds the sealed release it was built from: that is what the subsystem engineer accepts it
against.

`groups/l3_solar/` is the solar worked example (`groups/solar`), not solar's plain
export. *Said simply:* every group starts from what the design already holds, and
solar starts from the most.

**What the upload does.** A file missing from the drive is created; a file whose
bytes changed is replaced in place, so every link to it keeps working; a file
whose bytes are the same is left alone. **It never deletes.** A file the pack no
longer writes is named in the job's log and kept.

**What it does not do.** It does not read the drive back. A subsystem engineer's saved
structure or a node engineer's filled node file is theirs; the next release that
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
