## Equations

```
M_f107_long = closure(F107_req_long, F107_long, AtMost).margin
```

## Derivation

A closure is a comparison between what is required and what is achieved, and it is only checkable if both sides are rows. The requirement side is l3_solar_req_01, a declared commitment; this is the achieved side, and it computes nothing of its own — it restates the subsystem's conclusion on the side of the comparison the closure reads. The reason it exists rather than the closure reading sw_f107_design_long directly is that a closure should bind two rows of the same shape at the same layer, so that the comparison is visible on the tree instead of being an edge somebody has to trace.

WHICH conclusion it restates is the content of this row, because the subsystem has three F10.7 conclusions a reader could mistake for each other: sw_f107_design_long at 104.07 (the sustained level), sw_f107_design_short at 138.30 (a day inside it) and sw_f107_design at 200.14 (a persistence drift, which is now the achieved side of no closure at all). Restating the wrong one would move this closure without anything in the tree noticing.

1. The closure to be made checkable is F10.7 achieved against F10.7 required. Both sides must be rows, at the same layer, in the same quantity and unit, or the comparison is not something the tree can verify.
2. The requirement side is a declared commitment with a person's name against it. The achieved side must be the subsystem's own conclusion and nothing else, so this row is an identity on sw_f107_design_long. `F107_ach_flux = F107_long = 104.07 sfu`
3. Which makes the closure arithmetic trivial and the closure itself informative — though what it is informative ABOUT has changed. 104.07 achieved against 260 required is 150 per cent of room, where it was 9.6 per cent when the pair was written — but the ceiling is now the record's hottest rotation rather than a number picked above the chain, so the room is a statement about this window being quiet rather than about the requirement being slack. `104.07 <= 260 — the closure holds`
4. Note what the pass does NOT establish. The requirement is a ceiling on an INDEX, so a passing closure confirms the sky is one the design says it can operate in, not that the design is adequate: what a spacecraft feels is the density that index produces at its altitude, through a model this subsystem does not own. TWO closures in this group do not hold — the Ap single day at 90.55 against 80 and the Ap storm at 158.38 against 150 — and nothing in the tree says so about either, because the pairing is a convention the matrix draws rather than something the kernel evaluates.

## Assumptions

- It inherits every limitation of the row it restates, and a closure reading it sees none of them. Fails when: the same cost as the interface, and worth repeating on the row a closure actually binds. The number is sized on the cycle analogue at the mission's own epoch rather than on the record's unconditional mean, which this chain now reads the epoch to do; where it is a band it is 1.28 sigma, the 90th percentile, and not the 95 per cent the run is labelled; and where it is a return level its top end rests on two observations in 28.2 years. A margin computed from this row against a capability carries none of that, and will look like a clean number either way.

## Validity

From -10 to 1 One. Below: a margin of -10 is a requirement exceeded by eleven times its own value. Below that the comparison has stopped being a design margin and become a sign that one side is in the wrong unit, and it should refuse rather than report a number nobody will read as a fraction. Above: 1 is the whole of the requirement: an achieved value of zero against a positive bound. A margin above 1 under this sense would mean the achieved value is negative, which none of these drivers can be.

104.07 sfu at the declared window, against a ceiling of 260 — 150 per cent of room, and the room is now explained rather than accidental. l3_solar_req_01's 260 is the record's largest 27-day mean, 252.67 sfu centred 2024-08-13, rounded up; it says the design operates through any rotation the record has shown. Its predecessor, 250, was the next round number above whatever the chain happened to give and therefore moved with the design instead of with the sky.

It inherits every limitation of the row it restates and a closure reading it sees none of them. The centre beneath it is a cycle analogue at the mission's own epoch, scaled beyond one cycle past cycle 25's maximum by the mean amplitude of two completed cycles whose peaks differ by 41 per cent; and the band around it is 1.28 sigma, the 90th percentile, while the run is labelled 95 per cent. A margin computed from this row carries none of that and looks like a clean number either way — which is the standing hazard of a closure, and the reason the assumption is repeated on the row a closure actually binds.
