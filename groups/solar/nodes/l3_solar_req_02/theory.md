## Equations

```
F107_req_day = 350 sfu, above the record's largest daily value of 343
```

## Derivation

A requirement is a commitment, and its purpose is to be checkable against something measured. This row commits the design to surviving any single day the record holds, and the checking partner is l3_solar_ach_02.

The number is anchored in the record rather than set as a round figure above whatever the chain currently computes, and the difference matters. A ceiling placed above the achieved value moves whenever the achieved value moves and can never be violated — l3_solar_req_01 was set that way and its headroom has since drifted from 9.6 per cent to 140, which is a closure that cannot fail. A ceiling at 350 is a claim about the vehicle that stays still: the largest daily F10.7 in 28.2 years of record is 343 sfu, on 2023-02-17, so the commitment is that no day the record has ever shown would end the mission.

Why not the 400 the guards use: 400 is where the exospheric temperature relation stops being supported, not a day anybody has seen. A requirement written at the edge of a model's validity is a requirement nothing downstream can evaluate.

1. Start from what a requirement row must be able to do: be violated. If the committed number were computed from the design value it would track every change to the design and would always pass, which makes the closure a tautology. So the value is DECLARED — a person's number with their name against it — and the sense says which side of it is safe. `F107 <= F107_req_day (sense: <=)`
2. The commitment is about a single day, so the anchor is the worst single day the record holds rather than anything the design chain computes. `max(F107 over 10316 days) = 343 sfu, 2023-02-17`
3. Round up to the next fifty, so the number is a commitment rather than a transcription of one observation that a longer record would move. `F107_req_day = 350 sfu`
4. Which the achieved side clears comfortably, and the comfort is informative rather than embarrassing: the achieved side is where the window's ordinary band reaches, and this is where the record's extreme sits. `138.30 <= 350 — the closure holds`
5. Note what the pass does NOT establish. It is a ceiling on an INDEX. What a spacecraft feels is the density and the heating that index produces, through models this subsystem does not own.

## Assumptions

- 350 is anchored in the record's largest observed day, and a longer record would move it. Fails when: a day above 343 sfu is observed. The record is 28.2 years and covers two and a half cycles; cycle 19 in the late 1950s ran higher than anything in it, with F10.7 reported above 350. So this requirement is anchored in the SATELLITE-ERA record rather than in the observed history of the sun, and a design meant to survive a cycle-19 maximum needs a larger number. The rounding to 350 buys seven sfu against that, which is not much
- A single day and a sustained level are different commitments, and this is the single day. Fails when: somebody compares the achieved sustained flux against this row, or the achieved day against l3_solar_req_01. The two closures are 138.30 against 350 and 104.07 against 250, and crossing them reports a margin that belongs to neither
- It is a ceiling on the DRIVER and says nothing about what the driver does to the vehicle. Fails when: the requirement is read as a survivability statement. F10.7 is an index of solar radio flux; what a spacecraft actually feels is the density that flux produces at its altitude, through a model this subsystem does not own. A design that meets F10.7 <= 350 and is sized on a density model with the wrong drag coefficient has met this requirement and will still deorbit early
- One number for the whole mission, with no epoch and no phase in it. Fails when: the mission slips. The requirement does not move with the epoch and is not meant to — it is the vehicle's capability, and the sky's variation belongs on the achieved side of the closure where it can be seen

## Validity

From 60 to 400 One. Below: the same floor as env_f107: below 60 sfu has never been observed, so a requirement there could never be met and is not a requirement. Above: the same ceiling as env_f107: above 400 sfu every consumer of F10.7 is extrapolating, and a requirement written past the range its consumers support is not checkable.

350 against an achieved 138.30 at the declared window. That is a great deal of room and, unlike req_01's, it is room with a meaning: the achieved side is where the window's own band reaches, and the requirement is where the record's worst observed day sits. The gap between them is the distance between an ordinary design band and an extreme, and a design is entitled to have both.

The record's distribution behind the number: 343 is the maximum, 336 and 317 are next, and the 99.9th percentile of 10316 days is 282. So 350 clears every day observed, and clears the 99.9th percentile by 24 per cent.
