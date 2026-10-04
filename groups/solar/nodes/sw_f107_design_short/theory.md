## Equations

```
F107_short = F107_long + dF107_day
```

## Derivation

The two spreads are of different things and they stack rather than combine. sw_mean_band_spread is how wrong the pattern is about the LEVEL a rotation sits at — a forecast error. sw_daily_band_spread is how far a day departs from the rotation it belongs to — the sun's own variability, measured on days that already happened. A day is high because its rotation is high AND because it is a high day within that rotation, so the design level for a day is the sustained level with the daily departure added on top.

That is a deliberately conservative construction and the sheet says so. Adding two one-sided percentiles gives a bound further out than either — roughly the 99th rather than the 95th if the two were independent and normal — so the published day is rarer than the confidence attached to it suggests. The study does the same thing and the port follows it.

1. Take the sustained level the window is designed to. `F107_long = sw_f107_design_long`
2. Take how far above its rotation a day reaches, one day in twenty, AT THE LEVEL THAT ROTATION SITS AT. sw_daily_band_spread is a table rather than a number: the same question answered at 104 sfu and at 210 gives 20.07 and 46.93, a factor of two and a bit. `dF107_day = sw_daily_band_spread(F107_long) = 20.0723 sfu at a sustained 104.07`
3. Add them. A high day on a high rotation is both, and the design must survive the pair. `F107_short = F107_long + dF107_day`

## Assumptions

- The two spreads stack, and stacking two percentiles is not a percentile. Fails when: the published number is read as the 95th percentile of a day. It is not: it is the 90th percentile of the rotation level plus the 95th percentile of the daily departure, which for independent normals lands near the 99th. The bound is conservative, the label is not, and the honest statistic would be the percentile of the daily value itself rather than a sum of two
- The rotation error and the daily departure are independent. Fails when: they are not. Both widen with activity, so a window the pattern gets wrong on the high side is also a window whose days scatter most, and the true joint tail is fatter than the sum of two marginals suggests in one direction and thinner in the other
- One day in twenty is the day worth designing to. Fails when: the mission is long. Over a 365-day window a one-in-twenty day happens about eighteen times, so this is not a rare event but a routine one; the rare day a design might actually care about is further out and this row does not publish it

## Validity

From 60 to 400 One. Below: below 60 sfu has never been observed and every relation reading F10.7 has no support there; a single-day design level below it means a spread has been subtracted rather than added. Above: above 400 sfu the exospheric temperature relation is extrapolated past the largest recorded daily value — and this row, being the sustained level plus a daily excursion, is the one most likely to reach it, which is exactly why the guard is here.

Fed the study's own two numbers the arithmetic is exact: 175.6064196495 plus 34.1481481481 is 209.7545677976, which is the hotday scenario it published, to the last digit. That is what the parity grid beside this sheet checks, and it is a check on the RELATION.

What this row publishes is 124.14, not 209.75, and both terms moved to get there. The sustained level is 104.07 rather than 175.61 because sw_central_expectation reads the cycle analogue instead of holding the last rotation forecast forward. And the daily term is 20.07 rather than 34.23 because sw_daily_band_spread is now conditioned on the rotation level: the old single number was measured over days whose rotations averaged 136.63 sfu and applied unchanged at 104.07, which over-stated it by 71 per cent.

The cold side is not this row — sw_f107_cold_short is, and it is the case a design checks for a cold, thin atmosphere. It is NOT this row mirrored: the two daily tails differ at every level.
