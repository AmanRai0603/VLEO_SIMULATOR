# STATE

> **Answer first.** Where the relay to 1.0.0 stands at the end of the last shift: what merged, what is open, what is blocked and on whom, and what the next shift starts with. Each person writes only their own half.
>
> **Kind:** reference · **For:** the next shift

Plan: `docs/PLAN_F.md` (pull request 162) and the relay page Aman keeps.
Target 1.0.0 on Monday 19 October; Tuesday to Thursday are the buffer.

## Person A · day

Handed over Friday 16 October (S1 and S2: F1 to F3-c, and the key's proof). Nothing is merged: everything waits on Aman.

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

    The Opus review found most nodes could be signed unchecked (open, archived, empty stated) and an `agent-generated` case was never refused; all fixed in 3ebd978c, each with a test shown red. Four questions for Aman are in its body. CI is green.
  - **180 F3-c putting work back on the drive: high risk.** Stacked on 179.
    - `put_back::put_back` reports every file changed in the copy. **Only a node's file** goes back, and only over the drive's own file for the same node, and only when the drive's file is still as `copied.toml` recorded it.
    - The file must change only the node's own rows. It must keep the drive's signatures and add only the person's. It must not be older than the drive's revision, and at the same revision it may differ only by that signature.
    - It must carry the person's signature, counting through the drive's chain, made with a key that still counts at the moment it goes back.
    - It is written only on a second request naming exactly the report's files with their digests, under the drive's lock (`.vleo-put-back.lock`), with the chain read again under it.
    - Route: `POST /v1/put-back`. The scenario is `tools/put_back_check.py` in the panels job: two instances, one puts back, the other copies again and finds it signed.

    Two Opus review rounds, all fixed (1cb9cc87, 48d1cabc), each with a test shown red. Four questions for Aman are in its body. CI is green.
  - **181 the key's proof kept in its row: high risk.** Stacked on 180.
    - `person_key` gains `proof`. `Registry::key` refuses a key whose proof does not check, so it signs nothing anywhere the chain counts.
    - **FORMAT is 3.** A format-2 file is read as format 3 with every proof empty, and **a copy is kept** when it is written again. This reverses Aman's "no copy": the review found that a whole file's signatures go stale when it is read, because its digest covers the format and the columns.
    - `design/`'s 1424 files are converted to format 3; no answer moves. `request.kind` gains `breakdown`, for F3-d, so `design/` is converted once.
    - The pre-1.0 anchor counts only a programme manager's key that counts. `GET /v1/people` says `proof_holds` for each key.

    The Opus review found 10 points; nine are fixed in 9d22363e, each with a test shown red. The tenth (a proof is not bound to its file) and two more questions are for Aman in its body. CI's first run failed on `tools/files_check.mjs`, which still expected format 2; fixed in the same commit. CI on it is pending.
- **What 171 decides, which B's PRs must follow:**
  - **Ownership follows the mount.** Only the programme's file (mounting `systems`) and the systems' file (mounting each `l3_*`) mount anything.
  - **A group file registers only node engineers.** It holds nobody the programme's file registers, and no key it registers. `open_group_in` refuses anything else, by name.
  - **The test drive (165) must change.** It mounts the l3 groups in the programme's file, so they read as mounted twice. It should assign the mounting blocks in the systems' file instead, and register only node engineers in group files.
  - **The anchor** is accepted once per drive, with `accept_anchor` on `POST /v1/key/open`. A scenario that opens a key passes it once, and since 174 also `session`, read from `<keys>/open.html`. A face must make the person compare the fingerprint, never fill it in from the answer.
- **What 181 decides:**
  - **Every file is format 3.** A format-2 file is read with every key's proof empty, so nothing in it counts, and it is kept beside the file when written again.
  - **For B:** every test drive and key row B writes, 165's among them, stores a proof (`people::proof`). A drive written as format 2 is made again from the programme's file down.
  - **A key without its proof is replaced by a new key.** No route adds a proof to a key already registered; the program tells its holder so.
- **What 180 decides:**
  - **A group's file does not go back yet.** What a valve may change in it is not checked against the drive's file. So People's changes (F1) stay in the copy, and their messages say so. For Aman: a PR of its own in F3, or F5.
  - **The one walk of a folder is `vleo_files::put_back::digests`**, which the copy, its state and a put-back share.
    - It refuses links and keys.
    - It says a `.vleo-incoming-…` file, which is a put-back in progress or cut short.
    - It leaves out only the record, the drive's lock and the record being written, at the top.
  - **The chain never opens a group's file through a link** (`me::registries`).
  - **The small drive** (`examples/a_small_drive.rs`) gains Test Pia (key seed 68), assigned solar's `sw_activity_band`, a node whose checks hold. A scenario that counts solar's people or nodes must allow for her.
  - **Today's view shows a node put back only once its group seals it (F5)**: today's design reads releases, not loose node files.
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
- **Next for A, in order:**
  - **F3-d, asking** (the brief's item 4): a contract change or a breakdown, written as a request in the node file and signed on its own, so a node that cannot be signed yet still asks. Planned in `docs/briefs/F3.md` (b1e476e7, on 162), with two questions for Aman. Being built on `claude/a-f3d-ask`, stacked on 181; high risk, since it adds a scope to the chain and a path to put-back.
  - **F3-e** (item 5, a lesson's place in the node's file). Not planned yet; it starts with its plan.

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
