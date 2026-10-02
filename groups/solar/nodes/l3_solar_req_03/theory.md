## Equations

```
Ap_req = ap(G4) = 207
```

## Derivation

This row asks what the mission must not be DESTROYED by. That is a different question from what it OPERATES THROUGH, and separating the two is what lets both be answered honestly.

The vehicle operates through G3: sw_storm_design_level declares it and sw_ap_design turns it into a daily Ap of 132. The record expects 158.4 over the declared five-year mission, so the mission WILL meet a storm it cannot work through. That is a fact about the sky and no number written here changes it.

What this row commits to is that such a storm does not end the mission, and it is anchored at 207 — the published ap at Kp 8, the G4 threshold. The three exceedance rows then say what the operating violation costs, which is the question a designer actually faces: 0.284 days a year above 132, 1.42 days over the mission, in about 1.24 events averaging 1.14 days each.

This row used to be 132 plus a 14 per cent margin, 150, and the closure failed because 158.4 exceeded it. Raising it was rejected then on the grounds that a number above 158.4 "commits the vehicle on paper to a level it is not built for". That objection holds for an OPERATING commitment and is why sw_storm_design_level is untouched at G3. It does not hold for a SURVIVAL one, which is what this row is.

1. Begin from the capability, which is the only defensible starting point for a commitment: sw_ap_design at the declared G3 gives a daily Ap of 132, the value a day would reach with all eight slots at Kp 7.
2. Do not add a margin to it. A survival commitment is a different question from an operating capability, so it is anchored where the published scale puts the next band rather than at a percentage above what the vehicle works through. `Ap_req = ap(Kp 8) = 207, the G4 threshold`
3. Now compare it with what the record expects, which is where the row stops being arithmetic. sw_storm_return_level at five years gives 158.4 — above the 132 the vehicle operates through, below the 207 it must survive. `158.4 <= 207 — the closure holds, and 158.4 > 132 is the finding`
4. The alternative was available and was rejected. A margin chosen so the closure passes would have to exceed 158.4, which means about 200 — and 200 sits in G4 territory, committing the vehicle on paper to a level it is not built for. A requirement that passes because its number was chosen to pass is worse than one that fails honestly.
5. So the useful question becomes the size of the violation rather than its existence, and that is what the exceedance rows answer: how often the record goes past 132, for how long each time, and where in the cycle. `0.284 d/yr * 5 yr = 1.42 d, in ~1.24 events of ~1.14 d`
6. Thirty-four hours outside the design environment over five years is a number a programme can accept or reject. That is the whole purpose of writing the requirement below the expectation: it converts a vague inadequacy into a bounded one.

## Assumptions

- 207 is a SURVIVAL commitment and not an operating capability, and the two are easy to confuse. Fails when: a reader takes this as the level the vehicle works through. It is not. sw_storm_design_level declares G3 and sw_ap_design turns it into 132, and that is the operating level; nothing about it moved. This row says a G4 storm does not end the mission, which sw_storm_design_level's own reason_upper describes as the right treatment for G4 — handled 'by operating through the event rather than by building for it'. A design that claimed to OPERATE at G4 would change sw_storm_design_level, and four exceedance rows would move with it.
- This closure now holds and the OPERATING one still does not, and the second is the one that matters. Fails when: a passing closure here is read as the storm case being closed. 158.4 sits between the two levels: above the 132 the vehicle operates through, below the 207 it must survive. So the mission meets a storm it cannot work through, and the size of that is what a designer needs — 0.284 days a year above 132, 1.42 days over the mission, about 1.24 events averaging 1.14 days each, longest run in 29 years 2 days. Roughly thirty-four hours outside the operating environment across five years. Nothing in the tree compares 158.4 against 132 automatically; the three exceedance rows measure it and a person reads them.
- Switching the G level moves the design value and the exceedance statistics, and not this row. Fails when: somebody expects the requirement to follow the switch. sw_storm_design_level is the input a design turns to ask what a different storm level costs — G1 gives 48, G2 gives 80, G3 gives 132 — and sw_ap_design and all three exceedance rows move with it. At G2 the exceedance rate is 1.24 days a year, 6.21 days over the mission in 5.1 events. This row does not move: it is a commitment, and a commitment that silently tracked the design would never be violated and would therefore never be a requirement. Changing it is a separate, deliberate act.
- A daily mean, which is the wrong shape for what a storm does. Fails when: the storm is short or long. Daily Ap averages eight three-hourly slots, so a violent six-hour storm and a mild day-long disturbance can share a value, and a requirement written on the daily mean is satisfied by both. The atmosphere responds to the integral with a lag, not to the daily mean. The record's largest daily Ap is 273 — above this requirement — and it is one day in 29 years.
- It is met by the record and the record is 29 years long. Fails when: the mission meets something the record has not seen. The return level this is checked against is fitted through ranks 2 and 3 of a 28.2-year sample at the mission's five-year period, and the largest event in that sample, Ap 273, has an apparent return period of 28.2 years for no reason but that it is the largest thing in 28.2 years. Events well beyond this requirement are known from longer proxy records. A requirement that a 29-year record cannot violate is not thereby safe.

## Validity

From 20 to 400 One. Below: below 20 the requirement would be under the level at which the record's storms begin — the median day is Ap 7 and sw_storm_return_level's own floor is 20 — so a requirement there could not be met by any mission and is not a requirement. Above: 400 is the top of the published ap table, the value at Kp 9. A requirement above it is off the scale the G levels are defined on and could not be expressed as a G level at all.

The closure fails and the failure is bounded. 0.284 days a year above the design Ap of 132 is 1.42 days over the mission, in about 1.24 events averaging 1.14 days each, the longest run in 29 years being 2 days — roughly thirty-four hours outside the design environment across five years, on the declining side of the cycle where the declared epoch sits. If that is unacceptable the answer is to move sw_storm_design_level, not to raise this row until the arithmetic stops complaining. And note what this row does NOT do: it does not follow the G-level switch. A commitment that tracked the design would never be violated.
