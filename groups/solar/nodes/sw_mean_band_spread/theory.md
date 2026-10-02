## Equations

```
sigma_total = std(pred - truth) over 361 walk-forward next-rotation forecasts = 13.4544 sfu
```

## Derivation

A central expectation is only usable in a design if the part it CANNOT explain travels with it. The way to measure that part is to make the forecast the tool would have made, at every rotation of the record, using only what was known before that rotation, and look at how far it landed from the truth.

The forecast is a pattern and not a model of the sun. In each step the training slice is everything before the rotation being predicted. A phase-conditioned baseline is fitted in twenty bins of cycle phase, each bin pulled toward the global mean in proportion to how thin it is — a bin with four rotations in it is a noisy estimate and shrinkage is the standard remedy. That baseline is scaled to the level of the last thirteen rotations, so a cycle larger than average is not predicted down to the average. What is left after the baseline is an anomaly, and an AR(2) fitted on the training anomalies carries the rotation-to- rotation memory. The mean of the residuals ALREADY OBSERVED is subtracted as a causal debias.

The spread of what remains is this row. It is the honest width of the central expectation, and it is deliberately NOT a forecast error in the usual sense: there is no forecast in it, only the cycle and the memory of the last two rotations.

1. Cut the record into 27-day rotations and take the mean of each. The question is about the level a rotation sits at, not about the days inside it — those are sw_daily_band_spread's question. `rm(b) = mean(F107 over days 27b .. 27b+26)`
2. For each rotation in turn, fit only on the rotations before it. A baseline in twenty phase bins, each shrunk toward the global mean by n/(n+5), then scaled by the level of the last thirteen rotations and clamped so a thin stretch cannot swing the scale. `base(p) = w*mean_bin + (1-w)*mean_all, w = n/(n+5); sc = mean(rm[-13:]) / mean(base[-13:]), clamped to [0.4, 2.5]`
3. Fit an AR(2) on the training anomalies and predict the next one, then subtract the mean of the residuals already seen. Both halves are causal: nothing after the predicted rotation is used. `pred = base(p_next)*sc + a1*an[-1] + a2*an[-2] - mean(residuals so far)`
4. The spread of what is left, over every rotation the walk-forward scored. `sigma_total = std(pred - truth) = 13.4544 sfu over 361 rotations`
5. Phase past the end of the cycle table WRAPS on the mean cycle length rather than stopping at 1. Sixteen days of the record sit past cycle 25's tabulated end, and clamping them to phase 1 puts them at the cycle minimum: it moved the held-forward centre from 158.37 to 138.74, fourteen per cent, while barely touching this pooled number. `u = mod((t - start_last) / mean_cycle_length, 1), clamped to 0.999`

## Assumptions

- One sigma holds across the whole cycle. Fails when: it does not, and the source this was rebuilt from says so about its own number: 'sigma is NOT flat across the cycle; a design that uses one number is too tight somewhere and too loose somewhere else.' prf_rebuild reports sigma split into five phase bins for that reason. This row publishes the pooled number, so a design near solar maximum is given a band that is too narrow and one near minimum a band too wide
- The cycle table ends before the record does, and the phase past it is an extrapolation. Fails when: it is read as measured. Cycle 25 is tabulated to 2025-12-15 and the record runs to 2025-12-31, so the last sixteen days carry a phase computed from the MEAN length of the three cycles the table holds — one of which is itself incomplete. Every day the window is held forward from inherits that extrapolation
- The 273-day hole in 2017 is filled by straight-line interpolation before the rotations are cut. Fails when: those interpolated days are counted as observations. Nine months of invented flux sit inside about ten rotations, and they are smoother than the sun, so every one of those rotations is easier to predict than a real one and the pooled spread is a little narrower than the record can support
- It is the residual of a pattern, not of a forecast. Fails when: this is read as what a forecaster would get wrong. There is no forecast in it: no flare watch, no active-region count, no observation later than the rotation before. A real 27-day outlook does better, which is what sw_forecast_skill measures

## Validity

From 0 to 45 One. Below: a spread of zero would mean the pattern predicts every rotation exactly, which the record contradicts at every rotation it scores; below zero is not a spread. Above: above 45 sfu the residual would exceed the standard deviation of the rotation means themselves, so the pattern would be worse than predicting the record's own mean and the band would be arithmetic rather than physics. The bound WAS 60, which was wider than this same sentence justifies: the 382 rotation means of bundles/solar-weather@2026.09.14 have a standard deviation of 41.45 sfu, and 45 is that rounded up far enough that refreshing the bundle does not move the bound.

13.4544 sfu, against rotation means that run from about 70 to 240 over the record. The pattern explains most of where a rotation sits and misses by about fourteen units of flux, and a design band of centre +/- 1.28*sigma is roughly +/- 17 sfu.

The same method on Ap gives 3.5937. It is measured and recorded here but not published, for the same reason as the daily row: sw_ap_design_long does not exist yet, and a published number nothing reads is a number nobody checks.
