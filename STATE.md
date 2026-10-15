# STATE

> **Answer first.** Where the relay to 1.0.0 stands at the end of the last shift: what merged, what is open, what is blocked and on whom, and what the next shift starts with. Each person writes only their own half.
>
> **Kind:** reference · **For:** the next shift

Plan: `docs/PLAN_F.md` (pull request 162) and the relay page Aman keeps.
Target 1.0.0 on Monday 19 October; Tuesday to Thursday are the buffer.

## Person A · day

Handed over Thursday 15 October (S1, then F2 and F3-a and F3-b the same day). Nothing is merged: everything waits on Aman.

- **Open, in merge order:**
  - **162 the plan.** It now carries the nine briefs and F3's Plan (`docs/briefs/F3.md`).
  - **171 F1 keys and identity: high risk.** The Opus review ran four rounds; nothing blocks. Aman approves. The summary is in its last comment.
  - **172 F1 people and `base_version`, library only: high risk.** Stacked on 171, so it retargets to `developer` when 171 merges. Its Opus review is in its comments.
  - **174 F1 the copy and People's routes: high risk.** Stacked on 172. It adds `POST/GET /v1/copy` (`copied.toml`), `GET /v1/people`, `POST /v1/people/register|replace|revoke`, the launch's word (`session`, from `<keys>/open.html`) and the passphrase pause. Its Opus review ran three rounds; nothing blocks.
  - **175 F1 the shell and My work.** Stacked on 174. `/app`, the five workspaces, My work (`GET /v1/my-work`), keys, the anchor typed by the person, and People; proved by `tools/shell_check.py` in the panels job. Its Opus review ran three rounds; nothing blocks.
  - **176 F2 Explore: the chain checked, results kept, issues raised.** Stacked on 175, and it **includes B's 163**, merged in: 163 can close when 176 merges.
    - Today's design checks each release's seal from START HERE's anchor, and refuses what does not check.
    - It has a fingerprint of its own; two instances on one drive agree.
    - The base is read from the systems' file once that file checks.
    - Results kept name their design and engine.
    - `POST /v1/issues/raise` writes a signed `.vissue` into the copy.
    - /app's Explore holds Run, Inputs and Results kept.

    The Opus review found 12 issues; all are fixed in b5d001d5, each with a test shown red. The engine's identity moves (kernel `807ac9` → `a227fb`) because `which` lives in `vleo-modules`; no answer moves.
  - **177 F2 removals.** Stacked on 176.
    - Today's page loses the layer figure, the run view and the Inputs and Results tabs.
    - **Rows** lists every row to open its page, and links to the inputs or results open `/app#explore/<view>`.
    - The readers' folder and `xtask readers` go. The bundler moves to `xtask/src/bundle.rs`.
    - `crates/vleo-kernel-wasm` and its check go too (Aman's call, 7158a36f): nothing is compiled for the browser for the application. Lost on purpose: the check that `pmath` answers the same built for WebAssembly.

    Its Opus review's findings are fixed in 7158a36f; CI is green on it. Not run locally: `panel_check.py` and its self-test (disk); the panels job runs them.
  - **178 F3-a a revision saved in place.** Stacked on 175 (it needs nothing from F2).
    - `vleo_files::save::save` writes a node file in the copy as its next revision, by its assigned writer or their deputy, under the copy's lock.
    - A second writer is stopped by the bytes' SHA-256, with both versions answered.
    - A save changes the node's own rows and nothing else: no signature, block, mount or person rides in.
    - Routes: `GET /v1/node/<id>/file`, `POST /v1/node/<id>/save`; the scenario is `tools/save_check.py` in the panels job.

    The Opus review found 11 issues; all are fixed in 02bba1a6, each with a test shown red, except a test of the key checked at the moment of the save. CI was green on the first commit; the fixes' run is pending.
  - **179 F3-b signing the day's work: high risk.** Stacked on 178.
    - `chain::sign_node` refuses the wrong key and anyone neither assigned nor the assignee's deputy, then holds the signed file to `chain::check_node`.
    - `checks::check_node` is a node file's own checks, factored out of `check_release`.
    - `save::sign_in_place` signs the node in the copy only when it is the file opened, and refuses while its cases disagree, naming each check.
    - Route: `POST /v1/node/<id>/sign`. It refuses on the library's checks, not `/v1/parts` (that is B's 168, not under this stack).

    The Opus review found most nodes could be signed unchecked (open, archived, empty stated) and an `agent-generated` case was never refused; all fixed in 3ebd978c, each with a test shown red. Four questions for Aman are in its body.
- **What 171 decides, which B's PRs must follow:**
  - **Ownership follows the mount.** Only the programme's file (mounting `systems`) and the systems' file (mounting each `l3_*`) mount anything.
  - **A group file registers only node engineers.** It holds nobody the programme's file registers, and no key it registers. `open_group_in` refuses anything else, by name.
  - **The test drive (165) must change.** It mounts the l3 groups in the programme's file, so they read as mounted twice. It should assign the mounting blocks in the systems' file instead, and register only node engineers in group files.
  - **The anchor** is accepted once per drive, with `accept_anchor` on `POST /v1/key/open`. A scenario that opens a key passes it once, and since 174 also `session`, read from `<keys>/open.html`. A face must make the person compare the fingerprint, never fill it in from the answer.
- **What 178 and 179 decide:**
  - **The copy's lock is the library's** (`vleo_files::save::lock`). Anything that writes the copy takes it, and holds it from its first read to its rename.
  - **A node file is one block,** kept as `groups/<group_id>/nodes/<block id>.vnode`. A save or a signing refuses any other.
  - **`acts_for` holds for nobody when the holder is empty**, everywhere in the chain.
  - **Only the node engineer, or their deputy, signs a node.** The brief's scenario names Ola's deputy; that is put to Aman on 179.
- **What 176 decides:**
  - **Today's design takes releases from before 1.0 only** (`intake::Release::open` reads format 1). A release of the one schema is refused by name, and the chain's mount path waits for F5's intake.
  - **A drive that names an anchor and has no programme's file admits nothing.**
- **Deferred, on purpose:**
  - F3-c's put-back checks signatures against the drive's current files, never the copy's (the review of 174).
  - The race between `exists()` and the write when a key is made.
  - Issues raised in two copies at once can take one number until put-back exists.
  - The layer figure's dead CSS in `web/app.css` (177).
- **Not done:**
  - Run my code again (169) is still not limited to the person's own nodes. Aman decides.
  - Closing an issue (F5), and an issue in My work.
- **Lockfiles:** `crates/vleo-py` keeps its own, built `--locked` in CI, and so does `crates/vleo-kernel-wasm` until 177 removes it. A new dependency of `vleo-files` or `vleo-server` needs them refreshed, as 171 does.
- **Disk:** the container's allowance filled once on S1. Old worktrees' `target/` folders and `target/*/incremental` were deleted to make room.
- **Next for A:**
  - F3-c, putting work back on the drive (`docs/briefs/F3.md`), stacked on 179.
  - The key's proof kept in its row and checked on open (the review of 172).

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
