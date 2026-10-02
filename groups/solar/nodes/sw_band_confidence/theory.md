## Equations

None yet. The sheet's expression is empty and marked to be re-decided by a person, not inherited. What it would fix is the multiplier z in the design band, centre ± z·sigma, which sw_f107_design_long, sw_f107_cold_long, sw_ap_design_long and sw_ap_cold_long each write today as the literal 1.28 (Phi(1.28) = 0.8997).

## Derivation

This row is declared, not derived: its value is a decision, and §30 B2 of docs/MATLAB_PORT_PLAN.md says that decision needs a person. What the sheet records instead is what each answer costs, measured by tools/band_confidence_cost.py and set out in §44: moving z from 1.28 to 1.645 moves 47 rows inside solar and its layer-2 interface, raises the sustained F10.7 level 4.91 sfu from 104.07 to 108.98, reaches no KPI closure, leaves all five solar closures closing with 3.1 to 6.2 per cent less margin, and breaks no parity check in this repository.

## Assumptions

The design does not yet say; the node's author supplies this.

## Validity

From 0 to 0 .
