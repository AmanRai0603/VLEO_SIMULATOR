#!/usr/bin/env python3
"""The management and system layers, exactly as CD-06 declares them.

Every row here comes from `cd06/tree.json`, which `tools/cd06_extract.py` reads
out of the source document. Nothing in this file invents a variable: if a name
is on the tree it is on the document's tree, and if the document does not name
something then it is not named here either.

That last part matters more than it sounds. The document enumerates layers 1
and 2 completely — 228 rows and 367 rows, with 82, 78, 95 and 245 edges between
them — and specifies layer 3 by shape only: which subsystem layers exist, how
many targets each takes from layer 2, and how many rows each holds. So layer 3
gets its interface node and its target rows (those are derivable: a target
equals the parent's variable, one for one) and the remainder stay `to be named`
until somebody names them. A plausible label on a row nobody has specified is
the one thing that would make this tree lie.

## Identifiers

CD-06's own ids are positional — `m2_0`, `c1_9`, `a1k_3` — which is fine inside
one document and wrong as a directory name that has to survive a decade. Each
row therefore gets a readable id built from its group and its label, and keeps
the document's id in the sheet as `cd06` so the row can always be traced back.
Labels are verbatim; only the identifier is ours.
"""

import json
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# Which branch of the system tree a row belongs to decides its owner and the
# crate its folder lands in. Keyed on the depth-1 or depth-2 group id.
OWNER = {
    "svc": "systems", "cpt": "systems", "msn": "systems", "orb": "environment",
    "sat": "mass", "sub": "systems", "srv": "payload",
    "sb1": "propulsion", "sb2": "gnc", "sb3": "gnc", "sb4": "power",
    "sb5": "thermal", "sb6": "comms", "sb7": "payload",
    "cus": "systems", "plc": "programme", "seg": "systems",
    "std": "programme", "sup": "programme", "org": "programme",
}

TONE = {
    "svc": "amber", "cpt": "slate", "msn": "violet", "orb": "violet",
    "sat": "slate", "sub": "green", "srv": "amber",
    "cus": "teal", "plc": "green", "seg": "green",
    "std": "violet", "sup": "violet", "org": "violet",
}

# The four kinds CD-06 puts on a leaf, mapped onto the kinds the engine knows.
KIND = {
    "set here": "declared",
    "computed": "computed",
    "target — required": "required",
    "achieved — what the design delivers": "achieved",
}

SLUG_DROP = re.compile(r"[^a-z0-9]+")


def slug(s):
    s = (s.replace("&", " and ").replace("·", " ").replace("—", " ")
          .replace("’", "").replace("'", ""))
    return SLUG_DROP.sub("_", s.lower()).strip("_")


class Cd06:
    """The document's tree, indexed."""

    def __init__(self, path=None):
        self.d = json.load(open(path or os.path.join(ROOT, "cd06", "tree.json"),
                                encoding="utf-8"))
        self.sys = {r[0]: r for r in self.d["HN_SYS"]}
        self.mgt = {r[0]: r for r in self.d["HN_MGT"]}
        self.kids = {}
        for tab in (self.d["HN_SYS"], self.d["HN_MGT"]):
            for r in tab:
                self.kids.setdefault(r[2], []).append(r[0])
        self.nid = {}          # cd06 id -> our node/group id

    # -- shape ------------------------------------------------------------
    def is_group(self, i):
        return bool(self.kids.get(i))

    def branch(self, tab, i, depth=1):
        """The ancestor at `depth`, which decides owner, tone and crate."""
        chain = []
        r = tab[i]
        while r[2] is not None:
            chain.append(r[0])
            r = tab[r[2]]
        chain.reverse()
        return chain[depth - 1] if len(chain) >= depth else (chain[-1] if chain else i)

    def owner_of(self, tab, i):
        for depth in (2, 1):
            b = self.branch(tab, i, depth)
            if b in OWNER:
                return OWNER[b]
        return "systems"


def install(cd, layer, S, prefix_mgt="mgt", prefix_sys="sys"):
    """Emit every layer-1 and layer-2 row through the seeder's own helpers.

    Returns the id maps, so the caller can wire the edges once every row it
    might name actually exists.
    """
    for tab, root, lyr, pfx in ((cd.mgt, "mgm", 1, prefix_mgt),
                                (cd.sys, "prg", 2, prefix_sys)):
        _emit(cd, tab, root, lyr, pfx, layer, S)
    return cd.nid


def _emit(cd, tab, root, lyr, pfx, layer, S):
    # The root of each layer is a box; everything else inherits its branch tone.
    def rec(i, parent_id, group_slug):
        row = tab[i]
        cd_id, label, _, note = row[0], row[1], row[2], row[3]
        group = cd.is_group(cd_id)
        own = cd.owner_of(tab, cd_id)

        if group:
            gid = "%s_%s" % (pfx, slug(label)) if row[2] is None else \
                  "%s_%s" % (pfx, slug(label))
            gid = _unique(gid, cd.nid.values())
            cd.nid[cd_id] = gid
            b1 = cd.branch(tab, cd_id, 1)
            layer(gid, label, parent_id or "root", own, lyr,
                  box=(row[2] is None or cd_id in ("sub", "svc", "cus", "std")),
                  tone=TONE.get(b1, "slate"))
            for k in cd.kids.get(cd_id, []):
                rec(k, gid, slug(label))
            return

        nid = _unique("%s_%s_%s" % (pfx, group_slug, slug(label)),
                      cd.nid.values(), cd, tab, cd_id, pfx, label)
        cd.nid[cd_id] = nid
        S(nid, label, parent_id, pfx, own, lyr, kind=KIND.get(note, "declared"))

    rec(root, "", slug(tab[root][1]))


def _unique(base, taken, cd=None, tab=None, cd_id=None, pfx=None, label=None):
    """A colliding identifier is disambiguated by its grandparent, not a number.

    Three customers hold the same five headings and the same fifteen rows
    beneath them, so `Requirement discussion` occurs three times. A numeric
    suffix would settle the collision and tell a reader nothing; the group
    above it says which customer, which is the thing they wanted to know. Only
    if that still collides does a number appear, and that would be a tree with
    two genuinely indistinguishable rows in it.
    """
    taken = set(taken)
    if base not in taken:
        return base
    if cd is not None:
        row = tab[cd_id]
        gp = tab.get(tab[row[2]][2]) if row[2] and tab[row[2]][2] else None
        if gp:
            cand = "%s_%s_%s_%s" % (pfx, slug(gp[1]), slug(tab[row[2]][1]), slug(label))
            if cand not in taken:
                return cand
    n = 2
    while "%s_%d" % (base, n) in taken:
        n += 1
    return "%s_%d" % (base, n)
