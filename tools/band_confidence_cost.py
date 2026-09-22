#!/usr/bin/env python3
"""What the design band's confidence costs, measured through the whole tree.

    tools/band_confidence_cost.py            needs a daemon
    tools/band_confidence_cost.py --selftest needs neither

WHY THIS EXISTS. `sw_band_confidence` is seeded and unanswered: §30 B2 asks what
confidence the band is at, and the value needs a person. §30's own rule is that
where there is evidence for what a number should be, the step carries the
EVIDENCE and not a decision. This is that evidence, and it carries no
recommendation.

THE CONTRADICTION IS IN THE SOURCE. The legacy run's own header, recorded
verbatim in matlab/reference/mission_drivers.csv, says the run is at 95 per cent
confidence and that its mean band is centre +/- 1.28*sigma. Phi(1.28) = 0.8997.
The daily half of the same band does use 0.95. So the two halves of one scenario
are at different confidences and neither is at the label's.

WHAT THIS CAN AND CANNOT MEASURE, STATED FIRST. The multiplier is a LITERAL IN
FOUR HOLE BODIES and not a declared row — that is the whole of what §30 B2 asks
to fix — so it cannot be moved with a what-if override the way a declared value
can. `/v1/run?set=` on a computed row is accepted and silently ignored, which is
worth knowing on its own.

So the downstream figures below were measured by editing those four holes in a
working tree, rebuilding, running the whole tree, and reverting. They are
recorded here as a table and CHECKED against the arithmetic that produces them:
the four band rows are centre +/- z*sigma and nothing else, so given the engine's
own centre and sigma this file recomputes what each row becomes and fails if the
recorded figure has drifted. What it cannot recompute — the rows further down
that read those four — is marked as recorded rather than derived, and says so.
"""

import argparse
import csv
import json
import os
import sys
import urllib.error
import urllib.parse
import urllib.request

HOST = os.environ.get("VLEO_DAEMON", "http://localhost:7777")
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DECISION = "sw_band_confidence"
REF = os.path.join(ROOT, "matlab/reference/mission_drivers.csv")

#: The multiplier as the four holes carry it, and the one-sided 95 per cent
#: alternative §30 B2 names. Neither is a recommendation.
Z_NOW = 1.28
Z_95 = 1.645

#: The four rows the multiplier is written into, as (row, centre, sigma, sign).
#: The sign is which way the band goes; everything else about each row is read
#: off the engine.
BAND = [
    ("sw_f107_design_long", "sw_central_expectation", "sw_mean_band_spread", +1),
    ("sw_f107_cold_long", "sw_central_expectation", "sw_mean_band_spread", -1),
    ("sw_ap_design_long", "sw_ap_central_expectation", "sw_ap_mean_band_spread", +1),
    ("sw_ap_cold_long", "sw_ap_central_expectation", "sw_ap_mean_band_spread", -1),
]

#: What moving z to 1.645 was measured to do, by editing the four holes in a
#: working tree and running everything. Recorded, not derived — this file cannot
#: rebuild the engine. The four band rows above ARE derived and are checked.
RECORDED = {
    "rows that move": 47,
    "kpi closures that move": 0,
    "closures that stop closing": 0,
    "worst margin lost, per cent": 6.2,
}


def get(node, mode="branch"):
    q = HOST + "/v1/run?" + urllib.parse.urlencode({"node": node, "mode": mode})
    with urllib.request.urlopen(q, timeout=600) as fh:
        d = json.load(fh)
    return {v["id"]: v for v in d.get("values", [])} if d.get("ok") else None


def reference():
    """The legacy run's own answers, and the sigma implied by its own band."""
    rows = {}
    with open(REF, encoding="utf-8") as fh:
        for r in csv.DictReader(l for l in fh if not l.startswith("#")):
            rows[r["scenario"]] = {k: float(v) for k, v in r.items() if k != "scenario"}
    return rows


def label_says_95():
    """The header claims 95 per cent, and the construction says 1.28 sigma.

    Both sentences are in one file and they disagree. Read rather than asserted,
    because it is the single strongest piece of evidence for this decision and a
    reworded header would quietly remove it.
    """
    head = "".join(l for l in open(REF, encoding="utf-8") if l.startswith("#"))
    return "95% confidence" in head, "1.28*sigma_total" in head


def check():
    bad = []
    conf, band = label_says_95()
    print("the legacy run's own header: says 95%% confidence = %s; "
          "says the band is 1.28 sigma = %s" % (conf, band))
    if not (conf and band):
        bad.append("mission_drivers.csv no longer states both halves of the "
                   "contradiction this decision is about")

    vals = get("l3_solar_interface", mode="all")
    if vals is None:
        print("  FAIL the tree did not run")
        return 1

    print()
    print("the four rows the multiplier is written into, at z = %.3f and %.3f:" % (Z_NOW, Z_95))
    for row, centre, sigma, sign in BAND:
        for k in (row, centre, sigma):
            if k not in vals:
                bad.append("%s did not answer, so the band cannot be checked" % k)
                break
        else:
            c, s, got = vals[centre]["si"], vals[sigma]["si"], vals[row]["si"]
            want = c + sign * Z_NOW * s
            # THE ARITHMETIC IS THE CHECK. If the engine's row is not its own
            # centre plus or minus z sigma, the relation has changed and every
            # figure below it is about a relation that is gone.
            if abs(got - want) > 1e-6 * max(1.0, abs(want)):
                bad.append("%s answers %.6f and centre %s %.3f*sigma is %.6f — the "
                           "band relation has changed" % (row, got, "+-"[sign < 0], Z_NOW, want))
                continue
            at95 = c + sign * Z_95 * s
            print("   %-22s %10.4f -> %-10.4f  %+8.4f   (centre %.4f, sigma %.4f)"
                  % (row, got, at95, at95 - got, c, s))

    print()
    print("what moving to z = %.3f was measured to do, over the whole tree:" % Z_95)
    for k, v in RECORDED.items():
        print("   %-32s %s" % (k, v))
    print("   (recorded from a working-tree run, not recomputed here — the")
    print("    multiplier is a hole literal and no override can move it)")

    print()
    print("and what it does NOT touch: no parity check in this repository")
    print("compares a band value, so none of them breaks at either z.")

    for x in bad:
        print("  FAIL %s" % x)
    return 1 if bad else 0


def selftest():
    """The parts that need no daemon."""
    bad = 0

    # 1 · THE EVIDENCE IS STILL IN THE FILE IT IS QUOTED FROM. Both halves: the
    #     label and the construction. This is the whole basis of the decision.
    try:
        conf, band = label_says_95()
        if not conf:
            bad += 1
            print("  FAIL mission_drivers.csv no longer records the run's 95%% label")
        if not band:
            bad += 1
            print("  FAIL mission_drivers.csv no longer records the 1.28 sigma band")
    except OSError as exc:
        bad += 1
        print("  FAIL could not read the reference run: %s" % exc)

    # 2 · AND THE LEGACY RUN'S OWN NUMBERS STILL IMPLY 1.28. Its hotmean minus
    #     its nominal, over the sigma its parity file records, is the multiplier
    #     it used — a second statement of the same fact, from the data rather
    #     than the prose, so a reworded header cannot take the evidence with it.
    try:
        ref = reference()
        got = (ref["hotmean"]["f107"] - ref["nominal"]["f107"]) / 13.496881576668528
        if abs(got - Z_NOW) > 5e-4:
            bad += 1
            print("  FAIL the reference run's own band implies z = %.4f, not %.2f"
                  % (got, Z_NOW))
        got_ap = (ref["hotmean"]["ap"] - ref["nominal"]["ap"]) / 3.5865545525358034
        if abs(got_ap - Z_NOW) > 5e-4:
            bad += 1
            print("  FAIL the reference run's Ap band implies z = %.4f, not %.2f"
                  % (got_ap, Z_NOW))
    except (OSError, KeyError, ZeroDivisionError) as exc:
        bad += 1
        print("  FAIL could not read the reference scenarios: %s" % exc)

    # 2b · AND EACH ROW'S SIGN IS THE ONE ITS NAME CLAIMS. A cold edge recorded
    #      as an addition prints a band that never existed and still looks like a
    #      band: the numbers stay plausible and both edges land on the hot side.
    #      The first version of this case compared the reference run's hotmean
    #      against nominal + 1.28 sigma, which is true whatever this table says,
    #      so it passed against a flipped sign and proved nothing.
    for row, _, _, sign in BAND:
        cold = "_cold_" in row
        if cold != (sign < 0):
            bad += 1
            print("  FAIL %s is the %s edge and the table gives it sign %+d"
                  % (row, "cold" if cold else "hot", sign))

    # 3 · THE ROW THIS IS EVIDENCE FOR EXISTS AND IS SEEDED. If it were published
    #     with a value an agent supplied, that is the one defect nothing else in
    #     this repository can catch.
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
        print("  FAIL %s has no sheet; §30 B2 asks for it" % DECISION)

    # 4 · THE FOUR HOLES STILL CARRY THE MULTIPLIER, and still say it is declared
    #     in the sheet. Both halves matter: the first is what this row would
    #     replace, and the second is the claim that is not yet true.
    for row, _, _, _ in BAND:
        p = os.path.join(ROOT, "crates/vleo-mod-solar/nodes", row, "model.rs")
        try:
            src = open(p, encoding="utf-8").read()
            if "spread * %s;" % Z_NOW not in src:
                bad += 1
                print("  FAIL %s no longer multiplies the spread by %s — this file's "
                      "whole subject has moved" % (row, Z_NOW))
        except OSError as exc:
            bad += 1
            print("  FAIL could not read %s: %s" % (row, exc))

    # 5 · AND THE ROW IS CROSS-REFERENCED FROM WHERE THE NUMBER LIVES, so the
    #     two cannot drift.
    for row, _, _, _ in BAND:
        p = os.path.join(ROOT, "crates/vleo-mod-solar/nodes", row, "node.toml")
        try:
            if DECISION not in open(p, encoding="utf-8").read():
                bad += 1
                print("  FAIL %s does not cross-reference %s" % (row, DECISION))
        except OSError as exc:
            bad += 1
            print("  FAIL could not read %s's sheet: %s" % (row, exc))

    # 6 · NO PARITY CHECK COMPARES A BAND VALUE. This is the finding that
    #     corrects §30 B2's cost table, and it is a fact about two files rather
    #     than an opinion: if either check ever starts comparing one, the cost of
    #     this decision changes and this line must be re-read.
    for tool in ("mat_parity.py", "matlab_parity.py"):
        src = open(os.path.join(ROOT, "tools", tool), encoding="utf-8").read()
        named = [r for r, _, _, _ in BAND if r in src]
        if named:
            bad += 1
            print("  FAIL %s now names %s. A parity check that compares a band "
                  "value makes moving the confidence cost parity, which §44.3 "
                  "says it does not — re-measure before trusting that section"
                  % (tool, ", ".join(named)))

    print("selftest: %d cases, %s" % (18, "all as expected" if not bad else "%d FAILED" % bad))
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()
    if a.selftest:
        return selftest()
    try:
        return check()
    except (OSError, urllib.error.URLError) as exc:
        print("could not reach the engine: %s" % str(exc)[:160])
        print("a daemon must be running: cargo run --release -p vleo-daemon")
        return 1


if __name__ == "__main__":
    sys.exit(main())
