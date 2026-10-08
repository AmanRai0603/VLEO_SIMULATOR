#!/usr/bin/env python3
"""Check the files that tell the people maintaining this repository what to do.

    tools/docs_lint.py
    tools/docs_lint.py --selftest

The house rules are Markdown — AGENTS.md, the area files, CONTRIBUTING.md and
the operating model. That makes them an input to every change, and prose is the one
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
    "docs/OPERATING_1_0.md",
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


#: The four kinds of documentation (Diátaxis), docs/EXPLAINING.md E8.
KINDS = ["tutorial", "how-to", "reference", "explanation"]


def documents():
    """Every document a person reads: docs/, the top-level three, and the areas."""
    out = sorted((ROOT / "docs").glob("*.md"))
    out += [ROOT / f for f in ("README.md", "AGENTS.md", "CONTRIBUTING.md")]
    out += sorted((ROOT / "areas").glob("*.md"))
    return [p for p in out if p.is_file()]


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

    # Stale references. The one failure these files will really have. The
    # guide to changing the code is a map of paths, so it is held to them too.
    for p in files() + [p for p in (ROOT / "docs" / "CHANGING.md",) if p.is_file()]:
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
    # document that is merely out of date reads as authoritative. AGENTS.md
    # keeps only the commands that outlive the switch-over and sends the rest
    # to docs/PIPELINE.md, the table every command is generated from.
    xt = ROOT / "xtask" / "src" / "main.rs"
    if xt.is_file():
        m = re.search(r'match cmd \{(.*?)\n    \};', xt.read_text(), re.S)
        listed = set(re.findall(r'"([a-z]+)" =>', m.group(1))) if m else set()
        prose = "\n".join(
            (ROOT / f).read_text()
            for f in ("README.md", "AGENTS.md", "docs/USING_IT.md", "docs/PIPELINE.md")
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
            "explain", "simply", "breaks", "wrong", "version", "believed", "tested",
            "learned", "rests_on", "breaks_if", "risk", "since",
        }
        text = authoring.read_text()
        for f in sorted(AUTHOR_FIELDS):
            if f not in text:
                bad.append(("docs/NODE_AUTHORING.md", "does not explain the sheet field '%s'" % f))

    # REQUIRED DOCS. Every hand-written Rust file says what it is in its first
    # line: a file nobody can place is a file nobody reviews, and the two
    # 3,200-line files were split into parts precisely so each could be placed.
    # A row's generated files are the generator's to head, and they say so.
    for d in ("crates", "xtask"):
        for p in sorted((ROOT / d).rglob("*.rs")):
            rel = p.relative_to(ROOT)
            if "target" in rel.parts or (rel.parts[1].startswith("vleo-mod-") and "nodes" in rel.parts):
                continue
            if not p.read_text(errors="replace").startswith("//!"):
                bad.append((rel.as_posix(), "does not open with a `//!` line saying what it is"))

    # Every kind of figure the engine draws is in the guide to changing the
    # code, so the table a developer adds a kind from is the table that exists.
    fig = ROOT / "crates" / "vleo-modules" / "src" / "figure.rs"
    changing = ROOT / "docs" / "CHANGING.md"
    if fig.is_file():
        m = re.search(r"pub fn name\(self\)[^{]*\{(.*?)\n    \}", fig.read_text(), re.S)
        kinds = re.findall(r'Kind::\w+ => "([a-z0-9]+)"', m.group(1)) if m else []
        if not kinds:
            bad.append(("crates/vleo-modules/src/figure.rs", "Kind::name could not be read for its kinds"))
        guide = changing.read_text() if changing.is_file() else ""
        for k in kinds:
            if "| `%s` |" % k not in guide:
                bad.append(("docs/CHANGING.md", "does not name the figure kind `%s` in its table (§4)" % k))

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

    # THE EXPLANATION STANDARD, docs/EXPLAINING.md. Every document opens with
    # its answer (E1) and says what kind of reading it is (E8), in its first
    # block — so a reader knows in two lines whether this is the page they
    # need. A document that has to be read to the end to learn what it is for
    # is the failure these two lines exist to prevent.
    for p in documents():
        rel = p.relative_to(ROOT).as_posix()
        head = p.read_text().splitlines()[:16]
        if not any(l.startswith("> **Answer first.**") for l in head):
            bad.append((rel, "does not open with its answer — `> **Answer first.** …` in its first block (E1)"))
        kinds = [l for l in head if l.startswith("> **Kind:**")]
        if not kinds:
            bad.append((rel, "does not say what kind of reading it is — `> **Kind:** … · **For:** …` (E8)"))
        else:
            k = kinds[0][len("> **Kind:**"):].split("·")[0].strip()
            for part in (x.strip() for x in k.split("+")):
                if part not in KINDS:
                    bad.append((rel, "kind «%s» is not one of %s (E8)" % (part, ", ".join(KINDS))))

    # Every figure says where its picture stops being true (E4). The panels'
    # words live beside their code in web/js/solar.js, so the check reads them
    # there: each panel's `id` has a `breaks` before the next panel begins.
    solar = ROOT / "web" / "js" / "solar.js"
    if solar.is_file():
        src = solar.read_text()
        at = src.find("const PANELS = [")
        if at >= 0:
            end = src.find("\n];", at)
            body = src[at:end if end > 0 else len(src)]
            starts = [m.start() for m in re.finditer(r"\n    id: '([a-z0-9_-]+)',", body)]
            for i, s in enumerate(starts):
                block = body[s: starts[i + 1] if i + 1 < len(starts) else len(body)]
                pid = re.match(r"\n    id: '([a-z0-9_-]+)',", block).group(1)
                if "\n    breaks: '" not in block:
                    bad.append(("web/js/solar.js", "the panel «%s» does not say where its picture stops being true (E4)" % pid))

    # The roles, by their names (docs/PLAN_1_0.md, phase B). Five roles and a
    # deputy for each, and every page, screen and guide uses those and no
    # other: a page that says "the lead" or "the team" is describing an
    # organisation that no longer exists, and a reader takes it for the one
    # that does. The names stored in files change with the files (phase C).
    bad.extend(role_words())

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


#: The pages, screens and guides people read, where the five roles are the only
#: names for people (docs/OPERATING_1_0.md, section 2; docs/PLAN_1_0.md,
#: phase B). Globs from the root.
ROLE_PAGES = [
    "README.md", "AGENTS.md", "CONTRIBUTING.md", "areas/*.md",
    "docs/*.md", "docs/*.html", "docs/roles/*.html", "docs/manual.toml",
    "docs/examples/*.html", "web/*.html", "web/*.css", "web/js/*.js",
    "web/pages/*", "acceptances/README.md",
    ".github/pull_request_template.md", "groups/SPEC.toml",
    "groups/skill/*/*.md", ".claude/skills/*/*.md", "contract/README.md",
    "xtask/src/main.rs", "xtask/src/pipeline.rs",
]

#: Read, but not by this check — each with the reason. Nothing else is skipped.
ROLE_EXEMPT = {
    "web/group.html": "generated from web/pages and web/js, which are checked",
    "web/node.html": "generated from web/pages and web/js, which are checked",
    "docs/MATLAB_PORT_PLAN.md": "the record of the port as it was planned, kept as written",
    "docs/VARIABLES.md": "generated from the design's own sheets, whose words are their owners'",
    "docs/DERISK_NARRATIVE.md": "generated from the design's own version records, whose words are their owners'",
}

#: What is masked before the words are read: a name in backticks (or in
#: `<code>` on a page), a
#: placeholder, a word quoted as a word, an identifier, and the pipeline's
#: check *the author approved this exact change*, which is a job's name and
#: retires with the group loop at the switch-over.
ROLE_MASK = re.compile(
    r"`[^`\n]*`|<code>[^<]*</code>|<[a-z_]+>|“[^”]*”|‘[^’\n]*’"
    r"|the author approved this exact[\s\\]+change"
    r"|\bcommit authors?\b|\b(?:review|GitHub) teams?\b|\bmaintainer branch(?:es)?\b",
    re.I)

#: An old name used for a person. "Lead" is also a forecast's lead time, which
#: keeps its name, so it is read only in the shapes a person has.
OLD_ROLE = re.compile(
    r"(?<![\w.$!/<\[-])(?:"
    r"(?:the|a|an|its|their|your|each|every|one|this|that|no|whose|our)\s+"
    r"(?:users?|teams?|maintainers?|authors?)(?:['’]s)?"
    r"|(?:users|maintainers|authors|teammates?|team\s+members?)(?!\s*[=(\[:])"
    r"|(?:user|team|maintainer|author|lead)['’]s"
    r"|group\s+leads?"
    r"|(?:your|its|their)\s+lead"
    r"|the\s+lead(?=\s+(?:sets|edits|signs|seals|issues|accepts|opens|writes|decides|says|sees|keeps|assembles|owns|gave|set)\b)"
    r"|(?:the|an|its)\s+experts?(?=\s+(?:who|fills|writes|knows)\b)"
    r")(?![\w/>\]-]|\.\w)",
    re.I)


def role_words(root=None):
    """Every place a page names a person by a role that no longer exists."""
    root = root or ROOT
    out = []
    seen = set()
    for g in ROLE_PAGES:
        for p in sorted(root.glob(g)):
            rel = p.relative_to(root).as_posix()
            if rel in seen or not p.is_file() or rel in ROLE_EXEMPT:
                continue
            seen.add(rel)
            try:
                text = p.read_text(encoding="utf-8")
            except UnicodeDecodeError:
                continue
            if rel.endswith(".md"):
                text = re.sub(r'"[^"\n]*"', lambda m: " " * len(m.group(0)), text)
            text = ROLE_MASK.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)
            for n, line in enumerate(text.split("\n"), 1):
                for m in OLD_ROLE.finditer(line):
                    out.append((rel, "line %d names a person «%s» — the roles are programme manager, "
                                     "system engineer, subsystem engineer, node engineer, developer"
                                % (n, " ".join(m.group(0).split()))))
    return out


def _unname_publish(d):
    """Take `xtask publish` out of every document that names it."""
    for f in ("README.md", "AGENTS.md", "docs/USING_IT.md", "docs/PIPELINE.md"):
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
        ("a document that does not open with its answer",
         lambda d: (d / "docs" / "RUNBOOK.md").write_text(
             (d / "docs" / "RUNBOOK.md").read_text().replace("> **Answer first.**", "> Answer:")),
         "does not open with its answer"),
        ("a document of no known kind",
         lambda d: (d / "docs" / "ARCHITECTURE.md").write_text(
             (d / "docs" / "ARCHITECTURE.md").read_text().replace("**Kind:** explanation", "**Kind:** story")),
         "kind «story»"),
        ("a figure that does not say where it breaks",
         lambda d: (d / "web" / "js" / "solar.js").write_text(
             re.sub(r"\n    breaks: '[^\n]*", "", (d / "web" / "js" / "solar.js").read_text(), count=1)),
         "does not say where its picture stops being true"),
        ("a Rust file that does not say what it is",
         lambda d: (d / "crates" / "vleo-sheet" / "src" / "shell.rs").write_text(
             (d / "crates" / "vleo-sheet" / "src" / "shell.rs").read_text().replace("//!", "//", 1)),
         "crates/vleo-sheet/src/shell.rs does not open with a `//!`"),
        ("a figure kind the guide does not name",
         lambda d: (d / "crates" / "vleo-modules" / "src" / "figure.rs").write_text(
             (d / "crates" / "vleo-modules" / "src" / "figure.rs").read_text()
             .replace('Kind::Scene3d => "scene3d"', 'Kind::Scene3d => "volume"')),
         "figure kind `volume`"),
        ("a path in the guide to changing the code that is gone",
         lambda d: (d / "docs" / "CHANGING.md").write_text(
             (d / "docs" / "CHANGING.md").read_text() + "\nSee `web/js/gone.js`.\n"),
         "docs/CHANGING.md points at web/js/gone.js"),
        ("a page naming a person by an old role",
         lambda d: (d / "docs" / "RUNBOOK.md").write_text(
             (d / "docs" / "RUNBOOK.md").read_text() + "\nSend it to the group lead.\n"),
         "«group lead»"),
        ("a guide calling the people the team",
         lambda d: (d / "docs" / "manual.toml").write_text(
             (d / "docs" / "manual.toml").read_text().replace(
                 "reaches everyone in the next release", "reaches the team in the next release")),
         "«the team»"),
        ("a screen naming the author",
         lambda d: (d / "web" / "js" / "napp.js").write_text(
             (d / "web" / "js" / "napp.js").read_text().replace(
                 "Only its node engineer signs it.", "Only its author signs it.")),
         "«its author»"),
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
