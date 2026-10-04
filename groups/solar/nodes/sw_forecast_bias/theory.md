## Equations

```
B_f107(L) = mean over issues of (F107_forecast - F107_observed) at lead L
```

## Derivation

A skill score is built on squared errors and is therefore blind to sign: a forecast that is consistently 4 sfu low and one that is consistently 4 sfu high score identically. For a drag design the sign is the whole point. A low F10.7 means a thin atmosphere, less drag and a longer predicted lifetime, so an under-forecasting outlook does not add scatter to a design — it moves the design in the unsafe direction. This row measures the signed error, which is the part of the verification a margin cannot absorb.

1. Take the signed error rather than its square or its magnitude: forecast minus observed, averaged over issues at a fixed lead. Signed, so that a consistent lean shows up instead of cancelling into a variance. `B(L) = mean over issues of ( F107_forecast - F107_observed ) at lead L`
2. The first thing to check is whether the mean is distinguishable from zero at all, because a well-constructed forecast should have no systematic lean. This one does, and at every single lead: from -0.593 sfu at lead 1 to -3.968 at lead 26, never crossing zero across 26 leads and samples of 866 to 1257 pairs each.
3. A sign that is the same 26 times out of 26 is not scatter about a correct central value. The published outlook systematically under-forecasts F10.7, and a design reading it gets a thinner atmosphere, lower drag and a longer predicted lifetime than it will fly.
4. The magnitude is not monotone in lead, and the interior structure is real rather than noise: the bias deepens to -2.748 sfu at lead 9, recovers to -1.848 at lead 13, then deepens again to -3.968 by lead 26. With over 1200 pairs per lead, that shape is a property of how the outlook is constructed.
5. So the row is a table over measured leads. Fitting a trend through it would erase the recovery in the middle, which is the one part of the curve that tells a reader the outlook is not simply degrading with distance.
6. The declared range is worth noting as a piece of the argument. The upper bound is -0.5 rather than 0, because an answer of -0.1 sfu would be as wrong as +0.1 and a guard placed at zero would let it through.

## Assumptions

- The outlook is low at every lead in the window, and that is the unsafe direction. Fails when: the sign is read as incidental. Measured over 1997-2025 the bias is negative at all 26 verifiable leads, from -0.593 sfu at lead 1 to -3.968 sfu at lead 26, and it never crosses zero. So this is not scatter about a correct central value: the published outlook systematically under-forecasts F10.7, and a design reading it gets a thinner atmosphere, a lower drag and a longer predicted lifetime than it will fly. The remedy is to add the bias back, not to trust the outlook and widen a margin somewhere else
- One mean over 29 years of a varying Sun. Fails when: the bias is cycle-dependent, which it will be, because a forecaster's error on a 250 sfu day is not the error on a 70 sfu day. The sample is 866 to 1257 issue-target pairs per lead, pooled across cycles 23, 24 and the rise of 25 without conditioning on activity. A design at a known cycle phase is owed a phase-conditioned bias and this row does not give one; it gives the average over the record, which is the honest thing to publish from a pooled sample and is not the same thing
- The magnitude is small against the quantity and large against the margin. Fails when: it is compared to F10.7 itself. Minus 4 sfu on a mean of 115 sfu is 3.5%, which sounds negligible, and it is not: sw_uncertainty_growth already carries the spread a design must survive, and this bias sits underneath it as an offset that no amount of spread removes. A symmetric band around a biased centre is still centred in the wrong place
- Non-monotone in lead, and the middle of the curve is not noise. Fails when: somebody fits a straight line through it. The bias deepens to -2.748 sfu at lead 9, recovers to -1.848 at lead 13, then deepens again to -3.968 by lead 26. That interior minimum is present in a sample of over 1200 pairs per lead, so it is a property of how the outlook is constructed and not sampling scatter. The table interpolates between the measured leads rather than fitting a trend, because there is no trend to fit

## Validity

From -4 to -0.5 One. Below: the deepest measured bias is -3.968 sfu, at lead 26, the last verifiable lead. A value below -4 means the table was misread or the bundle changed underneath it. Above: the shallowest measured bias is -0.593 sfu, at lead 1. A value above -0.5 — and certainly a positive one — would say the outlook over-forecasts F10.7 somewhere in the window, which this record does not show at any lead. The guard is deliberately on the safe side of zero rather than at zero, because an answer of -0.1 sfu would be as wrong as +0.1 and a bound at zero would pass it.

Minus 4 sfu on a mean of 115 is 3.5 per cent, which sounds negligible and is not. sw_uncertainty_growth already carries the spread a design must survive, and this bias sits underneath it as an offset that no amount of spread removes: a symmetric band around a biased centre is still centred in the wrong place. The remedy is to add the bias back, not to trust the outlook and widen a margin somewhere else. It is also a single mean pooled across 29 years of a varying Sun, and a forecaster's error on a 250 sfu day is not the error on a 70 sfu day.
