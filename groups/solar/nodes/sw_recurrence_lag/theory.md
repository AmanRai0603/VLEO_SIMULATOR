## Equations

```
L_rot = argmax over lag of corr(F107', F107' shifted by lag) = 26 d
```

## Derivation

The Sun rotates, and an active region that faces the Earth today faces it again about a month later. That is the only source of genuine medium-term predictability in solar flux, and it is the reason the published outlook is 27 days long rather than 7 or 90. Measuring it means finding the lag at which a day best predicts a later day — but the eleven-year cycle already makes every day resemble its neighbours, so the cycle has to be removed first or the answer is simply 'lag 1'. The whole of the care in this row is in the removal and in reading the resulting peak correctly.

1. The quantity wanted is the autocorrelation of F10.7 against itself at a range of lags, and its first interior maximum. Applied to the raw series that fails: the eleven-year cycle dominates, correlation stays high for hundreds of days, and no rotation bump is visible against it.
2. So the cycle is removed first by subtracting a 365-day centred mean. The window length is the choice that makes the measurement possible: at 365 days it is thirteen rotations long, so it removes the cycle and the annual terms while leaving the rotation untouched. A window near 27 days would remove the rotation itself and the row would measure nothing. `F107'(d) = F107(d) - mean_365_centred(d); sd(F107') = 19.85 sfu, mean -0.03`
3. Correlate the residual with itself at each lag and look for the first interior peak. It appears where expected, at lag 26, with a correlation of +0.376. `L_rot = argmax_lag corr(F107', shift(F107', lag))`
4. Now the part that would be an error to skip. The residual's autocorrelation decays steeply on its own — +0.936 at lag 1 down to -0.066 at lag 13 — and the rotation bump rides on the tail of that decay. A bump sitting on a falling background has its apparent maximum pulled toward zero lag, so the measured peak is expected to come in slightly SHORT of the true period.
5. The harmonics settle it, because they sit far enough out that the decay no longer tilts them. The second peak is at lag 54 and the third at lag 81 — exactly 27.0 days per cycle in both cases. `54 / 2 = 27.0 81 / 3 = 27.0`
6. So the period is 27 days and this row's 26 is the first-peak estimate, one day short for a reason that is understood and measured. It reports the measured peak because that is what the study's own C.rot_peak_lag field holds, and the sheet says which number to use for which question.

## Assumptions

- 26 is where the first peak sits and 27 is the period, and the one-day gap is a known bias in the estimator rather than a disagreement. Fails when: the first peak is read as the rotation period. The autocorrelation of the detrended series decays steeply from +0.936 at lag 1 to -0.066 at lag 13, and the rotation bump rides on the tail of that decay, so its apparent peak is pulled toward zero lag. The harmonics settle it: the second peak is at lag 54 and the third at lag 81, both exactly 27.0 days per cycle, and they sit far enough out that the decay no longer tilts them. So the period is 27 days and the first-peak estimate is one day short. This row reports the measured peak, which is what the MATLAB field C.rot_peak_lag holds; a row that needs the period should use 27 and cite the harmonics
- The bump is broad, so the single lag overstates how sharp the recurrence is. Fails when: it is treated as a period a design can phase-lock to. Correlation exceeds +0.34 at every lag from 24 to 28 and exceeds +0.30 from 23 to 29 — a seven-day-wide shoulder. The Sun does not rotate as a solid body: the equator turns in about 25 days and mid-latitudes in about 28, and active regions emerge and decay within a rotation. So the recurrence is a tendency over a week-wide window, not a clock
- The 11-year cycle was removed with a 365-day centred mean, and that choice sets what is left. Fails when: the detrending window is comparable to the signal. At 365 days it is thirteen rotations long, so it removes the cycle and the annual terms while leaving the rotation untouched; the residual has a standard deviation of 19.85 sfu about a mean of -0.03. A window near 27 days would remove the rotation itself and this row would measure nothing. 3 of 10319 days in the record have no F10.7 and are excluded pairwise rather than interpolated

## Validity

From 20 to 35 Day. Below: below 20 days the autocorrelation is still on the steep descent from lag 1 and is falling, not peaking: it reads +0.186 at lag 20 against +0.376 at the peak. A value there would mean the detrending removed the rotation instead of the cycle. Above: above 35 days the first bump has closed — the correlation is +0.031 at lag 35 and negative by 36 — and anything beyond is the second harmonic near lag 54, which is the same signal counted twice. A value there would be reporting a harmonic as the fundamental.

26 is where the measured peak sits and 27 is the period, and the difference is a bias in the estimator rather than a disagreement about the Sun. A row that needs the PERIOD should use 27 and cite the harmonics. The bump is also broad rather than sharp: correlation exceeds +0.34 at every lag from 24 to 28 and +0.30 from 23 to 29. The Sun does not rotate as a solid body — the equator turns in about 25 days and mid-latitudes in about 28 — and active regions emerge and decay within a rotation, so the recurrence is a tendency over a week-wide window and not a clock a design can phase-lock to.
