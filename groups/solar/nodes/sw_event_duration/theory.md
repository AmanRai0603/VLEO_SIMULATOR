## Equations

```
D_burst = mean length of a run of consecutive days with F107/F107A >= S_thr = 3.3115 d
```

## Derivation

sw_spike_threshold turns a continuous flux into a yes-or-no test, and a crossing on its own is not a load. A design's exposure depends on the dwell: a single elevated day and a week of them are different loads on a drag budget and on a power one, even though both count as one crossing. So this row measures how long the record stays above the threshold once it goes above it, which is the step that turns a threshold into an event.

1. Take the days the spike test passes and group them into maximal runs of consecutive days. The mean length of those runs is the answer. `D_burst = mean length of maximal runs with F107/F107A >= S_thr`
2. As with the Ap exceedance rows, the rule that ends a run has no physics behind it and has to be declared. A run breaks on the first day that fails the test, so a two-week episode with one quiet day in the middle counts as two events rather than one, which shortens this mean. A one-day bridge would be equally defensible; nothing in the record says which is right.
3. At the declared threshold of 1.3049 the record gives 202 spike days in 61 runs, so the mean is an exact rational. `D_burst = 202 / 61 = 3.3115 d`
4. The run lengths are worth writing out, because the shape of the distribution is the finding and the mean hides it: 24 ones, 3 twos, 10 threes, 7 fours, 4 fives, 7 sixes, 2 sevens, 2 nines and 2 tens.
5. A distribution with its mode at 1, its median at 3 and its mean at 3.31 is strongly right-skewed, which is what a process of independent triggers with occasional persistent sources looks like. The mean is a poor summary of it and the longest runs are what a design is actually exposed to.
6. Two limits on the count itself. The 273 absent days can neither start nor end a run, so a burst inside 2017 is missing and one running into or out of the gap is truncated at it rather than followed — with 61 events, one or two truncations move the mean by about a tenth of a day, and that is not corrected for.

## Assumptions

- A mean over a distribution that is not remotely symmetric. Fails when: the mean is used as a typical event. The 202 spike days fall into 61 runs: 24 of them are a single day, and the tail runs to 10. The distribution is 24 ones, 3 twos, 10 threes, 7 fours, 4 fives, 7 sixes, 2 sevens, 2 nines and 2 tens — so the modal event is one day and the mean is 3.31 because a handful of long bursts pull it up. The median is 3. A design that sizes on 3.31 days is sizing on neither the common case nor the bad one, and the bad one is what matters
- It is defined entirely by sw_spike_threshold and moves when that moves. Fails when: this row is quoted without the threshold beside it. At the declared 1.3049 the record gives 61 events averaging 3.31 days; a lower threshold merges neighbouring runs into longer ones and a higher one splits them. The two rows are one definition in two places, and a change to either without the other makes the pair incoherent
- A one-day gap ends an event, and that is a choice with no physics behind it. Fails when: the sky dips below the threshold for a day and comes back. Runs are broken on the first day that fails the test, so a two-week episode with one quiet day in the middle is counted as two events rather than one, shortening the mean. Allowing a one-day bridge would be as defensible and would give a different answer; nothing in the record says which is right, and this row states the rule rather than pretending the number is unique
- The 273 absent days can neither start nor end a run. Fails when: an event straddled 2017. observed_daily.csv is missing 2017-01-01 to 2017-09-30, so a burst in those nine months is absent, and a burst that ran into 2017-01-01 or out of 2017-09-30 is truncated at the gap rather than followed. With 61 events over the record, one or two truncations would move the mean by a tenth of a day, which is the order of the effect and is not corrected for

## Validity

From 1 to 10 Day. Below: a run is at least one day by construction, because a run of zero days is not an event. A value below 1 means the run-finding is broken rather than that events are short. Above: the longest run in 29 years is 10 days. A mean above it would exceed every single event the record contains, which no averaging can produce.

The mean is the wrong statistic for this distribution and the sheet publishes it because the study's field holds it. The 202 spike days fall into 61 runs of which 24 are a single day, and the tail runs to 10 days: the modal event is one day, the median is 3, and the mean is 3.31 only because a handful of long bursts pull it up. A design that sizes on 3.31 days is sizing on neither the common case nor the bad one — and the bad one is what matters. The row is also defined entirely by sw_spike_threshold and moves when it moves, so quoting one without the other makes the pair incoherent.
