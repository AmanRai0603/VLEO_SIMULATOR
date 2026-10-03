---
name: vleo-group-release
description: Take a group's sealed release (.vleo) through the developer's loop in this repository — unpack, intake, apply, build from the methods, test against the group's own results, deliver a test application — and report back what the group must change. Use when a developer hands over a group release file, or asks to build, test or deliver a group.
---

# A group's release, from sealed file to test application

You are helping a **developer** of this repository take one group's sealed
release into the design. The group wrote everything that matters — methods,
derivations, cases, results, code. Your part is mechanical: run the loop, read
what each step prints, and report. `AGENTS.md` is the rule book; this skill does
not loosen any of it.

## The rules that stop you

1. **Never edit the release.** Not a file in the unpacked folder, not a result,
   not a tolerance. A release changed after sealing is refused by every
   command, and that refusal is the point.
2. **Never write a method, a derivation, a case or a result.** If one is
   wrong or missing, the group changes it and seals again. Write down what
   they must change; do not change it.
3. **Never apply under your own name.** Relation stamping refuses a checkout
   whose `git config user.name` is an assistant's. Applying is the
   developer's act: if `--apply` is refused for that reason, stop and hand
   the command to the developer to run.
4. **A test that disagrees is the group's evidence speaking.** Never change
   the engine, a tolerance or a case to make `group-test` pass. A
   disagreement goes back to the group with the report.
5. **Stop at the first refusal** and report it with the lines the command
   printed. Do not work around a refusal with another command.

## The loop

Run from the repository root. `<file>` is the `.vleo` the group sealed;
`<dir>` is a scratch folder outside the repository.

    node tools/group_db.mjs --unpack <file> --out <dir>
    cargo run -p xtask -- group-intake  <dir>            # 1 · the plan
    cargo run -p xtask -- group-intake  <dir> --apply    # 2 · the developer applies
    cargo run -p xtask -- group-build   <dir>            # 3 · from the methods
    cargo run -p xtask -- group-test    <dir>            # 4 · against their results
    cargo run -p xtask -- gate && cargo test             # 5 · the whole design
    git switch -c group/<group>-<version>                # 6 · the release's branch,
    git commit …                                         #     the developer commits
    cargo run -p xtask -- group-deliver <dir>            # 7 · the test application
    cargo run -p xtask -- group-accept <answer> --delivery <DELIVERY.toml>
                                                         # 8 · the group's answer

| Step | Read it for | Stop when |
| --- | --- | --- |
| 1 · plan | every `REFUSED` line and why; `not in the design` (a new node arrives by its own form) | anything is refused |
| 2 · apply | `applied — … the tree gated` per node | refused and put back — the message says why |
| 3 · build | `build-node: <id> is built from its method` per node; `not rerun — the entry function is not named` is a note for the group, not a failure | any node fails a stage |
| 4 · test | the four sections; every `FAIL` line names the node, the inputs, the expected and the found value | any `FAIL` |
| 5 · gate | `0 node check failure(s)` and every test passing | anything fails |
| 7 · deliver | the folder it wrote; `DELIVERY.md` is what the group reads | it refuses: commit on the group branch first |
| 8 · answer | `accepted … recorded in acceptances/`; an answer of **changes** prints the group's note | always: an acceptance is the group's to give. Record only the file the lead's page wrote — never write or edit one |

At 6 the developer commits the change on the branch named for the group and
version (`group/<group>-<version>`), with a message naming the group and its
signer: a delivery is built from a commit, so it can be built again, and the
group's acceptance binds that commit. A pull request from that branch passes
the pipeline only with the acceptance recorded on it.

## What to report

To the developer, in this order:

- the release (group, version, who sealed it, the fingerprint's first
  sixteen characters);
- how far the loop got, and the step that stopped it;
- for the group, each thing **they** must change, by file and line where the
  command named one — e.g. *"nodes/orbit_period/results/isolation.csv, case 3:
  the engine gives 5431.18 s from your method; your row says 5430.0"*;
- what was not checked, and why (an author's code not rerun because no entry
  function is named; a node not in the design yet).

Never report a step as passed that you did not see pass.

## Two parts of the build, and what is generated

- **Backend** — the engine. Each method is translated by fixed rules into
  `crates/vleo-core/src/physics/methods/<node>.rs`; nothing in it is written
  by hand, and a defect in it is a translator defect, fixed in
  `crates/vleo-sheet/src/method/` with its golden files written again on
  purpose.
- **Frontend** — the pages. Every node's page is rendered from its sheet when
  it is opened; there is no page file to edit, by hand or otherwise.

If either looks wrong, the fix is in the generator or in the group's release —
never in the generated file.
