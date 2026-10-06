# START HERE — Vleo Database

> **Answer first.** This folder is the VLEO spacecraft design, shared with every group: two pages that open its files, the design to read in a spreadsheet, and each group's own working files. A subsystem engineer opens their group in `apps/group.html`; a node engineer opens their node in `apps/node.html`. Nothing to install.
>
> **Kind:** how-to · **For:** everyone in a group, and the drive's owner

This page is the source of the Google Doc of the same name at the top of the
drive. `tools/drive.py pack` builds the six folders beside it
(docs/DRIVE_SETUP.md).

## What is here

| Folder | What it is | Whose it is |
| --- | --- | --- |
| `apps/` | `group.html` and `node.html`: download one, double-click it, and it opens in Chrome or Edge with no network | the repository's |
| `guides/` | the three role guides: user, maintainer, developer | the repository's |
| `design/design.vleo` | the whole released design as one file, for the tool and for Python | the repository's |
| `readable/` | the released design to read in any spreadsheet: `Groups.csv` (each group, and what its own checks still ask of it), `Nodes.csv` (every live node), `Interfaces.csv` (every value one group reads from another) | the repository's |
| `groups/<group>/` | the group's working files: `<group>.vgroup` (the structure), `nodes/<node>.vnode` (one per node), `releases/` (its **sealed** releases, and nothing else); `groups/READY.csv` is each group's first plan | the group's |
| `deliveries/<group>-<version>/` | a test application's record: `DELIVERY.toml`, `DELIVERY.md`, `group-test.csv`, and the subsystem engineer's answer beside them | the developer's, then the subsystem engineer's answer |

*Said simply:* `readable/` is what the design holds today; `groups/` is what
each group is working on, which can be ahead of it until the developer takes it
in.

## The loop, and who does each step

1. **Author** — open `apps/node.html`, then your `groups/<group>/nodes/<node>.vnode`.
   Fill it, sign it, save it, give it to your subsystem engineer.
2. **Lead** — open `apps/group.html`, then `groups/<group>/<group>.vgroup`.
   *Node files & release*: assemble the node files. *Sign & seal*: seal the
   release. Save the sealed file in `groups/<group>/releases/` and tell the
   developer.
3. **Developer** — takes the sealed release into the design, builds it, tests
   it against the group's own results, and adds `deliveries/<group>-<version>/`.
4. **Lead** — in `apps/group.html`, open the sealed release from `releases/`,
   then *Delivery & acceptance*, and choose `DELIVERY.toml` from the delivery's
   folder. Read `group-test.csv`: every check the build ran on your own
   results. Answer *Accepted* or *Changes*, say what you tried, *Write the
   answer*, and save `<group>-<version>.accept.toml` in the delivery's folder.
   Tell the developer.
5. **Developer** — records the answer and merges. The group's work reaches
   everyone in the next release.

## When a release comes

The drive's owner deletes `apps/`, `guides/`, `design/` and `readable/`, and
drags in the new ones from the release's zip. A new delivery is a new folder in
`deliveries/`. `groups/` is never replaced: it is the groups' own work. A new
group's folder is added beside the others.

## Where this breaks

- **Anyone with the folder's link can read every group's design.** Ask the
  owner before sharing it further.
- **A file of the same name dragged in beside another makes two.** Drive keeps
  both. Delete the old one first, or save under a new name.
- **The applications save by downloading.** Put the saved file back in its
  folder in the drive; the copy in your Downloads folder is not shared.
