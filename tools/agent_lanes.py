#!/usr/bin/env python3
"""Check what an agent changed against the lane it is allowed to change.

The roster gives every agent a "what it may not do". A sentence in a prompt is
not a control: the model that read it is the same model deciding whether this
case is the exception. A lane is a fact about a diff, so it holds whether the
agent understood the instruction, ignored it, or never saw it.

    tools/agent_lanes.py --agent hole-filler              # the working tree
    tools/agent_lanes.py --agent systems-backend --since HEAD~1
    tools/agent_lanes.py --list

Exit status is 0 when every changed path is inside the lane and 1 when any is
outside, so it can sit in a hook or a pipeline.

The hole filler used to be the interesting one: its lane included `model.rs`,
so a path check alone would have passed a rewrite of the whole file. It writes
nothing now — it returns hole bodies as text and `cargo xtask fill` splices
them — so its lane is simply closed.

The diff check that guarded it is still worth having, and is now separate from
any agent:

    tools/agent_lanes.py --holes-only HEAD~1

It reads every `model.rs` change in a range and requires each changed line to
fall inside a numbered `HOLE` block, whoever made it. A person editing the
generated region is the same defect as an agent doing it, and the regeneration
diff catches it a step later and less legibly.
"""

import argparse
import fnmatch
import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def lanes():
    return tomllib.loads((ROOT / "agents" / "lanes.toml").read_text())["agent"]


def changed(since):
    """Paths that changed, and the diff text, for the working tree or a range."""
    if since:
        names = ["git", "diff", "--name-only", since]
        text = ["git", "diff", since]
    else:
        names = ["git", "status", "--porcelain"]
        text = ["git", "diff", "HEAD"]
    out = subprocess.run(names, cwd=ROOT, capture_output=True, text=True).stdout
    if since:
        paths = [l.strip() for l in out.splitlines() if l.strip()]
    else:
        paths = [l[3:].strip() for l in out.splitlines() if l.strip()]
    diff = subprocess.run(text, cwd=ROOT, capture_output=True, text=True).stdout
    return paths, diff


def matches(path, patterns):
    for p in patterns:
        if fnmatch.fnmatch(path, p):
            return True
        # `a/**` should also cover `a/b`, which fnmatch does not do on its own.
        if p.endswith("/**") and (path == p[:-3] or path.startswith(p[:-2])):
            return True
    return False


def outside_holes(diff, path):
    """Changed lines in `path` that do not fall inside a numbered HOLE block.

    Read off the hunk headers rather than the file: what matters is where the
    change landed, and a file re-read after the fact has already lost that.
    """
    bad = []
    in_file = False
    inside = False
    for line in diff.splitlines():
        if line.startswith("diff --git "):
            in_file = line.endswith(" b/" + path)
            inside = False
            continue
        if not in_file or line.startswith(("---", "+++", "index ", "@@")):
            if line.startswith("@@"):
                inside = False          # a new hunk: position is unknown again
            continue
        body = line[1:] if line[:1] in "+- " else line
        if re.match(r"\s*//\s*----\s*HOLE\s+\d+", body):
            inside = True
        elif re.match(r"\s*//\s*----\s*end HOLE", body):
            inside = False
            continue
        if line.startswith(("+", "-")) and not inside:
            bad.append(body.strip()[:70] or "(blank line)")
    return bad


# The cases this file is expected to get right. A checker nobody has watched
# fail is a checker nobody knows works, and these are cheap enough to run every
# time, so the evidence is never older than the last run.
#
#   (agent, path, allowed?)
CASES = [
    # Refused, and this is the change that matters: the hole filler is not
    # given the file. It returns text and `xtask fill` splices it.
    ("hole-filler", "crates/vleo-mod-prop/nodes/prop_capture_efficiency/model.rs", False),
    ("hole-filler", "crates/vleo-mod-prop/nodes/prop_capture_efficiency/node.toml", False),
    ("hole-filler", "crates/vleo-core/src/physics/prop.rs", False),
    ("hole-filler", "xtask/src/main.rs", False),
    ("fixture-recorder", "crates/vleo-mod-aero/nodes/aero_drag_force/fixtures.toml", True),
    ("fixture-recorder", "crates/vleo-mod-aero/nodes/aero_drag_force/node.toml", False),
    ("fixture-recorder", "web/js/app.js", False),
    ("declaration-drafter", "crates/vleo-mod-aero/nodes/aero_drag_force/node.toml", True),
    ("declaration-drafter", "crates/vleo-mod-aero/nodes/aero_drag_force/fixtures.toml", False),
    ("declaration-drafter", "crates/vleo-core/src/physics/aero.rs", False),
    ("test-author", "crates/vleo-core/tests/pmath_accuracy.rs", True),
    ("test-author", "crates/vleo-mod-prop/nodes/prop_capture_efficiency/evidence.rs", False),
    ("test-author", "crates/vleo-mod-prop/nodes/prop_capture_efficiency/fixtures.toml", False),
    ("diagnostician", "crates/vleo-core/src/resolver.rs", False),
    ("diagnostician", "docs/RUNBOOK.md", False),
    ("systems-backend", "crates/vleo-daemon/src/main.rs", True),
    ("systems-backend", "xtask/src/main.rs", True),
    ("systems-backend", "tools/seed_tree.py", True),
    ("systems-backend", "crates/vleo-mod-prop/nodes/prop_capture_efficiency/model.rs", False),
    ("systems-backend", "web/js/matrix.js", False),
    ("frontend-visualisation", "web/js/matrix.js", True),
    ("frontend-visualisation", "web/app.css", True),
    ("frontend-visualisation", "crates/vleo-daemon/src/main.rs", False),
]

# A hole body, and the same file with a line smuggled in above the imports.
INSIDE = """diff --git a/m/model.rs b/m/model.rs
@@
 // ---- HOLE 1 : do the thing -> Ratio
-let x: Ratio = old();
+let x: Ratio = new();
 // ---- end HOLE 1
"""
OUTSIDE = """diff --git a/m/model.rs b/m/model.rs
@@
 use vleo_core::units::*;
+use std::f64::consts::PI;
 // ---- HOLE 1 : do the thing -> Ratio
 let x: Ratio = same();
 // ---- end HOLE 1
"""


def verdict(lane, path):
    if matches(path, lane.get("never", [])):
        return False
    return matches(path, lane.get("writes", []))


def selftest():
    all_lanes = {L["name"]: L for L in lanes()}
    bad = 0
    for name, path, want in CASES:
        got = verdict(all_lanes[name], path)
        if got != want:
            bad += 1
            print("   WRONG %-24s %-60s expected %s, got %s"
                  % (name, path, "allowed" if want else "refused",
                     "allowed" if got else "refused"))
    print("paths: %d case(s), %d wrong" % (len(CASES), bad))

    inside = outside_holes(INSIDE, "m/model.rs")
    outside = outside_holes(OUTSIDE, "m/model.rs")
    if inside:
        bad += 1
        print("   WRONG a change inside a hole was reported as outside: %s" % inside)
    if not any("consts::PI" in x for x in outside):
        bad += 1
        print("   WRONG a change outside a hole was not caught: %s" % outside)
    print("holes: 2 case(s), %d wrong" % (0 if not inside and outside else 1))

    # The hole filler writes nothing at all now. If that ever loosens, the
    # working model's one fully-enforced prohibition quietly becomes a request.
    c = all_lanes["hole-filler"]
    if c.get("writes") or c.get("never") != ["**"]:
        bad += 1
        print("   WRONG the hole filler has a write path again: writes=%r never=%r"
              % (c.get("writes"), c.get("never")))
    if "add a guard" not in c["may_not"]:
        bad += 1
        print("   WRONG the hole filler's guard prohibition is missing from its lane")
    for L in lanes():
        if "enforced_by" not in L:
            bad += 1
            print("   WRONG %s does not say what enforces its prohibition" % L["name"])
    print("lanes: %d agent(s) checked for a stated enforcement" % len(all_lanes))
    print("selftest: %s" % ("every case as expected" if bad == 0 else "%d FAILED" % bad))
    return 1 if bad else 0


def holes_only(since):
    """Every model.rs line changed outside a numbered HOLE block, by anyone."""
    paths, diff = changed(since)
    models = [p for p in paths if p.endswith("model.rs")]
    if not models:
        print("holes-only: no model.rs changed.")
        return 0
    bad = 0
    for p in models:
        out = outside_holes(diff, p)
        if out:
            bad += 1
            print("  OUT  %s" % p)
            for line in out:
                print("         %s" % line)
        else:
            print("  ok   %s — every changed line is inside a HOLE block" % p)
    if bad:
        print("\n%d file(s) edited outside a hole. That region is regenerated from the" % bad)
        print("sheet, so the edit is discarded by the next `xtask docs` and fails the")
        print("regeneration diff. `cargo xtask fill <node> --hole n --body -` is the way in.")
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--agent")
    ap.add_argument("--holes-only", nargs="?", const="", metavar="SINCE",
                    help="check every model.rs change in a range against the HOLE markers")
    ap.add_argument("--since", help="a commit or range; default is the working tree")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--selftest", action="store_true",
                    help="run the cases this checker is expected to get right")
    a = ap.parse_args()

    if a.selftest:
        return selftest()
    if a.holes_only is not None:
        return holes_only(a.holes_only or None)

    all_lanes = lanes()
    if a.list or not a.agent:
        for L in all_lanes:
            print("%-2s %-24s %s" % (L["id"], L["name"], L["does"]))
            print("%-27s may not: %s" % ("", L["may_not"]))
        return 0

    lane = next((L for L in all_lanes if L["name"] == a.agent), None)
    if lane is None:
        print("no lane for %r. Known: %s" % (a.agent, ", ".join(L["name"] for L in all_lanes)))
        return 2

    paths, diff = changed(a.since)
    if not paths:
        print("%s: nothing changed." % a.agent)
        return 0

    bad = []
    for p in paths:
        if matches(p, lane.get("never", [])):
            bad.append((p, "outside the lane — %s may not touch it" % lane["name"]))
        elif not matches(p, lane.get("writes", [])):
            bad.append((p, "not in %s's lane" % lane["name"]))

    ok = [p for p in paths if p not in {b[0] for b in bad}]
    print("%s: %d path(s) changed, %d inside the lane" % (a.agent, len(paths), len(ok)))
    for p in ok:
        print("   ok   %s" % p)
    for p, why in bad:
        print("   OUT  %s — %s" % (p, why))
    if bad:
        print("\n%s may not: %s" % (lane["name"], lane["may_not"]))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
