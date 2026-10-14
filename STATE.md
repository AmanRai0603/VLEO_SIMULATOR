# STATE

> **Answer first.** Where the relay to 1.0.0 stands at the end of the last shift: what merged, what is open, what is blocked and on whom, and what the next shift starts with. Each person writes only their own half.
>
> **Kind:** reference · **For:** the next shift

Plan: `docs/PLAN_F.md` (pull request 162) and the relay page Aman keeps.
Target 1.0.0 on Monday 19 October; Tuesday to Thursday are the buffer.

## Person A · day

- Not started. S1 begins with the nine stage briefs, then F1 (keys, people, My work, `base_version`).

## Person B · night

S0 ended Thursday 15 October, 08:00. Nothing is merged yet: each PR waits on Aman.

- **Open, for review:**
  - PR 163, Explore: the today, health, trace and compare routes and the Explore tab. This is F2 without keys, so A's F2 is now the chain check on screen, then results.
  - `claude/b-setup`: CLAUDE.md, this file, the reviewer and test-runner agents, and the deny rules.
  - `claude/b-test-drive`: `xtask test-drive`, the drive every scenario starts from. It has invented "Test …" people with fixed-seed keys, three groups from `design/`, and sealed Solar 1.1 and 1.2 beside the drive.
  - `claude/b-scenarios`, stacked on test-drive: `tools/scenarios.py` and a CI job. It starts the program once per person on a copy of the drive. Three scenarios pass (opens, two-people, refused-release), and its self-test goes red on a false claim.
- **For A's F1 (decide in it):**
  - `chain::owner_of` reads group mounts from the programme's file, but `design/` mounts the groups in the systems' file. The test drive follows the library for now.
  - `chain::open_group` admits only a subsystem engineer, so the systems' file, owned by the system engineer, cannot open.
- **For A's F1 and F2 proofs:** add a scenario to `SCENARIOS` in `tools/scenarios.py`. `world.person(name)` starts one program instance; `world.release(name)` puts a sealed release on the drive.
- **Blocked on Aman:** merging PR 162, and the reviews and merges above.
- **Next for B (S2):** F3 Node, from A's Opus plan.
