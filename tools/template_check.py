#!/usr/bin/env python3
"""Has any node folder drifted from the template?

    tools/template_check.py
    tools/template_check.py --selftest

Every node is the same eight files in one directory. That uniformity is what
makes adding a node a copy rather than a decision, ownership a path rule, and a
node's history the history of a directory — and it is the kind of property that
degrades one folder at a time without anything noticing, because each individual
departure looks harmless.

`xtask gate` already proves the GENERATED files match what the generators would
write. What it does not check is the shape of the folder itself: a stray file
somebody left behind, a fixtures.toml beside a declared row that generates no
tests from it, a sheet missing a section the template requires, a parity.csv
with nothing to be the parity of. Those are the drifts this looks for.

It is deliberately dumb about content. Anything that needs to understand a
number belongs in the gate; this understands only layout.

Exit status is 0 when nothing has drifted and 1 when anything has.
"""

import argparse
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

#: The template, and it is STATE-DEPENDENT. A seeded row generates no Rust —
#: "the folder exists, the row is on the tree, every tab opens and each one says
#: what goes in it" — so its folder is the sheet, its fixtures, the page and the
#: metadata. A published row generates the four Rust files as well. Checking the
#: Rust against a seeded folder is checking for something the template does not
#: promise, and a checker that reports 1078 nodes as broken is a checker nobody
#: will run twice.
BY_HAND = {"node.toml", "fixtures.toml"}
ALWAYS_GENERATED = {"page.html", "meta.json"}
WHEN_PUBLISHED = {"model.rs", "contract.rs", "mod.rs", "evidence.rs"}
GENERATED = ALWAYS_GENERATED | WHEN_PUBLISHED
#: Beside those, exactly one other file is allowed, and only with a reason.
OPTIONAL = {"parity.csv"}

#: Sections every published sheet must carry. A seeded sheet is exempt: it is
#: the honest state of most of a tree for most of a programme.
REQUIRED_TABLES = ["question", "maths", "output"]

#: Keys that must be present at the top of every sheet, whatever its state.
REQUIRED_KEYS = ["id", "label", "folder", "subsystem", "parent", "kind", "owner", "tier", "layer", "order"]


def node_dirs():
    return sorted(ROOT.glob("crates/*/nodes/*"))


def check_folder(d, names, published, findings):
    """The files in one node directory, against the template."""
    strays = names - BY_HAND - GENERATED - OPTIONAL
    for s in sorted(strays):
        findings.append(f"{d.name}: {s} is not part of the node template")
    want = ALWAYS_GENERATED | (WHEN_PUBLISHED if published else set())
    for g in sorted(want - names):
        findings.append(f"{d.name}: {g} is missing — the folder was never generated")
    # The other direction, which is the drift that actually hides: Rust beside a
    # row that is not published. Either the state was walked back and the
    # artefacts left behind, or the sheet says one thing and the folder another.
    if not published:
        for g in sorted(WHEN_PUBLISHED & names):
            findings.append(f"{d.name}: {g} exists but the sheet is not published")
    if "node.toml" not in names:
        findings.append(f"{d.name}: no sheet, so nothing here can be regenerated")
    for sub in (p for p in d.iterdir() if p.is_dir()):
        findings.append(f"{d.name}: {sub.name}/ — a node folder holds files, not directories")


def check_sheet(d, names, findings):
    """The sheet's own shape: the sections and keys the template requires."""
    p = d / "node.toml"
    if not p.is_file():
        return None
    try:
        with p.open("rb") as f:
            sh = tomllib.load(f)
    except Exception as e:  # a sheet that will not parse is the worst drift there is
        findings.append(f"{d.name}: node.toml does not parse — {e}")
        return None

    for k in REQUIRED_KEYS:
        if k not in sh:
            findings.append(f"{d.name}: the sheet has no `{k}`")
    # The folder name is frozen at seed and the sheet says so; a rename that
    # moved one and not the other is invisible until something cannot be found.
    if sh.get("folder") and sh["folder"] != d.name:
        findings.append(f"{d.name}: the sheet says folder = \"{sh['folder']}\"")

    seeded = sh.get("state", "") in ("", "empty")
    if not seeded:
        for t in REQUIRED_TABLES:
            if t not in sh:
                findings.append(f"{d.name}: published and has no [{t}]")
        # A parity grid is a claim about somebody else's implementation, and a
        # claim with nobody's name on it cannot be attributed.
        if "parity.csv" in names and not sh.get("migrated_from", "").strip():
            findings.append(f"{d.name}: parity.csv with no migrated_from")
        if sh.get("migrated_from", "").strip() and "parity.csv" not in names:
            findings.append(f"{d.name}: migrated_from with no parity.csv beside it")
    return sh


def check_generated_banner(d, findings):
    """Every generated file says so, at the top, in the same words."""
    for g in ("model.rs", "contract.rs", "mod.rs", "evidence.rs"):
        p = d / g
        if not p.is_file():
            continue
        head = p.read_text(errors="replace")[:400]
        if "GENERATED from node.toml" not in head:
            findings.append(f"{d.name}/{g}: no generated banner — a reader cannot tell not to edit it")


def check_page_tabs(d, findings, tabs_seen):
    """The page carries the same tab set for every node, in the same order."""
    p = d / "page.html"
    if not p.is_file():
        return
    html = p.read_text(errors="replace")
    tabs = re.findall(r'<button role="tab" class="tab[^"]*" data-tab="\d+">([^<]*)</button>', html)
    panels = re.findall(r'data-panel="(\d+)"', html)
    if not tabs:
        findings.append(f"{d.name}/page.html: no tabs at all")
        return
    tabs_seen.setdefault(tuple(tabs), []).append(d.name)
    if len(panels) != len(tabs):
        findings.append(f"{d.name}/page.html: {len(tabs)} tabs but {len(panels)} panels")


def run():
    findings = []
    tabs_seen = {}
    dirs = node_dirs()
    if not dirs:
        findings.append("no node folders found at all")
    states = {"published": 0, "seeded": 0}
    for d in dirs:
        if not d.is_dir():
            continue
        names = {p.name for p in d.iterdir() if p.is_file()}
        sh = check_sheet(d, names, findings)
        published = bool(sh) and (sh.get("state", "") not in ("", "empty"))
        states["published" if published else "seeded"] += 1
        check_folder(d, names, published, findings)
        check_generated_banner(d, findings)
        check_page_tabs(d, findings, tabs_seen)
    findings.insert(0, None)   # placeholder, replaced by the caller's summary
    findings.pop(0)

    # One tab set for the whole tree. More than one means the pages were
    # generated at different times and the template moved in between.
    if len(tabs_seen) > 1:
        findings.append(f"{len(tabs_seen)} different tab sets across the tree, not one:")
        for tabs, who in sorted(tabs_seen.items(), key=lambda kv: -len(kv[1])):
            findings.append(f"    {len(who):5d} nodes: {' · '.join(tabs)}  (e.g. {who[0]})")
    return dirs, findings, tabs_seen, states


def selftest():
    """Watch each check fail, on a folder built to fail it.

    A checker that has never been seen to refuse anything is a checker nobody
    should trust, and this one reports zero findings on the real tree — which is
    exactly the result that needs the most evidence behind it.
    """
    import shutil
    import tempfile
    import __main__

    def under(root):
        saved = __main__.ROOT
        __main__.ROOT = root
        try:
            return run()[1]
        finally:
            __main__.ROOT = saved

    cases = []
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        src = ROOT / "crates/vleo-mod-solar/nodes/sw_regime"
        seed_src = ROOT / "crates/vleo-mod-acs/nodes/l3_acs_ach_01"
        work = tmp / "crates/vleo-mod-solar/nodes/sw_regime"
        work.parent.mkdir(parents=True)
        shutil.copytree(src, work)
        seed = tmp / "crates/vleo-mod-acs/nodes/l3_acs_ach_01"
        seed.parent.mkdir(parents=True)
        shutil.copytree(seed_src, seed)

        def expect(label, wanted, mutate, restore):
            mutate()
            found = under(tmp)
            restore()
            hit = any(wanted in f for f in found)
            cases.append((label, hit))
            print(f"  {'caught' if hit else 'MISSED'}: {label}")
            if not hit:
                print(f"    wanted {wanted!r}, got {found[:4]}")

        base = under(tmp)
        cases.append(("a published node and a seeded one, untouched", not base))
        print(f"  {'clean' if not base else 'NOT CLEAN'}: an untouched copy of each kind")
        if base:
            print(f"    {base[:4]}")

        stray = work / "notes.txt"
        expect("a stray file in a node folder", "not part of the node template",
               lambda: stray.write_text("left behind\n"), lambda: stray.unlink())

        gone = work / "contract.rs"
        keep = gone.read_text()
        expect("a generated file missing from a published node", "never generated",
               lambda: gone.unlink(), lambda: gone.write_text(keep))

        intruder = seed / "model.rs"
        expect("Rust beside a row that is not published", "but the sheet is not published",
               lambda: intruder.write_text("// left over\n"), lambda: intruder.unlink())

        sheet = work / "node.toml"
        orig = sheet.read_text()
        expect("a sheet naming a different folder", "the sheet says folder",
               lambda: sheet.write_text(orig.replace('folder = "sw_regime"', 'folder = "elsewhere"')),
               lambda: sheet.write_text(orig))

        expect("a published sheet with no [maths]", "published and has no [maths]",
               lambda: sheet.write_text(orig.replace("[maths]", "[maths_disabled]")),
               lambda: sheet.write_text(orig))

        grid = work / "parity.csv"
        gk = grid.read_text()
        expect("migrated_from with no grid beside it", "migrated_from with no parity.csv",
               lambda: grid.unlink(), lambda: grid.write_text(gk))

        banner = work / "mod.rs"
        bk = banner.read_text()
        expect("a generated file with no banner", "no generated banner",
               lambda: banner.write_text(bk.replace("GENERATED from node.toml", "written by hand", 1)),
               lambda: banner.write_text(bk))

        page = work / "page.html"
        pk = page.read_text()
        expect("two different tab sets in one tree", "different tab sets",
               lambda: page.write_text(pk.replace(">theory<", ">THEORY<", 1)),
               lambda: page.write_text(pk))

        after = under(tmp)
        cases.append(("everything restored", not after))
        print(f"  {'clean' if not after else 'NOT CLEAN'}: after restoring every mutation")

    bad = [c for c, ok in cases if not ok]
    print(f"selftest: {len(cases)} cases, {len(cases) - len(bad)} as expected")
    for c in bad:
        print(f"  FAILED: {c}")
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()

    dirs, findings, tabs_seen, states = run()
    print(f"{len(dirs)} node folder(s) — {states['published']} published, {states['seeded']} seeded "
          f"— {len(findings)} finding(s)")
    if tabs_seen and len(tabs_seen) == 1:
        tabs = next(iter(tabs_seen))
        print(f"one tab set across every node, {len(tabs)} tabs: {' · '.join(tabs)}")
    for f in findings:
        print(f"  {f}")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
