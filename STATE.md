# STATE

> **Answer first.** Where the relay to 1.0.0 stands at the end of the last shift: what merged, what is open, what is blocked and on whom, and what the next shift starts with. Each person writes only their own half.
>
> **Kind:** reference · **For:** the next shift

Plan: `docs/PLAN_F.md` (pull request 162) and the relay page Aman keeps.
Target 1.0.0 on Monday 19 October; Tuesday to Thursday are the buffer.

## Person A · day

Handed over Thursday 15 October (S1). Nothing is merged: everything waits on Aman.

- **Open, in merge order:**
  - **162 the plan.** It now carries the nine briefs and F3's Plan (`docs/briefs/F3.md`).
  - **171 F1 keys and identity: high risk.** The Opus review ran four rounds; nothing blocks. Aman approves. The summary is in its last comment.
  - **172 F1 people and `base_version`, library only: high risk.** Stacked on 171, so it retargets to `developer` when 171 merges. Its Opus review is in its comments.
- **What 171 decides, which B's PRs must follow:**
  - **Ownership follows the mount.** Only the programme's file (mounting `systems`) and the systems' file (mounting each `l3_*`) mount anything.
  - **A group file registers only node engineers.** It holds nobody the programme's file registers, and no key it registers. `open_group_in` refuses anything else, by name.
  - **The test drive (165) must change.** It mounts the l3 groups in the programme's file, so they read as mounted twice. It should assign the mounting blocks in the systems' file instead, and register only node engineers in group files.
  - **The anchor** is accepted once per drive, with `accept_anchor` on `POST /v1/key/open`. A scenario that opens a key passes it once. A face must make the person compare the fingerprint, never fill it in from the answer.
- **Deferred, on purpose:**
  - The per-launch token and the pause after wrong passphrases land in F3-b, before anything signs. Until then `me::with_key` must stay uncalled.
  - The race between `exists()` and the write when a key is made.
- **Not done on S1:**
  - F1's program side of people: the register, replace and revoke routes, and `copied.toml` (the drive folder and every file's SHA-256 as copied).
  - F1 PR 3, the shell and My work, and its scenarios.
  - F2.
  - Run my code again (169) is still not limited to the person's own nodes. Aman decides.
- **Next for A (S2):** F1's program side of people with `copied.toml`, then F1 PR 3, then F2 on the chain check, then F3-a.

## Person B · night

Handed over Wednesday 14 October, 23:20 IST. Nothing is merged: each PR waits on Aman. CI is green where finished.

- **Merge order:** 165 before 166; 168 and 169 before 170. PRs 163, 168 and 169 each add 3 lines to the route table in `vleo-server/src/lib.rs`; the second to merge fixes a small conflict.
- **Open:**
  - 163 Explore (F2 without keys): routes for today, health, trace and compare, and the Explore tab. A's F2 is now the chain check on screen, then results.
  - 164 setup (CLAUDE.md, this file, agents, deny rules) · 165 `xtask test-drive` · 166 `tools/scenarios.py` and its CI job, green on its first CI run.
  - 167 `name@version` reserved · 168 `/v1/parts/<id>`, six parts as the gate says · 170 the node page shows them, with the button.
  - **169 run my code again: high risk.** It runs code from a design file. Aman decides whether F1 limits it to your own nodes, or asks first.
- **For A's F1 to decide:**
  - Group ownership: `chain::owner_of` reads the programme's mounts, but `design/` mounts groups in the systems' file.
  - `chain::open_group` admits only a subsystem engineer, so the systems' file cannot open.
- **F3 moved to A, after F1:** signing a node through the chain (needs an Opus review); saving a revision in place, refused if the file moved (needs identity and the drive copy). Both should start from the six parts (168) and the test drive (165).
- **Proving F1 and F2:** add a scenario to `SCENARIOS` in `tools/scenarios.py`. `world.person(name)` starts one program; `world.release(name)` puts a sealed release on the drive.
- **Blocked on Aman:** merging PR 162 and the reviews above.
- **Next for B:** no build work planned while B's seat is over budget; B answers review comments on 163–170 if asked.
