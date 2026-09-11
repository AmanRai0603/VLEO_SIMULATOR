#!/usr/bin/env python3
"""Check the files that tell agents what to do.

    tools/instruction_lint.py
    tools/instruction_lint.py --selftest

Agent behaviour is Markdown. That makes it an input to every node, and it is
the only input in this system that nothing was checking — which is gap W8.

The failure these files will actually have is not a contradiction; it is rot.
They point at `crates/vleo-sheet/src/emit.rs`, at `xtask docs`, at a lane
called `hole-filler`, and every one of those can be renamed by a commit that
never opens an instruction file. A stale reference is worse than a missing one:
it reads as authoritative and sends somebody to a path that no longer exists.

So the checks here are mostly about references being real, not about prose.

Exit status is 0 when everything holds and 1 when anything does not.
"""

import argparse
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

#: The instruction files the working model names, individually. A missing one
#: is a whole area with no house rules, which is how two agents come to disagree
#: about what "reviewed" means.
REQUIRED = [
    "AGENTS.md",
    "areas/generators.md",
    "areas/faces.md",
    "areas/document.md",
    "areas/data.md",
    "areas/numerics.md",
    "areas/release.md",
]

#: Frontmatter every agent definition must carry.
FRONTMATTER = ["name", "description", "tools", "model"]

#: A path-looking token inside backticks. Two shapes, checked differently:
#: something with a slash is a path and must resolve exactly; a bare filename
#: like `model.rs` is a *kind* of file — it appears in 1329 folders — so it is
#: satisfied by any file of that name anywhere in the tree. Treating the second
#: as a path is how a lint starts reporting things that are not wrong, and a
#: lint that cries wolf is a lint people turn off.
PATHISH = re.compile(
    r"`((?:crates|tools|xtask|web|docs|areas|agents|panels|bundles|sources|cases|\.github|\.claude)"
    r"/[A-Za-z0-9_./*-]+"
    r"|[A-Za-z0-9_-]+\.(?:rs|toml|py|md|json|html|js|yml|lock|sh|csv))`"
)


def files():
    """Every instruction file: the required set, plus the agent definitions."""
    out = [ROOT / r for r in REQUIRED]
    out += sorted((ROOT / ".claude" / "agents").glob("*.md"))
    return [p for p in out if p.is_file()]


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
    1329 folders and naming one of them in prose would be worse, not better. So
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

    # Every agent definition is complete, and every lane has one.
    try:
        lanes = tomllib.loads((ROOT / "agents" / "lanes.toml").read_text())["agent"]
    except Exception as e:
        bad.append(("agents/lanes.toml", "does not parse: %s" % e))
        lanes = []
    defs = {}
    for p in sorted((ROOT / ".claude" / "agents").glob("*.md")):
        rel = p.relative_to(ROOT).as_posix()
        text = p.read_text()
        if not text.startswith("---\n"):
            bad.append((rel, "has no frontmatter"))
            continue
        head = text.split("---\n", 2)[1]
        fields = dict(
            (k.strip(), v.strip())
            for k, _, v in (l.partition(":") for l in head.splitlines() if ":" in l)
        )
        for need in FRONTMATTER:
            if need not in fields:
                bad.append((rel, "frontmatter has no '%s'" % need))
        defs[fields.get("name", p.stem)] = fields

    for lane in lanes:
        if lane["name"] not in defs:
            bad.append(("agents/lanes.toml", "%s has a lane and no definition" % lane["name"]))
        if "enforced_by" not in lane:
            bad.append(("agents/lanes.toml", "%s does not say what enforces its prohibition" % lane["name"]))
    for name in defs:
        if not any(L["name"] == name for L in lanes):
            bad.append((".claude/agents/%s.md" % name, "has a definition and no lane"))

    # Every definition is registered, with somewhere to fall back to.
    try:
        prov = tomllib.loads((ROOT / "agents" / "provenance.toml").read_text())
    except Exception as e:
        bad.append(("agents/provenance.toml", "does not parse: %s" % e))
        prov = {"agent": []}
    registered = {a["name"] for a in prov.get("agent", [])}
    for name in defs:
        if name not in registered:
            bad.append(("agents/provenance.toml", "%s is not registered — no source, pin, owner or fallback" % name))
    for a in prov.get("agent", []):
        for need in ("source", "licence", "owner", "model", "fallback", "verified"):
            if not a.get(need):
                bad.append(("agents/provenance.toml", "%s has no %s" % (a["name"], need)))
        # A checker may not run the family of the thing it checks. A model
        # handed its own reasoning to grade approves it, which is the whole
        # reason the working model states this rule twice.
        if a.get("checks"):
            other = next((x for x in prov["agent"] if x["name"] == a["checks"]), None)
            if other is None:
                bad.append(("agents/provenance.toml", "%s checks %r, which is not an agent" % (a["name"], a["checks"])))
            elif other.get("model") == a.get("model"):
                bad.append((
                    "agents/provenance.toml",
                    "%s checks %s and both run %r — a checker on the producer's own family "
                    "approves the producer's own reasoning"
                    % (a["name"], a["checks"], a.get("model")),
                ))
        if a["name"] in defs and defs[a["name"]].get("model") != a.get("model"):
            bad.append((
                "agents/provenance.toml",
                "%s runs %r but the register says %r" % (
                    a["name"], defs[a["name"]].get("model"), a.get("model")),
            ))

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
        ("a lane with no definition",
         lambda d: (d / "agents" / "lanes.toml").write_text(
             (d / "agents" / "lanes.toml").read_text()
             + '\n[[agent]]\nid = "Z"\nname = "ghost"\ndoes = "x"\nwrites = []\nnever = []\n'
               'may_not = "x"\nenforced_by = "none"\n'),
         "no definition"),
        ("an unregistered definition",
         lambda d: (d / "agents" / "provenance.toml").write_text(
             (d / "agents" / "provenance.toml").read_text().replace('name = "hole-filler"', 'name = "someone-else"')),
         "not registered"),
        ("a model that disagrees with the register",
         lambda d: (d / ".claude" / "agents" / "hole-filler.md").write_text(
             (d / ".claude" / "agents" / "hole-filler.md").read_text().replace("model: sonnet", "model: opus")),
         "the register says"),
        ("a checker on the producer's family",
         lambda d: (d / "agents" / "provenance.toml").write_text(
             (d / "agents" / "provenance.toml").read_text().replace(
                 'checks = "hole-filler"\nsource = "ADOPT"\nupstream = "the test-generation',
                 'checks = "hole-filler"\nsource = "ADOPT"\nupstream = "XX the test-generation')
             .replace('model = "opus"\nwhy_model = "a checker', 'model = "sonnet"\nwhy_model = "a checker')),
         "approves the producer"),
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
