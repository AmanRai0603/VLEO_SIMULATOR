#!/usr/bin/env python3
"""Release notes, as a transform rather than an afternoon.

    tools/release_notes.py --since v0.1.0 --version 0.2.0
    tools/release_notes.py --version 0.2.0            # since the last tag
    tools/release_notes.py --selftest

This is the other half of the commit-message rule. The prefix on every subject
is machine-readable for exactly one reason: so that the notes for a release are
read off the log instead of remembered. If this script ever needs a human to
finish it, the rule upstream is not being enforced.

# What it refuses to do

It does not invent a section for a change with no type, and it does not quietly
drop one. An unrecognised subject is listed under "uncategorised" where a
person will see it, because a release note that silently omits a change is
worse than one that is untidy.

# Breaking changes

A `!` subject or a `BREAKING CHANGE:` trailer puts the change at the top, under
its own heading, with the trailer's text. Nothing else in the notes can be
above it.
"""

import argparse
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from commit_message import HEAD, GENERATED, TYPES  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent

#: The order sections appear in, and what each is called to a reader. A reader
#: of release notes wants to know what changed for them first.
SECTIONS = [
    ("feat", "New"),
    ("fix", "Fixed"),
    ("perf", "Faster"),
    ("docs", "Documentation"),
    ("test", "Evidence"),
    ("refactor", "Internal"),
    ("build", "Build"),
    ("ci", "Pipeline"),
    ("chore", "Housekeeping"),
]

BREAKING = re.compile(r"^BREAKING CHANGE:\s*(.+)$", re.M)


def git(*args):
    return subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=False
    ).stdout


def last_tag():
    t = git("describe", "--tags", "--abbrev=0").strip()
    return t or None


def messages(since):
    rng = "%s..HEAD" % since if since else "HEAD"
    out = git("log", "--no-merges", "--format=%x00%H%x01%B", rng)
    for chunk in out.split("\x00"):
        if not chunk.strip():
            continue
        sha, _, body = chunk.partition("\x01")
        yield sha[:9], body


def classify(entries):
    """(breaking, {type: [(sha, text)]}, [(sha, subject)] uncategorised)."""
    breaking, buckets, loose = [], {}, []
    for sha, body in entries:
        subject = body.strip().split("\n")[0]
        if GENERATED.match(subject):
            continue
        m = HEAD.match(subject)
        if not m or m["type"] not in TYPES:
            loose.append((sha, subject))
            continue
        text = m["text"]
        scope = m["scope"]
        line = "%s — %s" % (scope, text) if scope else text
        b = BREAKING.search(body)
        if m["bang"] or b:
            breaking.append((sha, line, b.group(1).strip() if b else ""))
        buckets.setdefault(m["type"], []).append((sha, line))
    return breaking, buckets, loose


def render(version, since, entries):
    breaking, buckets, loose = classify(entries)
    out = ["# %s" % version, ""]
    if since:
        out.append("_Everything since %s._" % since)
        out.append("")
    if not (breaking or buckets or loose):
        out.append("No changes.")
        return "\n".join(out)

    if breaking:
        out += ["## Breaking", ""]
        for sha, line, why in breaking:
            out.append("- **%s** (%s)" % (line, sha))
            if why:
                out.append("  %s" % why)
        out.append("")
    for key, heading in SECTIONS:
        rows = buckets.get(key)
        if not rows:
            continue
        out += ["## %s" % heading, ""]
        out += ["- %s (%s)" % (line, sha) for sha, line in rows]
        out.append("")
    if loose:
        # Never silently dropped. A release note that omits a change is worse
        # than one that is untidy.
        out += ["## Uncategorised", "",
                "_These subjects predate the commit-message rule, or do not follow it._", ""]
        out += ["- %s (%s)" % (subject, sha) for sha, subject in loose]
        out.append("")
    return "\n".join(out).rstrip() + "\n"


CASES = [
    # (entries, must appear, must not appear)
    ([("aaaaaaaaa", "feat(web): the tree reads in order")], "## New", "## Fixed"),
    ([("bbbbbbbbb", "fix(data): a bundle with no licence is refused")], "- data — a bundle", None),
    ([("ccccccccc", "Merge pull request #3 from x/y")], "No changes.", "Uncategorised"),
    ([("ddddddddd", "an old subject from before the rule")], "## Uncategorised", None),
    ([("eeeeeeeee", "feat(core)!: the sheet schema changed\n\nBREAKING CHANGE: old sheets fail.")],
     "old sheets fail.", None),
    # Breaking is above everything else, whatever order the log arrives in.
    ([("fffffffff", "feat(web): something"),
      ("ggggggggg", "fix(core)!: something else\n\nBREAKING CHANGE: it moved.")],
     "## Breaking", None),
]


def selftest():
    bad = 0
    for entries, want, unwanted in CASES:
        text = render("1.0.0", "v0.9.0", entries)
        if want not in text:
            bad += 1
            print("  FAIL %r missing from:\n%s" % (want, text))
        if unwanted and unwanted in text:
            bad += 1
            print("  FAIL %r should not appear" % unwanted)
    # Breaking is first, always. Nothing in a release note may sit above the
    # thing that will stop somebody's build.
    text = render("1.0.0", None, CASES[-1][0])
    if "## Breaking" not in text or text.index("## Breaking") > text.index("## New"):
        bad += 1
        print("  FAIL a breaking change was not at the top")
    # Nothing is dropped: every commit reaches exactly one section.
    entries = [(c[0][0][0], c[0][0][1]) for c in CASES[:2]] + [("hhhhhhhhh", "not a conventional subject")]
    text = render("1.0.0", None, entries)
    for sha, _ in entries:
        if sha not in text:
            bad += 1
            print("  FAIL %s was dropped from the notes" % sha)
    print("selftest: %d cases, %s" % (len(CASES) + 2, "all as expected" if not bad else "%d FAILED" % bad))
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--version", help="what to call this release")
    ap.add_argument("--since", help="the previous tag; default is the last one")
    ap.add_argument("--out", help="write here as well as to stdout")
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()
    if a.selftest:
        return selftest()
    if not a.version:
        print("--version is required", file=sys.stderr)
        return 2
    since = a.since or last_tag()
    text = render(a.version, since, list(messages(since)))
    print(text)
    if a.out:
        Path(a.out).write_text(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
