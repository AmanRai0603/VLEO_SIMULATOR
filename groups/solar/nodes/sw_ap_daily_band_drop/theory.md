## Equations

```
dAp_day_low(L) = -pctl(Ap - movmean(Ap, 27 d), 0.05) over days whose rotation sits within 15% of L
```

## Derivation

sw_ap_daily_band_spread's measurement, read from the other end: the same daily record, the same centred 27-day moving mean, the same bins of rotation level, the 5th percentile instead of the 95th, published as a magnitude.

The asymmetry is the reason this is a row and it is larger on Ap than anywhere else in the subsystem. At Ap 4 the tails are 6.02 up and 2.97 down; at Ap 11, 14.67 and 7.56; at Ap 26, 63.85 and 21.02. The high tail is roughly twice the low one at the bottom of the ladder and three times at the top. Ap is bounded below at 0 and a quiet day really does sit there, so a rotation whose mean is 20 cannot fall more than 20 and can rise without limit — the distribution is not merely skewed, it is truncated, and the truncation bites harder the lower the rotation.

The low tail is also much better behaved than the high one. It rises smoothly by a factor of seven across the ladder where the high tail rises by eleven with most of it in the last third, because a quiet day is a quiet day whatever the rotation and a storm day is not.

1. Take the daily Ap record and subtract the centred 27-day moving mean. The sample is identical to sw_ap_daily_band_spread's — the same days, the same moving mean, one measurement. `dev(t) = Ap(t) - movmean(Ap, 27)(t)`
2. Bin every day by the level of its own rotation, within 15 per cent of each knot, exactly as the high tail does. `bin(L) = { dev(t) : |movmean(Ap,27)(t) - L| <= 0.15 L }`
3. Read the 5th percentile instead of the 95th. Eight knots, Ap 4 to 26: 2.97, 4.17, 5.59, 7.56, 10.84, 14.30, 17.69, 21.02. `lo(L) = pctl(bin(L), 0.05)`
4. Publish its magnitude, interpolated between knots and clamped beyond the ends. The declared window sits between two knots, so this is an interpolation. `dAp_day_low(L) = -interp(L; knots) = 12.56 at L = 17.50`

## Assumptions

- The result stays above zero, and nothing in the arithmetic ensures it. Fails when: the sustained quiet level is small. With this table the crossing is at a sustained Ap near 15.6, and the declared window's 17.50 clears it by under two units of level — where before, with a single 10.59, the crossing was at 15.2. Conditioning on level did not move that crossing much because the drop falls with the level too, but the margin on the ANSWER narrowed from seven to five. sw_ap_cold_short's guard refuses rather than publishing a negative index, and it is the row most likely in this subsystem to fire
- Ap's departures are strongly asymmetric at every level, and this row exists because of it. Fails when: it is treated as the negation of sw_ap_daily_band_spread. The high tail is twice the low one at Ap 4 and three times at Ap 26. Anything that mirrors one onto the other is wrong by a factor of two to three, and in the direction that puts the cold day below zero
- A percentile of departures does not know that Ap floors at zero. Fails when: the rotation mean it is subtracted from is smaller than the drop. The table is measured from a sample that is itself truncated, so the low tail already reflects the floor implicitly — but the interpolation between knots does not, and neither does the subtraction in sw_ap_cold_short. The guard there is what stops it, not the statistic here
- A 15 per cent bin half-width, which trades resolution against sample size. Fails when: either matters more than the other. The declared window sits between the Ap 15 knot, holding 1558 days, and the Ap 20 knot, holding 803. Both are well populated, so this row's own answer is on firmer ground than its high-tail twin's, which is reading the table's top knot at 300 days
- The 27-day moving mean is the rotation. Fails when: Ap's recurrence is driven by coronal holes and high-speed streams whose period is nearer 27.0 days than the 27.27 of the equatorial photosphere. A 27-day window is the conventional round number rather than a measured period
- It is the record's own daily scatter, not a forecast error. Fails when: this is read as an uncertainty. It says how far below its rotation the geomagnetic field has gone, measured on days that already happened

## Validity

From 0 to 150 One. Below: a drop of zero would mean no day ever falls below its rotation mean, which the record contradicts on about half the days it holds; below zero is not a distance, and a negative value here means the tail has been read from the wrong end. Above: above 150 the value exceeds anything the record supports for this quantity, so it is an arithmetic error rather than a quiet sky.

12.56 at the declared window, where sw_ap_cold_long puts the sustained quiet level at 17.50 — between the Ap 15 and Ap 20 knots, so this is a genuine interpolation rather than a clamp, unlike its high-tail twin. The old single-number version gave 10.59.

sw_ap_cold_short therefore falls from 6.91 to 4.93. That is the narrowest margin to a declared bound anywhere in this subsystem: five units above a floor of zero, where before it was seven. With this table a sustained Ap below about 15.6 would put the cold day through the floor and the row would refuse.
