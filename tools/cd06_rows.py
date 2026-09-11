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

# The system layer's root, as this file names it. The management layer crosses here.
SYS_ROOT = "sys_vleo_multipayload"


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
        self.crossings = []    # the rows that cross between layers

    # -- shape ------------------------------------------------------------
    def is_group(self, i):
        return bool(self.kids.get(i))

    def variables_of(self, group):
        """The leaf variables directly under a layer-2 group, in document order."""
        return [(k, self.sys[k][1]) for k in self.kids.get(group, [])
                if not self.is_group(k)]

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
        # The one row in the management layer that crosses downward. CD-06
        # marks it by its note — "the door into this customer's engineering
        # layer" — and there is one per customer, which is the whole interface
        # between what a customer asked for and the architecture that answers
        # it. Nothing else in layer 1 can see layer 2.
        crosses = SYS_ROOT if "door into" in note else ""
        if crosses:
            cd.crossings.append(nid)
        S(nid, label, parent_id, pfx, own, lyr,
          kind=KIND.get(note, "declared"), crosses=crosses)

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


# Which layer-2 group each subsystem layer answers to.
#
# The document gives a target count per layer and a variable count per layer-2
# group, and every one of the fifteen pairs agrees: nineteen targets for
# propulsion, nineteen propulsion variables. That is the closure rule stated as
# an arithmetic fact rather than a claim, and it is what makes the target rows
# derivable instead of guessable.
LAYER3_GROUP = {
    "prop": "c2", "massaero": "c1", "payload": "c10", "power": "c3",
    "thermal": "c4", "fsw": "c6", "struct": "c5", "atthw": "c8",
    "orbmaint": "g3", "acs": "g2", "navod": "g4", "multipay": "c11",
    "ttc": "c7", "pointing": "g1", "navsense": "c9",
}

LAYER3_OWNER = {
    "prop": "propulsion", "massaero": "mass", "payload": "payload",
    "power": "power", "thermal": "thermal", "fsw": "avionics",
    "struct": "mass", "atthw": "gnc", "orbmaint": "propulsion",
    "acs": "gnc", "navod": "gnc", "multipay": "payload", "ttc": "comms",
    "pointing": "gnc", "navsense": "gnc",
}


def LAYER3_SOURCE(cd):
    """(id, label, owner, targets, total, layer-2 group) for each subsystem layer."""
    out = []
    for s in cd.d["layer3_shape"]:
        sid = s["id"]
        grp = LAYER3_GROUP[sid]
        n = len(cd.variables_of(grp))
        if n != s["targets"]:
            raise SystemExit(
                "%s: the document asks for %d targets and its layer-2 group holds %d "
                "variables. The closure rule says those are the same number, so one of "
                "the two has changed." % (sid, s["targets"], n))
        out.append((sid, s["label"], LAYER3_OWNER[sid], s["targets"], s["nodes"], grp))
    return out


def install_edges(cd, LAYERS, BY_ID):
    """The three graphs, wired once every row they might name exists.

    They stay three graphs. `ED_*` joins group to group and becomes a relation;
    `VE` joins variable to variable and becomes a derivation edge, declared on
    the consumer because knowing its inputs is what changes the consumer's
    implementation; `KE` joins variable to KPI and becomes a contribution,
    declared on the variable. Merging them would mean a KPI gets executed as
    though it were derived, and a navigation link counts as evidence.
    """
    stat = {"relation": 0, "derivation": 0, "contribution": 0, "skipped": 0}

    def as_group(tab, cd_id):
        """A relation names a scope. Where CD-06 puts one end on a row rather
        than a heading — every such edge in the document touches a customer's
        crossing node — the relation belongs to the scope that row sits in."""
        gid = cd.nid.get(cd_id)
        if gid in LAYERS:
            return gid
        parent = tab[cd_id][2]
        return cd.nid.get(parent) if parent else None

    for src, tab in (("ED_MGT", cd.mgt), ("ED_SYS", cd.sys)):
        for a, b, why in cd.d[src]:
            ga, gb = as_group(tab, a), as_group(tab, b)
            if ga in LAYERS and gb in LAYERS and ga != gb:
                if (ga, gb, why) not in LAYERS[ga]["relates"]:
                    LAYERS[ga]["relates"].append((ga, gb, why))
                stat["relation"] += 1
            else:
                stat["skipped"] += 1

    for a, b, why in cd.d["VE"]:
        na, nb = cd.nid.get(a), cd.nid.get(b)
        if na in BY_ID and nb in BY_ID:
            ins = BY_ID[nb]["inputs"]
            binding = _binding(BY_ID[na]["label"], {n for n, _ in ins})
            ins.append((binding, na))
            stat["derivation"] += 1
        else:
            stat["skipped"] += 1

    for a, b, _why in cd.d["KE"]:
        na, nb = cd.nid.get(a), cd.nid.get(b)
        if na in BY_ID and nb in BY_ID:
            if nb not in BY_ID[na]["kpis"]:
                BY_ID[na]["kpis"].append(nb)
            stat["contribution"] += 1
        else:
            stat["skipped"] += 1
    return stat


def _binding(label, taken):
    """A Rust-safe binding name for an input, from the producer's label."""
    b = slug(label)[:24].strip("_") or "x"
    if b[0].isdigit():
        b = "v_" + b
    if b in RESERVED:
        b += "_"
    c, n = b, 2
    while c in taken:
        c = "%s_%d" % (b, n)
        n += 1
    return c


RESERVED = {
    "as", "break", "const", "continue", "crate", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
    "move", "mut", "pub", "ref", "return", "self", "static", "struct", "super",
    "trait", "true", "type", "unsafe", "use", "where", "while", "async",
    "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
}
