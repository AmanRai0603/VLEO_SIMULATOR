## Equations

```
F107_obs = 150
```

## Derivation

A forecast needs a starting point, and for solar flux the starting point is what the Sun is doing now. That is a measurement, not a derivation, so this row declares it rather than computing it.

It is separate from the flux the design is built to because the two are different questions with different answers. Today's flux is an observation with a lifetime of about one solar rotation. The design flux is a statement about a five-year window that has not happened yet, and this subsystem exists to produce it.


## Assumptions

- One number stands for the current state of the Sun. Fails when: the question is asked at a lead short enough for persistence to carry weight. At a lead of days the flux on the day matters, the 27-day decay has barely started, and a round 150 would be doing real work badly. The declared lead range of this tree starts well past that, which is why it does not.
- It is a daily value, not an 81-day mean. Fails when: it is read as F10.7A. The two are different quantities with different ranges and the thermosphere relation uses both, weighting the mean 3.24 and the daily departure 1.3. sw_f107a_ratio is the row that relates them.

## Validity

From 60 to 400 One. Below: the same floor env_f107 declared and for the same reason: below 60 sfu has never been observed and every relation reading F10.7 has no support there. Above: the same ceiling env_f107 declared: above 400 sfu is beyond the largest recorded daily value, so anything reading it is extrapolating.

It is a SINGLE DAY'S LEVEL, and a stand-in one. The shipped climatology's 150 sfu is a round number chosen as a design default, not a reading taken on a date, and nothing here pretends otherwise — `source` names the publisher of the index rather than an observation.

That matters less than it looks, and the note above says by how much: the one estimator reading this row weights it into insignificance at every lead the tree can ask about. A person replacing 150 with a real observation would change sw_central_expectation in the thirtieth decimal place. The row is here so that the subsystem's input and the subsystem's output are not the same variable, and a real observation would make it honest rather than make it matter.
