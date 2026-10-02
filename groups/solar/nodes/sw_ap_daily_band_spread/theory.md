## Equations

```
dAp_day(L) = pctl(Ap - movmean(Ap, 27 d), 0.95) over days whose rotation sits within 15% of L
```

## Derivation

The method is sw_daily_band_spread's, applied to Ap: subtract the centred 27-day moving mean from the daily record, bin every day by the level of the rotation it sits in, take the 95th percentile inside each bin. Geometric ladder of knots, each bin every day within 15 per cent of its knot, a knot kept only while its bin holds at least 200 days and both tails stay monotone, the ladder stopping at the last knot that passes. That gives eight knots from Ap 4 to Ap 26.

What is different about Ap is the SHAPE of the dependence. On F10.7 the table rises smoothly by a factor of ten across its range. On Ap it rises by eleven and most of that is in the last third: 14.67 at Ap 11, 23.79 at 15, 28.68 at 20, 40.61 at 24, 63.85 at 26. A disturbed rotation is not a quiet rotation scaled up — it is a rotation containing storms, and a storm day departs from its rotation by an amount that has nothing to do with the rotation's own level.

That steepness is why the table stops at 26. Beyond it the record has 170 days at Ap 28 and 105 at Ap 30, across 27 rotation-sized blocks in 28.2 years, and a 95th percentile of 170 days is the ninth-largest of them. The ladder stops where the rule stops it and the answer is clamped above, which the assumptions below say plainly costs something at exactly the level the declared window sits at.

1. Take the daily Ap record and subtract the centred 27-day moving mean from each day. `dev(t) = Ap(t) - movmean(Ap, 27)(t)`
2. Bin every day by the level of its own rotation, within 15 per cent of each knot, on a geometric ladder. `bin(L) = { dev(t) : |movmean(Ap,27)(t) - L| <= 0.15 L }`
3. Read the 95th percentile inside each bin. Keep a knot only while its bin holds at least 200 days and both tails stay monotone; stop at the last knot that passes. Eight knots, Ap 4 to 26, the thinnest holding 300 days. `hi(L) = pctl(bin(L), 0.95) — 6.02 at Ap 4, 14.67 at 11, 28.68 at 20, 63.85 at 26`
4. Interpolate between knots and clamp beyond the ends. The declared window sits above the top knot, so its answer is the clamp. `dAp_day(L) = interp(L; knots) = 63.85 at L = 26.70, which is the clamp`

## Assumptions

- THE DECLARED WINDOW IS ABOVE THE TOP KNOT, so its answer is the clamp and not an interpolation. Fails when: the sustained Ap exceeds 26, as it does here at 26.70. The table holds its top value rather than extrapolating, and the segment below that top knot is the steepest in the table — 40.61 at Ap 24 to 63.85 at 26. Extrapolating that slope to 26.70 would give about 72 rather than 63.85, so the clamp is the CONSERVATIVE-DOWNWARD choice here and the published single-day Ap may be low. The record cannot settle it: Ap 28 holds 170 days and Ap 30 holds 105
- Conditioning on level more than doubled this row's consumer, and the previous value was not a margin but an error. Fails when: this is read as a refinement. sw_ap_design_short was 41.70 and is 90.55. A design sized on 41.70 for its own hot scenario was under-designed by a factor of two, because the band it used was measured over a sample whose rotations averaged Ap 11.64 and applied at 26.70
- The top of the table rests on very little record. Fails when: a design sits near it. The Ap 26 knot holds 300 days, and the whole record has only 27 rotation-sized blocks above Ap 22 in 28.2 years. A 95th percentile of 300 days is the fifteenth-largest of them, so the top of this table moves if one storm period is added to or removed from the record. The bottom knots hold thousands of days and are not like this, and the table does not say which regime a caller is in at the point of use
- A disturbed rotation is not a quiet rotation scaled up. Fails when: somebody fits a smooth law through these knots. The rise is a factor of eleven and most of it is in the last third, because high-Ap rotations contain storms and a storm day's departure has little to do with its rotation's own level. A power law or a constant ratio through this table would be wrong at both ends
- The 27-day moving mean is the rotation. Fails when: Ap's recurrence is driven by coronal holes and high-speed streams whose period is nearer 27.0 days than the 27.27 of the equatorial photosphere, and they persist for many rotations. A 27-day window is the conventional round number rather than a measured period
- It is the record's own daily scatter, not a forecast error. Fails when: this is read as an uncertainty. It says how variable the geomagnetic field is within a rotation, measured on days that already happened

## Validity

From 0 to 150 One. Below: a value below zero is not a spread, and Ap itself floors at zero — a quiet day really is Ap 0. Above: above 150 the value exceeds anything the record supports for this quantity, so it is an arithmetic error rather than an active sun.

63.85 at the declared window, where sw_ap_design_long puts the sustained disturbed level at 26.70 — just above the table's top knot, so the answer is the clamp. The old single-number version gave 15.00, measured over days whose rotations averaged Ap 11.64 and applied unchanged at 26.70.

sw_ap_design_short therefore moves from 41.70 to 90.55, and THAT BREAKS A CLOSURE: l3_solar_req_05 commits the design to surviving a single day at the G2 threshold, Ap 80, and the record says this window's own hot scenario reaches 91. Ap 91 is a G3-G4 day. It is consistent with the rest of the subsystem — sw_storm_return_level says the one storm expected in the whole mission is Ap 158 — and it means either the single-day commitment belongs at G3 or this window is outside what the design was built for. That is a decision, not an arithmetic consequence, and the requirement has NOT been moved to make the closure pass.
