## Equations

```
P_exc(Ap_design) = median cycle phase of the days with Ap >= Ap_design
```

## Derivation

How often and how long still leave a programme unable to plan. The third question is WHEN — because if exceedances were uniformly distributed through a cycle there would be nothing to schedule around, and if they cluster there is. The interesting part of this row is not the median it publishes but the test it survives: a timing statistic is only about the cycle if it does not move when the threshold moves, and this one barely moves across a factor of nearly three in threshold.

1. Take the days that exceed the design bound, convert each to its cycle phase on the same 0-to-1 scale sw_cycle_phase uses, and take the median of those phases. `P_exc = median{ phase(d) : Ap(d) >= Ap_design }`
2. The median rather than the mean, because phase is nearly circular and a handful of days near the ends of a cycle would drag a mean toward the middle regardless of where the mass sat.
3. Now the test that decides whether the answer means anything. If the median tracked the threshold, it would be telling us about where the line was drawn. It does not: 0.5717 at the G1 bound, 0.5763 at G2, 0.6027 at G3 — a drift of three hundredths across a threshold ratio of nearly three, on samples of 131, 35 and 8 days.
4. So the timing is a property of the solar cycle rather than of the design bound. Disturbed days cluster after maximum, on the declining phase, which is where coronal holes dominate the solar wind — and three independent samples agreeing to within 0.03 is better evidence for that than any one of them alone.
5. Two caveats about the phase scale itself, neither of which the median can show. Cycle 25's end in the record is the record's end rather than a real minimum, so phases inside it are computed against a length not yet known; and cycles 23 and 24 differ in length by 8 per cent, so equal phase is unequal time.
6. And the 2017 gap sits at phase 0.74 to 0.80 of cycle 24, just past the upper edge of the G3 exceedance band. Whatever fell there is absent from this median and from the rate that goes with it.

## Assumptions

- A median with no spread beside it, and the spread is the part a schedule needs. Fails when: 0.603 is read as when exceedances happen. It is where the middle one happened. At G3 the eight days run from phase 0.294 to 0.736 — a band nearly half a cycle wide, about five years — and they fall in phase deciles 0.2-0.3, 0.3-0.4, 0.5-0.6 twice, 0.6-0.7 three times and 0.7-0.8 once. Half of them sit between 0.2 and 0.6. A design that reads only this row plans for a date; a design that reads the assumption plans for a five-year window
- It is stable across G level, which means it is the cycle rather than the threshold. Fails when: the median is expected to move with the bound. It barely does: 0.571692 at the G1 bound, 0.576302 at G2 and 0.602697 at G3, a drift of three hundredths across a factor of nearly three in threshold. So the timing is a property of the solar cycle — disturbed days cluster after maximum, on the declining phase, where coronal holes dominate — and not of where the line is drawn. That stability is the reason the row is worth having: it would be meaningless if it tracked the threshold
- Eight days at G3, and a median of eight numbers is a coarse instrument. Fails when: precision is read into 0.603. The sample is 131 days at G1, 35 at G2 and 8 at G3, so at the declared level the median sits between the fourth and fifth of eight values. Moving one storm moves the median by a few hundredths. The three levels agreeing to within 0.03 on samples of 8, 35 and 131 is better evidence for the timing than any one of them alone
- Phase is folded on three cycles, one of which is incomplete, and 273 days are missing from the risky part. Fails when: the phase scale is assumed uniform. Cycle 25's end in solar_cycles.csv is the record's end rather than a real minimum, so phases inside it are computed against a cycle whose length is not yet known; cycles 23 and 24 differ in length by 8 per cent, so equal phase is unequal time. And the 2017 gap sits at phase 0.74 to 0.80 of cycle 24, just past the upper edge of the G3 exceedance band — so whatever fell there is absent from this median and from sw_exceedance_rate's count

## Validity

From 0 to 1 One. Below: cycle phase runs 0 at minimum to 1 at the next minimum, so 0 is the floor by definition. A median at 0 would mean every exceedance fell on the first day of a cycle. Above: 1 is the end of a cycle by the same definition. A median at 1 would mean every exceedance fell on the last day of one.

A median with no spread beside it, and the spread is the part a schedule needs. 0.603 is where the middle exceedance happened, not when exceedances happen. At G3 the eight days run from phase 0.294 to 0.736 — a band nearly half a cycle wide, about five years — with half of them between 0.2 and 0.6. A design reading only this row plans for a date; a design reading the spread plans for a five-year window. The declared epoch at phase 0.6194 sits inside that band, which is what makes the epoch a conservative choice rather than a convenient one.
