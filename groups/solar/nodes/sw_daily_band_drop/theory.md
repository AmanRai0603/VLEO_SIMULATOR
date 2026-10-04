## Equations

```
dF107_day_low(L) = -pctl(F107 - movmean(F107, 27 d), 0.05) over days whose rotation sits within 15% of L
```

## Derivation

sw_daily_band_spread takes the daily record, subtracts its own centred 27-day moving mean, bins every day by the level of the rotation it sits in, and reads the 95th percentile inside each bin. This reads the 5th percentile of the same bins, and publishes its magnitude. Same sample, same moving mean, same ladder of knots, same 15 per cent bins, same rule for where the ladder stops — one measurement, read from both ends.

That it is a second row rather than the same row negated holds at every level. At a rotation of 70 the two tails are 4.63 up and 4.39 down, six per cent apart; at 145 they are 34.61 and 32.39; at 210, 46.93 and 40.78, fifteen per cent apart. The reason is physical rather than statistical: F10.7 has a floor near 65 sfu that the quiet sun sits on, so there is less room below a rotation than above it, and the gap widens as the rotation rises above the floor.

The magnitude is published rather than the signed percentile so that the row reads as a distance, its declared floor is zero, and the sign lives in the relation that consumes it. A row whose value is negative and whose consumers must remember to add rather than subtract is a row waiting to be used the wrong way round once.

1. Take the daily record up to the day the window opens and subtract the centred 27-day moving mean. The sample is identical to sw_daily_band_spread's — the same days, the same moving mean, one measurement. `dev(t) = F107(t) - movmean(F107, 27)(t)`
2. Bin every day by the level of its own rotation, within 15 per cent of each knot, exactly as the high tail does. `bin(L) = { dev(t) : |movmean(F107,27)(t) - L| <= 0.15 L }`
3. Read the 5th percentile instead of the 95th. One call on one sorted bin gives both tails; this row is the lower one. `lo(L) = pctl(bin(L), 0.05)`
4. Publish its magnitude, interpolated between knots and clamped beyond the ends, so the row is a distance with a floor of zero and the sign travels with the relation that uses it. `dF107_day_low(L) = -interp(L; knots) = 4.39 at L = 69.63, which is the clamp`

## Assumptions

- The departure scales with the level, and the old single number was seven times too large at this window. Fails when: this is read as a refinement. 31.27 subtracted from 69.63 is 38.36 sfu, which has never been observed and which sw_f107_cold_short's guard refused. Conditioned on the level the drop is 4.39. A factor of seven is not a correction to a margin, it is a different answer
- The declared window sits at the clamp, so this row is reading the end of its own table. Fails when: the sustained cold level falls below 70 sfu, as it does here at 69.63. The answer is held at the lowest knot's 4.39 rather than extrapolated, which is the right choice — below 70 the record has few rotations and a line drawn onward would be invention — but it does mean this window's answer is the table's endpoint and not an interpolation. A colder window gets the same 4.39
- The departures are asymmetric, and this row exists because of it. Fails when: it is treated as the negation of sw_daily_band_spread. The gap runs from six per cent at a rotation of 70 to fifteen per cent at 210, because F10.7 has a floor near 65 and no ceiling, and the room below a rotation shrinks as the rotation approaches that floor. Mirroring the high tail onto the low one is asserting a symmetry the record refuses at every level
- A 15 per cent bin half-width, which trades resolution against sample size. Fails when: either matters more than the other. Narrower bins resolve the level dependence better and hold fewer days, so the percentile gets noisier; wider bins are the defect this row exists to fix, in miniature
- The 27-day moving mean is the rotation. Fails when: the solar rotation is 27.27 days at the equator and slower at the poles, and the active longitudes that drive F10.7 are not at one latitude. A 27-day window is the conventional round number rather than a measured period, and the departures it leaves carry whatever the mismatch contributes
- It is the record's own daily scatter, not a forecast error. Fails when: this is read as an uncertainty. It is not: it says how far below its rotation the sun has gone, measured on days that already happened. What a forecast of a future day would get wrong is sw_uncertainty_growth's question and a larger number

## Validity

From 0 to 120 One. Below: a drop of zero would mean no day ever falls below its rotation mean, which the record contradicts on about half the days it holds; below zero is not a distance, and a negative value here means the tail has been read from the wrong end. Above: above 120 sfu the departure exceeds the largest single-day excursion in the record in either direction, so a value there is an arithmetic error rather than a quiet sun.

4.39 sfu at the declared window, where sw_f107_cold_long puts the sustained cold level at 69.63 — just under the table's lowest knot, so the answer is the clamp. The old single-number version of this row gave 31.27 for the same window, seven times as much, because it was measured over days whose rotations averaged 136.63 and applied unchanged at 69.63.

That seven-fold error is what made sw_f107_cold_short refuse. It now publishes 65.24 sfu, which clears the floor of 60 by five and is a genuinely quiet day rather than an impossible one.
