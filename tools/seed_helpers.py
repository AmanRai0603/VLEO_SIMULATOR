#!/usr/bin/env python3
"""
The row helpers, and the lists they append to.

`xtask seed` in the delivery plan: run once, ever. It writes one folder per
node — the sheet, the initial hole bodies and the fixture table — and after
that the sheets are the source and this script is history. It is kept in the
repository because the alternative is that nobody can tell, in two years,
whether a field was authored or inherited from a seed.

Folder names are frozen here and recorded in the sheet. Editing a label never
moves a directory: renaming a heading would otherwise show up in version
control as hundreds of deletes and adds and would conflict with every open
branch.

Run:  python3 tools/seed_tree.py
"""
import os
import sys
import textwrap

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
NODES = []
LAYERS = {}


def esc(s):
    return s.replace("\\", "\\\\").replace('"', '\\"')


def toml_str(s):
    return '"%s"' % esc(s)


class Node(dict):
    pass


def D(nid, label, subsystem, parent, ty, unit, symbol, value, lo, hi, rlo, rhi,
      q, src, owner, conf, tier="A", kpis=(), note=""):
    """A declared value — a number a person picked.

    Two thirds of the tree is these. They are cheaper than a computed node and
    they are not free: a value with no unit, no range, no reason per bound, no
    source and no confirmation is an opinion that every margin downstream is
    built out of.
    """
    NODES.append(Node(
        id=nid, label=label, subsystem=subsystem, parent=parent, kind="declared",
        ty=ty, unit=unit, symbol=symbol, value=value, lo=lo, hi=hi,
        rlo=rlo, rhi=rhi, question=q, source=src, owner=owner, confirmed=conf,
        tier=tier, kpis=list(kpis), note=note, expression="%s = %g" % (symbol, value),
        assumptions=[], steps=[], inputs=[], fixtures=[], view=("number", {}),
        bundles=[], state="published", layer=2, crosses_to=""))


def C(nid, label, subsystem, parent, ty, unit, symbol, q, expr, src, owner,
      ins, steps, lo, hi, rlo, rhi, tier="B", assumptions=(), fixtures=(),
      kpis=(), view=("number", {}), bundles=(), kind="computed", note=""):
    """A computed node — one small question with one answer."""
    NODES.append(Node(
        id=nid, label=label, subsystem=subsystem, parent=parent, kind=kind,
        ty=ty, unit=unit, symbol=symbol, question=q, expression=expr, source=src,
        owner=owner, inputs=list(ins), steps=list(steps), lo=lo, hi=hi,
        rlo=rlo, rhi=rhi, tier=tier, assumptions=list(assumptions),
        fixtures=list(fixtures), kpis=list(kpis), view=view,
        bundles=list(bundles), confirmed="", value=None, note=note,
        state="published", layer=2, crosses_to=""))


def layer(gid, label, parent, owner, lyr, box=False, tone="slate", relates=(), cases=()):
    """One heading in the tree.

    `lyr` is which of the four layers it belongs to — 1 management,
    2 the system, 3 subsystem, 4 the run. Exactly one node crosses between any
    two layers; nothing else is shared, so no layer can be reasoned about
    wrongly from another.

    `box` says whether it is drawn as a nested box on the diagonal of the
    matrix. A mark inside a box is coupling that subtree owns; a mark outside
    it crosses a boundary, and that difference is the finding. A heading that
    is not a box is a label rather than a scope.
    """
    LAYERS[gid] = dict(id=gid, label=label, parent=parent, owner=owner,
                       layer=lyr, box=box, tone=tone, relates=list(relates),
                       cases=list(cases), order=len(LAYERS))


SEEDED = []


def S(nid, label, parent, subsystem, owner, lyr, kind="declared", crosses=""):
    """A seeded row.

    The folder exists, the row is on the tree, the eight tabs open and each one
    says what goes in it. Nothing is generated from it and it cannot run.

    This is the normal state of most of a tree for most of a programme. Six
    hundred grey rows on day one is not a failure and must not be drawn as one:
    the denominator was always there, and hiding the unstarted rows to make the
    tree look finished is the one thing the figure may never do.
    """
    NODES.append(Node(
        id=nid, label=label, subsystem=subsystem, parent=parent, kind=kind,
        ty="", unit="", symbol="", value=None, lo=0.0, hi=0.0, rlo="", rhi="",
        question="", source="", owner=owner, confirmed="", tier="", kpis=[],
        note="", expression="", assumptions=[], steps=[], inputs=[],
        fixtures=[], view=("number", {}), bundles=[], state="empty",
        layer=lyr, crosses_to=crosses))
    SEEDED.append(nid)


def seed_run(prefix, parent, subsystem, owner, lyr, labels):
    """A block of seeded rows under one heading."""
    for i, label in enumerate(labels, 1):
        S("%s_%02d" % (prefix, i), label, parent, subsystem, owner, lyr)


def unnamed(prefix, parent, subsystem, owner, lyr, n, what):
    """`n` rows the decomposition has a place for and nobody has named yet.

    Named `to be named` rather than given a plausible label, because a
    plausible label on an empty row is the one thing that would make this tree
    lie: a reader would take it for work that exists.
    """
    for i in range(1, n + 1):
        S("%s_%02d" % (prefix, i), "%s — to be named (%d)" % (what, i),
          parent, subsystem, owner, lyr)
