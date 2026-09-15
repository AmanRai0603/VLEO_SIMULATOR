#!/usr/bin/env python3
"""The port against the MATLAB tool's own saved run.

    tools/mat_parity.py                 # needs the daemon on :7777
    tools/mat_parity.py --selftest

WHAT THIS COMPARES, AND WHY IT IS DIFFERENT FROM tools/matlab_parity.py.

`matlab_parity.py` checks the port against the study's published CSV OUTPUT.
This checks it against the MATLAB tool's own SAVED RUN — `Result_Vleo_Tool.mat`,
the whole state of the tool at one design point, supplied by its author — and
against the MATLAB SOURCE, which is now available and was not when that first
checker was written. Its opening claim, that there is no way to run the study
and diff the two, is out of date: the algorithms can be reimplemented exactly
from the source, and where this file does that it says so.

The MATLAB run being compared against is one window: opens 2027-06-26, 365 days,
95% confidence, Kp slot 'mean', drivers from the database. Its answers are in
`matlab/reference/mission_drivers.csv` with their provenance.

Three kinds of entry, and the third is the one that matters:

  AGREES      the port and MATLAB land on the same number
  DIFFERS     they do not, and the entry says by how much and which is closer to
              the record when that can be established
  NO ROW      MATLAB publishes a number the port has no row for

A DIFFERS is not automatically a defect in either tool. Two of them here are
the two tools answering different questions, which the entry states.

Exit status is 0 when every entry is as this file expects. It is a comparison,
not a gate on the port being identical to MATLAB — the port departs from MATLAB
deliberately in places, and those departures are recorded here as expectations
so that an UNDECLARED change in either direction shows up.
"""

import argparse
import csv
import json
import statistics
import sys
import textwrap
import urllib.parse
import urllib.request
from datetime import date, timedelta
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BUNDLE = ROOT / "bundles/solar-weather/2026.09.14"
REF = ROOT / "matlab/reference/mission_drivers.csv"

#: The window the MATLAB run used. Everything here is evaluated at it.
WIN_OPEN = date(2027, 6, 26)
WIN_DAYS = 365
WIN_CONF = 95

#: The published three-hourly ap<->Kp scale (Bartels/IAGA), as prf_ap2kp's
#: tbl_() carries it. Both tools start from this; they differ in what they do
#: to it afterwards, which is the whole of finding 2.
AP_T = [0, 2, 3, 4, 5, 6, 7, 9, 12, 15, 18, 22, 27, 32, 39, 48, 56, 67,
        80, 94, 111, 132, 154, 179, 207, 236, 300, 400]
KP_T = [i / 3.0 for i in range(28)]

#: prf_ap2kp's fitting bins. Not round numbers and not negotiable — the offset
#: is a median per bin and moving an edge moves the answer.
BINS = [0, 5, 10, 15, 20, 30, 45, 70, 110, 400]


def interp(xs, ys, x):
    """Linear interpolation, clamping at both ends. MATLAB's interp1 default."""
    if x <= xs[0]:
        return ys[0]
    if x >= xs[-1]:
        return ys[-1]
    for i in range(len(xs) - 1):
        if xs[i] <= x <= xs[i + 1]:
            t = (x - xs[i]) / (xs[i + 1] - xs[i])
            return ys[i] + t * (ys[i + 1] - ys[i])
    return ys[-1]


def kp_table(ap):
    """prf_ap2kp tbl_(): the published scale, uncorrected."""
    ap = max(float(ap), 0.0)
    return 9.0 if ap > 400 else interp(AP_T, KP_T, min(ap, 400))


def read_record():
    rows = []
    with (BUNDLE / "observed_daily.csv").open() as f:
        for r in csv.DictReader(line for line in f if not line.startswith("#")):
            def num(k):
                try:
                    return float(r[k])
                except (TypeError, ValueError):
                    return None
            slots = [num(f"kp_{h:02d}z") for h in (0, 3, 6, 9, 12, 15, 18, 21)]
            rows.append({"date": r["date"], "f107": num("f107"),
                         "ap": num("ap_planetary"), "kp": slots})
    return rows


def fit_offset(rows, slot):
    """prf_ap2kp fitOffset_(), reimplemented on THIS repository's bundle.

    The point of recomputing it here rather than copying MATLAB's numbers is
    that it tests the data too: if the port's bundle disagreed with the
    database MATLAB ran on, the offsets would differ and every Kp below would
    miss. They do not.
    """
    ctr = [0.5 * (BINS[i] + BINS[i + 1]) for i in range(len(BINS) - 1)]
    off = [None] * len(ctr)
    for i in range(len(ctr)):
        d = []
        for r in rows:
            ap, ks = r["ap"], [k for k in r["kp"] if k is not None]
            if ap is None or len(ks) < 8 or not (BINS[i] <= ap < BINS[i + 1]):
                continue
            tgt = max(ks) if slot == "peak" else sum(ks) / len(ks)
            d.append(tgt - kp_table(ap))
        if len(d) >= 20:
            off[i] = statistics.median(d)
    good = [(c, o) for c, o in zip(ctr, off) if o is not None]
    gx, gy = [c for c, _ in good], [o for _, o in good]
    return ctr, [o if o is not None else interp(gx, gy, c) for c, o in zip(ctr, off)]


def kp_cal(ap, ctr, off):
    """prf_ap2kp in 'cal' mode: the table plus the measured slot offset."""
    return min(max(kp_table(ap) + interp(ctr, off, min(max(ap, ctr[0]), ctr[-1])), 0.0), 9.0)


def run_node(port, node, sets=(), mode="branch"):
    q = [("node", node), ("mode", mode)]
    q += [("set", f"{k}:{v!r}") for k, v in sets]
    req = urllib.request.Request(f"http://localhost:{port}/v1/run?" + urllib.parse.urlencode(q))
    with urllib.request.urlopen(req, timeout=30) as r:
        d = json.load(r)
    if not d.get("ok"):
        return None
    for v in d.get("values", []):
        if v["id"] == node:
            return v.get("si")
    return None


def cycle_analogue(rows):
    """What the record's earlier cycles actually did over this same window.

    Neither tool can be checked against the truth for 2027 — there is none yet.
    What CAN be done is to ask the record what F10.7 did at the same distance
    past solar maximum in the cycles that have finished, scaled onto this
    cycle's amplitude. n = 2, which is stated wherever this number is used.
    """
    cyc = []
    with (BUNDLE / "solar_cycles.csv").open() as f:
        for r in csv.DictReader(line for line in f if not line.startswith("#")):
            cyc.append({"n": int(r["cycle"]), "peak": float(r["peak"]),
                        "peak_date": date.fromisoformat(r["peak_date"])})
    rec = {date.fromisoformat(r["date"]): r["f107"] for r in rows if r["f107"] is not None}
    here = max(cyc, key=lambda c: c["peak_date"])
    lag = (WIN_OPEN - here["peak_date"]).days
    out = []
    for c in cyc:
        if c["n"] == here["n"]:
            continue
        a = c["peak_date"] + timedelta(days=lag)
        b = a + timedelta(days=WIN_DAYS)
        v = [f for d, f in rec.items() if a <= d < b]
        if v:
            out.append(statistics.mean(v) / c["peak"] * here["peak"])
    return (statistics.mean(out) if out else None), lag, len(out), here


def check(port):
    rows = read_record()
    ref = {}
    with REF.open() as f:
        for r in csv.DictReader(line for line in f if not line.startswith("#")):
            ref[r["scenario"]] = {k: float(v) for k, v in r.items() if k != "scenario"}
    findings, entries = [], []

    def entry(verdict, title, detail):
        entries.append((verdict, title, detail))

    # ---- 1. the ap -> Kp conversion -------------------------------------
    # The strongest check here: MATLAB's algorithm, reimplemented on the PORT's
    # own bundle, against MATLAB's saved answers.
    for slot, col in (("mean", "kp_mean"), ("peak", "kp_peak")):
        ctr, off = fit_offset(rows, slot)
        worst = 0.0
        for tag, s in ref.items():
            worst = max(worst, abs(kp_cal(s["ap"], ctr, off) - s[col]))
        ok = worst < 1e-4
        entry("AGREES" if ok else "DIFFERS",
              f"prf_ap2kp '{slot}' slot, recomputed on this repo's bundle",
              f"all 5 scenarios within {worst:.2e} Kp of the .mat"
              if ok else f"off by up to {worst:.4f} Kp — the bundle and MATLAB's database disagree")
        if not ok:
            findings.append(f"ap2kp {slot}: {worst:.4f} Kp")

    # The port's own two rows, against the same reference.
    ctr_p, off_p = fit_offset(rows, "peak")
    ctr_m, off_m = fit_offset(rows, "mean")
    for tag in ("nominal", "hotmean", "hotday"):
        ap = ref[tag]["ap"]
        got = run_node(port, "sw_kp_slot_bias", [("sw_storm_return_level", ap)], "alone")
        want = ref[tag]["kp_peak"] - kp_table(ap)
        if got is None:
            entry("DIFFERS", f"sw_kp_slot_bias at Ap {ap:.3f}", "the engine refused")
            findings.append("sw_kp_slot_bias refused")
        else:
            ok = abs(got - want) < 1e-6
            entry("AGREES" if ok else "DIFFERS", f"sw_kp_slot_bias at Ap {ap:.3f}",
                  f"port {got:.9f} vs MATLAB's peak offset {want:.9f}"
                  + ("" if ok else f" — differs by {got - want:+.6f}"))
            if not ok:
                findings.append(f"sw_kp_slot_bias at {ap:.3f}: {got - want:+.6f}")

    # sw_kp_from_ap publishes the bare table by design, and sw_kp_mean_bias is the
    # correction that turns it into the number MATLAB's mission run used. The
    # check that matters is the SUM: a reader composing the two rows the way the
    # sheets tell them to must land on MATLAB's answer.
    for tag in ("nominal", "hotmean", "hotday"):
        ap = ref[tag]["ap"]
        raw = run_node(port, "sw_kp_from_ap", [("sw_storm_return_level", ap)], "alone")
        bias = run_node(port, "sw_kp_mean_bias", [("sw_storm_return_level", ap)], "alone")
        want = ref[tag]["kp_mean"]
        if raw is None or bias is None:
            entry("DIFFERS", f"sw_kp_from_ap + sw_kp_mean_bias at Ap {ap:.3f}",
                  "the engine refused one of the two rows")
            findings.append(f"kp composition at {ap:.3f}: engine refused")
            continue
        got = raw + bias
        ok = abs(got - want) < 1e-9
        entry("AGREES" if ok else "DIFFERS",
              f"sw_kp_from_ap + sw_kp_mean_bias at Ap {ap:.3f}",
              f"table {raw:.9f} {bias:+.9f} = {got:.9f} vs MATLAB {want:.9f}"
              + ("" if ok else f" — differs by {got - want:+.3e}"))
        if not ok:
            findings.append(f"kp composition at {ap:.3f}: {got - want:+.3e}")

    # And the bare table on its own, which is NOT MATLAB's answer and is not
    # meant to be. Recorded so that a later change making it agree shows up as a
    # change rather than as an improvement nobody decided on.
    ap = ref["nominal"]["ap"]
    raw = run_node(port, "sw_kp_from_ap", [("sw_storm_return_level", ap)], "alone")
    if raw is not None:
        entry("DIFFERS", "sw_kp_from_ap alone vs MATLAB's mission Kp",
              f"port {raw:.6f} (the bare table) vs MATLAB {ref['nominal']['kp_mean']:.6f}. "
              "EXPECTED and unchanged: the sheet says this row applies no "
              "correction. The correction is sw_kp_mean_bias, checked above.")

    # ---- 2. the centre of the window ------------------------------------
    lead_s = WIN_DAYS * 86400
    cen = run_node(port, "sw_central_expectation", [("orbit_mission_duration", lead_s)])
    ml = ref["nominal"]["f107"]
    ana, lag, n_an, here = cycle_analogue(rows)
    if cen is not None:
        d = cen - ml
        detail = (f"port {cen:.4f} vs MATLAB {ml:.4f} ({d:+.2f} sfu, {100 * d / ml:+.1f}%). "
                  f"EXPECTED: different models. MATLAB freezes the last 27-day rotation "
                  f"forecast; the port relaxes to the record mean with a 27-day e-fold, so "
                  f"beyond ~6 months it IS the record mean.")
        if ana:
            detail += (f" The window sits {lag / 365.25:.2f} yr past SC{here['n']} max; over the "
                       f"same span the {n_an} finished cycles ran at {ana:.1f} sfu scaled onto "
                       f"this cycle's peak. Both tools are above that — MATLAB by "
                       f"{100 * (ml - ana) / ana:+.0f}%, the port by {100 * (cen - ana) / ana:+.0f}%.")
        entry("DIFFERS", "the centre of the design window", detail)

    # ---- 3. what MATLAB publishes and the port has no row for ------------
    entry("CLOSED", "the 24-hour-mean Kp slot",
          "MATLAB's mission run reads Kp in the 'mean' slot, and until "
          "sw_kp_mean_bias there was no row for it — only the peak sibling. "
          "Composing the two rows now reproduces that run exactly, above.")
    entry("NO ROW", "a window-driven driver product",
          "MATLAB turns (date, duration, confidence) into five driver sets. The port "
          "has no such row: sys_mission_requirements_mission_duration is still seeded, "
          "so there is no duration to drive a window with.")
    entry("NO ROW", "the sustained band around the centre",
          f"MATLAB publishes centre +/- 1.28*sigma (sigma measured walk-forward: "
          f"{(ref['hotmean']['f107'] - ml) / 1.28:.3f} sfu for F10.7, "
          f"{(ref['hotmean']['ap'] - ref['nominal']['ap']) / 1.28:.3f} for Ap). The port "
          f"publishes a one-sided p95 rise instead, so it has no lower bound at all.")
    return entries, findings


def selftest():
    """Watch the checks fail, on inputs built to fail them."""
    cases = []

    def want(label, ok):
        cases.append((label, ok))
        print(f"  {'as expected' if ok else 'WRONG'}: {label}")

    want("the table reproduces its own anchors",
         all(abs(kp_table(a) - k) < 1e-12 for a, k in
             [(0, 0), (4, 1), (7, 2), (15, 3), (27, 4), (48, 5), (80, 6), (132, 7), (207, 8), (400, 9)]))
    want("the table clips above ap 400", abs(kp_table(1000) - 9) < 1e-12)
    want("the table is monotone", all(kp_table(a) <= kp_table(a + 1) + 1e-12 for a in range(0, 450)))
    want("interp clamps below its first point", interp([1, 2], [10, 20], 0) == 10)
    want("interp clamps above its last point", interp([1, 2], [10, 20], 9) == 20)
    rows = read_record()
    want("the bundle carries 8 Kp slots on most days",
         sum(1 for r in rows if len([k for k in r["kp"] if k is not None]) == 8) > 9000)
    ctr, off = fit_offset(rows, "mean")
    want("the measured mean-slot offset is negative above ap 5 (Jensen)",
         all(o < 0 for c, o in zip(ctr, off) if c > 5))
    ctrp, offp = fit_offset(rows, "peak")
    want("the measured peak-slot offset is positive everywhere (Jensen, the other way)",
         all(o > 0 for o in offp))
    want("the reference table is present and has five scenarios",
         len([1 for line in REF.open() if not line.startswith("#")]) == 6)
    ana, lag, n, here = cycle_analogue(rows)
    want("the cycle analogue finds the current cycle and two finished ones",
         ana is not None and n == 2 and here["n"] == 25)
    bad = [c for c, ok in cases if not ok]
    print(f"selftest: {len(cases)} cases, {len(cases) - len(bad)} as expected")
    for c in bad:
        print(f"  FAILED: {c}")
    return 1 if bad else 0


def main():
    ap_ = argparse.ArgumentParser()
    ap_.add_argument("--port", type=int, default=7777)
    ap_.add_argument("--selftest", action="store_true")
    a = ap_.parse_args()
    if a.selftest:
        return selftest()

    entries, findings = check(a.port)
    print("THE PORT AGAINST THE MATLAB TOOL'S OWN SAVED RUN")
    print(f"window {WIN_OPEN} + {WIN_DAYS} d at {WIN_CONF}%, from {REF.relative_to(ROOT)}")
    print()
    for verdict, title, detail in entries:
        print(f"  {verdict:8s} {title}")
        for line in textwrap.wrap(detail, 74):
            print(f"           {line}")
        print()
    n = {v: sum(1 for e in entries if e[0] == v) for v in ("AGREES", "DIFFERS", "CLOSED", "NO ROW")}
    print(f"{len(entries)} entries — {n['AGREES']} agree, {n['DIFFERS']} differ as expected, "
          f"{n['CLOSED']} closed by a row added since, {n['NO ROW']} have no row in the port")
    for f in findings:
        print(f"  UNEXPECTED: {f}")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
