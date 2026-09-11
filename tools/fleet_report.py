#!/usr/bin/env python3
"""What the fleet actually costs, and which agents actually fire.

    tools/fleet_report.py
    tools/fleet_report.py --since 2026-09-01
    tools/fleet_report.py --selftest

Gaps W6 and W7. Token cost per agent and firing rate were both declared as
things to watch and neither had an instrument behind it — and a target with no
meter behind it is an intention. The consequence is specific: no agent can be
shown to earn its keep, so none is ever removed, and a definition that loads
into context and never runs is pure cost that nothing reports.

Everything here is read from the harness's own local session logs. Nothing is
sent anywhere and nothing is estimated from character counts.

# What it can and cannot attribute

**Exactly:** how many times each agent was invoked, when it last fired, and
every token the main session spent, by model.

**Not:** the tokens a subagent spent inside its own run. Those go to its own
context and the parent's log records the invocation and the result, not the
usage. So per-agent cost here is *invocations*, not tokens, and saying so is
the point — a number labelled "cost per agent" that is actually a guess from
character counts is worse than an honest count, because somebody will bill
against it.

**Deliberately not money.** A currency figure needs a price table that moves
without telling anybody. Tokens are the thing this repository can state as
fact.
"""

import argparse
import collections
import json
import os
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LOGS = Path(os.path.expanduser("~/.claude/projects"))


def roster():
    return [a["name"] for a in tomllib.loads((ROOT / "agents" / "lanes.toml").read_text())["agent"]]


def register():
    """What the fleet owner decided about each definition."""
    try:
        d = tomllib.loads((ROOT / "agents" / "provenance.toml").read_text())
    except Exception:
        return {}
    return {a["name"]: a for a in d.get("agent", [])}


def logs():
    return sorted(LOGS.glob("*/*.jsonl")) if LOGS.is_dir() else []


def read(paths, since=None):
    """(fired, tokens, sessions, turns) from the session logs."""
    fired = collections.Counter()
    last = {}
    tokens = collections.defaultdict(collections.Counter)
    sessions = set()
    turns = 0
    for p in paths:
        for line in p.read_text(errors="replace").splitlines():
            try:
                d = json.loads(line)
            except ValueError:
                continue
            ts = d.get("timestamp", "")
            if since and ts and ts[:10] < since:
                continue
            if d.get("sessionId"):
                sessions.add(d["sessionId"])
            if d.get("type") != "assistant":
                continue
            m = d.get("message", {})
            u = m.get("usage") or {}
            if u:
                turns += 1
                model = m.get("model", "unknown")
                for k in ("input_tokens", "output_tokens",
                          "cache_creation_input_tokens", "cache_read_input_tokens"):
                    tokens[model][k] += u.get(k) or 0
            for c in m.get("content", []) or []:
                if c.get("type") == "tool_use" and c.get("name") in ("Agent", "Task"):
                    name = (c.get("input") or {}).get("subagent_type", "?")
                    fired[name] += 1
                    if ts > last.get(name, ""):
                        last[name] = ts
    return fired, last, tokens, sessions, turns


def render(fired, last, tokens, sessions, turns, names, reg=None):
    reg = register() if reg is None else reg
    out = []
    out.append("firing rate — an agent that never fires is pure cost")
    out.append("")
    out.append("  %-24s %8s  %-12s %s" % ("agent", "fired", "last", "expected from"))
    waiting, unexplained = [], []
    for n in names:
        exp = (reg.get(n) or {}).get("expected_from", "")
        if fired.get(n):
            out.append("  %-24s %8d  %-12s %s" % (n, fired[n], last.get(n, "")[:10], exp))
        else:
            out.append("  %-24s %8s  %-12s %s" % (n, "0", "never", exp or "\x1b[31mnot recorded\x1b[0m"))
            (waiting if exp else unexplained).append(n)
    for n in sorted(set(fired) - set(names)):
        out.append("  %-24s %8d  %-12s (not on the roster)" % (n, fired[n], last.get(n, "")[:10]))

    if waiting:
        out.append("")
        out.append("  %d silent, each kept on purpose:" % len(waiting))
        for n in waiting:
            out.append("    %s — waits for: %s" % (n, reg[n]["expected_from"]))
            why = reg[n].get("why_kept", "")
            if why:
                for line in _wrap(why, 74):
                    out.append("      %s" % line)
        out.append("")
        out.append("  Silent before its own expected_from is an agent waiting, defined, for work")
        out.append("  that has not begun. Silent past it is a finding.")
    if unexplained:
        out.append("")
        out.append("  %d silent with no decision recorded: %s." % (len(unexplained), ", ".join(unexplained)))
        out.append("  Each still loads into context. Delete it, merge it into another, or record")
        out.append("  expected_from in agents/provenance.toml — those are the only three honest")
        out.append("  answers, and the third one has to be written down or nobody decided.")

    out.append("")
    out.append("tokens — the main session only; a subagent's own run is not attributable here")
    out.append("")
    out.append("  %-22s %12s %12s %12s %12s" % ("model", "in", "out", "cache write", "cache read"))
    total = collections.Counter()
    for model, t in sorted(tokens.items()):
        out.append("  %-22s %12d %12d %12d %12d" % (
            model, t["input_tokens"], t["output_tokens"],
            t["cache_creation_input_tokens"], t["cache_read_input_tokens"]))
        total.update(t)
    if len(tokens) > 1:
        out.append("  %-22s %12d %12d %12d %12d" % (
            "total", total["input_tokens"], total["output_tokens"],
            total["cache_creation_input_tokens"], total["cache_read_input_tokens"]))
    out.append("")
    out.append("  %d session(s), %d model turn(s)." % (len(sessions), turns))
    out.append("  No money figure: a currency number needs a price table that moves without")
    out.append("  telling anybody, and tokens are what this repository can state as fact.")
    return "\n".join(out)


def _wrap(text, n):
    words, line, out = text.split(), "", []
    for w in words:
        if len(line) + len(w) + 1 > n:
            out.append(line)
            line = w
        else:
            line = (line + " " + w).strip()
    if line:
        out.append(line)
    return out


#: (fired, roster, register, needle, present)
REG = {"test-author": {"expected_from": "the first declared property"}}
CASES = [
    ({"hole-filler": 3}, ["hole-filler", "test-author"], REG, "test-author", True),
    # Silent with a recorded expectation is a decision, not a finding.
    ({"hole-filler": 3}, ["hole-filler", "test-author"], REG, "kept on purpose", True),
    ({"hole-filler": 3}, ["hole-filler", "test-author"], REG, "no decision recorded", False),
    # Silent with nothing recorded is the finding W7 exists for.
    ({"hole-filler": 3}, ["hole-filler", "test-author"], {}, "no decision recorded", True),
    ({"hole-filler": 3}, ["hole-filler", "test-author"], {}, "kept on purpose", False),
    ({"hole-filler": 3, "test-author": 1}, ["hole-filler", "test-author"], {}, "silent", False),
    ({"ghost": 2}, ["hole-filler"], {}, "not on the roster", True),
]


def selftest():
    bad = 0
    for fired, names, reg, needle, want in CASES:
        text = render(collections.Counter(fired), {}, {}, set(), 0, names, reg)
        if (needle in text) != want:
            bad += 1
            print("  FAIL %r expected %s in:\n%s" % (needle, "present" if want else "absent", text))
    # An agent that never fired must be named, not merely counted.
    text = render(collections.Counter({"hole-filler": 1}), {}, {}, set(), 0, roster())
    for n in roster():
        if n not in text:
            bad += 1
            print("  FAIL %s is on the roster and not in the report" % n)
    # Every roster agent carries a decision. Without this the report goes quiet
    # the moment somebody adds an agent and forgets, which is the failure W7
    # names: a definition that loads and never runs, and nothing says so.
    reg = register()
    for n in roster():
        if not (reg.get(n) or {}).get("expected_from"):
            bad += 1
            print("  FAIL %s has no expected_from — its silence would mean nothing" % n)
    print("selftest: %d cases, %s" % (len(CASES) + 2, "all as expected" if not bad else "%d FAILED" % bad))
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--since", metavar="YYYY-MM-DD")
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()
    if a.selftest:
        return selftest()
    paths = logs()
    if not paths:
        print("no session logs under %s." % LOGS)
        print("The instrument reads the harness's own logs; with none there is nothing to report,")
        print("which is different from a fleet that costs nothing.")
        return 0
    fired, last, tokens, sessions, turns = read(paths, a.since)
    print(render(fired, last, tokens, sessions, turns, roster()))
    return 0


if __name__ == "__main__":
    sys.exit(main())
