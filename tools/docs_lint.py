#!/usr/bin/env python3
"""Check the files that tell the people maintaining this repository what to do.

    tools/docs_lint.py
    tools/docs_lint.py --selftest

The house rules are Markdown — AGENTS.md, the area files, CONTRIBUTING.md and
the work model. That makes them an input to every change, and prose is the one
input nothing else checks.

The failure these files will actually have is not a contradiction; it is rot.
They point at `crates/vleo-sheet/src/emit.rs`, at `xtask docs`, at a folder
that was removed, and every one of those can be renamed by a commit that never
opens an instruction file. A stale reference is worse than a missing one: it
reads as authoritative and sends somebody to a path that no longer exists.

So the checks here are mostly about references being real, not about prose.

Exit status is 0 when everything holds and 1 when anything does not.
"""

import argparse
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

#: The instruction files, individually. A missing one is a whole area with no
#: house rules, which is how two people come to disagree about what "reviewed"
#: means.
REQUIRED = [
    "AGENTS.md",
    "areas/generators.md",
    "areas/faces.md",
    "areas/document.md",
    "areas/data.md",
    "areas/numerics.md",
    "areas/release.md",
    "CONTRIBUTING.md",
    "docs/WORK_MODEL.md",
]

#: A path-looking token inside backticks. Two shapes, checked differently:
#: something with a slash is a path and must resolve exactly; a bare filename
#: like `model.rs` is a *kind* of file — it appears in every row's folder — so it is
#: satisfied by any file of that name anywhere in the tree. Treating the second
#: as a path is how a lint starts reporting things that are not wrong, and a
#: lint that cries wolf is a lint people turn off.
PATHISH = re.compile(
    r"`((?:crates|tools|xtask|web|docs|areas|agents|panels|bundles|sources|cases|\.github|\.claude)"
    r"/[A-Za-z0-9_./*-]+"
    r"|[A-Za-z0-9_-]+\.(?:rs|toml|py|md|json|html|js|yml|lock|sh|csv))`"
)


def files():
    """Every instruction file."""
    return [p for p in (ROOT / r for r in REQUIRED) if p.is_file()]


def _sources():
    """Every line of code that could name a file format."""
    if not hasattr(_sources, "cache"):
        text = []
        for d in ("crates", "xtask", "tools", "web"):
            for p in (ROOT / d).rglob("*"):
                if p.suffix in {".rs", ".py", ".js"} and "target" not in p.parts:
                    text.append(p.read_text(errors="replace"))
        _sources.cache = "\n".join(text)
    return _sources.cache


def resolves(token):
    """Does this exist, or is it a format the code implements?

    A bare filename is a kind of file rather than a location: `model.rs` sits in
    every row's folder and naming one of them in prose would be worse, not better. So
    it is satisfied by any file of that name.

    A format with no instance yet — `parity.csv` before the first node is
    migrated — is satisfied instead by the code knowing the name. That is the
    check worth making: it catches prose describing a mechanism nobody built,
    which is the failure mode of a document that runs ahead of its repository.
    """
    if "*" in token:
        return any(ROOT.glob(token))
    if "/" in token:
        return (ROOT / token).exists()
    if any(ROOT.glob("**/" + token)):
        return True
    return '"%s"' % token in _sources() or "'%s'" % token in _sources()


def check():
    bad = []

    for r in REQUIRED:
        p = ROOT / r
        if not p.is_file():
            bad.append((r, "named in the working model and missing"))
        elif len(p.read_text().strip()) < 400:
            # A stub is worse than nothing: it reads as though the area has
            # rules and it does not.
            bad.append((r, "is a stub — %d characters" % len(p.read_text().strip())))

    # Stale references. The one failure these files will really have.
    for p in files():
        rel = p.relative_to(ROOT).as_posix()
        for token in sorted(set(PATHISH.findall(p.read_text()))):
            if token.startswith(("http", "//")) or " " in token:
                continue
            if not resolves(token):
                bad.append((
                    rel,
                    "points at %s, which does not exist and which no code names" % token,
                ))

    # The agent roster was removed: every change to the design now arrives as
    # a node form and is applied by a developer. A rule that still points at a
    # lane or a definition reads as a control that no longer exists.
    readers = files() + [ROOT / f for f in ("README.md", "docs/USING_IT.md",
                                           "docs/NODE_AUTHORING.md", "docs/RUNBOOK.md")
                         if (ROOT / f).is_file()]
    for p in readers:
        rel = p.relative_to(ROOT).as_posix()
        for gone in ("agents/lanes.toml", "agents/provenance.toml", ".claude/agents",
                     "agent_lanes.py", "instruction_lint.py", "VLEO_ALLOW_WRITE"):
            if gone in p.read_text():
                bad.append((rel, "names %s, which was removed" % gone))

    # Every command the tool has is named in a document somebody reads. This is
    # the check for the staleness that actually happened: three commands were
    # added — declare, fill, ready — and docs/USING_IT.md, which calls itself
    # the page to read first, still described the workflow they replaced. A
    # document that is merely out of date reads as authoritative.
    xt = ROOT / "xtask" / "src" / "main.rs"
    if xt.is_file():
        m = re.search(r'match cmd \{(.*?)\n    \};', xt.read_text(), re.S)
        listed = set(re.findall(r'"([a-z]+)" =>', m.group(1))) if m else set()
        prose = "\n".join(
            (ROOT / f).read_text() for f in ("README.md", "AGENTS.md", "docs/USING_IT.md")
            if (ROOT / f).is_file()
        )
        for c in sorted(listed):
            if c in {"help"}:
                continue
            if ("xtask -- %s" % c) not in prose and ("xtask %s" % c) not in prose:
                bad.append(("docs", "`xtask %s` exists and no document names it" % c))

    # Every sheet field the loader reads is explained where an author reads.
    # This page had forty-five fields to explain and was missing fifteen,
    # including two added the same week. A reference with holes in it is worse
    # than none: it reads as complete.
    loader = ROOT / "crates" / "vleo-sheet" / "src" / "load.rs"
    authoring = ROOT / "docs" / "NODE_AUTHORING.md"
    if loader.is_file() and authoring.is_file():
        # Only the fields a person writes on a node sheet. The rest belong to
        # layers/, cases/ or sources/ and are documented with those.
        AUTHOR_FIELDS = {
            "label", "question", "expression", "source", "confirmed_by", "symbol",
            "unit", "lower", "upper", "reason_lower", "reason_upper", "kind",
            "owner", "tier", "criticality", "migrated_from", "parity_tolerance", "sense",
            "note", "contributes",
            "fails_when", "state",
        }
        text = authoring.read_text()
        for f in sorted(AUTHOR_FIELDS):
            if f not in text:
                bad.append(("docs/NODE_AUTHORING.md", "does not explain the sheet field '%s'" % f))

    # The row count is a fact about the tree, written in prose in nine places.
    # It went stale the moment a subsystem was added, and nothing said so: the
    # gate was green, the lint was clean, and four files claimed a size the tree
    # no longer had. A number repeated in prose is a number that will be wrong.
    rows = len([d for d in ROOT.glob("crates/vleo-mod-*/nodes/*") if d.is_dir()])
    if rows:
        stale = set()
        for f in ("README.md", "AGENTS.md", "areas/generators.md"):
            p = ROOT / f
            if not p.is_file():
                continue
            for n in re.findall(r"\b(1[0-9]{3})\b(?= rows| across)", p.read_text()):
                if int(n) != rows:
                    stale.add((f, n))
        for f, n in sorted(stale):
            bad.append((f, "says %s rows and the tree has %d" % (n, rows)))

    # The review policy is stated once. Two copies are two policies within a
    # month: they had already begun to differ, one listing tolerance changes and
    # bundle publication and the other listing tools/ scripts.
    owner = ROOT / "CONTRIBUTING.md"
    if owner.is_file() and "How many reviewers, by what changed" not in owner.read_text():
        bad.append(("CONTRIBUTING.md", "no longer states the review policy, which AGENTS.md points at"))
    for p in files():
        rel = p.relative_to(ROOT).as_posix()
        if rel == "CONTRIBUTING.md":
            continue
        t = p.read_text()
        if "| reviewers |" in t or "| how many reviewers |" in t.lower():
            bad.append((rel, "restates the review policy — CONTRIBUTING.md is its one home"))

    # Adopted libraries carry the two fields that exist because of real failures.
    try:
        lock = tomllib.loads((ROOT / "ADOPTION.lock").read_text())
    except Exception as e:
        bad.append(("ADOPTION.lock", "does not parse: %s" % e))
        lock = {"adopted": []}
    for r in lock.get("adopted", []):
        for need in ("upstream", "licence", "fallback", "verified"):
            if not r.get(need):
                bad.append(("ADOPTION.lock", "%s has no %s" % (r.get("name", "?"), need)))
    return bad


def _unname_publish(d):
    """Take `xtask publish` out of every document that names it."""
    for f in ("README.md", "AGENTS.md", "docs/USING_IT.md"):
        p = d / f
        p.write_text(p.read_text().replace("xtask -- publish", "xtask -- xxpub")
                     .replace("xtask publish", "xtask xxpub"))


def selftest():
    """The lint has to fail on the things it exists to catch."""
    import tempfile, shutil, os
    global ROOT
    real = ROOT
    bad = 0
    cases = [
        ("a missing area file", lambda d: (d / "areas" / "numerics.md").unlink(), "numerics"),
        ("a stub area file", lambda d: (d / "areas" / "faces.md").write_text("# faces\n"), "stub"),
        ("a stale path reference",
         lambda d: (d / "AGENTS.md").write_text(
             (d / "AGENTS.md").read_text() + "\nSee `crates/vleo-gone/src/lib.rs`.\n"),
         "does not exist"),
        ("a rule pointing at the removed lanes",
         lambda d: (d / "AGENTS.md").write_text(
             (d / "AGENTS.md").read_text() + "\nRead your lane in agents/lanes.toml.\n"),
         "which was removed"),
        ("an xtask command no document names", _unname_publish, "`xtask publish` exists"),
        ("an adopted row with no licence",
         lambda d: (d / "ADOPTION.lock").write_text(
             (d / "ADOPTION.lock").read_text().replace('licence = "MIT OR Apache-2.0"', 'licence = ""', 1)),
         "has no licence"),
    ]
    for label, break_it, needle in cases:
        with tempfile.TemporaryDirectory() as tmp:
            work = Path(tmp) / "tree"
            shutil.copytree(real, work, ignore=shutil.ignore_patterns(".git", "target"))
            break_it(work)
            ROOT = work
            try:
                found = check()
            finally:
                ROOT = real
            if not any(needle in (f + " " + why) for f, why in found):
                bad += 1
                print("  FAIL %s was not caught (looked for %r)" % (label, needle))
    # And the control: the real tree is clean.
    if check():
        bad += 1
        print("  FAIL the repository itself does not lint clean:")
        for f, why in check():
            print("        %s — %s" % (f, why))
    print("selftest: %d cases, %s" % (len(cases) + 1, "all as expected" if not bad else "%d FAILED" % bad))
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()
    if a.selftest:
        return selftest()
    bad = check()
    n = len(files())
    for f, why in bad:
        print("  %-34s %s" % (f, why))
    print("%d instruction file(s), %d finding(s)" % (n, len(bad)))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
