## Equations

```
Ap_short = Ap_long + dAp_day
```

## Derivation

The two spreads stack for the same reason they do for F10.7: one is how wrong the pattern is about the rotation's level, the other is how far a day departs from the rotation it sits in, and a bad day is both.

For Ap the daily term DOMINATES, and conditioning it on level made that far more true. 63.85 added to a sustained 26.70 is seventy per cent of the answer, where F10.7's 20.07 on 104.07 is under a sixth. Geomagnetic activity is a burst process: the level is quiet and the day is not, so a design that reads only the sustained number is not slightly wrong about a storm day, it is wrong by the size of the storm.


## Assumptions

- The rotation-level error and a day's rise within its rotation stack, because a bad day is both a bad stretch and a bad day within it. Fails when: either half is read alone. For Ap the daily term dominates — 63.85 on a sustained 26.70, seventy per cent of the answer — so a design that reads only the sustained level is wrong by the size of the storm
- The daily rise is read from its measured table at the sustained level, and is held at the top of that table. Fails when: more record at high levels shows a larger daily rise, in which case this answer is low
- The window's own hot scenario is one the design was built for. Fails when: the answer, 90.5, passes the single-day commitment of l3_solar_req_05 at Ap 80, as it now does. The requirement has not been moved to make the closure pass; whether the commitment belongs at G3 or the window is outside what the design was built for is a decision, not arithmetic

## Validity

From 0 to 400 One. Below: Ap floors at zero, so a single-day design level below it means a spread has been subtracted rather than added. Above: 400 is the top of the Ap index itself; a value above it is not a geomagnetic index at all, and this row — a sustained level plus a daily excursion — is the one in the subsystem most likely to reach for it.

90.5472 at the declared window, and THIS IS THE LARGEST CORRECTION THIS SUBSYSTEM HAS MADE. It was 41.6972 until sw_ap_daily_band_spread was conditioned on the rotation level, within 0.4 per cent of the 41.5352 the study publishes for its own window.

The old daily term, 15.00, was one percentile measured over days whose rotations averaged Ap 11.64, applied at a sustained Ap of 26.70. At that level the record says a one-in-twenty day departs by 63.85, more than four times as much, because high-Ap rotations are storm-driven and their days swing enormously. So the previous Ap single-day design level was less than half what the record supports for its own hot scenario, and a design sized on it was under-designed by a factor of two.

AND IT BREAKS A CLOSURE. l3_solar_req_05 commits the design to surviving a single day at the G2 threshold, Ap 80; this row now says the window's own hot scenario reaches 91, which is a G3-G4 day. The requirement has NOT been moved to make the closure pass. Either the single-day commitment belongs at G3, or this window is outside what the design was built for, and that is a decision.

Ap 91 is consistent with the rest of the subsystem rather than at odds with it: sw_storm_return_level puts the one storm expected in the whole mission at Ap 158, and a disturbed rotation throwing G3 days is what the record shows.
