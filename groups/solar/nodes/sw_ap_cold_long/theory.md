## Equations

```
Ap_cold_long = Ap_central - 1.28 * sigma_ap
```

## Derivation

sw_ap_design_long with the sign of one term reversed, and a separate row for the same reason its F10.7 counterpart is: the two edges are read in different places by different consumers, and a single row with a sign flag would be a row whose meaning depends on a value somebody has to go and look up.

The mean band is symmetric because sigma is a standard deviation and a standard deviation has no sides. On Ap that symmetry is in tension with the quantity itself, which is bounded below at zero and unbounded above, and the tension is visible in the numbers: the DAILY Ap band reaches 15.00 up and only 10.59 down, forty-two per cent apart, while this mean band is exactly as far down as up. One of the two is describing the sky's shape and the other is describing a spread, and only the first can be asymmetric.

That is not a defect to be fixed here. A standard deviation of a residual is what sw_ap_mean_band_spread measures and 1.28 of it is what the study subtracts; reproducing the port means reproducing both. It IS a reason to read this row's assumptions rather than only its number, because at a low enough centre the symmetric subtraction reaches below zero and the guard, not the arithmetic, is what stops it.

1. Take the centre the cycle analogue gives for Ap over the window. `Ap_central = sw_ap_central_expectation`
2. Take the spread the pattern has historically missed a rotation's Ap by. The same number the hot edge uses, because it is one spread and not two. `sigma_ap = sw_ap_mean_band_spread = 3.5937`
3. Subtract 1.28 of them. Symmetric with the hot edge, unlike the daily band two rows down, where the two tails are forty-two per cent apart because Ap floors at zero. `Ap_cold_long = Ap_central - 1.28 * sigma_ap`

## Assumptions

- 1.28 is the confidence, and it is the 90th percentile while the run is called 95 per cent. Fails when: a reader takes the published band as a 95 per cent bound. Phi(1.28) = 0.8997, so this edge is the 10th percentile and not the 5th. A one-sided 95 per cent bound is 1.645 sigma, a further 1.3 down at this sigma. The daily half of the same band DOES use 0.95, so the two halves are not at one confidence, and this row reproduces that rather than silently repairing it Which number that is, is now a row of its own — sw_band_confidence, seeded by §30 B2 and unanswered, because the value needs a person. Until it carries one the multiplier is a literal in this hole and three others, each saying it is declared in the sheet while no sheet declares it. §44 measures what each answer costs: moving to 1.645 moves 47 rows and no KPI closure, leaves all five solar closures closing with 3.1 to 6.2 per cent less margin, and breaks no parity check in this repository
- A symmetric band on a quantity truncated at zero. Fails when: the centre is small. Ap cannot be negative, so the true low edge of any band is bounded by the centre itself, and a symmetric subtraction of 1.28 sigma ignores that. With this sigma the crossing is at a centre near 4.6, which is a deep-minimum window rather than an impossible one. The guard catches it; the arithmetic does not know about it
- One sigma covers the whole window. Fails when: sigma is not flat across the cycle, and Ap's is least flat of all — geomagnetic activity peaks in the DECLINING phase rather than at maximum, when coronal holes are largest and high-speed streams recur. A window spanning that transition is given one width where it needs two
- The quiet edge is a design case and not a nuisance. Fails when: it is read as the harmless side. A quiet field is the coolest, thinnest thermosphere, the weakest signal a magnetometer has to work with and the least torque a magnetorquer can generate. The last of those sizes an actuator

## Validity

From 0 to 300 One. Below: Ap floors at zero — a perfectly quiet day is Ap 0 and there is nothing below it. On this row the guard is load-bearing rather than decorative: a symmetric band subtracted from a small centre reaches below zero, and what comes out then is arithmetic rather than sky. Above: above 300 the level exceeds the largest daily Ap in the record, 273. A COLD level there means the spread has been added rather than subtracted, which is the one failure this row has that its twin does not.

At the study's own window the arithmetic is exact: its nominal 22.0925383902 minus 1.28 * 3.5865545525 is 17.5017485630, which is the coldmean Ap it published to the last digit. This repository's own chain gives 17.4955, 0.04 per cent away, because its centre and its sigma are its own measurements.

Ap 17.5 is an ordinary quiet-to-unsettled field. The guard at zero has three and a half units of room at this centre — a centre below about 4.6 with this sigma would reach it — so for the declared window it is a check rather than a constraint. A window at deep solar minimum is where it earns its place.
