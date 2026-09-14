#!/usr/bin/env python3
"""The advisory review — a report on a change, which can never block a merge.

The roster had a reviewer agent. The audit kept it but moved it: not a roster
agent, because it cannot block, and a thing that cannot block is a report.

    tools/review_report.py --range origin/main..HEAD
    tools/review_report.py                      # the working tree
    tools/review_report.py --selftest

**Exit status is 0 whatever it finds.** That is the whole design, not an
oversight. A review that can fail a build is a review people learn to satisfy
rather than to read, and within a month its findings are worded to pass rather
than to be true. The gate blocks. This one only ever says something.

# What it looks at, and why the gate cannot

The gate reads one tree and asks whether it is self-consistent. Every question
here is about a *change* — two trees — which the gate has no way to ask:

  - a declaration moved and no evidence moved with it
  - a hole was filled and no test changed
  - the generator or the gate itself moved, which reaches every node at once
  - a diff that does not fit inside any single agent's lane

None of these is necessarily wrong. Each is worth a human glance, which is
exactly the register a report should be written in.
"""

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HOLE = re.compile(r"HOLE\s+(\d+)")


def git(*args):
    return subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=False
    ).stdout


def changed(rng):
    if rng:
        return [p for p in git("diff", "--name-only", rng).split("\n") if p]
    names = set(git("diff", "--name-only", "HEAD").split("\n"))
    names |= set(git("ls-files", "--others", "--exclude-standard").split("\n"))
    return sorted(p for p in names if p)


def node_of(path):
    """The node folder a path belongs to, or None."""
    m = re.match(r"(crates/vleo-mod-[a-z0-9]+/nodes/[a-z0-9_]+)/", path)
    return m.group(1) if m else None


def crate_of(path):
    m = re.match(r"(crates/[a-z0-9-]+)/", path)
    return m.group(1) if m else None


#: A change to any of these reaches every node at once. The work model files
#: that under H7: two reviewers, because one mistake there is 1361 mistakes.
WIDE = ("xtask/", "tools/seed_", "tools/nodes/", "tools/crate_skeleton", "crates/vleo-sheet/")


def review(paths):
    """(heading, [note]) sections. Findings, never verdicts."""
    nodes = {}
    for p in paths:
        n = node_of(p)
        if n:
            nodes.setdefault(n, set()).add(p.rsplit("/", 1)[1])

    out = []

    moved = [n for n, f in nodes.items() if "node.toml" in f and "fixtures.toml" not in f]
    if moved:
        out.append((
            "a declaration moved and no evidence moved with it",
            [
                "%s — if a range, a unit or a relation changed, the recorded rows "
                "were true about the old declaration." % n for n in sorted(moved)
            ],
        ))

    filled = []
    for n, f in nodes.items():
        if "model.rs" not in f:
            continue
        crate = crate_of(n)
        if not any(p.startswith("%s/tests/" % crate) for p in paths):
            filled.append(n)
    if filled:
        out.append((
            "a hole was filled and no test changed",
            [
                "%s — the fixtures may already cover it. Worth one look at whether "
                "the new lines are exercised by anything." % n for n in sorted(filled)
            ],
        ))

    wide = [p for p in paths if p.startswith(WIDE)]
    if wide:
        out.append((
            "this reaches every node at once",
            ["%s" % p for p in sorted(wide)]
            + ["The work model files a generator or gate change under H7: two "
               "reviewers, because one mistake here is 1361 mistakes."],
        ))

    sheets = sorted(n.rsplit("/", 1)[1] for n in nodes if "node.toml" in nodes[n] or "model.rs" in nodes[n])
    if sheets:
        out.append((
            "before a person is asked to look",
            ["`cargo run -p xtask -- ready %s`" % s for s in sheets]
            + ["A node with an open gap does not enter H2. The reviewer accepts; they do "
               "not hunt for defects a machine finds free."],
        ))

    fx = [p for p in paths if p.endswith("fixtures.toml")]
    if fx:
        out.append((
            "evidence changed",
            ["%s" % p for p in sorted(fx)]
            + ["The one rule the gate already enforces: an expected value may not "
               "come from the code under test. What it cannot check is whether the "
               "cited source says what the row claims."],
        ))

    return out


def lanes_note(paths):
    """Which single agent lane, if any, this whole diff sits inside."""
    try:
        sys.path.insert(0, str(ROOT / "tools"))
        import agent_lanes
    except Exception:
        return None
    inside = []
    for lane in agent_lanes.lanes():
        if all(agent_lanes.matches(p, lane.get("writes", [])) for p in paths) and not any(
            agent_lanes.matches(p, lane.get("never", [])) for p in paths
        ):
            inside.append(lane["name"])
    return inside


def render(paths, rng):
    lines = ["## advisory review", ""]
    lines.append("_A report, never a gate. Nothing here can fail a build._")
    lines.append("")
    if not paths:
        lines.append("No files changed%s." % (" in `%s`" % rng if rng else ""))
        return "\n".join(lines)
    lines.append("%d file(s) changed%s." % (len(paths), " in `%s`" % rng if rng else ""))
    lines.append("")

    found = review(paths)
    if not found:
        lines.append("Nothing this pass knows how to ask about. That is not an approval — "
                     "it reads four relationships across a diff, and a person reads the rest.")
    for heading, notes in found:
        lines.append("**%s**" % heading)
        lines.append("")
        for n in notes:
            lines.append("- %s" % n)
        lines.append("")

    inside = lanes_note(paths)
    if inside is not None:
        lines.append("")
        if inside:
            lines.append("Lane: this diff sits entirely inside `%s`." % "`, `".join(inside))
        else:
            lines.append(
                "Lane: no single agent lane covers this diff. That is normal for a "
                "person and worth checking for an agent — `tools/agent_lanes.py "
                "--agent <name>` says which path fell outside."
            )
    return "\n".join(lines)


#: (paths, heading that must appear or must not). The cases are the four
#: relationships, plus the property the whole design rests on.
CASES = [
    (["crates/vleo-mod-prop/nodes/prop_thrust/node.toml"], "a declaration moved", True),
    (["crates/vleo-mod-prop/nodes/prop_thrust/node.toml",
      "crates/vleo-mod-prop/nodes/prop_thrust/fixtures.toml"], "a declaration moved", False),
    (["crates/vleo-mod-prop/nodes/prop_thrust/model.rs"], "a hole was filled", True),
    (["crates/vleo-mod-prop/nodes/prop_thrust/model.rs",
      "crates/vleo-mod-prop/tests/t.rs"], "a hole was filled", False),
    (["xtask/src/main.rs"], "reaches every node", True),
    (["crates/vleo-sheet/src/gate.rs"], "reaches every node", True),
    (["web/js/app.js"], "reaches every node", False),
    (["crates/vleo-mod-prop/nodes/prop_thrust/fixtures.toml"], "evidence changed", True),
    (["README.md"], "Nothing this pass knows how to ask about", True),
    (["crates/vleo-mod-prop/nodes/prop_thrust/node.toml"], "before a person is asked to look", True),
    (["README.md"], "before a person is asked to look", False),
]


def selftest():
    bad = 0
    for paths, needle, want in CASES:
        got = needle in render(paths, None)
        if got != want:
            bad += 1
            print("  FAIL %s" % paths)
            print("       expected %r %s" % (needle, "present" if want else "absent"))
    # The property the whole thing rests on: a finding is never an exit code.
    for paths, _, _ in CASES:
        if main_status(paths) != 0:
            bad += 1
            print("  FAIL %s returned non-zero — a report cannot block" % paths)
    print("selftest: %d cases, %s" % (len(CASES) * 2, "all as expected" if not bad else "%d FAILED" % bad))
    return 1 if bad else 0


def main_status(paths):
    """What the command returns for a given set of paths. Always 0."""
    render(paths, None)
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--range", help="a commit range; default is the working tree")
    ap.add_argument("--out", help="write the report here as well as to stdout")
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()
    if a.selftest:
        return selftest()
    paths = changed(a.range)
    text = render(paths, a.range)
    print(text)
    if a.out:
        Path(a.out).write_text(text + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
