## In one line

Everything the solar-weather subsystem concludes, carried up to the system level as one set: five design scenarios (expected, sustained hot, sustained quiet, hottest day, quietest day), each with a daily solar-flux level, its 81-day mean, and a geomagnetic disturbance level.

## Said simply

- It feeds **sys_space_environment**: it decomposes the solar-weather record into the drivers the system designs to.
- It feeds **l3_x_envorbit**: F10.7, the 81-day mean and Kp are what the exospheric temperature is computed from.

## Picture it

{{fig record}}

Two full solar cycles and most of a third: the slow rise and fall is what every design number in this group is built on.

## Guess first

{{guess How many design scenarios does the solar-weather subsystem hand up to the system level, and why not just one? || Five: expected, sustained hot, sustained quiet, hottest day and quietest day. A design sizes some things on a level it sits in for months and others on a single day, so publishing one number would decide for the reader which problem they have.}}

## Where it breaks

Every value inherits the limits of the row beneath it, and a system reader sees none of them: the sustained scenarios are 90th-percentile bands though the run is labelled 95 per cent, and the day scenarios are nearer one-in-a-hundred days. Every requirement here is a ceiling on an index of solar or geomagnetic activity, not on the air density, heating and torque the spacecraft actually feels. And one refused member refuses the whole set.

## Common misreading

Reading the five requirements as promises the subsystem must reach, like everywhere else in the tree. Nobody builds the Sun: here a requirement is the worst sky the design can sustain, and a closure passes when the achieved sky stays under it (`sense = "<="`). Read the other way, the same numbers report a comfortable margin for a spacecraft in trouble.
