## Equations

```
C(L) = RMS[F107(t+L) - 114.8437], from the record
```

## Derivation

A structure function on its own says how fast persistence degrades and not whether persistence is worth using. That needs the other cheap forecast beside it: ignore today entirely and predict the record's mean. Where the two curves cross is the predictability horizon — the lead beyond which knowing today's flux buys nothing. This row is the second curve, and its defining property is one that looks like a defect until the purpose is clear: it barely depends on the lead at all.

1. Define the climatology forecast as predicting the record's unconditional mean for every day, regardless of lead, and take its RMS error at the same leads the persistence curve uses. `C(L) = RMS[ F107(t + L) - 114.8437 ]`
2. Because the forecast does not use the lead, its error cannot depend on the lead except through which target days happen to be in the sample. So the curve should be flat, and a baseline that degrades with lead would be the wrong baseline to compare against.
3. Measured, it is flat to within 1 sfu across three orders of magnitude in lead — 44.39 at one day, 45.32 at two years. The whole variation comes from the target-day set shifting rather than from anything becoming harder.
4. The value it is flat AT is the record's own standard deviation about its mean, which makes it a consistency check on the persistence row: the saturation limit quoted there, sqrt(2) times sigma, is sqrt(2) times this number. `C(L) ~ sigma = 44.39 sfu; sqrt(2) * 44.39 = 62.77`
5. Put the two curves together and the horizon falls out. Persistence starts far below climatology — 7.1 against 44.4 at a one-day lead — rises past the rotation dip, and crosses somewhere between 547 and 730 days.
6. One caveat that shortens the answer rather than lengthening it. This baseline knows nothing about the solar cycle, and a phase-conditioned climatology would be much better than 114.84 everywhere — so the horizon this pair implies is the horizon against a deaf baseline, and a fair one would be reached sooner. The date needed to build the fair baseline now exists; this row deliberately does not use it, because changing the baseline would change what the horizon means without changing its name.

## Assumptions

- It is nearly flat, and it is the record's own standard deviation. Fails when: the climatology forecast ignores the lead entirely, so its error is the spread of the target days about the mean and nothing more: 44.39 sfu at a one-day lead, 45.32 at two years. The whole variation across eighteen leads is under 1 sfu, and it comes from the set of target days shifting rather than from anything getting harder. Anyone expecting a baseline that degrades with lead is expecting the wrong baseline — that is the point of using it as one.
- The crossover is the design answer, and this row does not state it. Fails when: read against sw_horizon_persistence, persistence is the better forecast out to about 547 days and is beaten by 730 — so today's flux is worth something for roughly a year and a half, which is longer than the 27-day outlook horizon would suggest. That crossover is the number a designer wants and neither row publishes it: it is a property of the pair, and the tree carries one answer per row. A third row could state it; none does yet.
- The mean is the record's unconditional mean, so this baseline knows nothing about the cycle. Fails when: a climatology that knew the solar cycle phase would be a much better baseline than 114.84 sfu everywhere, and would beat persistence sooner. That needs a date, and the date now exists: sys_mission_requirements_mission_epoch is published and sw_mean_cycle_level already turns it into a phase-conditioned level. So a fair baseline is buildable and is not built here — this row deliberately keeps the deaf baseline, because changing it would change what the horizon below means without changing its name. So the horizon this pair implies is the horizon against a DEAF baseline, and a fair baseline would shorten it.
- Only pairs of REAL observations exactly L days apart, which is again not what the MATLAB does. Fails when: prf_design and prf_horizon both build a full daily grid and fill the record's 273 absent days by linear interpolation. Interpolated days have no variability, so every error statistic spanning them is understated. This row pairs only days that were both observed, as sw_uncertainty_growth does, and for the same reason: a number that is partly invented is not a measurement of the record. It is therefore expected to disagree slightly with prf_horizon, which is why it carries no parity grid.
- Pinned to solar-weather@2026.09.14, and the [data] declaration can only pin the name. Fails when: crates/vleo-modules compares bundle NAMES, so a run with a later solar-weather satisfies the precondition and still uses this table. Every entry must be re-measured if the bundle version changes. The same obligation sits on every measured row in this group.

## Validity

From 0 to 50 One. Below: an RMS cannot be negative, and this one cannot approach zero: it is the spread of the record about its own mean, which is 44 sfu. Above: the measured entries run 44.39 to 45.32 sfu and the relation is a table that clamps, so no input can produce more. 50 is above both and tight enough to catch a broken table, which a bound of 65 would not.

The crossover is the design answer and this row does not state it. Read against sw_horizon_persistence, persistence is the better forecast out to about 547 days and is beaten by 730 — so today's flux is worth something for roughly a year and a half, which is far longer than a 27-day outlook horizon would suggest. That number is a property of the pair and the tree carries one answer per row, so a third row could state it and none does yet. And the horizon is measured against a DEAF baseline: a climatology that knew the cycle phase would beat persistence sooner.
