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

The hole filler is the interesting one. Its lane includes `model.rs` — it has
to, that is where a hole body goes — so the path check alone would pass a
rewrite of the whole file. `holes_only` reads the diff instead and requires
every changed line to fall inside a numbered `HOLE` block.
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


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--agent")
    ap.add_argument("--since", help="a commit or range; default is the working tree")
    ap.add_argument("--list", action="store_true")
    a = ap.parse_args()

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

    if lane.get("holes_only"):
        for p in paths:
            if p.endswith("model.rs") and not matches(p, lane.get("never", [])):
                for line in outside_holes(diff, p):
                    bad.append((p, "changed outside a HOLE block: %s" % line))

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
