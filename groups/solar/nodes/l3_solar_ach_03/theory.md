## Equations

```
M_ap_survive = closure(Ap_req_survive, Ap_T, AtMost).margin
```

## Derivation

This is the achieved side of the closure that fails, and the reason it fails is visible in how it is built rather than in any arithmetic. The F10.7 pair compares a design value against a requirement and both are constructed the same way, so they agree. The Ap pair compares what the RECORD expects against what the VEHICLE is built for, and those are independent quantities with no reason to agree — so the closure is a real test rather than a consistency check, and it is the one that fails.

1. The achieved side restates the subsystem's geomagnetic conclusion, which is sw_storm_return_level read at the declared mission length. `Ap_ach = Ap_T(5 yr) = 158.38`
2. Set against the requirement the closure holds, and WHERE it sits is the finding. 158.38 is above the 132 the vehicle operates through and below the 207 it must not be destroyed by, so the mission meets a storm it cannot work through and survives it. `132 < 158.38 <= 207 — survived, not operated through`
3. The two sides come from unrelated places, and that is what makes this closure worth having. The requirement is a published threshold on the G scale; the achieved value is a tail quantile of a 28-year record. Nothing ties them, so there is no reason for them to agree — and when they do, the agreement means something. This pair was 150 against 158.38 and FAILED until 2026-09-16, when the requirement was re-anchored at the G4 threshold as a survival commitment rather than as the operating capability plus 14 per cent.
4. All five requirements in this group are now anchored on something a reader can check — two on the record's own extremes, three on published G thresholds — so none of the five closures can pass merely because its number was picked above whatever the chain happened to give. That was true of exactly one of them before, and l3_solar_req_01 spent two revisions drifting from 9.6 per cent of headroom to 140 before it was re-anchored.
5. And what the closure holding does NOT say is that the storm case is closed. The mission is still outside its OPERATING environment for 1.42 days over five years, in about 1.24 events of roughly one day each. Nothing in the tree compares 158.38 against 132 automatically; the three exceedance rows measure it and a person reads them.

## Assumptions

- It inherits every limitation of the row it restates, and a closure reading it sees none of them. Fails when: the same cost as the interface, and worth repeating on the row a closure actually binds. The number is sized on the cycle analogue at the mission's own epoch rather than on the record's unconditional mean, which this chain now reads the epoch to do — it was an open decision on sw_central_expectation and it has been made, moving this row by 28.0 sfu; where it is a percentile it is the 95th and not a worst case; and where it is a return level its top end rests on two observations in 28.2 years. A margin computed from this row against a capability carries none of that, and will look like a clean number either way.

## Validity

From -10 to 1 One. Below: a margin of -10 is a requirement exceeded by eleven times its own value. Below that the comparison has stopped being a design margin and become a sign that one side is in the wrong unit, and it should refuse rather than report a number nobody will read as a fraction. Above: 1 is the whole of the requirement: an achieved value of zero against a positive bound. A margin above 1 under this sense would mean the achieved value is negative, which none of these drivers can be.

It has no confidence attached, unlike the F10.7 pair, and the absence is a statement rather than an omission: geomagnetic activity has no usable long-term forecast, so the study designs to a return period instead of to a percentile. The number therefore means 'the storm expected once in a mission of this length', and its top end rests on two observations in 28.2 years. A margin computed against it carries none of that.
