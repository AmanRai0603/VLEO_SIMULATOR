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


def render(fired, last, tokens, sessions, turns, names):
    out = []
    out.append("firing rate — an agent that never fires is pure cost")
    out.append("")
    out.append("  %-24s %8s  %s" % ("agent", "fired", "last"))
    silent = []
    for n in names:
        if fired.get(n):
            out.append("  %-24s %8d  %s" % (n, fired[n], last.get(n, "")[:10]))
        else:
            silent.append(n)
            out.append("  %-24s %8s  %s" % (n, "0", "never"))
    for n in sorted(set(fired) - set(names)):
        out.append("  %-24s %8d  %s   (not on the roster)" % (n, fired[n], last.get(n, "")[:10]))
    if silent:
        out.append("")
        out.append("  %d never fired: %s." % (len(silent), ", ".join(silent)))
        out.append("  Each still loads into context. Delete it, merge it into another, or")
        out.append("  say why it is being kept — those are the only three honest answers.")

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


CASES = [
    # (fired counter, roster, needle, present)
    ({"hole-filler": 3}, ["hole-filler", "test-author"], "test-author", True),
    ({"hole-filler": 3}, ["hole-filler", "test-author"], "1 never fired", True),
    ({"hole-filler": 3, "test-author": 1}, ["hole-filler", "test-author"], "never fired", False),
    ({"ghost": 2}, ["hole-filler"], "not on the roster", True),
]


def selftest():
    bad = 0
    for fired, names, needle, want in CASES:
        text = render(collections.Counter(fired), {}, {}, set(), 0, names)
        if (needle in text) != want:
            bad += 1
            print("  FAIL %r expected %s in:\n%s" % (needle, "present" if want else "absent", text))
    # An agent that never fired must be named, not merely counted.
    text = render(collections.Counter({"hole-filler": 1}), {}, {}, set(), 0, roster())
    for n in roster():
        if n not in text:
            bad += 1
            print("  FAIL %s is on the roster and not in the report" % n)
    print("selftest: %d cases, %s" % (len(CASES) + 1, "all as expected" if not bad else "%d FAILED" % bad))
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
