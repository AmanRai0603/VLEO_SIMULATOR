## Equations

```
Ap_long = Ap_central + 1.28 * sigma_ap
```

## Derivation

Ap_central says where the window sits, sw_ap_mean_band_spread says how wrong the pattern has been about that, and 1.28 of the spread is the margin. The multiplier is the same 1.28 as its F10.7 twin and carries the same problem: it is the one-sided 90th percentile of a normal while the run is labelled 95 per cent, and the daily half of the band uses the true 0.95.

Which number that is, is now a row of its own — sw_band_confidence, seeded by §30 B2 and unanswered, because the value needs a person. Until it carries one the multiplier is a literal in this hole and three others, each saying it is declared in the sheet while no sheet declares it. §44 measures what each answer costs: moving to 1.645 moves 47 rows and no KPI closure, leaves all five solar closures closing with 3.1 to 6.2 per cent less margin, and breaks no parity check in this repository.

Ap makes the normal assumption worse than F10.7 does. The index floors at zero, a quiet rotation cannot undershoot far, and an active one overshoots a long way — so the residuals are strongly right-skewed and a symmetric z multiplier understates exactly the tail a design is sized against.

That is true of the far tail and NOT of this multiplier, which §45 measures: at 1.28 sigma the Ap band holds 0.9148 to 0.9239 of the exceedances against a normal's 0.8997, so it over-covers by 1.5 to 2.4 points. The understatement starts beyond it — the 99th percentile of exceedance runs 2.53 to 2.87 sigma where a normal says 2.33.


## Assumptions

- The margin is 1.28 spreads, which is the one-sided 90th percentile of a normal while the run is labelled 95 per cent. Fails when: the band is read as a 95 per cent bound. The daily half of the same band uses the true 0.95; which number both halves should read is sw_band_confidence, which is unanswered because the value needs a person
- A symmetric multiplier is a usable margin for an index that floors at zero and swings a long way upward. Fails when: a design is sized on the far upper tail. At 1.28 spreads the Ap band holds 0.9148 to 0.9239 of the exceedances against a normal's 0.8997, so it slightly over-covers; further out the record is heavier, with the 99th percentile of exceedance at 2.53 to 2.87 spreads where a normal says 2.33
- One spread, pooled over the record, stands for the mission window. Fails when: the window falls in the declining phase, where geomagnetic activity is burstier and the pooled spread gives a band too narrow — the assumption sw_ap_mean_band_spread states about its own number

## Validity

From 0 to 300 One. Below: Ap floors at zero — a perfectly quiet day is Ap 0 — so a design level below it means a spread has been subtracted rather than added. Above: above 300 the level exceeds the largest daily Ap in the record, 273, so a SUSTAINED level there is not a window this tool can model.

26.6953 at the declared window, against the study's hotmean of 26.6833 — 0.045 per cent, and the construction is exact on the study's own inputs: 22.0925383902 + 1.28 * 3.5865545525 = 26.6833282174.
