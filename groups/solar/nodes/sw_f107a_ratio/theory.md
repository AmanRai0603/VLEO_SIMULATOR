## Equations

```
sd of F107 / F107A over the record = 0.122210
```

## Derivation

sw_f107_81day gives a density model the smooth driver and can only give it a climatology. This row is the half of the same question the record CAN answer with no prediction in it: how far a single day departs from its own 81-day mean. The two are a pair on purpose — one is the level, the other is the width — and a design that takes the level without the width is designing to the average day of the average cycle. It is expressed as a RATIO rather than an absolute scatter because the departures scale with the level: a fixed number of sfu would be meaningless at minimum and negligible at maximum.

1. Pair every day with its own 81-day centred mean and take the ratio. A ratio rather than a difference, because the size of a departure scales with the level of activity — a 15 sfu excursion is large at a minimum level of 70 and unremarkable at a maximum level of 200 — so a dimensionless spread is the only form that applies at every phase. `r(d) = F107(d) / F107A(d)`
2. Before measuring the spread, check the centre, because the centre is a test of the computation rather than a result. A centred mean is an unbiased estimate of the day at its centre, so the mean ratio must come out at one; anything else would mean the window was trailing or misaligned. `mean r = 0.999340 over 10284 days`
3. The six-parts-in-ten-thousand shortfall is not noise either. Flux spikes up and decays down, so the 81 days around a typical day contain a slightly higher mean than the day itself — the record's own asymmetry surviving the average.
4. Then the spread, which is the row's answer: the standard deviation of that ratio over the record. A typical day sits twelve per cent away from its own baseline. `sd_ratio = sd(r) = 0.122210`
5. The day count is 10284 rather than the 10592 the record spans, and the shortfall is two different things kept separate. 273 days are the 2017 gap. The rest are days too near an end of the record for a full centred window to exist. Nothing is interpolated across the gap: a day with fewer than 57 of its 81 neighbours present is left undefined rather than computed from a short window, because a mean of half a window is a different statistic wearing the same name.

## Assumptions

- The ratio is centred on one to six parts in ten thousand, and that is a check rather than a coincidence. Fails when: the 81-day window were not centred, or the record were trending within it. Measured over 10284 days the mean ratio is 0.999340. A centred mean is an unbiased estimate of the day at its centre, so a mean ratio at one is what a correct computation must produce, and a departure would have meant the window was trailing or misaligned. The 0.00066 shortfall is the record's own asymmetry — flux spikes up and decays down — surviving the average
- It is a standard deviation of a distribution that is not normal. Fails when: 0.1222 is used to build a symmetric interval. The ratio runs to 2.11 at the top, which is nine standard deviations above the mean, and cannot go below zero at all: the distribution is bounded on one side and has a long tail on the other. Two standard deviations does not mean 95 per cent here. The measured percentiles are 1.2188 at the 95th and 1.3711 at the 99th, which are the numbers to use for a band
- Pooled across the whole cycle, and the scatter is not constant across it. Fails when: a design at a known cycle phase wants the scatter it will actually see. Active regions produce the departures, so the ratio's spread is larger near maximum than near minimum, and this row averages a quiet 2008 with a busy 2002 into one number. The mean-cycle level at the epoch's phase is 108.14 sfu, so a twelve per cent scatter there is about 13 sfu — but that is the pooled twelve per cent applied at one phase, not a phase-conditioned measurement
- The 273 absent days are not in it. Fails when: the count is read as the whole record. The ratio is defined on 10284 of the 10592 calendar days the record spans: 273 are the 2017 gap, and the rest are days too near an end of the record for a full 81-day centred window to exist. No value is interpolated across the gap — a day whose window overlaps it uses the days that are there, and a day with fewer than 57 of its 81 is left undefined rather than computed from a short window

## Validity

From 0.05 to 0.25 One. Below: below 0.05 the daily flux would sit within five per cent of its 81-day mean on a typical day, which would make the daily driver and the smoothed one interchangeable. The record's 99th percentile ratio is 1.371, so it is not. Above: above 0.25 the typical day would be a quarter away from its own baseline and the 81-day mean would not be describing the same quantity as the day. A value there means the window or the pairing is wrong rather than that the Sun is variable.

It is a standard deviation of a distribution that is not normal, so it must not be doubled to make a 95 per cent band. The ratio cannot fall below zero and reaches 2.11 at the top, nine standard deviations above the mean — bounded on one side, long-tailed on the other. The measured percentiles are the numbers to use: 1.2188 at the 95th and 1.3711 at the 99th. It is also pooled across the whole cycle, and the scatter is not constant across one: active regions produce the departures, so a quiet 2008 and a busy 2002 are averaged into a single figure.
