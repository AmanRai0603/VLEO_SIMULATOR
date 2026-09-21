#!/usr/bin/env python3
"""Every editable input in a subsystem, driven through its own active branches.

    tools/branch_audit.py --sub solar      audit one subsystem, end to end
    tools/branch_audit.py                  audit every subsystem that has
                                           published rows
    tools/branch_audit.py --selftest       needs no daemon

The face lets a person change a declared row and run the design. This asks the
same questions of every such row at once, against a running daemon, and reports
what nobody would find by clicking: an input nothing reads, a published row that
refuses where it stands, a branch that cannot be run end to end.

WHAT IT DOES NOT DO IS DECIDE WHICH BRANCHES EXIST. That rule lives in the
engine, at `/v1/branches`, because the node page needs the same answer and a
rule with two implementations is a rule that drifts — the face used to carry its
own copy in JavaScript and this tool would have been the third. Every branch
below is the engine's own answer.

ACTIVE IS NOT THE SAME AS WILL SUCCEED, and the difference is the most useful
thing here. Active says every row in the closure is published, so the tree is
filled in that far. A published row can still refuse, because its own value
lands outside its own declared domain: raising sw_mean_band_spread past about
22 drives sw_f107_cold_long below its 60 sfu floor and takes 33 rows with it.
That is the design answering, not the tool failing, so it is reported as a
finding with its bound rather than as an error.

A PERTURBATION IS NOT A PROOF. Where a row is read through a table that clamps —
sw_recurrence_lag above lead 26, sw_storm_design_level between its integer G
levels — a value moved a little way returns the same answer, and reading that as
"this input is inert" is wrong. So the probe walks the declared range rather
than nudging: a row that responds anywhere in its domain is not reported as
inert. The first version of this did nudge, and reported two false findings.
"""

import argparse
import json
import os
import pathlib
import sys
import urllib.error
import urllib.parse
import urllib.request

HOST = os.environ.get("VLEO_DAEMON", "http://localhost:7777")
# Enough points to cross a table's breakpoints. A row read through a lookup is
# flat between them, and three points can land on one plateau.
PROBE = 9


def get(path):
    with urllib.request.urlopen(HOST + path, timeout=300) as r:
        return json.load(r)


def run(node, mode, sets=None):
    p = [("node", node), ("mode", mode), ("case", "nominal")]
    for k, v in sets or []:
        p.append(("set", "%s:%.17g" % (k, v)))
    req = urllib.request.Request(HOST + "/v1/run",
                                 data=urllib.parse.urlencode(p).encode())
    with urllib.request.urlopen(req, timeout=600) as r:
        return json.load(r)


def drawn_by_a_figure(node_id, root=None):
    """Whether a figure names this row.

    A row nothing in the tree reads is not automatically a loose end. Seven of
    the solar rows are exactly that by design: their answer is read by a person
    off a chart, so there is no downstream node and there never will be. The
    difference between that and a genuinely unread row is whether a panel spec
    or the face names it, which is a question the repository can answer — so it
    is answered here rather than left for a reader to check by hand, because a
    finding a reader has to go and disprove seven times is a finding they stop
    reading.
    """
    base = pathlib.Path(root or os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    for d in ("panels", "web/js"):
        p = base / d
        if not p.is_dir():
            continue
        for f in p.rglob("*"):
            if f.is_file() and f.suffix in (".toml", ".js"):
                try:
                    if node_id in f.read_text(encoding="utf-8", errors="ignore"):
                        return True
                except OSError:
                    pass
    return False


def editable(row):
    """The rows the face offers a field for. The same test inputs.js applies."""
    return (row["kind"] == "declared" and row["state"] == "published"
            and row["hi"] > row["lo"])


def audit(sub=None):
    index = get("/v1/index")
    rows = [r for r in index["rows"] if sub is None or r["sub"] == sub]
    if not rows:
        print("no rows in subsystem %r" % sub)
        return 1
    pub = [r for r in rows if r["state"] == "published"]
    inputs = [r for r in pub if editable(r)]

    # One full run is the ground truth everything below is read against.
    anchor = pub[0]["id"] if pub else rows[0]["id"]
    base = run(anchor, "all")
    if not base["ok"]:
        print("the design does not run at all: %s" % base.get("message", ""))
        return 1
    BV = {v["id"]: v for v in base["values"]}
    BB = {b["id"]: b for b in base["blocked"]}

    findings = []
    name = sub or "the whole tree"
    print("%s — %d rows, %d published, %d editable" % (name, len(rows), len(pub), len(inputs)))

    # 1 · a published row that returns neither a number nor a named refusal is
    #     the one state nothing can act on.
    silent = [r for r in pub if r["id"] not in BV and r["id"] not in BB]
    refused = [r for r in pub if r["id"] in BB]
    print("   at the declared values: %d answer, %d refuse, %d are silent"
          % (len(pub) - len(silent) - len(refused), len(refused), len(silent)))
    for r in silent:
        findings.append((r["id"], "neither a value nor a refusal in a full run"))
    for r in refused:
        findings.append((r["id"], "refuses where it stands: " + BB[r["id"]]["message"][:90]))

    # 2 · every editable input, through the engine's own branches.
    print()
    print("   %-38s %6s %7s %6s %6s" % ("input", "reads", "branch", "moves", "breaks"))
    print("   " + "-" * 68)
    for r in inputs:
        b = get("/v1/branches?node=" + urllib.parse.quote(r["id"]))
        heads = b.get("branches", [])
        read_by = int(b.get("read_by", 0))

        # Walk the declared range rather than nudging: see the note at the top.
        moved, broke, seen_any = set(), set(), False
        held = []                 # where in the range every branch still answered
        lo, hi = r["lo"], r["hi"]
        for k in range(PROBE):
            x = lo + (hi - lo) * k / (PROBE - 1)
            res = run(r["id"], "all", [(r["id"], x)])
            if not res["ok"]:
                continue          # the engine refusing the INPUT is its own bound, not a finding
            seen_any = True
            V = {v["id"]: v for v in res["values"]}
            lost_here = False
            for h in heads:
                i = h["id"]
                if i in V and i in BV and V[i]["si"] != BV[i]["si"]:
                    moved.add(i)
                elif i not in V and i in BV:
                    broke.add(i)
                    lost_here = True
            held.append(not lost_here)
        print("   %-38s %6d %7d %6d %6d"
              % (r["id"][:38], read_by, len(heads), len(moved), len(broke)))

        if read_by == 0 and not drawn_by_a_figure(r["id"]):
            findings.append((r["id"], "nothing reads it: no node, and no figure either"))
        elif heads and seen_any and not moved and not broke:
            findings.append((r["id"],
                             "%d active branch(es) and none responds anywhere in its "
                             "declared range" % len(heads)))
        elif read_by and not heads:
            findings.append((r["id"], "%d row(s) read it, none in a fully published branch"
                             % read_by))
        # ONE FINDING PER INPUT, NOT ONE PER ROW IT TOOK DOWN. Three solar inputs
        # each knock out the same five layer-2 rows, and printing that fifteen
        # times buries the thing worth reading: how much of the declared range
        # the design actually closes over. A declared domain is a claim that the
        # relation is valid there, and nothing checks it against what happens
        # downstream — so the extent is measured and stated.
        if broke:
            n = len(held)
            ok_n = sum(1 for x in held if x)
            where = ("only at the bottom of its range" if held and held[0] and not held[-1]
                     else "only at the top of its range" if held and held[-1] and not held[0]
                     else "in part of its range")
            findings.append((r["id"],
                             "%d branch(es) go from a number to blocked inside its own "
                             "declared range %g … %g — every branch answers %s, at %d of "
                             "%d sampled points (%s)"
                             % (len(broke), lo, hi, where, ok_n, n,
                                ", ".join(sorted(broke)[:3]))))

    print()
    print("FINDINGS: %d" % len(findings))
    for i, w in findings:
        print("   %-40s %s" % (i[:40], w))
    # A finding is something for a person to read, not a reason to fail a build:
    # every one of them above is a true statement about the design.
    return 0


def selftest():
    """The parts that do not need a daemon: the editable test, and the probe."""
    bad = 0

    cases = [
        ({"kind": "declared", "state": "published", "lo": 0.0, "hi": 1.0}, True,
         "a published declared row with room to move"),
        ({"kind": "computed", "state": "published", "lo": 0.0, "hi": 1.0}, False,
         "a computed row is never editable"),
        ({"kind": "declared", "state": "empty", "lo": 0.0, "hi": 1.0}, False,
         "a seeded row has nothing to change"),
        ({"kind": "declared", "state": "deprecated", "lo": 0.0, "hi": 1.0}, False,
         "a retired row is not offered"),
        ({"kind": "declared", "state": "published", "lo": 5.0, "hi": 5.0}, False,
         "no room between the bounds"),
    ]
    for row, want, why in cases:
        if editable(row) != want:
            bad += 1
            print("  FAIL %s: editable() said %s" % (why, editable(row)))

    # The probe has to reach both ends, or a bound is never tested.
    lo, hi = 2.0, 10.0
    pts = [lo + (hi - lo) * k / (PROBE - 1) for k in range(PROBE)]
    if pts[0] != lo or pts[-1] != hi:
        bad += 1
        print("  FAIL the probe does not reach both bounds: %r" % pts[:2])
    if len(set(pts)) != PROBE:
        bad += 1
        print("  FAIL the probe repeats a point")
    # And it must be fine enough to cross a plateau. sw_storm_design_level is
    # three integer levels over 1..3; a probe that never lands off the declared
    # end would have called it inert.
    lv = [1.0 + 2.0 * k / (PROBE - 1) for k in range(PROBE)]
    if not any(abs(x - 1.0) < 1e-9 for x in lv) or not any(abs(x - 2.0) < 1e-9 for x in lv):
        bad += 1
        print("  FAIL the probe misses the integer levels of a lookup row")

    print("selftest: %d cases, %s"
          % (len(cases) + 3, "all as expected" if not bad else "%d FAILED" % bad))
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--sub", help="one subsystem, e.g. solar")
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()
    if a.selftest:
        return selftest()
    try:
        return audit(a.sub)
    except urllib.error.URLError as e:
        print("no daemon at %s: %s" % (HOST, e))
        print("start one with: cargo run --release -p vleo-daemon")
        return 1


if __name__ == "__main__":
    sys.exit(main())
