## Equations

```
M_ap_long = closure(Ap_req_long, Ap_long, AtMost).margin
```

## Derivation

A closure is a comparison between what is required and what is achieved, and it is only checkable if both sides are rows. The requirement side is l3_solar_req_04, a declared commitment about the vehicle; this is the achieved side, and it is an identity on sw_ap_design_long.

Which of the subsystem's Ap conclusions it restates is the whole content of the row, because the subsystem has three that a reader could mistake for each other: the sustained level, the worst day of the design band, and the one storm the mission is expected to meet. This one is the sustained level. Its siblings are sw_ap_design_short and sw_storm_return_level, and at the declared window the three are 26.70, 41.70 and 158.38 — far enough apart that restating the wrong one would move a closure from passing to failing or back.

1. The closure to be made checkable is sustained Ap achieved against sustained Ap required. Both sides must be rows, at the same layer, in the same quantity and unit, or the comparison is not something the tree can verify.
2. The requirement side is a declared commitment with a person's name against it. The achieved side must be the subsystem's own conclusion and nothing else, so this row is an identity on sw_ap_design_long. `Ap_ach_long = sw_ap_design_long = 26.6953`
3. Note what a passing closure does NOT establish. Both numbers are ceilings on an INDEX, and what a spacecraft feels is the heating, the density and the torque that index produces, through models this subsystem does not own. `26.6953 <= 48 — the closure holds`

## Assumptions

- It inherits every limitation of the row it restates, and a closure reading it sees none of them. Fails when: a margin is computed from this row against a capability. The centre beneath it is a cycle analogue scaled, beyond one cycle past cycle 25's maximum, by the mean amplitude of two completed cycles whose peaks differ by 41 per cent; the band is 1.28 sigma, which is the 90th percentile and not the 95 per cent the run is labelled; and the residuals it is a sigma of are skewed. None of that travels across the closure, and the margin looks like a clean number either way
- It restates the sustained level and not one of its two siblings. Fails when: somebody reads it as the other. The subsystem publishes three Ap conclusions — 26.70 sustained, 41.70 for the worst day of the design band, 158.38 for the one storm expected in the mission — and they answer three different questions. Restating the wrong one would move this closure from passing to failing or back without anything in the tree noticing
- Nothing compares this row with its requirement automatically. Fails when: a reader assumes the tree checks the closure. The pairing is a convention the matrix draws; `sense` is declared on the requirement row and the gate checks only that it is present. The Ap storm closure beside this one, l3_solar_req_03 against l3_solar_ach_03, currently FAILS at 158.38 against 150 and nothing in the tree says so

## Validity

From -10 to 1 One. Below: a margin of -10 is a requirement exceeded by eleven times its own value. Below that the comparison has stopped being a design margin and become a sign that one side is in the wrong unit, and it should refuse rather than report a number nobody will read as a fraction. Above: 1 is the whole of the requirement: an achieved value of zero against a positive bound. A margin above 1 under this sense would mean the achieved value is negative, which none of these drivers can be.

26.6953 at the declared window, against a ceiling of 48. It inherits every limitation of the row it restates and a closure reading it sees none of them: the centre beneath it is a cycle analogue at the mission's own epoch, the band around that centre is 1.28 sigma — the 90th percentile, while the run is labelled 95 per cent — and the within-rotation term is a percentile of a sample that is truncated at zero. A margin computed from this row carries none of that and looks like a clean number either way.
