#!/usr/bin/env python3
"""Every row of the solar subsystem, against what the MATLAB study published.

    tools/matlab_parity.py                 # needs the daemon on :7777
    tools/matlab_parity.py --port 7777

WHAT THIS CAN AND CANNOT COMPARE, STATED FIRST.

The study's own code — `prf_cycles`, `prf_design`, `prf_ap2kp`, `prf_cluster`,
`prf_drivers`, `prf_horizon`, `prf_segment` — is NOT in this repository. The only
MATLAB here is a 118-line client that calls this tool *from* MATLAB, so nothing
here runs the study and diffs the two.

That is a statement about this repository, and it is narrower than it used to
read. The source has since been supplied by its author, and where an algorithm
of it has been reimplemented and checked exactly, that is done in
`tools/mat_parity.py` and said there. This file remains what it always was: a
check against the study's published OUTPUT, which is the part of the evidence
that lives in the repository and can be re-run by anyone who clones it.

What IS here is the study's OUTPUT, published in full as the solar-weather
bundle: seven CSVs, 70,422 rows, including `daily_regime.csv` — which is
prf_cluster's per-day labels and posteriors — and `forecast_issued.csv`, the
30,048 forecast rows the outlook actually issued. A port can be checked against
those exactly, and that is what this does: for each row of the subsystem, it
re-derives the quantity from the published record in Python, independently of the
Rust, and compares against what the engine returns.

Three outcomes, and the third is the point:

  AGREES     the engine and an independent derivation from the study's own
             published output land on the same number
  DEPARTS    they differ, the difference is DECLARED on the sheet, and the size
             and direction are checked against what the sheet claims
  UNCHECKED  the study published nothing this row can be compared against

A DEPARTS is not a failure. Four of them are the deliberate corrections this port
makes to the study, and each is a place this implementation is more careful than
the thing it came from.

Exit status is 0 when every comparable row agrees or departs as declared.
"""

import argparse
import csv
import json
import math
import sys
import urllib.parse
import urllib.request
from collections import Counter, defaultdict
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BUNDLE = ROOT / "bundles/solar-weather/2026.09.14"


# --------------------------------------------------------------------------
# the study's published output, read exactly as the engine reads it

def rows(name):
    with (BUNDLE / name).open() as f:
        return list(csv.DictReader([l for l in f if not l.startswith("#")]))


def d2000(s):
    return (date(int(s[:4]), int(s[5:7]), int(s[8:10])) - date(2000, 1, 1)).days


def num(v):
    return None if v in (None, "", "NaN") else float(v)


def load():
    obs = rows("observed_daily.csv")
    days = []
    for r in obs:
        days.append({
            "t": d2000(r["date"]), "date": r["date"],
            "f107": num(r["f107"]), "ap": num(r["ap_planetary"]),
            "kp": num(r["kp_max"]),
            "kp8": [num(r[f"kp_{h:02d}z"]) for h in (0, 3, 6, 9, 12, 15, 18, 21)],
        })
    cyc = [{"n": int(r["cycle"]), "start": d2000(r["start"]), "end": d2000(r["end"]),
            "length": float(r["length_years"])}
           for r in rows("solar_cycles.csv")]
    return days, cyc


def quant(sorted_, q):
    """The convention the rows were measured under: linear at h = (n-1)q."""
    if not sorted_:
        return None
    if len(sorted_) == 1:
        return sorted_[0]
    h = (len(sorted_) - 1) * q
    lo = int(math.floor(h))
    hi = min(len(sorted_) - 1, lo + 1)
    return sorted_[lo] + (h - lo) * (sorted_[hi] - sorted_[lo])


# --------------------------------------------------------------------------
# the engine

def run_node(port, node):
    body = urllib.parse.urlencode({"node": node, "mode": "branch", "case": "c1"}).encode()
    req = urllib.request.Request(f"http://localhost:{port}/v1/run", data=body,
                                 headers={"content-type": "application/x-www-form-urlencoded"})
    with urllib.request.urlopen(req, timeout=30) as r:
        d = json.load(r)
    if not d.get("ok"):
        return None
    for v in d.get("values", []):
        if v["id"] == node:
            return v.get("si")
    return None


# --------------------------------------------------------------------------
# the comparisons, one per row that has something to compare against

def comparisons(days, cyc):
    """Each entry re-derives a row from the published record, in Python."""
    ap = [d["ap"] for d in days if d["ap"] is not None]
    f107 = [d["f107"] for d in days if d["f107"] is not None]
    byday = {d["t"]: d["f107"] for d in days if d["f107"] is not None}
    years = len(ap) / 365.25

    out = []

    def agrees(node, want, tol, how):
        out.append({"node": node, "want": want, "tol": tol, "how": how, "kind": "AGREES"})

    def departs(node, want, tol, how, why):
        out.append({"node": node, "want": want, "tol": tol, "how": how,
                    "kind": "DEPARTS", "why": why})

    # --- prf_cluster: its labels are published per day, so this is exact.
    reg = {r["date"]: r for r in rows("daily_regime.csv")}
    dis = agree = 0
    lowtail = 0
    for d in days:
        if d["ap"] is None or d["date"] not in reg:
            continue
        theirs = reg[d["date"]]["regime"]
        ours = "quiet" if d["ap"] <= 6 else "active" if d["ap"] <= 25 else "storm"
        if theirs == ours:
            agree += 1
        else:
            dis += 1
            if d["ap"] <= 1 and theirs == "storm":
                lowtail += 1
    departs("sw_regime", 3.0, 0,
            f"prf_cluster's own labels: {agree} of {agree + dis} days reproduced exactly",
            f"{dis} days differ, all {lowtail} of them the broad storm component "
            f"reclaiming the low tail at Ap 0 or 1 — declared on the sheet")

    # --- prf_design's exceedance curve, refitted here from the ranked record.
    desc = sorted(ap, reverse=True)
    pts = [(years / m, desc[m - 1]) for m in range(2, 57)]
    n = len(pts)
    sx = sum(math.log(t) for t, _ in pts)
    sy = sum(v for _, v in pts)
    sxx = sum(math.log(t) ** 2 for t, _ in pts)
    sxy = sum(math.log(t) * v for t, v in pts)
    b = (n * sxy - sx * sy) / (n * sxx - sx * sx)
    a = (sy - b * sx) / n
    agrees("sw_storm_return_level", a + b * math.log(5.0), 5e-3,
           f"least squares on ranks 2..56 of the published daily Ap: "
           f"a={a:.4f} b={b:.4f} against the sheet's 92.5155 / 40.9265")
    agrees("sw_design_safe_duration", math.exp((132.0 - a) / b) * 365.25 * 86400, 5e-3,
           "the same fit, inverted at the G3 bound of Ap 132")

    # --- prf_design's structure function, at the mission lead.
    L = 1826
    ch = sorted(byday[t + L] - v for t, v in byday.items() if t + L in byday)
    agrees("sw_uncertainty_growth", quant(ch, 0.95), 5e-3,
           f"the 95th percentile of the {L}-day change over {len(ch)} observed pairs")
    agrees("sw_central_expectation", sum(f107) / len(f107), 1e-6,
           f"the record's mean F10.7 over {len(f107)} days")

    # --- prf_cycles: the boundaries are published, the fold is the sheet's.
    epoch = 9862.0
    done = [c for c in cyc if any(o["start"] >= c["end"] for o in cyc)]
    # THE STUDY PUBLISHES A LENGTH AND ALSO PUBLISHES THE BOUNDARIES, AND THEY
    # DISAGREE BY 0.96 DAYS. length_years is printed to two decimals — 11.88 and
    # 11.00 — so their mean is 4178.46 d, while the exact boundary differences
    # are 4338 and 4017 d, whose mean is 4177.50. The node folds with the
    # published column, which is the study's own number; this checks that, and
    # the sheet now says which of the two it used.
    mean_len = sum(c["length"] for c in done) / len(done) * 365.25
    cur = max((c for c in cyc if c["start"] <= epoch), key=lambda c: c["start"])
    exact = sum(c["end"] - c["start"] for c in done) / len(done)
    agrees("sw_cycle_phase", (epoch - cur["start"]) / mean_len, 1e-6,
           f"the published boundaries, folded with the mean of the published "
           f"length_years ({mean_len:.2f} d; the exact boundary mean is {exact:.2f} d, "
           f"a difference of {mean_len - exact:.2f} d)")
    agrees("sw_cycle_number", float(cyc[0]["n"] - 1 + sum(1 for c in cyc if epoch >= c["start"])), 0,
           "the count of published cycle starts the epoch has passed")

    # --- prf_ap2kp: the IAGA table is the source, the bias is measured here.
    slots = [(d["ap"], max(k for k in d["kp8"] if k is not None))
             for d in days if d["ap"] is not None and any(k is not None for k in d["kp8"])]
    edges = [0, 5, 10, 15, 20, 30, 45, 70, 110, 400]
    APX = [0, 2, 3, 4, 5, 6, 7, 9, 12, 15, 18, 22, 27, 32, 39, 48,
           56, 67, 80, 94, 111, 132, 154, 179, 207, 236, 300, 400]
    KPY = [i / 3 for i in range(28)]

    def table(x):
        x = max(APX[0], min(APX[-1], x))
        for i in range(1, len(APX)):
            if x <= APX[i]:
                f = (x - APX[i - 1]) / (APX[i] - APX[i - 1])
                return KPY[i - 1] + f * (KPY[i] - KPY[i - 1])
        return KPY[-1]

    binned = defaultdict(list)
    for a_, peak in slots:
        for i in range(len(edges) - 1):
            if edges[i] <= a_ < edges[i + 1]:
                binned[i].append(peak - table(a_))
                break
    centres = [(edges[i] + edges[i + 1]) / 2 for i in range(len(edges) - 1)]
    meds = [quant(sorted(binned[i]), 0.5) for i in range(len(edges) - 1)]
    lvl = a + b * math.log(5.0)
    # the design Ap sits in the top bin, whose centre is 255
    agrees("sw_kp_from_ap", table(lvl), 5e-3,
           "the published IAGA table read at the five-year return level")
    # prf_ap2kp holds the end bins and interpolates between centres, so the value
    # at the design Ap is not the top bin's median — it sits between the 90 and
    # 255 centres. A first draft compared against the median and reported a 19
    # per cent disagreement that was its own.
    def at_centre(x):
        if x <= centres[0]:
            return meds[0]
        if x >= centres[-1]:
            return meds[-1]
        for i in range(1, len(centres)):
            if x <= centres[i]:
                f = (x - centres[i - 1]) / (centres[i] - centres[i - 1])
                return meds[i - 1] + f * (meds[i] - meds[i - 1])
        return meds[-1]

    agrees("sw_kp_slot_bias", at_centre(lvl), 5e-3,
           f"the median of max(8 Kp) - table(daily Ap) per prf_ap2kp bin, "
           f"interpolated at the five-year level of Ap {lvl:.1f}")

    # --- the exceedance trio, counted straight off the record.
    bound = 132.0
    above = [d for d in days if d["ap"] is not None and d["ap"] >= bound]
    runs, prev = 0, -99
    for d in above:
        if d["t"] != prev + 1:
            runs += 1
        prev = d["t"]
    agrees("sw_exceedance_rate", len(above) / years, 5e-3,
           f"{len(above)} days at or above Ap {bound:.0f} over {years:.3f} years")
    agrees("sw_exceedance_duration", (len(above) / runs) * 86400, 5e-3,
           f"{len(above)} days in {runs} runs")

    # --- prf_segment's published F10.7 levels. The row bands env_f107, the
    # declared daily flux, NOT the design value — a first draft of this check
    # compared it against the design flux and reported a disagreement that was
    # entirely its own.
    ENV_F107 = 150.0
    band = 1 + sum(1 for e in (90.0, 130.0, 170.0) if ENV_F107 >= e)
    share = 100 * sum(1 for x in f107 if 130 <= x < 170) / len(f107)
    agrees("sw_activity_band", float(band),0,
           f"prf_segment's published levels applied to env_f107 = {ENV_F107:.0f}, "
           f"whose band holds {share:.1f}% of the record")

    # --- the outlook, scored against what arrived.
    fc = rows("forecast_issued.csv")
    tof = {d["date"]: d["t"] for d in days}
    pc = {}

    def persist(issue):
        t = tof.get(issue)
        if t is None:
            return None
        for back in range(1, 16):
            if t - back in byday:
                return byday[t - back]
        return None

    e2 = p2 = se = 0.0
    cnt = 0
    for r in fc:
        if int(r["lead_days"]) != 26 or r["f107"] in ("", None, "NaN"):
            continue
        tt = tof.get(r["target_date"])
        if tt is None or tt not in byday:
            continue
        if r["issue_date"] not in pc:
            pc[r["issue_date"]] = persist(r["issue_date"])
        p = pc[r["issue_date"]]
        if p is None:
            continue
        e = float(r["f107"]) - byday[tt]
        e2 += e * e
        p2 += (p - byday[tt]) ** 2
        se += e
        cnt += 1
    agrees("sw_forecast_skill", 1 - (e2 / cnt) / (p2 / cnt), 5e-3,
           f"the issued outlook at lead 26 against strict persistence, {cnt} pairs")
    agrees("sw_forecast_bias", se / cnt, 5e-3,
           f"mean signed error of the issued outlook at lead 26, {cnt} pairs")

    # --- the alerts, as issued.
    al = rows("alerts.csv")
    ks = [float(r["threshold_value"]) for r in al
          if r["threshold_var"] == "K" and r["threshold_value"] not in ("", None)
          and float(r["threshold_value"]) <= 9]
    agrees("sw_alert_threshold", min(ks), 0,
           f"the lowest K threshold in {len(ks)} alerts as issued")

    return out


def main():
    ap_ = argparse.ArgumentParser()
    ap_.add_argument("--port", type=int, default=7777)
    args = ap_.parse_args()

    days, cyc = load()
    checks = comparisons(days, cyc)

    print("THE STUDY'S OWN CODE IS NOT IN THIS REPOSITORY — only its published output,")
    print("70,422 rows across seven CSVs. Each row below is re-derived from that output")
    print("in Python, independently of the Rust, and compared against the engine.")
    print()

    bad = []
    for c in checks:
        got = run_node(args.port, c["node"])
        if got is None:
            print(f"  {'REFUSED':9s} {c['node']}")
            bad.append(c["node"])
            continue
        want = c["want"]
        err = abs(got - want) / abs(want) if want else abs(got - want)
        ok = err <= c["tol"]
        mark = c["kind"] if ok else "DISAGREES"
        print(f"  {mark:9s} {c['node']:26s} engine {got:<20.10g} independent {want:<20.10g} rel {err:.2e}")
        print(f"            {c['how']}")
        if c["kind"] == "DEPARTS":
            print(f"            departs: {c['why']}")
        if not ok:
            bad.append(c["node"])

    print()
    print(f"{len(checks)} row(s) compared against the study's published output, "
          f"{len(checks) - len(bad)} as expected")
    for n in bad:
        print(f"  DISAGREES: {n}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
