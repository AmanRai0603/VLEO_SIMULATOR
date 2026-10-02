## Equations

```
Ap_central = the last rotation forecast, held forward = 22.0954
```

## Derivation

The window this tool designs for opens after the record ends, so there is no observation of it and no forecast reaching it. What the study does, and what is done here, is to hold the LAST rotation forecast forward: the pattern's estimate of the level the sun is at when the record stops, carried across the window unchanged.

That is a weak statement and it is meant to be. It says the mission will sit where the sun was last seen sitting, and its whole uncertainty is sw_ap_mean_band_spread. It does not decline with the cycle, it does not rise into the next maximum, and a window opening five years out gets the same centre as one opening next month.

1. Cut the record into 27-day rotations and predict each from the ones before it, by the method sw_ap_mean_band_spread describes. `pred(b) from rotations 1..b-1`
2. Take the last such prediction and hold it forward across the window. `Ap_central = pred(last) = 22.0954`

## Assumptions

- The level the sun was last at is the level it will be at. Fails when: the window is long or far out. This carries no cycle trend at all: the same number is published for a window opening next month and one opening in 2032, and over a 365-day window the sun demonstrably moves
- A rotation-mean level stands in for a daily level. Fails when: a design reads it as a day. It is the mean of 27 days; half the days in the window are above it by construction, which is what sw_ap_daily_band_spread exists to say
- The phase past the cycle table is an extrapolation. Fails when: it is read as measured. The last rotations sit past cycle 25's tabulated end and their phase wraps on the mean length of three cycles, one of which is incomplete

## Validity

From 1 to 40 One. Below: below 1 the level is beneath anything the record holds over a rotation: the quietest of 381 rotations in bundles/solar-weather@2026.09.14 has a mean Ap of 1.48. The bound WAS 0, carrying a reason about DAILY Ap — a single quiet day really is Ap 0 — which is the wrong quantity for this row. A rotation mean of 0 needs 27 consecutive Ap-0 days and the record has never held two. Above: above 40 the value exceeds anything the record supports for this quantity, so it is an arithmetic error rather than an active sun. This row is a rotation-scale level, and the largest rotation mean of Ap in the 381 rotations of bundles/solar-weather@2026.09.14 is 36.96; the bound WAS 80, which is more than twice a level the sun has never reached over a rotation.

22.0954, against a record whose daily Ap runs from 0 to over 200. The study's own nominal scenario for its window is 22.0925 — 0.013 per cent away, from a derivation that shares no code with it.
