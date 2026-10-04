## Equations

```
M_f107_short = closure(F107_req_short, F107_short, AtMost).margin
```

## Derivation

A closure is a comparison between what is required and what is achieved, and it is only checkable if both sides are rows. The requirement side is l3_solar_req_02, a declared commitment about the vehicle; this is the achieved side, and it is an identity on sw_f107_design_short.

WHICH conclusion it restates is the content of the row, because the subsystem has three F10.7 conclusions a reader could mistake for each other: sw_f107_design_long at 104.07 (the level the mission sustains for months), sw_f107_design_short at 124.14 (a day inside that level) and sw_f107_design at 200.14 (a one-sided persistence drift from today's value, which since §20 step 5 is the achieved side of no closure at all). Restating the wrong one would move this closure without anything in the tree noticing, because all three are fluxes in the same range with the same unit and the same declared domain.

1. The closure to be made checkable is single-day F10.7 achieved against single-day F10.7 required. Both sides must be rows, at the same layer, in the same quantity and unit, or the comparison is not something the tree can verify.
2. The requirement side is a declared commitment with a person's name against it. The achieved side must be the subsystem's own conclusion and nothing else, so this row is an identity on sw_f107_design_short. `F107_ach = F107_short = 124.14 sfu`
3. Which makes the closure arithmetic trivial and the closure itself informative: 124.14 achieved against 350 required, and the room between them is the room between what the window presents and what the record's worst day has shown. `124.14 <= 350 — the closure holds`
4. Note what the pass does NOT establish. It is a ceiling on an INDEX, and what a spacecraft feels is the density that index produces at its altitude, through a model this subsystem does not own. It also stacks two one-sided percentiles, so the day it describes is rarer than the confidence attached to it.

## Assumptions

- It inherits every limitation of the row it restates, and a closure reading it sees none of them. Fails when: a margin is computed from this row against a capability. The centre beneath it is a cycle analogue at the mission's own epoch, scaled beyond one cycle past cycle 25's maximum by the mean amplitude of two completed cycles whose peaks differ by 41 per cent; the band around it is 1.28 sigma, the 90th percentile, while the run is labelled 95 per cent; and the daily term stacked on top is a separate one-sided percentile, so the combination is nearer a 1-in-100 day than a 1-in-20 one. None of that travels across the closure, and the margin looks like a clean number either way
- It restates the single-day level and not one of its two siblings. Fails when: somebody reads it as the sustained level or as the persistence drift. The three are 124.14, 104.07 and 200.14 — all fluxes, same unit, same declared domain — so nothing in the tree would catch the substitution, and each would report a different margin against the same requirement
- Nothing compares this row with its requirement automatically. Fails when: a reader assumes the tree checks the closure. The pairing is a convention the matrix draws; `sense` is declared on the requirement row and the gate checks only that it is present. The Ap storm closure in this same group, l3_solar_req_03 against l3_solar_ach_03, currently FAILS at 158.38 against 150 and nothing in the tree says so

## Validity

From -10 to 1 One. Below: a margin of -10 is a requirement exceeded by eleven times its own value. Below that the comparison has stopped being a design margin and become a sign that one side is in the wrong unit, and it should refuse rather than report a number nobody will read as a fraction. Above: 1 is the whole of the requirement: an achieved value of zero against a positive bound. A margin above 1 under this sense would mean the achieved value is negative, which none of these drivers can be.

124.14 sfu at the declared window, against a ceiling of 350. The gap is large and, unlike ach_01's, it means something: the achieved side is where the window's ORDINARY band reaches and the requirement is where the record's worst observed day sits, 343 sfu on 2023-02-17. A design is entitled to have both, and the distance between them is the distance between a design band and an extreme.

The two terms behind the number are worth separating. 104.07 of it is the sustained level — a centre of 86.85 with 1.28 standard deviations of the rotation-forecast residual — and 20.07 is the within-rotation daily excursion AT THAT LEVEL. This row read 138.30 until sw_daily_band_spread was conditioned on the rotation level; the old single number, 34.23, was measured over rotations averaging 136.63 sfu and applied at 104.07.

So the daily term is under a sixth of the answer here, where on the Ap side it is seventy per cent: a design that treats the daily excursion as a detail is making a different mistake on flux than on the geomagnetic index.
