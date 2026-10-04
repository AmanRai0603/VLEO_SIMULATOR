## Equations

```
dF107_day(L) = pctl(F107 - movmean(F107, 27 d), 0.95) over days whose rotation sits within 15% of L
```

## Derivation

A rotation mean is an average over 27 days, and averages hide the days they are made of. The sun turns, an active region faces us for a week and is gone, and F10.7 swings tens of sfu around a rotation mean that never moves. A design told only the mean is told the quiet part.

The measurement is deliberately not a model. Take the daily record, subtract its own centred 27-day moving mean, and what is left is each day's departure from the rotation it belongs to — the sun's rotation removed by construction rather than by a fitted period. The 95th percentile of that sample is how far above its rotation a day reaches, one day in twenty.

WHAT IS NEW IS THAT THE SAMPLE IS CONDITIONED ON THE LEVEL. Every day of the record is binned by the level of the rotation it sits in, and the percentile is taken inside each bin. The knots are a geometric ladder; each bin is every day whose rotation sits within 15 per cent of the knot; a knot is kept only while its bin holds at least 200 days and the table stays monotone in both tails, and the ladder stops at the last knot that passes both. That gives seven knots from 70 to 210 sfu, the thinnest holding 850 days.

The 15 per cent half-width is a compromise and it is worth naming: narrower bins resolve the level dependence better and hold fewer days, so the percentile gets noisier. 15 per cent keeps every bin above two thousand days except the top one and still separates 70 from 210 by a factor of ten in the answer.

1. Take the daily record and subtract the centred 27-day moving mean from each day. What remains is the departure from the rotation that day sits in, with the rotation removed by construction rather than by a fitted period. `dev(t) = F107(t) - movmean(F107, 27)(t)`
2. Bin every day by the level of its own rotation, on a geometric ladder, each bin holding every day whose rotation sits within 15 per cent of the knot. `bin(L) = { dev(t) : |movmean(F107,27)(t) - L| <= 0.15 L }`
3. Read the 95th percentile inside each bin. Keep a knot only while its bin holds at least 200 days and both tails stay monotone; stop at the last knot that passes. `hi(L) = pctl(bin(L), 0.95) — 7 knots, 70 to 210 sfu, thinnest bin 850 days`
4. Interpolate between knots and clamp beyond the ends, because the record does not continue past them and a straight line drawn onward would be this row inventing sky. `dF107_day(L) = interp(L; knots) = 20.07 at L = 104.07`

## Assumptions

- The departure scales with the level, and a single number for it is wrong away from its own sample's mean. Fails when: this is read as a refinement. It is a correction. Over the whole record the 95th-percentile departure runs 4.63 at a rotation level of 70 to 46.93 at 210 — a factor of ten — and the previous version of this row published one number, 34.23, measured on a 2001-day window whose rotations averaged 136.63. Applied at 69.63 it took the cold chain below the floor of 60 and sw_f107_cold_short refused. The refusal was the construction failing, not the guard being tight
- The table is clamped beyond its ends rather than extrapolated. Fails when: a design level falls outside 70 to 210 sfu. Below 70 the answer is held at 4.63 and above 210 at 46.93. The record's own rotation means run from about 65 to 253, so the top of that range is reachable by a mission at cycle maximum, and a level above 210 gets a band measured at 210 — narrower than the record supports. The clamp is deliberate and it is the same choice sw_kp_from_ap makes at the ends of the published Kp scale
- A 15 per cent bin half-width, which trades resolution against sample size. Fails when: either matters more than the other. Narrower bins resolve the level dependence better and hold fewer days, so the percentile gets noisier; wider bins are the defect this row exists to fix, in miniature. 15 per cent keeps every bin above 2000 days except the top one, at 850, and still separates the ends by a factor of ten
- The knots rest on the whole record, and the record is 28.2 years. Fails when: the top knot is leaned on. 210 sfu holds 850 days against 3061 at 70, and those days are concentrated in two cycle maxima. A design at a high sustained level is reading a percentile with far less behind it than one at a low level, and the table does not say so at the point of use
- The 27-day moving mean is the rotation. Fails when: the solar rotation is 27.27 days at the equator and slower at the poles, and the active longitudes that drive F10.7 are not at one latitude. A 27-day window is the conventional round number rather than a measured period, and the departures it leaves carry whatever the mismatch contributes
- It is the record's own daily scatter, not a forecast error. Fails when: this is read as an uncertainty. It is not: it says how variable the sun is within a rotation, measured on days that already happened. What a forecast of a future day would get wrong is sw_uncertainty_growth's question and a larger number

## Validity

From 0 to 120 One. Below: a spread of zero would mean every day equals its rotation mean, which the record contradicts on every day it holds; below zero is not a spread. Above: above 120 sfu the departure exceeds the largest single-day excursion in the record, so a value there is an arithmetic error rather than an active sun.

20.07 sfu at the declared window, where sw_f107_design_long puts the sustained hot level at 104.07. The old single-number version of this row gave 34.23 for the same window — 71 per cent higher — because it was measured on days whose rotations averaged 136.63 and applied unchanged at 104.

So sw_f107_design_short falls from 138.30 to 124.14. The correction goes the other way on the cold side, where it matters more: sw_daily_band_drop gives 4.39 at a rotation of 69.63 against the old 31.27, and sw_f107_cold_short goes from REFUSING to publishing 65.24.
