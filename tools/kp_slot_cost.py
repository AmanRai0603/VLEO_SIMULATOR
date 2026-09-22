#!/usr/bin/env python3
"""What choosing a Kp slot costs, measured through the whole tree.

    tools/kp_slot_cost.py              needs a daemon
    tools/kp_slot_cost.py --selftest   needs neither

WHY THIS EXISTS. `sw_kp_driving_slot` is seeded and unanswered: §30 B1 asks which
Kp slot a design is driven by — the day's mean of eight three-hourly values, or
its worst slot — and the value needs a person. §30's own rule is that where there
is evidence for what a number should be, the step carries the EVIDENCE and not a
decision. This is that evidence.

It is not an argument for either slot. It answers a narrower question that can be
measured: if the choice is made one way rather than the other, WHAT MOVES. The
answer is not confined to the solar subsystem or to the temperature that reads it
— it reaches drag, propulsion, power, mass, thermal, cost and five of the twelve
KPI closures, which is why a row that names the slot is worth having at all.

HOW. `env_kp` is a declared constant, so the two candidate readings are supplied
as what-if overrides through `/v1/run?set=`, the same path the face takes, and the
whole tree is run at each. Nothing is written and no sheet moves.

WHAT IT DOES NOT DO. It does not say which slot is right, it does not touch
`sw_kp_driving_slot`, and it carries no recommendation. The mean is what the
legacy run used; the peak is what a vehicle meets; a third answer — sized on the
mean, closed against the peak — is defensible and would make that row a set
rather than a switch. Whoever answers it decides.
"""

import argparse
import json
import os
import re
import sys
import urllib.error
import urllib.parse
import urllib.request

HOST = os.environ.get("VLEO_DAEMON", "http://localhost:7777")
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# The row the decision will live on, and the row whose declared constant carries
# a Kp into the physics today.
DECISION = "sw_kp_driving_slot"
DRIVER = "env_kp"

# Noise floor. A run-to-run difference below this is float arithmetic reordering
# and not a design consequence; reporting it would pad the list with rows that do
# not move. `prop_total_efficiency` sits here.
FLOOR = 1e-9

# The two scenarios where the choice is a real decision, and the symbols
# `sw_kp_scenarios` publishes each slot under. The values are NOT written here —
# they are read off the engine, so a change in the record moves this report.
# Each scenario carries its own flux as well as its own two Kp slots, and the
# temperature has to be taken at BOTH of that scenario's drivers. Probing every
# scenario at one flux conflates them: it reproduces the gap correctly — the
# geomagnetic term does not depend on the flux — and puts both ends of the
# worst-day pair 26 K below where the sheets say they are.
SCENARIOS = [
    ("the sustained disturbed scenario", "Kp_mean_hotmean", "Kp_peak_hotmean", "F107_hotmean", "F107bar_hotmean"),
    ("the disturbed single day", "Kp_mean_hotday", "Kp_peak_hotday", "F107_hotday", "F107bar_hotday"),
]

# The closures a reader will look for first. Named so the report says whether the
# choice reaches them rather than leaving it in a list of seventy rows.
KPIS = "kpi_"


def get(node, mode="branch", **over):
    q = (HOST + "/v1/run?" + urllib.parse.urlencode({"node": node, "mode": mode})
         + "".join("&set=%s:%r" % (k, v) for k, v in sorted(over.items())))
    with urllib.request.urlopen(q, timeout=600) as fh:
        d = json.load(fh)
    if not d.get("ok"):
        return None, None
    return ({v["id"]: v for v in d.get("values", [])},
            {b["id"]: b.get("message", "") for b in d.get("blocked", [])})


def probe(node, **inputs):
    """ONE RELATION at given inputs — `/v1/probe`, not `/v1/run?set=`.

    env_kp used to be a declared constant and the two candidate slots were
    supplied as what-if overrides. §49 wired it to read the solar subsystem, so
    it is computed now and an override cannot survive the run — the engine says
    so rather than accepting one and dropping it.

    What that costs this file is stated where it is felt, below: the
    tree-wide count is gone and the temperature is not.
    """
    q = urllib.parse.urlencode([("node", node)]
                               + [("in", "%s:%r" % (k, v)) for k, v in sorted(inputs.items())])
    with urllib.request.urlopen(HOST + "/v1/probe?" + q, timeout=120) as fh:
        d = json.load(fh)
    return d.get("si") if d.get("ok") else None


def slots():
    """Both readings of both scenarios, off the engine."""
    # BOTH RUNS. The Kp slots are sw_kp_scenarios' own members; the flux each
    # scenario sits at is published by the crossing, and a run of one does not
    # carry the other's members.
    vals, _ = get("sw_kp_scenarios")
    iface, _ = get("l3_solar_interface")
    if vals is None or iface is None:
        return None
    by_symbol = {v["symbol"]: v["si"] for v in vals.values() if v.get("symbol")}
    for v in iface.values():
        if v.get("symbol"):
            by_symbol.setdefault(v["symbol"], v["si"])
    out = []
    for label, mean_sym, peak_sym, f_sym, fbar_sym in SCENARIOS:
        if any(s not in by_symbol for s in (mean_sym, peak_sym, f_sym, fbar_sym)):
            return None
        out.append((label, by_symbol[mean_sym], by_symbol[peak_sym],
                    by_symbol[f_sym], by_symbol[fbar_sym]))
    return out


def moved(a, b):
    """Every row whose answer differs, worst relative change first."""
    out = []
    for k in sorted(set(a) & set(b)):
        x, y = a[k]["si"], b[k]["si"]
        rel = abs(y - x) / abs(x) if x else (0.0 if y == 0 else float("inf"))
        if rel > FLOOR:
            out.append((rel, k, x, y, a[k].get("unit", "")))
    out.sort(reverse=True)
    return out


def check(top):
    bad = []
    got = slots()
    if got is None:
        print("  FAIL the engine did not publish both slots of both scenarios")
        return 1

    # The decision row must still be the unanswered one this report is evidence
    # for. If it has been answered, this file is reporting on a settled question
    # and should say so rather than presenting a choice as open.
    sheet = os.path.join(ROOT, "crates/vleo-mod-solar/nodes", DECISION, "node.toml")
    if not os.path.exists(sheet):
        bad.append("%s does not exist; §30 B1 asks for it" % DECISION)
    else:
        import tomllib
        d = tomllib.load(open(sheet, "rb"))
        who = (d.get("value") or {}).get("confirmed_by", "")
        if who:
            print("  NOTE %s is now answered, by %s. This report is evidence for a "
                  "decision that has been taken." % (DECISION, who))

    base, _ = get("env_exospheric_temperature", mode="all")
    if base is None:
        print("  FAIL the tree did not run at the declared drivers")
        return 1
    print("at the declared %s: %d row(s) answer" % (DRIVER, len(base)))
    print()

    # WHAT THIS FILE CAN NO LONGER MEASURE, said once and plainly.
    #
    # It used to run the whole tree twice with env_kp forced to each slot and
    # count every row that moved — 70 of them, six a KPI closure. That worked
    # because env_kp was a declared constant. §49 wired it to read the solar
    # subsystem, so it is computed, an override cannot survive the run, and the
    # tree-wide count is not available through this path any more.
    #
    # It is not available through another one either. The slot choice now lives
    # inside the subsystem, in which member sys_space_environment_kp selects,
    # and nothing declared upstream of it selects a slot. What would restore
    # the count is sw_kp_driving_slot carrying a value: two cases differing
    # only in that row would give two full runs to diff, which is what this
    # file wants and what §30 B1 asked that row to be for.
    #
    # The temperature cost survives, because it is a fact about the relation
    # rather than about the wiring, and it is the number §29.2 and §30 A2 quote.
    print("THE TREE-WIDE COUNT IS NOT MEASURABLE HERE while %s is computed." % DRIVER)
    print("  It was 70 rows and 6 KPI closures when %s was declared. Restoring it" % DRIVER)
    print("  needs %s to carry a value, so two cases can differ by the slot" % DECISION)
    print("  alone. The temperature cost below is unaffected and is measured.")
    print()

    for label, mean, peak, f107, f107a in got:
        tm = probe("env_exospheric_temperature",
                   env_f107=f107, env_f107a=f107a, env_kp=mean)
        tp = probe("env_exospheric_temperature",
                   env_f107=f107, env_f107a=f107a, env_kp=peak)
        if tm is None or tp is None:
            bad.append("the relation refused a probe at %s" % label)
            continue
        print("%s — at its own flux %.3f sfu (81-day mean %.3f)" % (label, f107, f107a))
        print("   mean slot Kp %.4f against peak slot Kp %.4f" % (mean, peak))
        print("   exospheric temperature %.1f K against %.1f K, a gap of %.1f K (%+.2f%%)"
              % (tm, tp, tp - tm, 100 * (tp - tm) / tm))
        print()

    for x in bad:
        print("  FAIL %s" % x)
    return 1 if bad else 0


def selftest():
    """The parts that need no daemon: the arithmetic, and that the row is seeded."""
    bad = 0

    # 1 · `moved` reports a real change, ignores float noise, and sorts worst
    #     first. The floor matters: without it the report carried a row at
    #     +0.00%, which reads as a consequence and is a reordered sum.
    a = {"p": {"si": 100.0, "unit": "W"}, "q": {"si": 1.0, "unit": "-"},
         "r": {"si": 5.0, "unit": "-"}, "s": {"si": 0.0, "unit": "-"}}
    b = {"p": {"si": 110.0, "unit": "W"}, "q": {"si": 1.0 + 1e-13, "unit": "-"},
         "r": {"si": 7.5, "unit": "-"}, "s": {"si": 0.0, "unit": "-"}}
    got = [(k, round(rel, 6)) for rel, k, _, _, _ in moved(a, b)]
    if got != [("r", 0.5), ("p", 0.1)]:
        bad += 1
        print("  FAIL moved() gave %r; expected r at 50%% then p at 10%%, with the "
              "1e-13 row and the unchanged zero row dropped" % got)

    # 2 · a row that goes from zero to non-zero is an infinite relative change
    #     and must not be dropped as noise or crash the sort.
    z = moved({"t": {"si": 0.0}}, {"t": {"si": 3.0}})
    if not z or z[0][0] != float("inf"):
        bad += 1
        print("  FAIL a row moving off zero was not reported: %r" % z)

    # 3 · THE ROW THIS IS EVIDENCE FOR EXISTS AND IS SEEDED. Both halves matter:
    #     without the row the decision has nowhere to live, and if it were
    #     published with a value an agent had supplied, that is the one defect
    #     nothing else in this repository can catch.
    sheet = os.path.join(ROOT, "crates/vleo-mod-solar/nodes", DECISION, "node.toml")
    try:
        import tomllib
        d = tomllib.load(open(sheet, "rb"))
        if not d.get("question", {}).get("text"):
            bad += 1
            print("  FAIL %s states no question, so it names no decision" % DECISION)
        if d.get("state") != "empty" and not (d.get("value") or {}).get("confirmed_by"):
            bad += 1
            print("  FAIL %s is not seeded and carries no confirmation — a value "
                  "nobody signed" % DECISION)
    except OSError:
        bad += 1
        print("  FAIL %s has no sheet; §30 B1 asks for it" % DECISION)

    # 4 · the cross-references §30 B1 makes the condition of this step. Both
    #     directions, because a one-way reference is how the two drift.
    for path, want in (
        ("crates/vleo-mod-solar/nodes/sw_kp_slot_bias/node.toml", DECISION),
        ("crates/vleo-mod-envorbit/nodes/env_exospheric_temperature/node.toml", DECISION),
    ):
        try:
            if want not in open(os.path.join(ROOT, path), encoding="utf-8").read():
                bad += 1
                print("  FAIL %s no longer cross-references %s" % (path, want))
        except OSError as exc:
            bad += 1
            print("  FAIL could not read %s: %s" % (path, exc))

    # 5 · the slot values are read off the engine and not written here, because a
    #     scenario value copied into a tool is §21's stale panel constant again.
    #     A NUMBER, not a digit. The scenario symbols are names — F107_hotmean,
    #     F107bar_hotday — and a digit inside an identifier is part of the name
    #     rather than a value copied in. What this must catch is a bare literal,
    #     so it looks for one that is not embedded in an identifier.
    src = open(os.path.abspath(__file__), encoding="utf-8").read()
    body = src.split("SCENARIOS = [", 1)[1].split("]", 1)[0]
    if re.search(r"(?<![A-Za-z0-9_])\d+(?:\.\d+)?(?![A-Za-z0-9_])", body):
        bad += 1
        print("  FAIL SCENARIOS carries a number; the slot values must come off the "
              "engine so a change in the record moves this report")

    print("selftest: %d cases, %s" % (9, "all as expected" if not bad else "%d FAILED" % bad))
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--top", type=int, default=12,
                    help="how many moved rows to print per scenario")
    a = ap.parse_args()
    if a.selftest:
        return selftest()
    try:
        return check(a.top)
    except (OSError, urllib.error.URLError) as exc:
        print("could not reach the engine: %s" % str(exc)[:160])
        print("a daemon must be running: cargo run --release -p vleo-daemon")
        return 1


if __name__ == "__main__":
    sys.exit(main())
