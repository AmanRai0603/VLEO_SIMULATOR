## Equations

```
Ap_cold_short = Ap_cold_long - dAp_day_low
```

## Derivation

The two spreads are of different things and they stack rather than combine. sw_ap_mean_band_spread is how wrong the pattern is about the LEVEL a rotation's Ap sits at — a forecast error. sw_ap_daily_band_drop is how far a day falls below the rotation it belongs to — the field's own variability, measured on days that already happened. A day is quiet because its rotation is quiet AND because it is a quiet day within that rotation, so the design level for a quiet day is the sustained quiet level with the daily drop taken off.

Two things about this row are specific to Ap rather than inherited from its F10.7 counterpart.

The first is how much of the answer the daily term is. From 17.50 down to 4.93 is a drop of twelve and a half units on a level of seventeen and a half — seventy-two per cent of the level. On F10.7 the same construction moves 69.63 to 65.24, six per cent. For the quiet Ap case the within-rotation term is not a correction to the rotation level, it is most of the answer.

The second is the floor. Ap cannot be negative. A symmetric band already ignores that, and stacking a daily drop on top of it makes the crossing close: with this subsystem's measured spread and its level-conditioned drop, a sustained quiet level below about 15.6 puts this row at or under zero. The declared window's 17.50 clears it by under two units of LEVEL, which is under five units of ANSWER. That is the narrowest margin to a declared bound anywhere in this subsystem, and a deep-minimum window would fail the guard here — correctly, because what the arithmetic would otherwise publish is not a geomagnetic index.

1. Take the sustained cold Ap level: the centre less 1.28 standard deviations of the rotation-forecast residual. `Ap_cold_long = sw_ap_cold_long`
2. Take how far a day falls below its own rotation, at the declared confidence AND AT THE LEVEL THAT ROTATION SITS AT. The LOW tail read at the QUIET level: 12.56 at Ap 17.50, against the high tail's 63.85 read at Ap 26.70. `dAp_day_low = sw_ap_daily_band_drop(Ap_cold_long) = 12.5635 at a sustained 17.50`
3. Subtract it. On Ap this term is seventy-two per cent of the level it is taken from, against six per cent on F10.7: for the quiet Ap case it is most of the answer rather than a correction to it. `Ap_cold_short = Ap_cold_long - dAp_day_low`

## Assumptions

- The result stays above zero, and nothing in the arithmetic ensures it. Fails when: the sustained quiet level is small. With this subsystem's level-conditioned drop the crossing is at a sustained Ap near 15.6, and the declared window's 17.50 clears it by under two units of level. The guard refuses rather than publishing a negative index, which is right, but it means this row is the one most likely in the subsystem to fire on an ordinary input rather than on a mistake
- The two spreads stack rather than combine. Fails when: a reader takes the result as a 95 per cent day. Stacking a 10th-percentile rotation level with a 5th-percentile day inside it is nearer a 1-in-100 day than a 1-in-20 one, assuming independence — and a quiet rotation is made of quiet days, so they are not independent. The study does this and the port reproduces it; the number is conservative and its label is wrong
- The quiet day is not the disturbed day mirrored. Fails when: somebody builds it by negating sw_ap_design_short's daily term. Two things make that wrong. The tails differ by a factor of two to three at every level; and the two rows read the table at DIFFERENT LEVELS, Ap 17.50 here against 26.70 there, so the terms are 12.56 and 63.85 — a factor of five. Mirroring would put this scenario at Ap -46, which is not a sky
- The daily drop measured before the window applies inside it. Fails when: the window spans a different part of the cycle. Ap's within-rotation variability peaks in the DECLINING phase, when coronal holes are largest and high-speed streams recur, rather than at maximum. A drop measured on one phase is the wrong depth for another

## Validity

From 0 to 400 One. Below: Ap floors at zero — a perfectly quiet day is Ap 0 and there is nothing below it. This is the row in the subsystem closest to its own floor, seven units clear at the declared window, and the only one where the guard is likely to fire on an ordinary input rather than on a mistake. Above: 400 is the top of the Ap index itself; a value above it is not a geomagnetic index at all. On the QUIETEST of the five scenarios a value anywhere near it means a sign is wrong somewhere in the chain above.

Fed the study's own two numbers the arithmetic is exact: its coldmean 17.5017485630 minus its own daily drop of 10.6296296296 is 6.8721189333, which is the coldday Ap it published to the last digit. That is what the parity grid beside this sheet checks, and it is a check on the RELATION.

What this row publishes is 4.93. It was 6.91 until sw_ap_daily_band_drop was conditioned on the rotation level: the drop at a sustained Ap of 17.50 is 12.56 rather than the single 10.59 measured over rotations averaging Ap 11.64.

Ap 4.93 is a quiet but wholly ordinary day; the record holds long stretches below it. The guard at zero has under five units of room, which is the narrowest margin any row in this subsystem carries — and narrower than before the correction, because the drop grew faster than the level it is taken from.
