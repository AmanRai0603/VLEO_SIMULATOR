## Equations

```
F107_cyc(phase) = mean over complete cycles of F10.7 at that phase
```

## Derivation

With a phase in hand a design can ask for a level instead of a distribution. The obstacle is that the record holds 28 years and not 28 cycles, so there is no way to average 'days like this one' unless something is chosen that makes two days alike. Phase is that thing, and the construction below is superposed-epoch analysis: express every day as a phase, stack the cycles on top of one another, and average what lands in each bin. What comes out is the shape of the average cycle, which is a different and far more useful object than the average of the record.

1. The question is the expected F10.7 at a given point in a cycle. Averaging the record answers a different question — the expected F10.7 of an unknown day — and gives 114.84 sfu, which is the average of maxima and minima together and describes no part of a cycle.
2. Superposed-epoch analysis: convert every day of every COMPLETE cycle into a phase, bin the phases, and take the mean F10.7 in each bin. Stacking on phase rather than on days-since-minimum is the step that lets cycles of different lengths be compared at all — cycle 23 ran 4338 days and cycle 24 ran 4017, so the same day-number is a different part of the cycle in each. `F107_cyc(p) = mean{ F107(d) : d in cycles 23, 24 and phase(d) in bin(p) }`
3. Twenty equal-width bins, which is a resolution-against-sample-size choice rather than a convention. Twenty bins over two cycles of about 11.4 years each puts roughly 418 days in every bin — enough for a mean to be steady — while a bin still spans only about 209 days of a single cycle, which is short against the years over which a cycle changes. `bin width = 0.05 in phase ~ 209 days per cycle, ~418 days pooled`
4. The counts are not all 418, and they are published rather than smoothed. The 0.775 bin holds 217 days because the two cycles have different lengths and one contributes fewer days there, and 273 consecutive days are missing from the record in 2017. Every fixture beside this node carries its bin's day count, so a thin bin is visible to a reader instead of being hidden inside a mean.
5. Between bin centres the answer is interpolated linearly, and this changes how the row must be read. At the declared epoch's phase of 0.6194 the answer is NOT the nearest bin's mean of 105.84: it is nine tenths of the way from the 0.575 bin toward the 0.625 one. `F107_cyc(0.6194) = 126.23 + 0.889 * (105.84 - 126.23) = 108.14 sfu`
6. The curve that results is asymmetric — 71.8 sfu at phase 0.025, 165.3 at 0.425, 67.7 at 0.975 — a fast rise and a slow decline. That is the known shape of a solar cycle appearing in the record rather than an artefact of the binning, and it is the reason a phase cannot be read as a proxy for activity: phase 0.2 and phase 0.8 are both 'mid-cycle' and are owed 111 and 76 sfu.

## Assumptions

- Two cycles, stacked on phase, in twenty bins. Fails when: cycles 23 and 24 are the only complete ones in the record, so every bin is the mean of two cycles and nothing more. Two samples cannot separate a cycle's shape from a cycle's individuality: cycle 23 peaked at 196 sfu and cycle 24 at 146, a 34% difference, and this row averages them into one curve that matches neither. The bins hold 358 to 418 days each except the 0.775 bin, which holds 217 because the two cycles' lengths differ and the stacking leaves it thin
- It is a mean and not a band. Fails when: half the days at any phase sit above this line. It is the CENTRE for a design value, and sw_uncertainty_growth supplies the spread that makes it safe. Sizing anything on this row alone would be sizing on the average day of the average cycle, which is the one thing a mission is guaranteed not to get
- The curve is not symmetric and the asymmetry is real. Fails when: the mean rises from 71.8 sfu at phase 0.025 to 165.3 at 0.425 and falls to 67.7 by 0.975 — a fast rise and a slow decline, which is the known shape of a solar cycle and not a binning artefact. A design at phase 0.2 and one at phase 0.8 are both 'mid-cycle' and are owed 111 and 76 sfu respectively

## Validity

From 60 to 400 One. Below: the same floor as env_f107: below 60 sfu has never been observed and no relation reading F10.7 has support there. Above: the same ceiling as env_f107: above 400 sfu every consumer of F10.7 is extrapolating. The largest binned mean is 165.3, so this bound is unreachable by the relation and catches a broken table.

It is a mean, so half the days at any phase are above it. That makes it a centre for a design value and not a design value: sw_uncertainty_growth supplies the spread that turns one into the other, and sizing anything on this row alone is sizing for the average day of the average cycle, which is the one sky a mission is guaranteed not to get. With two cycles in the record the curve also matches neither of them — cycle 23 peaked near 196 sfu and cycle 24 near 146 — and sw_cycle_repeatability is the row that says how much that costs.
