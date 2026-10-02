## Equations

```
F107_window_peak(T_e, L) = max over t in [T_e, T_e+L] of A(t)
```

## Derivation

A centre and a design value are different questions and a mission long enough to cross solar-cycle phases separates them sharply. Over the declared five-year window the analogue opens near 105 sfu and ends near 73; its mean is 86.8 and its maximum 110.7. Drag is not linear in flux, a vehicle does not get to average its propellant over five years, and the sizing case for almost everything downstream — thrust, propellant, intake area, achievable lifetime — is the busiest sustained period rather than the typical one.

So the maximum is published as its own row rather than left for a reader to infer from the centre. It is not a worst case and does not pretend to be: it is the highest the EXPECTED level goes, with no excursion on top, and the excursion is sw_uncertainty_growth's business.

1. Take the same analogue sw_central_expectation averages: the completed cycles' mean shape, wrapped on one mean cycle length, scaled by cycle 25's own peak inside cycle 25 and by the completed-cycle mean outside it. `A(t) = amp(t) * R((t - T_max) mod P)`
2. Maximise it over the window rather than integrating. The relation and the data are identical to the centre's; only the statistic differs, which is why the two rows share one implementation in the kernel and cannot drift apart. `F107_window_peak = max over [T_e, T_e + L]`
3. The maximum is EXACT rather than sampled, and it is worth saying why that is possible. The analogue is linear between knots, so on any interval its largest value is at an end of the interval or at a knot inside it — a finite list, not a search.
4. With one exception, which is the part that is easy to get wrong. The amplitude hands over with a step DOWN at half a period either side of the maximum, so the analogue's supremum just inside cycle 25 is not attained at any knot and is larger than the knot value there. Checking ends and knots alone looks safe and is not: over 6432 windows spanning the declared domain the handover's limit sets the answer in 18 of them, by as much as 4.05 sfu.
5. So the candidate list is the two ends, every knot strictly inside, and the limit at every handover strictly inside. That list is exhaustive, so the answer carries no discretisation error of its own — unlike the centre, whose window integral is numerical.

## Assumptions

- This is the peak of the EXPECTATION, not a peak of the sky. Fails when: the analogue is a mean over completed cycles of a smoothed level. The record's daily F10.7 reaches 343 sfu and this row's ceiling anywhere is 225.1; a single rotation can exceed this row's answer by a factor of two and nothing here is wrong when it does. A design wanting a level it will not see exceeded needs this plus an excursion, which is sw_uncertainty_growth, or the 95th percentile the design row already composes.
- Outside cycle 25 the amplitude is the mean of TWO completed cycles, and their spread is a factor of 1.41. Fails when: this row inherits the assumption from the analogue it reads. Cycle 23 peaked at 226.8 sfu and cycle 24 at 160.9, so the 193.9 used for every future cycle is the midpoint of two numbers 41% apart — and because this row reports a MAXIMUM, a long window's answer is usually exactly that amplitude rather than something averaged near it. The fifteen-year case returns 193.858 sfu, which is the assumption showing through undiluted. If the next cycle runs like cycle 23 this row is 17% low.
- A maximum is not a duration. Fails when: the row says how high the expected level gets and says nothing about how long it stays there. A window whose peak is a brief crossing of a cycle maximum and one that sits at maximum for two years return the same number. Anything sizing a propellant budget or a lifetime needs the integral, which is sw_central_expectation, and anything sizing a thermal or power case may need neither.

## Validity

From 60 to 400 One. Below: the analogue's smallest value anywhere is its smallest shape on its smallest amplitude, 0.324026 x 193.8580 = 62.8 sfu, and a maximum over any window is at least that. 60 is env_f107's floor, below which no F10.7 has been observed, so an answer under it means the shape table or an amplitude has been corrupted rather than that the Sun is quiet. Above: the analogue's largest value anywhere is 1.000000 x 225.1358 = 225.1 sfu, cycle 25's own 81-day peak, because the shape is normalised to one at a cycle maximum. 400 is env_f107's ceiling and is unreachable by this relation; it catches a broken amplitude rather than an extreme sky.

A ceiling on the expectation, not on the sky. A single disturbed day can and does exceed this by a large margin — the record reaches 343 sfu — and nothing about this number bounds one. What it bounds is the level a design must hold for months at a time somewhere inside its window.

Read it beside sw_central_expectation and the gap between them says how much the mission's own length is costing it. The two are equal for a window short enough to sit on one part of a cycle, and a fifteen-year window puts 105.7 against 193.9 — at which point a design sized to the centre is sized to roughly half the level it will actually meet.
