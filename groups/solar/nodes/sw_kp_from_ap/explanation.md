## In one line

Converts the daily geomagnetic index the design carries into Kp, a grade of disturbance from 0 to 9 set every three hours, which the atmosphere model wants.

## Said simply

Converts the daily geomagnetic index the design carries into Kp, a grade of disturbance from 0 to 9 set every three hours, which the atmosphere model wants. It reads the published 28-point table and nothing more; what flows in is the design storm, so what flows out is the grade of that storm.

## Picture it

{{fig kp_ap}}

The published table (the curve this node reads backwards) sits inside the measured spread at every Kp.

## Guess first

{{guess A day's average activity Ap is 48. What Kp does the published table give for it? || Kp 5. Ap 48 is one of the table's own 28 points, the storm threshold, so nothing is interpolated.}}

## Where it breaks

The table is meant for three-hour values but is fed a daily average, so it reads about one grade below the day's worst three hours, and 1.6 below on disturbed days; that is not corrected here. Above the table's top every storm reads 9, and a negative input reads back as 0, the quietest possible sky.

## Common misreading

That the Kp this row returns is the day's worst three-hour grade. It is the published table applied to a daily average, so it sits slightly above the day's typical grade and about one grade below its worst three hours (1.6 below on disturbed days). That offset is measured separately, in sw_kp_slot_bias, for a consumer to add; it is not corrected here.
