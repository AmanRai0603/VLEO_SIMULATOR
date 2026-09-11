#!/usr/bin/env python3
"""Extract the CD-06 node tree from the source document into `cd06/tree.json`.

The document (`CD06_VLEO_Tool_Plan_and_Working_Model.html`) carries the tool's
own tree as data inside its first script block, not as prose: six arrays that
between them give every row of the management and system layers and every edge
between them. This reads those arrays and writes them out as JSON so the seeder
builds from the document rather than from anybody's memory of it.

Run it against the document to re-derive:

    python3 tools/cd06_extract.py path/to/CD06_VLEO_Tool_Plan_and_Working_Model.html

The output is committed, because the document is not in this repository and the
tree has to be reviewable in a diff. Re-running against the same document must
produce a byte-identical file; if it does not, the document changed and the
change is the thing to look at.

  HN_MGT  228 rows  the management layer   [id, label, parent, note, extra]
  ED_MGT   82 edges                        [from, to, why]
  HN_SYS  367 rows  the system layer       [id, label, parent, note, extra]
  ED_SYS   78 edges branch to branch       [from, to, why]
  VE       95 edges variable to variable   [from, to, why]   the derivation graph
  KE      245 edges variable to KPI        [from, to, why]   the contribution graph

Layer 3 is deliberately absent: the document specifies it by shape only — which
subsystem layers exist, how many targets each takes from layer 2 and how many
nodes each holds — and never names the rows. That table is `LAYER3` below, and
it is transcribed from section 21 rather than parsed, because it is prose there.
"""

import ast
import json
import re
import sys
from pathlib import Path

WANTED = ["HN_MGT", "ED_MGT", "HN_SYS", "ED_SYS", "VE", "KE"]

# Section 21 of the document, "SUBSYSTEM LAYERS": the layer, how many targets it
# takes from layer 2, and how many nodes it holds. The names of those nodes are
# not in the document — only the counts — so this is the whole of what layer 3
# can honestly be seeded from.
LAYER3 = [
    ("prop",     "Propulsion · ICP plasma thruster", 19, 95),
    ("massaero", "Mass and aero",                    19, 62),
    ("payload",  "Payload",                          12, 68),
    ("power",    "Power",                            13, 68),
    ("thermal",  "Thermal",                          11, 47),
    ("fsw",      "Flight software",                   3, 52),
    ("struct",   "Structure",                         4, 48),
    ("atthw",    "Attitude hardware",                 9, 42),
    ("orbmaint", "Orbit maintenance",                10, 40),
    ("acs",      "Attitude control sizing",           9, 39),
    ("navod",    "Navigation & OD",                   9, 38),
    ("multipay", "Multi-payload",                     9, 38),
    ("ttc",      "TT&C and downlink",                 8, 37),
    ("pointing", "Pointing error budget",             6, 32),
    ("navsense", "Navigation sensing",                6, 32),
]


def grab(src: str, name: str):
    """The array literal assigned to `name`, matched by bracket depth.

    A regex cannot do this: the arrays contain brackets inside strings, and a
    non-greedy match stops at the first one of those.
    """
    m = re.search(r"\bvar\s+" + re.escape(name) + r"\s*=\s*\[", src)
    if not m:
        raise SystemExit(f"{name}: not found in the document")
    start = m.end() - 1
    depth = 0
    i = start
    while i < len(src):
        c = src[i]
        if c == "[":
            depth += 1
        elif c == "]":
            depth -= 1
            if depth == 0:
                break
        elif c == '"':
            i += 1
            while i < len(src) and src[i] != '"':
                if src[i] == "\\":
                    i += 1
                i += 1
        i += 1
    return ast.literal_eval(src[start : i + 1].replace("null", "None"))


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    doc = Path(sys.argv[1]).read_text(encoding="utf-8", errors="replace")
    block = re.findall(r"<script[^>]*>(.*?)</script>", doc, re.S | re.I)[0]

    out = {
        "_source": "CD06_VLEO_Tool_Plan_and_Working_Model.html, first script block",
        "_note": (
            "Layers 1 and 2 are the document's own rows and edges, verbatim. "
            "Layer 3 is present as shape only — the document gives counts, never names."
        ),
        "layer3_shape": [
            {"id": i, "label": l, "targets": t, "nodes": n} for i, l, t, n in LAYER3
        ],
    }
    for name in WANTED:
        out[name] = grab(block, name)

    # What the document itself claims, checked against what was parsed. A count
    # that drifts is a document that changed under us.
    expect = {"HN_MGT": 228, "ED_MGT": 82, "HN_SYS": 367, "ED_SYS": 78, "VE": 95, "KE": 245}
    for k, n in expect.items():
        got = len(out[k])
        if got != n:
            raise SystemExit(f"{k}: expected {n} entries, the document gave {got}")

    # Every edge must land on a row that exists, or the graph is not a graph.
    for edges, rows, what in (
        ("ED_MGT", "HN_MGT", "management"),
        ("ED_SYS", "HN_SYS", "system"),
        ("VE", "HN_SYS", "derivation"),
        ("KE", "HN_SYS", "contribution"),
    ):
        ids = {r[0] for r in out[rows]}
        loose = [e for e in out[edges] if e[0] not in ids or e[1] not in ids]
        if loose:
            raise SystemExit(f"{edges}: {len(loose)} edge(s) name a row that does not exist, e.g. {loose[0]}")

    total3 = sum(n for _, _, _, n in LAYER3)
    print(
        f"cd06: {len(out['HN_MGT'])} management rows, {len(out['HN_SYS'])} system rows, "
        f"{len(LAYER3)} subsystem layers holding {total3} rows by count"
    )
    print(
        f"      {len(out['ED_MGT'])} management edges, {len(out['ED_SYS'])} relation, "
        f"{len(out['VE'])} derivation, {len(out['KE'])} contribution"
    )
    Path("cd06").mkdir(exist_ok=True)
    Path("cd06/tree.json").write_text(
        json.dumps(out, indent=1, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    print("      -> cd06/tree.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
