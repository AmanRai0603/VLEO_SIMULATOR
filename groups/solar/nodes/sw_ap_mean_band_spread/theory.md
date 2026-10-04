## Equations

```
sigma_ap = std(pred - truth) over 361 walk-forward next-rotation forecasts = 3.5937
```

## Derivation

Identical in method to sw_mean_band_spread and different in what it measures: a phase-conditioned baseline shrunk toward the global mean, scaled to the last thirteen rotations, plus an AR(2) on the anomalies, causally debiased, fitted only on rotations before the one predicted. The spread of what is left is this row.

Ap is the harder of the two to predict from a pattern and the number says so. 3.59 against a mean daily Ap near 12 is a relative spread far worse than F10.7's 13.45 against 130 — geomagnetic activity is driven by what the solar wind happens to do at Earth, and a cycle-phase baseline knows nothing about that.

1. Cut the record into 27-day rotations and take the mean Ap of each. `rm(b) = mean(Ap over days 27b .. 27b+26)`
2. Predict each from the rotations before it, never from itself. `pred(b) = base(phase)*scale + AR(2) on anomalies - causal debias`
3. The spread of what is left, over every rotation scored. `sigma_ap = std(pred - truth) = 3.5937 over 361 rotations`

## Assumptions

- One sigma holds across the whole cycle. Fails when: it does not, and the source says so about its own number. Geomagnetic activity is burstier near the declining phase than at minimum, so a window there is given a band too narrow
- The residual spread is a usable margin for a non-negative index. Fails when: Ap floors at zero and its residuals are strongly skewed — a quiet rotation cannot undershoot far but an active one can overshoot a long way. A symmetric sigma understates the high tail, which is the tail a design is sized against. MEASURED the same way, and the same correction applies: at 1.28 sigma the Ap band holds 0.9148 to 0.9239 of the exceedances against a normal's 0.8997, so it OVER-covers by 1.5 to 2.4 points rather than understating. Ap's skew is stronger than F10.7's, -0.84 to -1.32 against -0.61 to -1.01, and its far tail is milder: the 99th percentile of exceedance runs 2.53 to 2.87 sigma against F10.7's 2.93 to 3.49. §45
- The 273-day hole in 2017 is interpolated before the rotations are cut. Fails when: those days are counted as observations. Interpolated Ap is far smoother than real Ap, so the rotations covering them are easier to predict than any real rotation and the pooled spread is narrower than the record supports

## Validity

From 0 to 6 One. Below: a value below zero is not a spread, and Ap itself floors at zero — a quiet day really is Ap 0. Above: above 6 the residual would exceed the standard deviation of the rotation means themselves, so the pattern would be worse than predicting the record's own mean. This row is identical in method to sw_mean_band_spread and takes that row's criterion rather than the vaguer one it carried before: the 381 Ap rotation means of bundles/solar-weather@2026.09.14 have a standard deviation of 4.96, and 6 is that rounded up. The bound WAS 20, which is four times what the record supports for a residual of this quantity.

3.5937. prf_rebuild's own figure for the same quantity is 3.5866 — 0.20 per cent apart, between two implementations of one method sharing no code.
