## Equations

```
M_ap_short = closure(Ap_req_short, Ap_short, AtMost).margin
```

## Derivation

A closure is a comparison between what is required and what is achieved, and it is only checkable if both sides are rows. The requirement side is l3_solar_req_05, a declared commitment about the vehicle; this is the achieved side, and it is an identity on sw_ap_design_short.

Which of the subsystem's Ap conclusions it restates is the whole content of the row, because the subsystem has three that a reader could mistake for each other: the sustained level, the worst day of the design band, and the one storm the mission is expected to meet. This one is the single-day level. Its siblings are sw_ap_design_long and sw_storm_return_level, and at the declared window the three are 26.70, 90.55 and 158.38 — far enough apart that restating the wrong one would move a closure from passing to failing or back.

1. The closure to be made checkable is single-day Ap achieved against single-day Ap required. Both sides must be rows, at the same layer, in the same quantity and unit, or the comparison is not something the tree can verify.
2. The requirement side is a declared commitment with a person's name against it. The achieved side must be the subsystem's own conclusion and nothing else, so this row is an identity on sw_ap_design_short. `Ap_ach_short = sw_ap_design_short = 90.5472`
3. Which makes the closure arithmetic trivial: 90.55 achieved against 132 required. The ceiling is the G3 threshold, raised from G2 once the level-conditioned band showed this window's own hot scenario reaching a G3-G4 day. `90.55 <= 132 — the closure holds`

## Assumptions

- It inherits every limitation of the row it restates, and a closure reading it sees none of them. Fails when: a margin is computed from this row against a capability. The centre beneath it is a cycle analogue scaled, beyond one cycle past cycle 25's maximum, by the mean amplitude of two completed cycles whose peaks differ by 41 per cent; the band is 1.28 sigma, which is the 90th percentile and not the 95 per cent the run is labelled; and the residuals it is a sigma of are skewed. None of that travels across the closure, and the margin looks like a clean number either way
- It restates the single-day level and not one of its two siblings. Fails when: somebody reads it as the other. The subsystem publishes three Ap conclusions — 26.70 sustained, 90.55 for the worst day of the design band, 158.38 for the one storm expected in the mission — and they answer three different questions. Restating the wrong one would move this closure from passing to failing or back without anything in the tree noticing
- Nothing compares this row with its requirement automatically. Fails when: a reader assumes the tree checks the closure. The pairing is a convention the matrix draws; `sense` is declared on the requirement row and the gate checks only that it is present. All five closures in this group hold as of 2026-09-16, and two of them did not the day before — this one at 90.55 against 80, and the Ap storm at 158.38 against 150. Both were invisible to every machine check in this repository, and both were found by a person reading the numbers rather than by anything running

## Validity

From -10 to 1 One. Below: a margin of -10 is a requirement exceeded by eleven times its own value. Below that the comparison has stopped being a design margin and become a sign that one side is in the wrong unit, and it should refuse rather than report a number nobody will read as a fraction. Above: 1 is the whole of the requirement: an achieved value of zero against a positive bound. A margin above 1 under this sense would mean the achieved value is negative, which none of these drivers can be.

90.5472 at the declared window, against a ceiling of 132. THE CLOSURE HOLDS, with 46 per cent of room, and it did not until 2026-09-16.

It held at 41.70 before that, on a number that was wrong: the daily band was one percentile measured over days whose rotations averaged Ap 11.64 and applied at a sustained Ap of 26.70, where the record says a one-in-twenty day departs by 63.85. Conditioning the band on the rotation level took this row to 90.55 and the closure failed at the old ceiling of 80, the G2 threshold.

The ceiling was raised to 132, the G3 threshold, by decision — not to make the arithmetic pass but because a one-day commitment at G2 is below what this window presents, and 90.55 is a G3-G4 day. The ladder stays anchored in the published scale and stays ordered: 48 operate indefinitely, 132 survive one day, 207 not destroyed by the one storm.

It inherits every limitation of the row it restates and a closure reading it sees none of them: the centre beneath it is a cycle analogue at the mission's own epoch, the band around that centre is 1.28 sigma — the 90th percentile, while the run is labelled 95 per cent — and the within-rotation term is read at the top knot of a table whose top knot rests on 300 days.
