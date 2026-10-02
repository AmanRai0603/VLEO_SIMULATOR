## Equations

```
Ap_req_long = ap(G1) = 48
```

## Derivation

A requirement is not a prediction and not a capability: it is a commitment, and its purpose is to be checkable against something measured. This row commits the design to operating continuously in a geomagnetic field up to the G1 threshold, and the checking partner is l3_solar_ach_04.

The number is taken from the published G scale rather than chosen as a round number above the achieved value, which is a deliberate departure from how l3_solar_req_01's 250 sfu was picked. A ceiling set above whatever the chain currently computes moves whenever the chain does and can never be violated; a ceiling set at a published threshold is a claim about the vehicle that stays still while the environment estimate moves underneath it. Both are in this group, side by side, and the difference is worth seeing.

1. The closure to be made checkable is sustained Ap achieved against sustained Ap required. Both sides must be rows, at the same layer, in the same quantity and unit.
2. The requirement side is a commitment about the vehicle, so it cannot be computed from the record. It is taken from the published G scale at the minor-storm threshold. `Ap_req_long = ap(Kp 5) = 48`
3. Which the achieved side clears at the declared window, and would not clear at a cycle maximum. `26.70 <= 48 — the closure holds, with 80 per cent of room`

## Assumptions

- It is a ceiling on the driver and says nothing about what the driver does to the vehicle. Fails when: a reader takes a passing closure as evidence the design is adequate. Ap is an index; what a spacecraft feels is the heating, the density and the torque it produces, through models this subsystem does not own. A design that meets Ap <= 48 and is sized with the wrong drag coefficient has met this requirement and will still deorbit early
- The G1 threshold is a reasonable place for a CONTINUOUS-operation commitment, and nothing here establishes that. Fails when: the vehicle's real limit is elsewhere. 48 is a published threshold rather than an arbitrary round number, which makes it checkable, but checkable is not the same as correct: the level at which a particular design must stop operating comes from its thermal, its torque authority and its propellant, none of which this subsystem sees. What this row guarantees is that a reader can see what the commitment IS
- A sustained level and a single day are different commitments, and this is the sustained one. Fails when: somebody compares the achieved single-day Ap against this row. That is l3_solar_req_05's closure and its ceiling is 80. Reading the wrong one of the two reports a failure where there is none, or a pass where there is not

## Validity

From 0 to 400 One. Below: Ap floors at zero, and a sustained requirement of zero would commit the design to operating only in a perfectly quiet field, which no mission window presents. Above: 400 is the top of the published ap table, the value at Kp 9. A requirement above it is off the scale the G levels are defined on and could not be expressed as a G level at all.

48 against an achieved 26.70 at the declared window — 80 per cent of room, which sounds generous and is not the point. The achieved side is a 1.28-sigma band on a centre that is itself a cycle analogue; if the epoch moved to a cycle maximum the sustained Ap would roughly double and this closure would come close.

It is a ceiling on the DRIVER and says nothing about what the driver does to the vehicle. Ap is an index of geomagnetic disturbance; what a spacecraft feels is the heating, the density and the torque that disturbance produces, through models this subsystem does not own.
