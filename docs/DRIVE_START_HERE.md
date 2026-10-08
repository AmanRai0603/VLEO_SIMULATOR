# START HERE — Vleo Database

> **Answer first.** This folder is the VLEO spacecraft design, shared with every group: two pages that open its files, the design to read in a spreadsheet, and each group's own working files. A subsystem engineer opens their group in `apps/group.html`; a node engineer opens their node in `apps/node.html`. Nothing to install.
>
> **Kind:** how-to · **For:** everyone in a group, and the drive's owner

This page is the source of the Google Doc of the same name at the top of the
drive. `tools/drive.py pack` builds the four folders beside it
(docs/DRIVE_SETUP.md).

## What is here

| Folder | What it is | Whose it is |
| --- | --- | --- |
| `apps/` | `group.html` and `node.html`: download one, double-click it, and it opens in Chrome or Edge with no network | the repository's |
| `guides/` | the three role guides: user, maintainer, developer | the repository's |
| `readable/` | the released design to read in any spreadsheet: `Groups.csv` (each group, and what its own checks still ask of it), `Nodes.csv` (every live node), `Interfaces.csv` (every value one group reads from another) | the repository's |
| `groups/<group>/` | the group's working files: `<group>.vgroup` (the structure), `nodes/<node>.vnode` (one per node), `releases/` (its **sealed** releases, and nothing else); `groups/READY.csv` is each group's first plan | the group's |

*Said simply:* `readable/` is the design the repository released; `groups/` is
what each group is working on, and the tool builds today's design from each
group's latest sealed release in it.

## The loop, and who does each step

1. **Node engineer** — open `apps/node.html`, then your
   `groups/<group>/nodes/<node>.vnode`. Fill it, sign it, save it, give it to
   your subsystem engineer.
2. **Subsystem engineer** — open `apps/group.html`, then
   `groups/<group>/<group>.vgroup`. *Node files & release*: assemble the node
   files. *Sign & seal*: seal the release. Save the sealed file in
   `groups/<group>/releases/`.
3. **The tool** — when it opens on the drive, it builds today's design from
   every group's latest sealed release that passes its checks: its seal, its
   content, and each node against the design. A group whose latest release is
   refused is built from its last good release, and the tool says which
   release each group's part is and why one was refused. Nobody takes a
   release in by hand: fix what it names, seal again, and save the new release
   beside the old one.

## When a release comes

The drive's owner deletes `apps/`, `guides/`, `design/` and `readable/`, and
drags in the new ones from the release's zip. `groups/` is never replaced: it is the groups' own work. A new
group's folder is added beside the others.

## Where this breaks

- **Anyone with the folder's link can read every group's design.** Ask the
  owner before sharing it further.
- **A file of the same name dragged in beside another makes two.** Drive keeps
  both. Delete the old one first, or save under a new name.
- **The applications save by downloading.** Put the saved file back in its
  folder in the drive; the copy in your Downloads folder is not shared.
