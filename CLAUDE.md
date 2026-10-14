# CLAUDE.md

> **Answer first.** An assistant in this repository works to `AGENTS.md`, and this file only adds how the two people building 1.0.0 run their sessions: read `STATE.md` first, take one stage brief, work in its worktree, prove it, and leave `STATE.md` true for the next shift.
>
> **Kind:** how-to · **For:** the developer's assistant sessions, by day and by night

`AGENTS.md` is the rules. Nothing here relaxes one of them: every formula in
the kernel, portable maths, a refusal never a substitution, no expected value
from the code under test, today's answers unchanged, and no assistant supplies
a relation.

## Start of every session

1. Read `STATE.md`: what merged, what is open, what is blocked, whose turn.
2. Read the brief for your stage (`docs/PLAN_F.md` names the stages) and
   nothing else you do not need. Do not read `target/debug/`,
   `target/release/`, `bundles/`, `design/` file contents, `web/group.html`
   or `web/node.html`: they are large, generated or data.
3. `git fetch origin developer` and start from it.

## One pull request

- One branch per pull request, from `developer`: `claude/a-<topic>` by day,
  `claude/b-<topic>` by night. A pull request goes into `developer` only.
- Write the test first and show it failing against a deliberately broken
  implementation, then make it pass (`AGENTS.md`, Testing).
- Before every push: `cargo run -p xtask -- gate && cargo test`. A file
  that inlines another (`web/app.css` into the group pages) is rebuilt by
  its command: `cargo run -p xtask -- group-app`.
- Recording contract examples rewrites unrelated ones with new temporary
  paths and times: keep only the examples the change meant to move.
- A separate reviewer session reads every pull request before it is asked
  for review: the `reviewer` agent in `.claude/agents/`. It runs the gate
  itself and reports; it never pushes.
- The `test-runner` agent runs the long suites and reports failures only.
- Nobody merges their own pull request. Work Aman marks high risk (keys,
  signing, the seal, writing a release, the design pointer, the dry run)
  waits for his approval.

## Two lanes, two worktrees

Each person keeps the checkout plus one worktree, so a long test run never
blocks the next change:

    git worktree add ../vleo-lane2 origin/developer    # the second lane
    git worktree remove ../vleo-lane2                  # when the shift ends

Two lanes never edit the same file in one shift. Files both people touch,
`crates/vleo-server/src/lib.rs` (the route table), `contract/routes.toml`,
`docs/manual.toml` and `web/js/app.js`, take a few lines each: the work goes in
a new file beside them.

## End of every shift

Update `STATE.md`, in your own half only, in under 40 lines: merged, open with
its pull request, blocked and on whom, and the next step. Commit it on the
branch of your last pull request, or on its own, so the next shift reads it on
`developer`.
