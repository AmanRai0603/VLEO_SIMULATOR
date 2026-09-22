#!/usr/bin/env python3
"""The 361 walk-forward rotation residuals, and what their tail actually is.

    tools/rotation_residuals.py            # F10.7 and Ap
    tools/rotation_residuals.py --dump f107 > residuals.csv
    tools/rotation_residuals.py --why      # what the gap is NOT
    tools/rotation_residuals.py --shape    # what the band covers, scale-free

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

TWO DISAGREEMENTS. The rotation COUNT is 376 here and 361 on the sheet. The
record is 10592 days on a dense grid, which is 392 rotations of 27, and this file
scores every one past a warm-up of `LEVEL + 3`. Reaching 361 from 392 needs a
warm-up of 31 and the sheet does not state one. Dropping the 13 rotations that
contain an interpolated day would give 379, and the sheet's third assumption says
those days WERE counted, so that is not it.

AND NEITHER OF THE TWO CANDIDATES FOR THEM CLOSES THE GAP. `--why` measures each
one the sheet leaves open, and every one of them moves the two channels in
OPPOSITE directions, which no single choice can do:

  * the warm-up. F10.7 comes closest at a further drop of 43 and Ap at 1. Take
    instead the drop that gives the published count of 361 and F10.7 is +6.23%
    while Ap goes from -1.87% to -3.36% — the sheet's own sample size makes the
    channel that nearly agreed agree LESS.
  * the estimator. The sheet says "an AR(2) fitted on the training anomalies"
    without naming one; least squares instead of Yule-Walker moves Ap to -0.41%
    and F10.7 the wrong way to +6.91%. `--ar ls` runs it for the whole report.
  * theory step 2's scale. Disabling it is worth +6.65% on F10.7 and +4.01% on
    Ap, so a stronger scale pulls BOTH down and Ap is already under.

So the gap is not one systematic choice this file got wrong, and the remaining
work is not a search here: it is either `prf_rebuild.m` itself, which is not in
this repository, or a person deciding what the sheet should say.

This is published as a starting point with its disagreement recorded rather than
tuned until it matched. A walk-forward whose free parameters were chosen to
reproduce a number agrees with that number by construction and is evidence for
nothing — and F10.7 sigma spans 25% across a choice of scored window alone,
which is four times the gap, so landing on 13.4544 by choosing one would say
nothing at all about the method. `--why` prints that span for scale.

The recipe is `sw_mean_band_spread`'s five theory steps, followed exactly; where
this file had to choose something the sheet does not state, it says so.

AND WHAT CAN STILL BE SAID WHILE SIGMA IS UNREPRODUCED. `--shape` divides sigma
out and reports what the band COVERS rather than how wide it is. That fraction is
stable where sigma is not: across the same variants sigma spans 32 per cent and
the coverage at 1.28 sigma spans under two points. It answers the question
section 30 B3 exists to ask -- does the normal multiplier under-cover the tail a
design is sized against -- without answering B3.2's rows, which want a quantile in
sfu and cannot have one from a sample this is not.

The answer is not the one three sheets stated. At 1.28 sigma, the multiplier four
design rows use, the band OVER-covers: 0.9149 to 0.9339 against a normal's 0.8997
for F10.7 and 0.9148 to 0.9239 for Ap. The heavy tail is real and starts further
out, at 2.93 to 3.49 sigma for the 99th percentile where a normal says 2.33. The
three sheets now say that.

Nothing here writes a sheet. A value measured by an agent is not a value a sheet
may carry -- see AGENTS.md -- so the output is evidence for a person to confirm,
and the rows that would hold it are section 30 B3.2. A COVERAGE FRACTION IS NOT
THAT VALUE: it is a shape, measured on this file's walk and not the study's, and
it is recorded in the sheets as a correction to a claim rather than as a number
any row publishes.
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


def _sd(res):
    """The population standard deviation — ONE definition.

    There were three: a local in `why`, an inline in `report`, and this. They
    agreed, which is the only reason nobody noticed; three statements of one
    fact is how the sheet's sense and the hole's sense came to differ elsewhere
    in this repository.
    """
    m = sum(res) / len(res)
    return math.sqrt(sum((e - m) ** 2 for e in res) / len(res))


def walk_forward(rm, rp, debias=True):
    """Predict each rotation from the ones before it, and return the residuals.

    rm — the rotation means. rp — each rotation's phase. Every step is causal:
    the slice fitted on is strictly before the rotation being predicted.

    `debias` is the sheet's own last sentence — "the mean of the residuals ALREADY
    OBSERVED is subtracted as a causal debias" — and is a parameter only so that
    `--selftest` can show the step is applied and what it is worth. It is not a
    free choice, so `--why` does not vary it.
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
        if debias and seen_res:
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
    iteration; least squares on the lagged design matrix gives slightly different
    coefficients. `--ar ls` runs that one, and `--why` measures what the choice
    is worth: not enough, and in opposite directions on the two channels.

    `--selftest` checks this recovers a known AR(2) rather than trusting it, for
    the reason that the whole of §31.4 rests on it. An estimator that is simply
    wrong would produce a disagreement with the published sigma too.
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


# The two estimators, by the name the sheet would use. `--ar` swaps the global
# `ar2` that walk_forward reaches, which is fine for a run of the whole report
# and wrong for a diagnostic that prints one column per estimator: read back
# through the global, the Yule-Walker column of `--ar ls --why` would have been
# least squares under a Yule-Walker heading.
AR2 = {"yw": ar2, "ls": ar2_ls}


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


def why(days, cycles):
    """Which of the named candidates closes the gap. Not one of them does.

    §31.4 named two — the warm-up rotation the walk begins scoring at, and the
    AR(2) estimator. This measures both, and theory step 2's scale with them,
    and in every case the two channels move in OPPOSITE directions. A walk has
    one warm-up, one estimator and one scale rule. It cannot have one that suits
    F10.7 and another that suits Ap.

    So this is a diagnostic and not a search, and `--ar` does not reach it: each
    line names the estimator it ran. The last block says why searching harder
    would be worse than useless — sigma moves further under a choice of scored
    window than the whole disagreement being chased, so a walk that lands within
    a few per cent by choosing one has demonstrated nothing about its method.
    """
    series_of = {}
    for col in ("f107", "ap"):
        t0, vals = series(days, col)
        ph = phases(t0, len(vals), cycles)
        n = len(vals) // ROT
        series_of[col] = ([sum(vals[b * ROT:(b + 1) * ROT]) / ROT for b in range(n)],
                          [ph[b * ROT + ROT // 2] for b in range(n)])

    def resid(col, est="yw", clamp=CLAMP):
        """The walk's residuals under a named estimator and a named scale clamp."""
        keep_ar, keep_cl = globals()["ar2"], globals()["CLAMP"]
        globals()["ar2"], globals()["CLAMP"] = AR2[est], clamp
        try:
            return walk_forward(*series_of[col])
        finally:
            globals()["ar2"], globals()["CLAMP"] = keep_ar, keep_cl

    pub = {"f107": 13.4544, "ap": 3.5937}
    off = lambda v, c: 100.0 * (v - pub[c]) / pub[c]

    print("WHICH NAMED CANDIDATE CLOSES THE GAP")
    print()
    print("1 · the warm-up. One drop serves the whole walk.")
    for col in ("f107", "ap"):
        res = resid(col)
        best = min(range(0, 120), key=lambda d: abs(_sd(res[d:]) - pub[col]))
        print("      %-5s comes closest to its published sigma at a further drop of"
              " %3d, leaving n = %3d" % (col, best, len(res) - best))
    res, res_ap = resid("f107"), resid("ap")
    d361 = len(res) - 361
    sf, sa = _sd(res[d361:]), _sd(res_ap[d361:])
    print("      the count 361 needs a further drop of %d — a warm-up of %d in all;"
          % (d361, LEVEL + 3 + d361))
    print("      there F10.7 sigma is %.4f, %+.2f%%, and Ap sigma is %.4f, %+.2f%%"
          % (sf, off(sf, "f107"), sa, off(sa, "ap")))
    print("      -> no single drop gives 361 and either channel's sigma.")
    print()

    print("2 · the estimator, Yule-Walker against least squares.")
    for col in ("f107", "ap"):
        print("      %-5s  yule-walker %+6.2f%%    least squares %+6.2f%%"
              % (col, off(_sd(resid(col, "yw")), col), off(_sd(resid(col, "ls")), col)))
    print("      -> least squares moves Ap toward its number and F10.7 away.")
    print()

    print("3 · the scale of theory step 2, measured by disabling it.")
    for col in ("f107", "ap"):
        on, no = _sd(resid(col)), _sd(resid(col, clamp=(1.0, 1.0)))
        print("      %-5s  scaling is worth %+6.2f%% of sigma;"
              " as written %+6.2f%%, unscaled %+6.2f%%"
              % (col, 100.0 * (no - on) / on, off(on, col), off(no, col)))
    print("      -> a stronger scale pulls BOTH down, and Ap is already under.")
    print()

    print("4 · what a choice of window is worth, for scale.")
    res = resid("f107")
    vals = [_sd(res[d:]) for d in range(0, 200, 5)]
    print("      F10.7 sigma over further drops 0 to 200: %.4f to %.4f, a span of %.0f%%"
          % (min(vals), max(vals), 100.0 * (max(vals) - min(vals)) / min(vals)))
    print("      -> a window alone moves sigma further than the gap being chased,")
    print("         so landing within a few per cent is not evidence of one method.")
    return 0


def coverage(res, z):
    """The fraction of design-relevant exceedances a band of z sigma holds.

    SCALE-FREE, WHICH IS THE WHOLE POINT. Sigma is what this file cannot
    reproduce — §31.4 — so every absolute quantile it could print is a quantile
    of a sample that is not the study's, and it refuses to print them. A
    coverage FRACTION divides that scale out: it says what shape the residuals
    have rather than how wide they are, and shape is the thing §30 B3 is
    actually asking about.
    """
    s = _sd(res)
    return sum(1 for e in res if -e <= z * s) / float(len(res))


def verdict(lo, hi, phi):
    """How a measured coverage span sits against what a normal would give.

    A SPAN CAN STRADDLE, and the first version of this did not say so: it tested
    `lo < phi` alone and reported 0.9432-0.9550 around 0.9500 as "UNDER-covers by
    0.7 to 0.5 points", which is two opposite verdicts printed as one range.
    """
    if lo < phi < hi:
        return ("straddles it — under by %.1f at one end, over by %.1f at the other"
                % (100 * (phi - lo), 100 * (hi - phi)))
    if hi <= phi:
        return "UNDER-covers by %.1f to %.1f points" % (100 * (phi - hi), 100 * (phi - lo))
    return "OVER-covers by %.1f to %.1f points" % (100 * (lo - phi), 100 * (hi - phi))


def shape(days, cycles):
    """Does the normal multiplier under-cover the tail a design is sized against?

    §30 B3 asks for the empirical quantile BESIDE the z-multiplier, and step
    B3.2's two rows want it in sfu — which needs the sample reproduced, which it
    is not. This answers the question the rows exist to settle without answering
    it in sfu, and it is worth something only because it is STABLE where sigma
    is not: the variants below span sigma by 25 per cent and coverage by under
    two points.

    It is a property of THIS file's walk, not of the study's. That caveat cannot
    be measured away; what makes the number usable is that every free parameter
    known to differ moves sigma far more than it moves this.
    """
    import math as _m

    series_of = {}
    for col in ("f107", "ap"):
        t0, vals = series(days, col)
        ph = phases(t0, len(vals), cycles)
        n = len(vals) // ROT
        series_of[col] = ([sum(vals[b * ROT:(b + 1) * ROT]) / ROT for b in range(n)],
                          [ph[b * ROT + ROT // 2] for b in range(n)])

    def variants(col):
        base = walk_forward(*series_of[col])
        out = [("as written", base)]
        keep = globals()["ar2"]
        globals()["ar2"] = ar2_ls
        out.append(("least squares", walk_forward(*series_of[col])))
        globals()["ar2"] = keep
        keep_c = globals()["CLAMP"]
        globals()["CLAMP"] = (1.0, 1.0)
        out.append(("no scale", walk_forward(*series_of[col])))
        globals()["CLAMP"] = keep_c
        for d in (15, 43, 100, 200):
            out.append(("drop %d" % d, base[d:]))
        return out

    # Phi(1.28) and Phi(1.645): what a normal would hold at the two multipliers
    # this tree actually uses.
    NORMAL = ((1.28, 0.899727), (1.645, 0.950015))

    print("WHAT THE BAND ACTUALLY HOLDS, AND WHAT A NORMAL SAYS IT HOLDS")
    print()
    print("No absolute quantile is printed. Sigma here is not the study's, so a")
    print("quantile in sfu would be a quantile of another sample — the refusal in")
    print("`report` stands. A coverage fraction divides the scale out.")
    for col in ("f107", "ap"):
        print()
        print("%s — sigma is NOT reproduced (§31.4); these fractions are shape, not scale."
              % col)
        span = {z: [1.0, 0.0] for z, _ in NORMAL}
        sig = [9e9, 0.0]
        for name, res in variants(col):
            s = _sd(res)
            sig = [min(sig[0], s), max(sig[1], s)]
            row = "   %-15s n=%3d  sigma %8.4f " % (name, len(res), s)
            for z, _ in NORMAL:
                c = coverage(res, z)
                span[z] = [min(span[z][0], c), max(span[z][1], c)]
                row += "  %.2fs %.4f" % (z, c)
            print(row + "   skew %+.3f" % skew(res))
        print("   sigma spans %.4f to %.4f, which is %.0f per cent"
              % (sig[0], sig[1], 100.0 * (sig[1] - sig[0]) / sig[0]))
        for z, phi in NORMAL:
            lo, hi = span[z]
            # STRADDLING IS ITS OWN ANSWER and the first version of this line did
            # not have it: it tested `lo < phi` alone and called a span of
            # 0.9432 to 0.9550 around a normal's 0.9500 "UNDER-covers by 0.7 to
            # 0.5 points", which is two different verdicts read as one.
            print("   at %.3f sigma a normal says %.4f; measured %.4f to %.4f — %s"
                  % (z, phi, lo, hi, verdict(lo, hi, phi)))
        # AND WHERE THE HEAVY TAIL ACTUALLY BITES, printed rather than asserted.
        # This is a ratio, not a quantile in sfu: the refusal stands.
        far = [quantile(sorted(-e for e in res), 0.99) / _sd(res)
               for _, res in variants(col)]
        print("   the 99th percentile of exceedance is %.2f to %.2f sigma, where a "
              "normal says 2.33" % (min(far), max(far)))
    print()
    print("READ IT AT THE MULTIPLIER THE DESIGN USES. Four rows band at 1.28 sigma.")
    print("There the measured coverage is ABOVE the normal's in every variant of")
    print("both channels: the multiplier is conservative, not under-covering. At")
    print("1.645 sigma the measurement straddles the normal. The heavy tail the")
    print("sheets warn about is real and bites further out, as the last line of")
    print("each block shows — which is a different statement from the one four")
    print("sheets make about the multiplier they use.")
    return 0


def refuses(off, n, n_published):
    """Whether quantiles are withheld. Either gate alone is enough to withhold.

    A QUANTILE OF THE WRONG SAMPLE IS WORSE THAN NO QUANTILE, because it is a
    number of the right magnitude in the right units and nothing about it
    announces that it came from a different walk-forward. BOTH have to match, and
    the count is the stricter of the two: a sigma that lands while the count does
    not is a sample of a different size that happens to have a similar spread —
    which is the Ap column under least squares, at -0.41 per cent on 376
    rotations against the sheet's 361. Printing its quantiles because one of the
    two agreed would be reporting a coincidence.
    """
    return abs(off) > TOL or n != n_published


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
    sd = _sd(resid)
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
    # Withheld unless BOTH agree — see `refuses`, which is where that rule lives.
    if refuses(off, n, n_published):
        why = []
        if n != n_published:
            why.append("the count is %d against %d" % (n, n_published))
        if abs(off) > TOL:
            why.append("sigma is %.2f%% out against a tolerance of %.1f%%"
                       % (100 * abs(off), 100 * TOL))
        print("   NO QUANTILES: %s." % ", and ".join(why))
        print("   This is not the sample sw_mean_band_spread measured, so its quantiles are")
        print("   quantiles of something else. `--why` says which candidates it is NOT.")
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


def selftest():
    """The maths, against things whose answers are known independently.

    §31.4 reports that this file's sigma disagrees with the published one, and
    that no named candidate closes the gap. That finding is only worth anything
    if the estimators and the walk are right, so this checks them against
    synthetic series with known coefficients rather than against the record — a
    broken AR(2) or a leak of future values would produce a disagreement with
    13.4544 as surely as a different warm-up would, and would look identical.

    Needs no bundle and no daemon.
    """
    bad = 0

    def noise(n, seed=12345):
        """A deterministic standard-normal-ish stream: Box-Muller on an LCG."""
        out, x = [], seed
        for _ in range((n + 1) // 2):
            x = (1103515245 * x + 12345) % (1 << 31)
            u1 = (x + 1) / (1 << 31)
            x = (1103515245 * x + 12345) % (1 << 31)
            u2 = (x + 1) / (1 << 31)
            r = math.sqrt(-2.0 * math.log(u1))
            out.append(r * math.cos(2 * math.pi * u2))
            out.append(r * math.sin(2 * math.pi * u2))
        return out[:n]

    # 1, 2 · both estimators recover a known AR(2). The coefficients are inside
    # the stationary triangle, so a long draw pins them.
    A1, A2 = 0.5, 0.3
    e = noise(40000)
    x = [0.0, 0.0]
    for i in range(2, len(e)):
        x.append(A1 * x[-1] + A2 * x[-2] + e[i])
    x = x[500:]                      # past the transient
    for name, fn in (("yule-walker", ar2), ("least squares", ar2_ls)):
        g1, g2 = fn(x)
        if abs(g1 - A1) > 0.02 or abs(g2 - A2) > 0.02:
            bad += 1
            print("  FAIL %s recovered (%.4f, %.4f) from an AR(2) of (%.2f, %.2f)"
                  % (name, g1, g2, A1, A2))

    # 3 · THE WALK IS CAUSAL. Changing one rotation mean may move that rotation's
    # residual and the ones after it; it may never move an earlier one. This is
    # the property the whole method rests on, and a leak makes sigma too SMALL —
    # which is the direction the published number sits in.
    rm = [100.0 + 40.0 * math.sin(i / 11.0) + 3.0 * v for i, v in enumerate(noise(120, 777))]
    rp = [(i * 0.017) % 1.0 for i in range(len(rm))]
    a = walk_forward(rm, rp)
    hit = 80
    rm2 = list(rm)
    rm2[hit] += 500.0
    b = walk_forward(rm2, rp)
    if len(a) != len(b):
        bad += 1
        print("  FAIL the walk changed length under one perturbed rotation")
    else:
        first = next((i for i, (u, v) in enumerate(zip(a, b)) if abs(u - v) > 1e-9), None)
        offset = len(rm) - len(a)            # the warm-up, in rotations
        if first is None:
            bad += 1
            print("  FAIL perturbing rotation %d moved no residual at all" % hit)
        elif first + offset < hit:
            bad += 1
            print("  FAIL the walk is NOT causal: perturbing rotation %d moved the"
                  " residual of rotation %d" % (hit, first + offset))
        else:
            # AND IT DOES NOT PREDICT A ROTATION FROM ITSELF. The residual is
            # `pred - rm[k]` and pred cannot see rm[k], so perturbing rm[k] by
            # d moves that residual by exactly -d. A `rm[:k + 1]` training slice
            # is the easiest leak to write and moves no EARLIER residual, so the
            # check above cannot see it; this one can.
            d = (b[hit - offset] - a[hit - offset]) + 500.0
            if abs(d) > 1e-6:
                bad += 1
                print("  FAIL rotation %d is predicted partly from itself: its"
                      " residual moved by %.6f, not -500" % (hit, b[hit - offset] - a[hit - offset]))

    # 3c · THE CAUSAL DEBIAS IS APPLIED. It is the sheet's own last theory
    # sentence, and nothing else in this selftest would notice it missing: a walk
    # without it computes, is causal, and reports a sigma. On a ramp, where the
    # phase baseline lags by construction, subtracting the residuals already seen
    # has to pull the mean residual materially toward zero.
    ramp = [100.0 + 2.0 * i for i in range(120)]
    with_, without = (walk_forward(ramp, rp, debias=d) for d in (True, False))
    mw = abs(sum(with_) / len(with_))
    mo = abs(sum(without) / len(without))
    if not mo * 0.75 > mw:
        bad += 1
        print("  FAIL the causal debias is not being applied: mean residual %.4f"
              " with it and %.4f without" % (mw, mo))

    # 3d · AND IT IS THE MEAN OF THEM, EXACTLY. The prediction does not depend on
    # any residual, so the undebiased run gives the pure prediction error u, and
    # the debiased residual must satisfy e[k] = u[k] - mean(e[:k]) at every step.
    # An identity rather than an inequality, because "the mean of the residuals
    # already observed" and "the last residual" are both causal, both shrink the
    # mean on a ramp, and only one of them is what the sheet says.
    for k in range(1, 6):
        want = without[k] - sum(with_[:k]) / k
        if abs(with_[k] - want) > 1e-9:
            bad += 1
            print("  FAIL the debias at step %d is not the mean of the residuals seen:"
                  " residual %.6f, the mean rule gives %.6f" % (k, with_[k], want))
            break

    # 4 · the warm-up is the one the header states, so §31.4's arithmetic on the
    # rotation count is arithmetic about this file.
    if len(rm) - len(a) != LEVEL + 3:
        bad += 1
        print("  FAIL the walk scores past a warm-up of %d, not LEVEL + 3 = %d"
              % (len(rm) - len(a), LEVEL + 3))

    # 4b · COVERAGE IS SCALE-FREE, which is the only reason §45 can say anything
    #      while sigma is unreproduced. Doubling every residual must not move it.
    r = [float(x) for x in noise(400, 99)]
    a, b = coverage(r, 1.28), coverage([2.0 * x for x in r], 1.28)
    if abs(a - b) > 1e-12:
        bad += 1
        print("  FAIL coverage moved from %.6f to %.6f when every residual doubled;"
              " it must divide the scale out" % (a, b))
    # and it reads the DESIGN-RELEVANT side. `e = pred - truth`, so the exceedance
    # is -e: a sample of purely negative residuals is all exceedance and nothing
    # is covered. Read the other way round this returns 1.0 and looks fine.
    if coverage([-5.0, -5.0, -5.0, -5.0, 5.0], 0.0) > 0.25:
        bad += 1
        print("  FAIL coverage is counting the wrong side of the residual")
    # On a standard normal it must land near the normal's own figure, or the
    # comparison §45 draws is against the wrong reference.
    if not 0.87 < coverage(r, 1.28) < 0.93:
        bad += 1
        print("  FAIL coverage of a normal sample at 1.28 sigma is %.4f, nowhere "
              "near Phi(1.28) = 0.8997" % coverage(r, 1.28))

    # 4c · THE SIGMA CONVENTION IS THE POPULATION ONE. `report` compares against
    #      13.4544 with it and every coverage figure divides by it; n-1 would
    #      move both and nothing else would notice.
    if abs(_sd([1.0, 2.0, 3.0, 4.0, 5.0]) - math.sqrt(2.0)) > 1e-12:
        bad += 1
        print("  FAIL _sd of 1..5 is %.6f; the population sigma is %.6f"
              % (_sd([1.0, 2.0, 3.0, 4.0, 5.0]), math.sqrt(2.0)))

    # 4d · and a coverage span that straddles the normal is reported as
    #      straddling rather than as one direction.
    for lo, hi, phi, want in ((0.94, 0.96, 0.95, "straddles"),
                              (0.90, 0.94, 0.95, "UNDER"),
                              (0.96, 0.98, 0.95, "OVER"),
                              (0.95, 0.95, 0.95, "UNDER")):
        if want not in verdict(lo, hi, phi):
            bad += 1
            print("  FAIL a span of %.2f-%.2f against %.2f reads %r; expected %s"
                  % (lo, hi, phi, verdict(lo, hi, phi), want))

    # 5 · the quantile convention, at h = (n-1)q.
    for q, want in ((0.0, 1.0), (0.5, 3.0), (1.0, 5.0), (0.25, 2.0), (0.875, 4.5)):
        got = quantile([1.0, 2.0, 3.0, 4.0, 5.0], q)
        if abs(got - want) > 1e-9:
            bad += 1
            print("  FAIL quantile at %.3f gave %.4f, not %.4f" % (q, got, want))

    # 6 · skew has the sign the report reads it for.
    if abs(skew([-2.0, -1.0, 0.0, 1.0, 2.0])) > 1e-9:
        bad += 1
        print("  FAIL skew of a symmetric sample is not zero")
    if skew([0.0, 0.0, 0.0, 0.0, 10.0]) <= 0:
        bad += 1
        print("  FAIL skew of a right-tailed sample is not positive")
    # AND THE OTHER SIGN. The report prints skew as the evidence for the sheet's
    # claim that the residuals lean one way, so a skew that lost its sign would
    # report the lean of a sample leaning the other way, which is the whole
    # claim. A right-tailed case alone passes against `abs()`.
    if skew([-10.0, 0.0, 0.0, 0.0, 0.0]) >= 0:
        bad += 1
        print("  FAIL skew of a left-tailed sample is not negative")

    # 7 · phases stay in [0, 1) across the table's end, which is theory step 5.
    cyc = [("1990-01-01", "2000-01-01"), ("2000-01-01", "2011-01-01"),
           ("2011-01-01", "2021-01-01")]
    ph = phases(epoch("2019-01-01"), 4000, [(a, b) for a, b in cyc])
    if not all(0.0 <= u < 1.0 for u in ph):
        bad += 1
        print("  FAIL a phase left [0, 1) past the table's end")
    if max(ph) > 0.9995:
        bad += 1
        print("  FAIL a phase reached 1.0 rather than clamping below it")

    # 8 · THE REPORT REFUSES ON EITHER GATE, not only on both at once. Called,
    # not grepped for: the probe that stood here looked for the gate's source
    # line, which the probe itself spelled, so it matched its own text and passed
    # against a file whose `or` had been changed to `and`.
    for off, n, want, what in ((0.0, 361, False, "a sample that agrees on both"),
                               (0.06, 361, True, "sigma 6% out on the right count"),
                               (0.0, 376, True, "sigma exact on the wrong count"),
                               (0.06, 376, True, "both wrong")):
        if refuses(off, n, 361) != want:
            bad += 1
            print("  FAIL quantiles are %s for %s"
                  % ("withheld" if want else "printed", what))

    print("selftest: %d cases, %s" % (29, "all as expected" if not bad else "%d FAILED" % bad))
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dump", choices=("f107", "ap"), help="print the residuals and stop")
    ap.add_argument("--ar", choices=("yw", "ls"), default="yw",
                    help="the AR(2) estimator: Yule-Walker (default) or least squares")
    ap.add_argument("--why", action="store_true",
                    help="test each named candidate for the gap; none of them closes it")
    ap.add_argument("--shape", action="store_true",
                    help="what the band covers, scale-free — the part of §30 B3 "
                         "that does not need sigma reproduced")
    ap.add_argument("--selftest", action="store_true",
                    help="check the estimators and the walk against known answers")
    a = ap.parse_args()
    if a.selftest:
        return selftest()
    if a.ar == "ls":
        globals()["ar2"] = ar2_ls
    days, cycles = read_days()
    if a.dump:
        report(a.dump, 0.0, 0, days, cycles, dump=True)
        return 0
    if a.why:
        return why(days, cycles)
    if a.shape:
        return shape(days, cycles)
    report("f107", 13.4544, 361, days, cycles)
    print()
    report("ap", 3.5937, 361, days, cycles)
    print()
    print("The first line of each block is the test, and it does not pass yet. Until")
    print("sigma matches, this reproduces a walk-forward and not THE walk-forward, and")
    print("no number from it belongs on a sheet. `--why` tests each candidate the")
    print("header names and reports that not one of them closes the gap.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except BrokenPipeError:
        # `--dump | head` is the normal way to look at a few of these, and a
        # traceback on a closed pipe is noise rather than news.
        os._exit(0)
