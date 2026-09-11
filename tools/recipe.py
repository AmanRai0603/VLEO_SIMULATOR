#!/usr/bin/env python3
"""Rung two of the escalation ladder: a transform solved once, replayed.

    tools/recipe.py --list
    tools/recipe.py --run <name>          # apply it to the working tree
    tools/recipe.py --check               # every recipe, on a copy, idempotent
    tools/recipe.py --selftest

# Why this is plain scripts

The working model points at OpenRewrite for this rung, and its argument is
exactly ours: when an agent can call a deterministic recipe it no longer has to
invent the upgrade path. The tool itself does not apply — it has no Rust
support, and it is licensed in tiers rather than being uniformly open. So the
pattern is kept and the tool is not: for a Rust codebase, rung two is plain
scripts, which is cheaper anyway and leaves the ladder intact.

# What a recipe is

A file in `tools/recipes/` named `<name>.py`, with two functions:

    WHY = "one line: the fix this replays, and where it came from"
    def apply(root): ...      # make the change; return how many files moved
    def verify(root): ...     # raise if the tree is not in the state apply() leaves

`--check` runs `apply` twice and requires the second to change nothing. A
transform that is not idempotent cannot be replayed safely, and replaying is
the entire point of the rung.

# Why the folder starts empty

A recipe is written when a fix has been needed twice, not in case it might be.
The ratchet in the working model runs one way — every messy fix becomes a
recipe, so rung-three work migrates down to rung two — and a folder seeded with
imagined transforms runs it backwards: it fills the rung with recipes nobody
needed and teaches people that the folder is decoration.
"""

import argparse
import importlib.util
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RECIPES = Path(__file__).resolve().parent / "recipes"


def load(path):
    spec = importlib.util.spec_from_file_location("recipe_" + path.stem, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    for need in ("WHY", "apply", "verify"):
        if not hasattr(mod, need):
            raise SystemExit("%s has no %s — see tools/recipe.py for the form" % (path, need))
    return mod


def discover(folder=RECIPES):
    return sorted(p for p in folder.glob("*.py") if not p.name.startswith("_"))


def check_one(path, root):
    """Apply twice on a copy. The second must change nothing."""
    mod = load(path)
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp) / "tree"
        shutil.copytree(root, work, ignore=shutil.ignore_patterns(".git", "target"))
        first = mod.apply(work)
        mod.verify(work)
        second = mod.apply(work)
        if second:
            return "not idempotent: a second run changed %d file(s)" % second
        mod.verify(work)
    return None if first is not None else "apply() returned nothing; it must say how many files moved"


def selftest():
    """Watch the runner accept a good recipe and refuse each bad one."""
    GOOD = (
        'WHY = "the example that tests the runner"\n'
        "def apply(root):\n"
        '    p = root / "probe.txt"\n'
        '    if p.exists() and p.read_text() == "done":\n'
        "        return 0\n"
        '    p.write_text("done")\n'
        "    return 1\n"
        "def verify(root):\n"
        '    assert (root / "probe.txt").read_text() == "done", "not as apply() left it"\n'
    )
    # Writes on every run, so a replay is never a no-op.
    NEVER_SETTLES = (
        'WHY = "writes every time"\n'
        "def apply(root):\n"
        '    p = root / "probe.txt"\n'
        '    p.write_text(p.read_text() + "x" if p.exists() else "x")\n'
        "    return 1\n"
        "def verify(root):\n"
        "    pass\n"
    )
    # Leaves the tree in a state its own verify() rejects.
    LIES = GOOD.replace('"not as apply() left it"', '"no"').replace(
        'assert (root / "probe.txt").read_text() == "done"',
        'assert (root / "probe.txt").read_text() == "something else"',
    )
    cases = [("good", GOOD, None), ("never_settles", NEVER_SETTLES, "not idempotent"),
             ("lies", LIES, "raises")]
    bad = 0
    with tempfile.TemporaryDirectory() as tmp:
        d = Path(tmp)
        (d / "tree").mkdir()
        (d / "tree" / "a.txt").write_text("x")
        for name, body, want in cases:
            p = d / ("%s.py" % name)
            p.write_text(body)
            try:
                got = check_one(p, d / "tree")
            except AssertionError as e:
                got = "raises: %s" % e
            ok = (got is None) if want is None else (got is not None and want in got)
            if not ok:
                bad += 1
                print("  FAIL %s: expected %r, got %r" % (name, want, got))
        # A recipe missing one of the three required names is refused outright.
        (d / "incomplete.py").write_text('WHY = "no apply"\n')
        try:
            load(d / "incomplete.py")
            bad += 1
            print("  FAIL incomplete: a recipe with no apply() was accepted")
        except SystemExit:
            pass
        # A leading underscore is a shared helper, not a recipe.
        (d / "_helper.py").write_text(GOOD)
        seen = [q.stem for q in discover(d)]
        want = ["good", "incomplete", "lies", "never_settles"]
        if seen != want:
            bad += 1
            print("  FAIL discovery: %s, wanted %s" % (seen, want))
    print("selftest: %d cases, %s" % (len(cases) + 2, "all as expected" if not bad else "%d FAILED" % bad))
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--run", metavar="NAME")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()

    if a.selftest:
        return selftest()
    found = discover()
    if a.list or not (a.run or a.check):
        if not found:
            print("0 recipes.")
            print("Rung two fills from rung three: a recipe is written when a fix has")
            print("been needed twice. Nothing has needed it twice yet.")
            return 0
        for p in found:
            print("  %-28s %s" % (p.stem, load(p).WHY))
        return 0
    if a.run:
        p = RECIPES / ("%s.py" % a.run)
        if not p.is_file():
            print("no recipe '%s'. tools/recipe.py --list" % a.run, file=sys.stderr)
            return 1
        mod = load(p)
        n = mod.apply(ROOT)
        mod.verify(ROOT)
        print("%s: %d file(s) changed" % (a.run, n))
        return 0
    worst = 0
    for p in found:
        why = check_one(p, ROOT)
        print("  %-28s %s" % (p.stem, why or "idempotent, and verifies"))
        worst |= 1 if why else 0
    print("%d recipe(s) checked" % len(found))
    return worst


if __name__ == "__main__":
    sys.exit(main())
