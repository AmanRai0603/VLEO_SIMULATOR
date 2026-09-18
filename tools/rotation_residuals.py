#!/usr/bin/env python3
"""The 361 walk-forward rotation residuals, and what their tail actually is.

    tools/rotation_residuals.py            # F10.7 and Ap
    tools/rotation_residuals.py --dump f107 > residuals.csv

WHY THIS EXISTS. `sw_mean_band_spread` publishes ONE number from this sample —
sigma = 13.4544 sfu — and four design rows turn it into a band with a normal
multiplier, `centre + 1.28*sigma`. That multiplier is a percentile only if the
residuals are normal, and the row's own sheet says they are not:

    the residuals of a forecast that misses hardest when activity is highest are
    skewed, and a normal multiplier under-covers the high tail -- which is the
    tail a design is sized against. The empirical percentile of the residuals
    would be the honest statistic, and sw_mean_band_spread publishes only their
    standard deviation.

The sample exists. This recomputes it and prints its empirical quantiles beside
the normal ones, so the size of that under-coverage is a measured number rather
than a worry.

THE ONE TEST THAT MATTERS IS THE FIRST LINE OF OUTPUT, AND AS WRITTEN IT FAILS.
If sigma does not come back at 13.4544 sfu the walk-forward here is not the
walk-forward that produced the published number, and every quantile below it is
a quantile of something else. So this file REFUSES to print quantiles unless
sigma agrees to within `TOL`. It currently does not:

    F10.7   376 rotations, sigma 14.3139   against 13.4544 published   +6.39%
    Ap      376 rotations, sigma  3.5264   against  3.5937 published   -1.87%

TWO DISAGREEMENTS, AND THEY ARE THE WORK LEFT. The rotation COUNT is 376 here
and 361 on the sheet. The record is 10592 days on a dense grid, which is 392
rotations of 27, and this file scores every one past a warm-up of `LEVEL + 3`.
Reaching 361 from 392 needs a warm-up of 31 and the sheet does not state one.
Dropping the 13 rotations that contain an interpolated day would give 379, and
the sheet's third assumption says those days WERE counted, so that is not it.

The estimator is the other candidate and it is not enough on its own: the sheet
says "an AR(2) fitted on the training anomalies" without naming one, and
least-squares instead of Yule-Walker moves Ap to -0.41% and F10.7 the wrong way
to +6.91%. `--ar ls` runs it. That Ap nearly lands and F10.7 does not says the
difference is not one systematic choice.

This is published as a starting point with its disagreement recorded rather than
tuned until it matched. A walk-forward whose free parameters were chosen to
reproduce a number agrees with that number by construction and is evidence for
nothing. The recipe is `sw_mean_band_spread`'s five theory steps, followed
exactly; where this file had to choose something the sheet does not state, it
says so, and those are the places to vary.

Nothing here writes a sheet. A value measured by an agent is not a value a sheet
may carry -- see AGENTS.md -- so the output is evidence for a person to confirm,
and the rows that would hold it are section 30 B3.2.
"""

import argparse
import csv
import math
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BUNDLE = os.path.join(ROOT, "bundles/solar-weather/2026.09.14")
ROT = 27           # days in a rotation, the synodic period the study works in
BINS = 20          # phase bins, from theory step 2
SHRINK = 5.0       # the n/(n+5) pull toward the global mean, from theory step 2
LEVEL = 13         # rotations the scale is taken over, from theory step 2
CLAMP = (0.4, 2.5) # the scale clamp, from theory step 2
TOL = 0.005        # how close sigma must come before a quantile is worth printing


def read_days():
    """The daily record, and the cycle table, as the face reads them."""
    days = []
    with open(os.path.join(BUNDLE, "observed_daily.csv")) as f:
        for r in csv.DictReader(l for l in f if not l.startswith("#")):
            days.append((r["date"], r["f107"], r["ap_planetary"]))
    cycles = []
    with open(os.path.join(BUNDLE, "solar_cycles.csv")) as f:
        for r in csv.DictReader(l for l in f if not l.startswith("#")):
            cycles.append((r["start"], r["end"]))
    return days, cycles


def epoch(s):
    """Days since 2000-01-01, written out so a timezone cannot move a day."""
    y, m, d = int(s[0:4]), int(s[5:7]), int(s[8:10])
    yy = y - 1 if m <= 2 else y
    era = yy // 400
    yoe = yy - era * 400
    doy = (153 * (m + (-3 if m > 2 else 9)) + 2) // 5 + d - 1
    doe = yoe * 365 + yoe // 4 - yoe // 100 + doy
    return era * 146097 + doe - 730425


def series(days, col):
    """One column on a dense day grid, with the record's holes filled.

    THE 273-DAY HOLE IS FILLED BY STRAIGHT LINE BEFORE THE ROTATIONS ARE CUT,
    which is what `sw_mean_band_spread`'s third assumption says was done and
    warns about: those interpolated days are smoother than the sun, so the
    rotations containing them are easier to predict than real ones and the
    pooled spread comes out a little narrow.
    """
    have = {}
    for date, f107, ap in days:
        v = f107 if col == "f107" else ap
        if v not in ("", "NaN", None):
            have[epoch(date)] = float(v)
    if not have:
        raise SystemExit("no %s in the record" % col)
    lo, hi = min(have), max(have)
    out, known = [], sorted(have)
    j = 0
    for t in range(lo, hi + 1):
        if t in have:
            out.append(have[t])
            continue
        while j + 1 < len(known) and known[j + 1] < t:
            j += 1
        a, b = known[j], known[j + 1]
        f = (t - a) / (b - a)
        out.append(have[a] + f * (have[b] - have[a]))
    return lo, out


def phases(t0, n, cycles):
    """Cycle phase per day, wrapping past the table's end on the mean length.

    Theory step 5: the sixteen days past cycle 25's tabulated end WRAP rather
    than clamp, because clamping them to phase 1 puts them at the cycle minimum.
    """
    cs = [(epoch(a), epoch(b)) for a, b in cycles]
    done = [(a, b) for a, b in cs if any(s >= b for s, _ in cs)]
    mean_len = sum(b - a for a, b in done) / len(done) if done else 4000.0
    last_start = cs[-1][0]
    out = []
    for i in range(n):
        t = t0 + i
        u = None
        for a, b in cs:
            if a <= t < b:
                u = (t - a) / (b - a if (a, b) in done else mean_len)
                break
        if u is None:
            u = ((t - last_start) / mean_len) % 1.0
        out.append(min(u, 0.999))
    return out


def walk_forward(rm, rp):
    """Predict each rotation from the ones before it, and return the residuals.

    rm — the rotation means. rp — each rotation's phase. Every step is causal:
    the slice fitted on is strictly before the rotation being predicted.
    """
    resid, preds = [], []
    seen_res = []
    for k in range(len(rm)):
        train = rm[:k]
        if len(train) < LEVEL + 3:      # enough for the level scale and the AR(2)
            continue
        tp = rp[:k]
        # the phase baseline, shrunk toward the global mean bin by bin
        allm = sum(train) / len(train)
        sums = [0.0] * BINS
        cnts = [0] * BINS
        for v, p in zip(train, tp):
            b = min(BINS - 1, int(p * BINS))
            sums[b] += v
            cnts[b] += 1
        base = []
        for b in range(BINS):
            if cnts[b]:
                w = cnts[b] / (cnts[b] + SHRINK)
                base.append(w * (sums[b] / cnts[b]) + (1 - w) * allm)
            else:
                base.append(allm)
        at = lambda p: base[min(BINS - 1, int(p * BINS))]
        # scaled to the level of the last thirteen, clamped
        bl = sum(at(p) for p in tp[-LEVEL:]) / LEVEL
        sc = (sum(train[-LEVEL:]) / LEVEL) / bl if bl else 1.0
        sc = max(CLAMP[0], min(CLAMP[1], sc))
        # the anomalies the AR(2) is fitted on
        an = [v - at(p) * sc for v, p in zip(train, tp)]
        a1, a2 = ar2(an)
        pred = at(rp[k]) * sc + a1 * an[-1] + a2 * an[-2]
        if seen_res:
            pred -= sum(seen_res) / len(seen_res)   # the causal debias
        e = pred - rm[k]
        seen_res.append(e)
        resid.append(e)
        preds.append(pred)
    return resid


def ar2(x):
    """AR(2) by Yule-Walker on the sample autocorrelations.

    NOT STATED BY THE SHEET, and it is the one place this file had to choose.
    The sheet says 'an AR(2) fitted on the training anomalies' and not by which
    estimator. Yule-Walker is the usual one for a short causal fit and needs no
    iteration; least squares on the lagged design matrix would give slightly
    different coefficients. If sigma below does not land on 13.4544 this is the
    first thing to vary.
    """
    n = len(x)
    m = sum(x) / n
    d = [v - m for v in x]
    c0 = sum(v * v for v in d) / n
    if c0 == 0:
        return 0.0, 0.0
    r1 = sum(d[i] * d[i - 1] for i in range(1, n)) / n / c0
    r2 = sum(d[i] * d[i - 2] for i in range(2, n)) / n / c0
    den = 1 - r1 * r1
    if abs(den) < 1e-12:
        return r1, 0.0
    return (r1 * (1 - r2) / den, (r2 - r1 * r1) / den)


def ar2_ls(x):
    """AR(2) by least squares on the lagged design matrix — the other candidate.

    Offered because the sheet names the model and not the estimator, and the two
    do not agree: on Ap this lands within half a per cent of the published sigma
    and on F10.7 it moves further away. Selected with `--ar ls`.
    """
    n = len(x)
    if n < 6:
        return 0.0, 0.0
    s11 = s12 = s22 = b1 = b2 = 0.0
    for i in range(2, n):
        y, x1, x2 = x[i], x[i - 1], x[i - 2]
        s11 += x1 * x1
        s12 += x1 * x2
        s22 += x2 * x2
        b1 += x1 * y
        b2 += x2 * y
    det = s11 * s22 - s12 * s12
    if abs(det) < 1e-12:
        return 0.0, 0.0
    return (b1 * s22 - b2 * s12) / det, (b2 * s11 - b1 * s12) / det


def quantile(sorted_, q):
    """Linear interpolation at h = (n-1)q — the convention the tree measures under."""
    if not sorted_:
        return None
    if len(sorted_) == 1:
        return sorted_[0]
    h = (len(sorted_) - 1) * q
    lo = int(math.floor(h))
    hi = min(len(sorted_) - 1, lo + 1)
    return sorted_[lo] + (h - lo) * (sorted_[hi] - sorted_[lo])


def report(col, published, n_published, days, cycles, dump=False):
    t0, vals = series(days, col)
    ph = phases(t0, len(vals), cycles)
    nrot = len(vals) // ROT
    rm = [sum(vals[b * ROT:(b + 1) * ROT]) / ROT for b in range(nrot)]
    rp = [ph[b * ROT + ROT // 2] for b in range(nrot)]
    resid = walk_forward(rm, rp)
    if dump:
        print("residual")
        for e in resid:
            print("%.10f" % e)
        return
    n = len(resid)
    mean = sum(resid) / n
    sd = math.sqrt(sum((e - mean) ** 2 for e in resid) / n)
    # THE SIGN CONVENTION. `pred - truth` is what the sheet says sigma is the
    # spread of. A design is sized against the truth coming in ABOVE the
    # prediction, which is a NEGATIVE residual under that convention, so the
    # tail a band must cover is the low tail of this sample. Reported as the
    # exceedance the band has to hold, which is -residual, so a bigger number is
    # a worse miss in the direction that costs propellant.
    over = sorted(-e for e in resid)
    off = (sd - published) / published
    print("== %s" % col)
    print("   %d rotations scored%s"
          % (n, "" if n == n_published else "   <- the sheet says %d" % n_published))
    print("   sigma = %.4f   (the sheet publishes %.4f, %+.2f%%)"
          % (sd, published, 100 * off))
    print("   mean residual %.4f, skew %.4f" % (mean, skew(resid)))
    # A QUANTILE OF THE WRONG SAMPLE IS WORSE THAN NO QUANTILE, because it is a
    # number of the right magnitude in the right units and nothing about it
    # announces that it came from a different walk-forward. So it is not printed.
    # BOTH HAVE TO MATCH, and the count is the stricter of the two. A sigma that
    # lands while the count does not is a sample of a different size that happens
    # to have a similar spread — which is the Ap column under least squares, at
    # -0.41 per cent on 376 rotations against the sheet's 361. Printing its
    # quantiles because one of the two agreed would be reporting a coincidence.
    if abs(off) > TOL or n != n_published:
        why = []
        if n != n_published:
            why.append("the count is %d against %d" % (n, n_published))
        if abs(off) > TOL:
            why.append("sigma is %.2f%% out against a tolerance of %.1f%%"
                       % (100 * abs(off), 100 * TOL))
        print("   NO QUANTILES: %s." % ", and ".join(why))
        print("   This is not the sample sw_mean_band_spread measured, so its quantiles are")
        print("   quantiles of something else. See the header for what is open.")
        return
    print("   the band a design needs, against what a normal multiplier gives:")
    for q, z in ((0.80, 0.8416), (0.90, 1.2816), (0.95, 1.6449), (0.99, 2.3263)):
        emp = quantile(over, q)
        nor = z * sd
        print("     %4.0f%%   empirical %8.4f   normal z*sigma %8.4f   %+7.4f  (%+6.1f%%)"
              % (100 * q, emp, nor, emp - nor, 100 * (emp - nor) / nor))


def skew(x):
    n = len(x)
    m = sum(x) / n
    s = math.sqrt(sum((v - m) ** 2 for v in x) / n)
    return sum((v - m) ** 3 for v in x) / n / s ** 3 if s else 0.0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dump", choices=("f107", "ap"), help="print the residuals and stop")
    ap.add_argument("--ar", choices=("yw", "ls"), default="yw",
                    help="the AR(2) estimator: Yule-Walker (default) or least squares")
    a = ap.parse_args()
    if a.ar == "ls":
        globals()["ar2"] = ar2_ls
    days, cycles = read_days()
    if a.dump:
        report(a.dump, 0.0, 0, days, cycles, dump=True)
        return 0
    report("f107", 13.4544, 361, days, cycles)
    print()
    report("ap", 3.5937, 361, days, cycles)
    print()
    print("The first line of each block is the test, and it does not pass yet. Until")
    print("sigma matches, this reproduces a walk-forward and not THE walk-forward, and")
    print("no number from it belongs on a sheet. See the header for the two open")
    print("differences: the rotation count, and which AR(2) estimator was used.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except BrokenPipeError:
        # `--dump | head` is the normal way to look at a few of these, and a
        # traceback on a closed pipe is noise rather than news.
        os._exit(0)
