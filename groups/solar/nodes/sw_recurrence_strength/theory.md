## Equations

```
r_rot = corr(F107', F107' shifted by L_rot) = 0.375988
```

## Derivation

The previous row establishes that a rotation signal exists and where it is. Its existence is not the design question: a signal can be perfectly located and useless. What matters is how much of a future day it actually determines, and the answer decides whether a 27-day outlook is a planning tool or a curiosity. The measurement is the correlation at the peak, and the honest way to report it is squared.

1. Read the correlation of the detrended series with itself at the recurrence lag. Nothing new is computed — this is the height of the peak whose position the previous row reports. `r_rot = corr(F107', shift(F107', 26 d)) = 0.375988`
2. A correlation is not a fraction of a signal, and quoting it as one overstates the recurrence by a factor of nearly three. The share of variance a linear predictor explains is the correlation SQUARED, which is the number a design should hear. `r^2 = 0.1414 — one seventh of the detrended variance`
3. The signal does not die within one rotation, which is worth knowing because this row reports only the first peak. The second harmonic at lag 54 still carries +0.136 and the third at lag 81 carries +0.067.
4. That decay across harmonics — 0.376, 0.136, 0.067 — is roughly geometric, which is the physical lifetime of an active region showing up in the statistics: each rotation the same region returns a little weaker, and after three it is faint but not gone.
5. The upper bound of the declared range is set by a consistency argument rather than by an extreme. A rotation-ahead correlation above 0.7 would exceed the same series' five-day correlation of +0.531, which would mean F10.7 is more predictable 26 days out than 5 days out. That is the shape of answer this guard exists to refuse.

## Assumptions

- A correlation of 0.376 explains 14 per cent of the variance, and that is the number a design should hear. Fails when: the correlation is read as the fraction of the signal that recurs. Squared it is 0.1414, so a rotation ahead the recurrence accounts for one seventh of the detrended variation and the other six sevenths is new. It is enough to make a 27-day outlook better than nothing — sw_forecast_skill measures +0.018 still at lead 23 — and nowhere near enough to size anything on
- It is the strength at the FIRST peak and the signal keeps going. Fails when: the recurrence is assumed to die within one rotation. The second harmonic at lag 54 still carries +0.136 and the third at lag 81 carries +0.067, so an active region is faintly detectable three rotations out. This row reports only the first peak, which is what the MATLAB field C.rot_peak_r holds. The decay across harmonics — 0.376, 0.136, 0.067 — is roughly geometric and is the physical lifetime of an active region showing up in the statistics
- Pooled over cycles 23, 24 and the rise of 25, and the recurrence is certainly not constant across them. Fails when: a design at a known cycle phase wants the recurrence it will actually see. At solar maximum there are many active regions and their overlap blurs the rotation signal; near minimum a single long-lived region can dominate it. This is one correlation over 10242 detrended days spanning all three, so it averages regimes in which the mechanism differs. A phase-conditioned version would be a separate row and would need the epoch, which now exists

## Validity

From 0.1 to 0.7 One. Below: below 0.1 the rotation bump would be indistinguishable from the noise floor of the detrended series, whose correlation sits between -0.09 and +0.03 across lags 36 to 47. A value there means the detrending removed the signal. Above: above 0.7 a rotation-ahead correlation would be stronger than the measured one-day correlation of the same series at lag 5 (+0.531), which would mean F10.7 is more predictable 26 days out than 5 days out. The measured value is 0.376 and nothing in the record approaches 0.7.

0.376 squared is 0.141, so a rotation ahead the recurrence accounts for one seventh of the detrended variation and the other six sevenths is new. That is enough to make a 27-day outlook better than nothing — sw_forecast_skill still measures +0.018 at lead 23 — and nowhere near enough to size anything on. It is also pooled across cycles 23, 24 and the rise of 25, and the mechanism is not the same in each: at maximum many active regions overlap and blur the signal, while near minimum a single long-lived region can dominate it.
