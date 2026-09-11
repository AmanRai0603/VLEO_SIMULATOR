#!/usr/bin/env python3
"""The commit-message check — a fixed transform, done by a script.

The roster had an agent for this. It should not: writing a commit subject is a
fixed transform with one right answer per diff, and a model is the wrong tool
for a job that has no judgement in it. What the job actually needs is a rule
that fails, so this is the rule.

    tools/commit_message.py --file .git/COMMIT_EDITMSG    # the hook
    tools/commit_message.py --range origin/main..HEAD     # the pipeline
    tools/commit_message.py --selftest

Exit status is 0 when every message passes and 1 when any fails, and every
failure names the line it is about.

# The form

    type(scope): a sentence saying what changed

    Why it changed, wrapped, in prose. The subject says what a reader sees in
    a log; the body says what they cannot.

The prefix is machine-readable so release notes are a transform rather than an
afternoon. The rest is a sentence, because the log is read by people far more
often than by the release script.

# Scopes are read from the repository, not from a list here

A scope is a crate under `crates/` with the `vleo-` dropped, or one of the few
places that are not crates. A list maintained by hand goes stale the week
somebody adds a crate, and then the check is teaching people to pass it rather
than to be accurate.

# The adoption boundary

Everything up to and including `ADOPTED_AFTER` predates the rule. Retro-failing
history would
mean either rewriting it — which is the one thing a repository hand may never
do — or an unfixable red check, and an unfixable red check is one people learn
to scroll past.
"""

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

#: The last commit written before the rule existed. It and its ancestors are
#: exempt; everything after it is checked.
ADOPTED_AFTER = "59aff9f"

#: What a change can be. Short on purpose: a type nobody can choose between is
#: a type that gets chosen at random, and then the release notes are noise.
TYPES = {
    "feat": "new behaviour a user can see",
    "fix": "behaviour that was wrong is now right",
    "docs": "prose only — no code path changes",
    "test": "evidence only — a test, a fixture, a golden vector",
    "refactor": "the same behaviour, differently arranged",
    "perf": "the same answer, faster",
    "build": "the toolchain, the manifests, the generators' own plumbing",
    "ci": "the pipeline and the checks it runs",
    "chore": "everything with no reader — housekeeping",
}

SUBJECT_MAX = 72
BODY_MAX = 80

HEAD = re.compile(r"^(?P<type>[a-z]+)(?:\((?P<scope>[a-z0-9._-]+)\))?(?P<bang>!)?: (?P<text>.+)$")

#: A merge or a revert subject is written by git, not by a person.
GENERATED = re.compile(r"^(Merge |Revert |fixup! |squash! )")

#: Lines a body may exceed the wrap on: a trailer, a URL, a table, an indented
#: block. Re-wrapping any of those breaks them.
UNWRAPPABLE = re.compile(r"^(\s|\||[A-Za-z-]+: \S|\S+://|`)")


def scopes():
    """Every scope this repository actually has."""
    out = {"docs", "web", "tools", "agents", "ci", "xtask", "gate", "tree", "bundles"}
    for p in sorted((ROOT / "crates").glob("*/Cargo.toml")):
        out.add(p.parent.name.removeprefix("vleo-"))
    return out


def check(message, valid=None):
    """Every reason this message fails, in the order a reader meets them."""
    valid = scopes() if valid is None else valid
    lines = [l.rstrip() for l in message.strip("\n").split("\n")]
    # A comment block is what git puts under the subject in an editor.
    lines = [l for l in lines if not l.startswith("#")]
    while lines and not lines[-1]:
        lines.pop()
    bad = []
    if not lines or not lines[0].strip():
        return ["the message is empty"]
    subject = lines[0]
    if GENERATED.match(subject):
        return []

    m = HEAD.match(subject)
    if not m:
        return ["the subject is not 'type(scope): a sentence': %r" % subject]
    if m["type"] not in TYPES:
        bad.append(
            "'%s' is not a change type. One of: %s" % (m["type"], ", ".join(sorted(TYPES)))
        )
    if m["scope"] and m["scope"] not in valid:
        bad.append(
            "'%s' is not a scope in this repository. One of: %s"
            % (m["scope"], ", ".join(sorted(valid)))
        )
    text = m["text"]
    if len(subject) > SUBJECT_MAX:
        bad.append("the subject is %d characters; %d is the limit" % (len(subject), SUBJECT_MAX))
    if text.endswith("."):
        bad.append("the subject ends in a full stop — it is a title, not a sentence in prose")
    if text[:1].isupper() and not text.split()[0].isupper():
        bad.append("the subject starts with a capital: %r" % text.split()[0])

    if len(lines) > 1:
        if lines[1].strip():
            bad.append("there is no blank line between the subject and the body")
        for i, l in enumerate(lines[2:], start=3):
            if len(l) > BODY_MAX and not UNWRAPPABLE.match(l):
                bad.append("line %d is %d characters; wrap the body at %d" % (i, len(l), BODY_MAX))
    if m["bang"] and "BREAKING CHANGE:" not in message:
        bad.append("a '!' subject must say what breaks in a 'BREAKING CHANGE:' trailer")
    return bad


def git(*args):
    return subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=False
    ).stdout


def commits(rng):
    """(sha, message) for a range, minus everything at or before the boundary."""
    out = []
    for sha in git("rev-list", "--no-merges", rng).split():
        if _is_ancestor(sha, ADOPTED_AFTER):
            continue
        out.append((sha[:9], git("log", "-1", "--format=%B", sha)))
    return out


def _is_ancestor(a, b):
    return (
        subprocess.run(
            ["git", "merge-base", "--is-ancestor", a, b], cwd=ROOT, capture_output=True
        ).returncode
        == 0
    )


#: (message, how many reasons it should fail for). Each bad case breaks exactly
#: one thing about the good one, so a failure names the rule that caught it.
CASES = [
    ("fix(data): an incomplete bundle is refused, not warned about", 0),
    ("feat: the tree reads in the order it was written", 0),
    ("docs(web): how the panels are arranged\n\nA body, wrapped.\n", 0),
    ("Merge pull request #3 from somewhere/branch", 0),
    ("Revert \"fix(data): something\"", 0),
    ("an incomplete bundle is refused", 1),
    ("fix(data) an incomplete bundle is refused", 1),
    ("wibble(data): an incomplete bundle is refused", 1),
    ("fix(nosuchcrate): an incomplete bundle is refused", 1),
    ("fix(data): An incomplete bundle is refused", 1),
    ("fix(data): an incomplete bundle is refused.", 1),
    ("fix(data): " + "x" * 70, 1),
    ("fix(data): a subject\nthe body, with no blank line", 1),
    ("fix(data): a subject\n\n" + "y" * 90, 1),
    ("fix(data)!: the manifest fields changed", 1),
    ("fix(data)!: the manifest fields changed\n\nBREAKING CHANGE: old manifests fail.", 0),
    ("", 1),
    ("   \n\n  ", 1),
    # Long lines a body is allowed to keep: a trailer, a URL, a table row.
    ("fix(data): a subject\n\nCo-Authored-By: " + "z" * 80, 0),
    ("fix(data): a subject\n\nhttps://example.invalid/" + "z" * 80, 0),
    ("fix(data): a subject\n\n| a | " + "z" * 80 + " |", 0),
]


def selftest():
    """A checker nobody has watched fail is a checker nobody knows works."""
    valid = scopes()
    bad = 0
    for message, want in CASES:
        got = check(message, valid)
        ok = (len(got) == 0) == (want == 0)
        if not ok:
            bad += 1
            print("  FAIL %r" % message[:56])
            print("        expected %s, got %s" % ("a pass" if want == 0 else "a failure", got))
    print("selftest: %d cases, %s" % (len(CASES), "all as expected" if not bad else "%d FAILED" % bad))
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--file", help="a message file, as the commit-msg hook gets it")
    ap.add_argument("--range", help="a commit range, as the pipeline gets it")
    ap.add_argument("--selftest", action="store_true", help="check the checker")
    ap.add_argument("--types", action="store_true", help="print the change types and scopes")
    a = ap.parse_args()

    if a.selftest:
        return selftest()
    if a.types:
        for t, why in sorted(TYPES.items()):
            print("  %-9s %s" % (t, why))
        print("\nscopes: %s" % ", ".join(sorted(scopes())))
        return 0
    if a.file:
        bad = check(Path(a.file).read_text())
        for b in bad:
            print("commit message: %s" % b, file=sys.stderr)
        if bad:
            print("\nThe form is:  type(scope): a sentence saying what changed", file=sys.stderr)
            print("Types and scopes:  tools/commit_message.py --types", file=sys.stderr)
        return 1 if bad else 0
    if a.range:
        n = worst = 0
        for sha, message in commits(a.range):
            n += 1
            bad = check(message)
            if bad:
                worst = 1
                print("%s  %s" % (sha, message.split("\n")[0][:64]))
                for b in bad:
                    print("            %s" % b)
        print("%d commit(s) checked after %s, %s" % (n, ADOPTED_AFTER, "all good" if not worst else "see above"))
        return worst
    ap.print_help()
    return 2


if __name__ == "__main__":
    sys.exit(main())
