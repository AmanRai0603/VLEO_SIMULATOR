## Equations

```
G_design = 3   (NOAA G3, strong, Kp 7)
```

## Derivation

A vehicle cannot be built for the worst storm physically possible, and a design that says so vaguely has made no decision. The NOAA G scale exists to make the decision nameable: five levels, each tied to a Kp, each with documented consequences for power grids, navigation and spacecraft. Declaring a G level is how a programme states what class of event it is engineered through rather than operated through, and putting that declaration on its own row is what makes it a switch — every number downstream moves when it moves.

1. The G scale is defined on Kp and it grades the WORST three-hourly slot of a day: G1 at Kp 5, G2 at Kp 6, G3 at Kp 7, G4 at Kp 8, G5 at Kp 9. So a G level is a statement about a day's peak disturbance, not about its average.
2. Choosing a level is choosing where to stop building and start operating, and the frequencies are what make that a real decision. In 29 years the record holds 49 days at G4 and 16 at G5 — rare enough that flying through them with a procedure is cheaper than carrying the structure for them, and common enough that a mission must have such a procedure.
3. G3 is the highest level this vehicle class is sized for. That is a statement about this class and not about the scale, which is why the declared upper bound is 3: a mission that genuinely requires G4 changes the bound on purpose and re-runs the closure, and the change is visible in a diff rather than buried in a selection.
4. The row is a switch rather than a constant. sw_ap_design converts it, and the three exceedance rows all read that conversion, so moving this one number from 3 to 2 moves the design Ap from 132 to 80 and the exceedance from 1.42 days over the mission to 6.21. Asking what a different storm level costs is a one-row edit.

## Assumptions

- G3 is a choice and the record says a five-year mission will exceed it. Fails when: the choice is read as sufficient. sw_storm_return_level, fitted on the record and read at the declared five-year mission, gives a daily Ap of 158.4; the days in the record nearest that value all reached Kp_max 9, which is G5. So the storm a five-year mission should expect is two levels above what this row designs to. That is not an error in either row — it is the gap between what the sky does and what a vehicle can be built for, and putting both numbers in the tree is the point. What closes it is operations, not structure
- The scale is on Kp and the design quantity is daily Ap, which are not the same measurement. Fails when: a G level is converted to a daily Ap as though the mapping were exact. Kp is three-hourly and the G level is the WORST slot in a day; daily Ap is the mean of eight slots. So a G3 day has one slot at Kp 7 and seven that may be anything below it, and the record shows exactly that spread: days whose Kp_max is 7 run from daily Ap 15 to 96, median 51. sw_ap_design takes the ceiling of that band rather than its median, and the sheet there says why
- Stopping at 3 is a statement about this vehicle class, not about the scale. Fails when: a mission that must survive G4 or G5 reads this row. The declared range refuses 4 and 5 rather than letting them be selected quietly, because a design sized for G4 is a different vehicle and the rest of this group's numbers — the return level, the band, the requirement — would all need revisiting together. A mission with that requirement should change this bound deliberately and re-run the closure, which is a visible act

## Validity

From 1 to 3 One. Below: G1 is the bottom of the NOAA scale. Below it there is no storm to design for — Kp 4 is 'active' and is where alerts begin, which sw_alert_threshold carries, not where storms do. Above: G3 is the highest level this vehicle class is sized for. G4 occurs on 49 days in 29 years and G5 on 16, and both are handled by operating through the event rather than by building for it. A design that genuinely requires G4 changes this bound on purpose and re-runs the closure.

G3 is a choice and the record says a five-year mission will exceed it. The return level at the declared mission is Ap 158.4, and the days in the record nearest that value all reached Kp_max 9, which is G5 — two levels above what this row designs to. That is not an error in either row; it is the gap between what the sky does and what a vehicle can be built for, and what closes it is operations rather than structure. The range refuses 4 and 5 deliberately: a design sized for G4 is a different vehicle, and the return level, the band and the requirement would all need revisiting together.
