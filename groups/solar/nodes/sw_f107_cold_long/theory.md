## Equations

```
F107_cold_long = F107_central - 1.28 * sigma_total
```

## Derivation

This is sw_f107_design_long with the sign of one term reversed, and it is a separate row rather than a parameter on that one for a reason that is about reading rather than about arithmetic. A design reads the hot level in one place and the cold level in another, and the two are consumed by different subsystems for different purposes. A single row with a sign flag would be a row whose meaning depends on a value somebody has to go and look up.

The band is symmetric HERE and asymmetric one row down, which is worth holding in mind. Both edges of the mean band are one sigma away from the centre because sigma is a standard deviation and a standard deviation has no sides. The DAILY band is not symmetric — sw_daily_band_spread reaches 34.23 above and sw_daily_band_drop only 31.27 below — because those are percentiles of a skewed sample rather than multiples of a spread. So the cold-day scenario is not the mirror of the hot-day one, while the cold-mean scenario is the mirror of the hot-mean one, and the two facts sit in adjacent rows.

The multiplier is 1.28, inherited from the same decision its twin records: the study forms the mean band as centre +/- 1.28*sigma while labelling the run 95 per cent, and in the same function forms the daily band at the true 0.95. That is reproduced rather than repaired, and reproducing it means using the same 1.28 on this side. Changing it would have to change both edges at once or the band would stop being a band.

1. Take the centre the cycle analogue gives for the window. `F107_central = sw_central_expectation`
2. Take the spread the pattern has historically missed a rotation level by. The same number the hot edge uses, because it is one spread and not two. `sigma_total = sw_mean_band_spread = 13.4544 sfu`
3. Subtract 1.28 of them. Symmetric with the hot edge, because a standard deviation has no sides; the DAILY band one row down is not symmetric, because percentiles of a skewed sample do. `F107_cold_long = F107_central - 1.28 * sigma_total`

## Assumptions

- 1.28 is the confidence, and it is the 90th percentile while the run is called 95 per cent. Fails when: a reader takes the published band as a 95 per cent bound. Phi(1.28) = 0.8997, so this edge is the 10th percentile and not the 5th. A one-sided 95 per cent bound is 1.645 sigma, which at this sigma is a further 4.9 sfu DOWN. The daily half of the same band DOES use 0.95, so the two halves are not at one confidence, and this row reproduces that rather than silently repairing it Which number that is, is now a row of its own — sw_band_confidence, seeded by §30 B2 and unanswered, because the value needs a person. Until it carries one the multiplier is a literal in this hole and three others, each saying it is declared in the sheet while no sheet declares it. §44 measures what each answer costs: moving to 1.645 moves 47 rows and no KPI closure, leaves all five solar closures closing with 3.1 to 6.2 per cent less margin, and breaks no parity check in this repository
- The band is symmetric because sigma is a standard deviation. Fails when: the residuals are skewed, which they are. A forecast that misses hardest when activity is highest has a long high tail and a short low one, so the true 10th percentile of the residuals is nearer the centre than 1.28 sigma and this edge is a little too cold. It errs toward the conservative on THIS side — a colder cold case is a harder case for drag authority — but it is the wrong number, not a safe one
- One sigma covers the whole window. Fails when: sigma is not flat across the cycle — the source says so about its own number — so a window spanning a rise or a fall is given one width where it needs two
- The cold edge is a design case and not a nuisance. Fails when: it is read as the harmless side. Thin air is the case with the least aerodynamic control authority, the slowest differential-drag phasing, the longest end-of-life de-orbit and the least array flux. Two of those are mission-ending in their own way

## Validity

From 60 to 400 One. Below: below 60 sfu has never been observed and every relation reading F10.7 has no support there. On this row the guard means something specific: a sustained level below the record's own floor says the band is wider than the sky, which happens when a window near solar minimum is given a sigma measured across a whole cycle. Above: above 400 sfu the exospheric temperature relation is extrapolated past the largest recorded daily value. A COLD level there is arithmetic rather than sky — it means the spread has been added rather than subtracted, which is the one failure this row has that its twin does not.

Fed the study's own two numbers the arithmetic is exact: 158.3304112313 minus 1.28 * 13.4968815767 is 141.0544028132, which is the coldmean scenario it published to the last digit. With this repository's sigma in place of the study's it is 141.1088, 0.04 per cent away. That is what the parity grid beside this sheet checks, and it is a check on the RELATION.

The answer this row actually publishes is not 141. On the nominal case it is 69.6281 sfu, because sw_central_expectation does not hold the last rotation forecast forward the way the study does — it reads the cycle analogue over the mission's own dates and gives 86.8497 at the declared epoch, a deliberate and recorded departure. So this row's own number and the scenario it is the parity of are 71 sfu apart, and the whole of that gap is upstream of this relation.

69.6 sfu clears the guard at 60 by nine and a half. Its short sibling does not: take the daily drop off and the result is 38.36, which the guard refuses. That refusal is the correct outcome and it is documented in sw_f107_cold_short rather than hidden, because it says something true — a symmetric band with a stacked daily percentile is not a valid construction at a centre this low.
