# STATE

> **Answer first.** Where the relay to 1.0.0 stands at the end of the last shift: what merged, what is open, what is blocked and on whom, and what the next shift starts with. Each person writes only their own half.
>
> **Kind:** reference · **For:** the next shift

Plan: `docs/PLAN_F.md` (pull request 162) and the relay page Aman keeps.
Target 1.0.0 on Monday 19 October; Tuesday to Thursday are the buffer.

## Person A · day

Handed over Saturday 17 October (S1 to S3: F1 to F3-f, the key's proof, F3-g1, the Node workspace, F4, the breakdown, F3-g2, writing the node, and F3-h, the node view's removal). Nothing is merged: everything waits on Aman.

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

    The Opus review found 10 points; nine are fixed in 9d22363e, each with a test shown red. The tenth (a proof is not bound to its file) and two more questions are for Aman in its body. CI's first run failed on `tools/files_check.mjs`, which still expected format 2; fixed in the same commit. CI is green.
  - **182 F3-d asking: high risk.** Stacked on 181.
    - A node engineer, or their deputy, asks for a contract change or a breakdown: a `request` row in the node's file, signed on its own under the scope `request:<uid>`. The node is not touched, so a node that cannot be signed yet still asks.
    - `chain::check_request` is the one rule of a request: one of its uid, named `<node>-r<n>`, of a contract or a breakdown, saying something, signed by its asker at the moment it says.
    - Put-back takes a file whose only change is the person's own signed requests, at the drive's revision. On a later revision they go back with it, signed.
    - Route: `POST /v1/node/<id>/ask`. My work lists every request to the group's owner and their deputy; one said to be answered is refused, since no answer is signed yet. The scenario is `tools/ask_check.py` in the panels job.

    The Opus review found 11 points, one blocking (two rows of one uid let an unsigned request count); all fixed in 51e21f13, each with a test shown red, except what its body names. Two questions for Aman are in its body. CI is green.
  - **183 F3-e a lesson's place in the node's file.** Stacked on 182.
    - A lesson is a `text` of kind `lesson` on the node's block, served to the loader as the `lesson.toml` its folder held, so the lesson route, the gate and the form read it unchanged. No schema change; no file in `design/` has one.
    - It is signed with the node, and checked with it: the node's own checks run the gate's lesson check but for the rows it names, which the lesson route, the plain save and the signing check against the design.
    - Route: `POST /v1/node/<id>/lesson`, a filled form or a bare `lesson.toml`, saved as the node's next revision.

    The Opus review found nothing blocking and 7 points to fix; all fixed in 60af1199, each with a test shown red. `vleo-sheet`'s `lesson.rs` is now one of the page library's sources, so both page libraries and the group pages were rebuilt. CI is green.
  - **184 F3-f the removals, part one.** Stacked on 183.
    - The single-node form is gone: its pages, the half of `vleo_sheet::template` that wrote and read it, and the new-node and known-value paths only its reader filled. The plan rules today's design holds a release's nodes to stay, with their tests rewritten against `Form` values (`a_node_s_change_is_held_to_the_plan_rules.rs`).
    - How an assistant helped is now checked by the plan rules, not the form's reader: `none`, `wording`, `relation` or `transcribed`, and an empty answer is refused.
    - **The node view stays** until `/app`'s Node workspace replaces it (PLAN_F, decision 1).

    The Opus review found nothing blocking and 6 points to fix, with 3 nits; all fixed in df052a0a but the role guides' loop picture (phase G), each new test shown red. CI is green.
  - **185 F3-g1 the Node workspace in /app.** Stacked on 184, on `claude/a-f3g-node`, which merges 177 and B's 168 to 170 into the stack (846d3d21, 4249c4fd), all checks green on the merges first.
    - `/app#node/<id>` and My work open one node: its six parts (B's drawing), its page as today's design holds it (`node.js` `drawNode`, the node view's own drawing), and its file in your copy: whose it is by `Registry::may_sign_node` with a key that counts, whether it is signed as it is now, signatures, requests, findings and one next step.
    - Sign, ask and a lesson form on screen; a second writer shown with nothing written. My work lists your nodes and puts signed work back, report first.
    - `GET /v1/node/<id>/file` and `GET /v1/my-work` gain fields, additive. Proved by `tools/node_check.py` (panels job) and a program test.

    The Opus review found one blocking point (an empty assignment made a node everyone's: the screen had re-derived who may write it) and eight more; all fixed in a088a3ce, each shown red but four named untested. CI's panels job then failed on `manual_check.py`, which B's 170 left numbering a row's page as before; fixed there too. CI is green on a088a3ce.
  - **186 F4-a the breakdown, the library: high risk.** Stacked on 185, on `claude/a-f4-breakdown`.
    - `vleo_files::breakdown`: add, fill, break down, move, rename, split, fold back and remove, each writing the group file and nothing else, by the branch's owner or their deputy (`chain::may_frame`, which People now uses too), each with a why in its `change` row, signed afresh.
    - A node is broken down only into children that reproduce its own cases: its signed cases, from a file its checks pass, run through its children in the method language, each case judged by `judge_outcome`.
    - `edit_in_place` writes it in the copy under the lock; a second writer is stopped by the digest.

    The Opus review found two blocking points (an edit re-signed a group file changed by hand; a node's cases were taken unsigned and unchecked) and eleven more; all fixed in 0ed6a202, each with a test shown red but the paths named untested. Nine questions for Aman are in its body. CI is green on the fixes; c2c34627, from 187's review, makes the test's invented child say it is invented.
  - **187 F4-b the branch in the System workspace: high risk.** Stacked on 186, on `claude/a-f4b-system`.
    - `GET /v1/branch?group=` and `POST /v1/branch/<id>`, every rule the library's.
    - `/app`'s System workspace: the tree, the block chosen with its cases beside the estimate, and the edit forms for the owner or their deputy only.
    - Proved by a program test and `tools/branch_check.py` (panels job, ports 7869 and 7870).

    The Opus review found two blocking points (which file mounts a group was decided on the screen's side, not by `chain::mounted_by`; node files that could not be read were left out of the tree) and six more; all fixed in 1bc260c7 to a4bf3329, each with a test shown red but two named untested. CI is green.
  - **188 F3-g2a writing the node, checked as it is typed: high risk.** Stacked on 187, on `claude/a-f3g2-edit` (it reads F4-a's split of the method's checks).
    - `vleo_files::edit`: the question, the explanation's three texts, the method and the cases of a release-shaped node file, each refused by name for rules 4 and 6, a decision with no record of which belief broke (a row of the node's own `versions.csv`), an edit that changes nothing, and a node of `design/`'s shape until F3-g0. Saved through `save::save`, split into `would_save` and the write; `try_edit` is the dry run.
    - `POST /v1/node/<id>/edit` with `dry_run`; the file route says the node's content. The Node workspace writes it, checked a moment after typing stops, never redrawn under the caret.
    - Proved by a library test, a program test and `tools/edit_check.py` (panels job, port 7873).

    The Opus review found two blocking points (the screen saved every field it first read over whatever revision was open, so a second writer's text was reverted; one section's text could begin, hide or double another) and seven more; all fixed in 6984cf1f, each with a test shown red but four paths named untested. Questions 16 and 17 for Aman are in its body. CI is green on the fixes.
  - **189 F3-g2b the page of the copy's revision: high risk.** Stacked on 188, on `claude/a-f3g2b-page`.
    - `vleo_files::intake::copy_form`: a node file of the copy laid out as a release's folder (`copy_release`) and taken by `node_form`, today's design's own intake, and nothing else.
    - `GET /v1/node/<id>/live`: planned as today's design plans a release, laid over it for that request only, drawn by the page's own fragment, said so, with what it changes, what is withheld and why, and what a release does not bring in. Nothing of today's design is written or run.
    - The Node workspace's centre has two pages, today's and the copy's revision's, the sheet alone on the second.
    - Proved by a library test, a program test, `tools/edit_check.py` and `tools/node_check.py`. Questions 18 and 19 for Aman are in its body.

    The Opus review found two blocking points (a plan that withheld one change still drew the others, though today's design refuses such a release whole; the copy was never held to the node's own checks) and four more; all fixed in eedf56c6, each with a test shown red but the screen's stale-answer guard named untested. CI is pending on the fixes.
  - **190 F3-h1 the panels drawn in `/app`: low risk.** Stacked on 189, on `claude/a-f3h1-panels`. `/app` has `window.openNode`; the Node workspace may grow to 118rem and its page stops at the node view's 74rem, so at `APP_WIDTH` (1920px) a figure is the size its references were recorded at, and none is recorded again (Aman's choice). Every panel names `page = "/app"`. 11 panels, 0 findings; the self-test 14 cases. The Opus review found nothing blocking; its four fixes are in a1010ff3.
  - **191 F3-h2 the case, the depth and a what-if in the Node workspace: low risk.** Stacked on 190, on `claude/a-f3h2-workspace`. `casebar.js` draws the case and the what-if bar on both pages; the depth switch and the banners in `/app`; a node's links to a group, the inputs and a kept result followed. Proved by `tools/workspace_check.py` (a CI step). The Opus review found nothing blocking; its three fixes (the what-if bar's words escaped, a group opened in its own tab, Explore asked for its inputs) are in 38ff29dd.
  - **192 F3-h3 today's tool page gives its node view to the workspace: low risk.** Stacked on 191, on `claude/a-f3h3-removal`. A row's link on `/`, and `#row/<id>` (in its own entry's place), open `/app#node/<id>`. `/`'s node view goes, with its markup, `window.openNode`, its status line, its parts heading, its styles and `/`'s depth switch. `input_check`, `mock_check` and `manual_check`'s node steps drive the workspace, and the manual's browser sections say where a row's page is. The Opus review found nothing blocking; its four fixes are in c40506a6, each re-pointed check shown red. CI was green on the first commit; the fixes' run is pending.
  - 190 to 192 are all built on the brief's proposed answer to question 7.
  - **195 F6-a the frame and Start group: high risk.** Stacked on 192, on `claude/a-f6a-frame`, built on the brief's proposed answers to questions 20 to 32.
    - `vleo_files::frame`: mount a group on a leaf nobody writes, in the systems' file only, by the system engineer or their deputy, a subsystem's leaf too (W13), whose owner is then the valve above it; allocate on a mount, with its closure's sense, never defaulted.
    - `vleo_files::start`: the owner the mount names, or their deputy, starts `groups/<g>/<g>.vgroup`, one heading, signed, opened through the chain, in an empty folder.
    - A mount is read across files: the branch that holds the leaf does not change it, and put-back does not take its node file. Fill refuses `allocated`.
    - `POST /v1/branch/mount|allocate`, `POST /v1/group/start`; `GET /v1/branch` says the frame, or why it cannot; My work says *Start <g>*. The System workspace draws it; `tools/start_group_check.py` in the panels job.

    The Opus review found two blocking points (a mount on a systems' issued node let its owner sign that file; My work's new kinds were not in the contract) and three more; all fixed before the push, each shown red but the guard that draws only the latest reading of a branch. CI is pending.
- **What 171 decides, which B's PRs must follow:**
  - **Ownership follows the mount.** Only the programme's file (mounting `systems`) and the systems' file (mounting each `l3_*`) mount anything.
  - **A group file registers only node engineers.** It holds nobody the programme's file registers, and no key it registers. `open_group_in` refuses anything else, by name.
  - **The test drive (165) must change.** It mounts the l3 groups in the programme's file, so they read as mounted twice. It should assign the mounting blocks in the systems' file instead, and register only node engineers in group files.
  - **The anchor** is accepted once per drive, with `accept_anchor` on `POST /v1/key/open`. A scenario that opens a key passes it once, and since 174 also `session`, read from `<keys>/open.html`. A face must make the person compare the fingerprint, never fill it in from the answer.
- **What 188 decides:**
  - **A page never writes a node file; it sends what was typed.** The library applies it (`vleo_files::edit`) and saves it as a save is; a dry run answers what a save would do, by the same checks.
  - **The record of which belief broke lives in the node's own `versions.csv`** until Aman says otherwise (question 12); which edits need one is `vleo_sheet::derisk::about_of`.
  - **`checks::ORIGINS` and `checks::NOT_A_REFERENCE` are the one list of each.**
  - **For B:** the Node workspace's right side gains *write it* (`.nw-live`, `data-edit` fields); a scenario that counts its parts must allow for it.
- **What 195 decides:**
  - **Every mount is in the systems' file**, W13's too; who owns a group is still read from the file that mounts it.
  - **A group's file is written first by its owner**, never by the system engineer: one writer signs it.
  - **Whether a group is to be started is `start::to_start`**: no file, and a folder absent or empty. Anything else is the refusal it was.
  - **For B:** the small drive's `sys_structure` gains an output, `structure_mass`. My work says *Start l3_struct* and *Start l3_prop* in place of *refused*, and `shell_check` reads them so. A scenario that counts My work's refusals must allow for them. F5's group-file put-back is how a started group reaches the drive (question 26).
- **What 189 decides:**
  - **A copy's revision is drawn only as today's design would take it**: the same intake as a release, the node's own checks first, and nothing drawn when anything is withheld. So its question and its explanation's texts are not on its page until intake takes them (question 18).
  - **For B:** the Node workspace's centre names its page on a button (`.nw-view.sel`), not in its heading; `drawNode` takes `opts.fragment` and then draws the sheet alone.
- **What 186 and 187 decide:**
  - **A breakdown is the group file's**: one wire into each output of the node from the child that answers it. The node's own file is not touched until F5 issues it again.
  - **A node's cases are evidence only once its node engineer has signed them as they are**, and its file's checks pass. A breakdown into children before that is refused, said why. So every breakdown waits for children F5 issues (question 6 on 186).
  - **A group file is framed only when it opens through the chain as it is.** Anything that signs a group file afresh opens it first.
  - **Every file of the copy is read for an edit**, releases among them, and one that cannot be read refuses it: a reader is never missed.
  - **The page library's sources are every crate compiled into it**, the kernel among them: a kernel change makes `web/files.wasm.gz` stale, and `xtask files-wasm` must run after `cargo fmt`, not before.
  - **A branch is read by one reader**, `breakdown::group_file_in` and `nodes_in`, for what is shown and for what is edited; which file mounts a group is always `chain::mounted_by`.
  - **For B:** the small drive gains two unassigned children of Pia's band (`sw_activity_count`, and `sw_activity_count_late`, wrong at 135 sfu on purpose). A scenario that counts solar's nodes must allow for them.
- **What 185 decides:**
  - **Who may write a node is asked of the library, never worked out again on a screen** (`Registry::may_sign_node`, with `Registry::key`). The review found a second copy of the rule had lost its guard.
  - **A page never reads a node file's bytes.** What a screen shows of a file, the program says as JSON; writing a node from a page (F3-g2) goes through the library in the program.
  - **No node of `design/` can be signed yet**: `check_node` wants a release's shape. Which shape is the one is the brief's question 9 for Aman, before F3-g0.
  - **For B:** `tools/manual_check.py` reads a row's page by the manual's step numbers; a step added to the manual moves them.
- **What 184 decides:**
  - **A change says how an assistant helped, or nothing is planned.** Anything that builds a `template::Form` (F4's breakdown, F5's intake) sets `ai` from the node's declaration, as `intake::declared_help` does.
  - **A transcription taken from a release loses its record when its method changes**: the plan names the release's author and clears what it was copied from. Older than 184; put to Aman on 184, for a pull request of its own.
  - **Nothing adds a node from a change.** A new node is its group's, in F4.
- **What 183 decides:**
  - **A lesson is its node's, signed with it**: a lesson changed after signing needs the node signed again. Put to Aman on 183.
  - **A release does not carry a lesson into today's design yet**: today's design takes a release's sheets only, until F5's intake.
- **What 182 decides:**
  - **A request's name is its node's**, `<node>-r<n>`, and its status and answer are signed by nobody yet: F4, answering, signs an answer of its own.
  - **Only the node's writer or their deputy asks in a node file**; anyone else asks through an issue (W11), until Aman decides otherwise.
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
- **Disk:** the container's allowance filled on S1 and again on S2. `xtask files-wasm` and the scenarios leave a `target/` of 10 GB or more inside the worktree they run in; delete it after a push. Old worktrees' `target/` folders and `target/*/incremental` were deleted to make room.
- **Next for A, in order:**
  - **F6-b** (the integration checks and *Release design*, high risk) and **F6-c**, building on B's F5 and on 195.
  - **For Aman:** the brief's questions 9 (one node shape, F3-g0) and 10 (ports), 7 and 8 (where today's page's Rows, Architecture, Forms and manual go), 11 to 19 (F3-g2), and 20 to 32 (F6-a).
  - **F8 and F9** after F6. **For B, now:** F5, on the critical path of F6, F7 and F9; what it needs (F1's owner, F3's signed nodes, F4's tree) is in 175, 179 and 187.

## Person B · night

Handed over Sunday 18 October (S4: F5). Nothing is merged: each waits on A's review and Aman. Every PR was green on `gate && cargo test --no-fail-fast` before its push. 163 is in 176, and 168 to 170 are in 185.

- **Two F5 stacks, both on A's 187:**
  - **Release line:** 193 F5-a the seal → 194 F5-b1 a release put back → 196 F5-b2 today's design takes it (W5 proved end to end) → 200 F5-d today's view and impact → 205 F5-g1 seal on screen. 202 F5-e2 answers (W7) is on 194. **High risk:** 193, 194, 196, 202, 205.
  - **Edit line:** 197 F5-c1 assign, wire → 198 c2 issue a node file → 199 c3 the sheet → 201 e1 contract change (W4) → 203 e3 update base (W12) → 204 f1 issues assigned, resolved (W11).
  - Each line conflicts with the other only in `branch.rs`, `lib.rs`'s route table, `workspace_system.js`, `manual.toml` and `files.wasm.stamp`: small, additive. Regenerate the stamp with `xtask files-wasm`.
- **Changed in A's code, for A to check:**
  - `chain::checked_release` (193): a block no assignment names is a heading, vouched for by the seal.
  - `chain::check_answer` (202) is new, beside `check_design`.
  - `release::node_in` (200) keeps the text kinds a node does not carry.
- **What F5 decides:**
  - **A seal makes every check today's design makes** (`check_release`), the group's `versions.csv` row for the version among them.
  - **A release and an answer go back as new files only.** Each is checked through the drive's chain as it is then, and never written over.
  - **A contract change** re-issues the node's file, and the node reads behind until it is re-signed. **Issue** moves a framed block's rows into its own node file. **The sheet** sets up many blocks, all or none.
  - **An issue's state is computed, never written.** It is assigned in the group file, and resolved by the release whose `versions.csv` row names it under `resolves`.
- **For Aman, found while building:**
  - **No node of `design/` can be signed.** The converted files have no declaration, no isolation results, inputs without defaults, and unit `One` where the method language reads `1`. Their method is a `method` text where the checks read `pseudocode` (same body). (The question 9 A names.)
  - Group files still do not go back to the drive. So a contract change, an assignment or an update base stays in the copy until a release.
- **Not done:**
  - **F5-f2:** `GET /v1/issues`, the state from today's design and the health map. It needs both lines merged.
  - **F5-g2:** remove `web/group.html`, `web/node.html` and `xtask group-app`. 37 files, `AGENTS.md` among them; it overlaps A's 192. One person, after 192.
  - The test drive (165) per 171 and 181: assign mounts in the systems' file, and give keys their proofs.
  - Neighbours' values on update base; My work naming moved groups (needs F6's previews); the health map of the view.
- **Next for B:** F5-f2 once the lines merge; the 165 fix; review comments on 193 to 205.
