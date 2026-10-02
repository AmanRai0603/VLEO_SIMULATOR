## Equations

```
Ap_req_short = ap(G3) = 132
```

## Derivation

A requirement is a commitment, and its purpose is to be checkable against something measured. This row commits the design to surviving one day at the G3 threshold, and the checking partner is l3_solar_ach_05.

Why the ladder has three rungs rather than one is the substance of this row. sw_ap_design_short and sw_storm_return_level both answer "the worst Ap" and they are not the same question: the first is the top of an ordinary design band — a sustained level with a within-rotation percentile stacked on it, reached routinely — and the second is an extreme-value return level, the one storm expected in the whole mission. At the declared window they differ by a factor of nearly four, 41.70 against 158.38. A single ceiling covering both would have to be the storm one, and the design would then be claiming to operate through a G3 storm, which is a much stronger claim than anybody made.

1. The closure to be made checkable is single-day Ap achieved against single-day Ap required. Both sides must be rows, at the same layer, in the same quantity and unit.
2. The requirement side is a commitment about the vehicle, so it cannot be computed from the record. It is taken from the published G scale at the strong-storm threshold. `Ap_req_short = ap(Kp 7) = 132`
3. Higher than the sustained ceiling, because a transient is easier to survive than a permanent condition. Lower than the storm ceiling, because the storm is a different question. `48 (sustained) < 132 (one day) < 207 (the one storm)`
4. Which the achieved side clears at the declared window. `90.55 <= 132 — the closure holds, with 46 per cent of room`

## Assumptions

- It is a ceiling on the driver and says nothing about what the driver does to the vehicle. Fails when: a reader takes a passing closure as evidence the design is adequate. Ap is an index; what a spacecraft feels is the heating, the density and the torque it produces, through models this subsystem does not own
- The worst day of a design band and the one storm of a mission are different questions. Fails when: somebody collapses this row into l3_solar_req_03. At the declared window the two achieved sides differ by a factor of nearly four, 41.70 against 158.38, because one is a percentile of ordinary variation and the other an extreme-value return level. A single ceiling covering both would have to be the storm one, and the design would then be claiming to operate through a G3 storm
- The G3 threshold is a reasonable place for a one-day survival commitment, and nothing here establishes that. Fails when: the vehicle's real limit is elsewhere. 132 is a published threshold rather than an arbitrary round number, which makes it checkable, but checkable is not the same as correct: the level at which a particular design must stop is a thermal, torque and propellant question this subsystem does not see

## Validity

From 0 to 400 One. Below: Ap floors at zero, and a single-day requirement of zero would commit the design to surviving only a perfectly quiet day, which is not a survival requirement. Above: 400 is the top of the published ap table, the value at Kp 9. A requirement above it is off the scale the G levels are defined on and could not be expressed as a G level at all.

132 against an achieved 90.55 at the declared window — 46 per cent of room. As with its sustained sibling the room is a property of the epoch rather than of the design: the achieved side is a band on a cycle-analogue centre, and a window at cycle maximum would narrow it sharply.

Worth noting against l3_solar_req_03: THAT closure does NOT hold. Its achieved side, sw_storm_return_level, gives 158.38 against a ceiling of 150. Nothing in this tree compares a required row with its achieved row automatically — the pairing is a convention the matrix draws — so that failure is visible to a reader of the matrix and to nothing else.
