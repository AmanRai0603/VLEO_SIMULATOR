## Equations

```
D_exc(Ap_design) = mean length of a run of consecutive days with Ap >= Ap_design
```

## Derivation

A rate says how often the bound is crossed and nothing about what a crossing is like. Those are different design problems: 1.42 days of exceedance spread as one day at a time is a transient an attitude and drag budget can absorb, and the same 1.42 days arriving as a single continuous siege is a different vehicle. So the second of the three exceedance numbers is the dwell — how long a run above the bound lasts once it starts — and the answer is what makes the violated requirement acceptable rather than fatal.

1. Define an event as a maximal run of consecutive days that all reach the bound, and measure the mean length of those runs. The definition needs one rule that has no physics behind it: what ends a run. `D_exc = mean length of maximal runs with Ap >= Ap_design`
2. The rule taken is that the first day failing the test ends the run. So a disturbed week with one quieter day in the middle counts as two events rather than one, which shortens this mean and raises the event count derived from sw_exceedance_rate. Allowing a one-day bridge would be equally defensible and would give a different pair of numbers; the rule is stated rather than the answer being presented as unique. It is the same rule sw_event_duration uses for F10.7, so the two rows are at least consistent with each other.
3. Measured at the three design levels the answer is just over a day everywhere, and it SHORTENS as the bound rises. `G1: 1.2718 d (max 5) G2: 1.2069 d (max 3) G3: 1.1429 d (max 2)`
4. The shortening is not a coincidence, it is what a threshold cutting further into a peaked distribution must do. A storm's Ap profile rises and falls; the higher the horizontal line drawn across it, the narrower the slice above the line. So the trend is a check on the measurement rather than a finding about the Sun.
5. The design consequence is in how close the three numbers are. Switching G level changes how OFTEN the bound is crossed by a factor of sixteen and how LONG each crossing lasts by eleven per cent — so the lever a designer has is the rate, not the dwell.
6. Which settles the question the requirement left open. At G3 the vehicle is above its design Ap for about one day at a time, at most two: long enough to matter to an attitude budget or a drag transient, not long enough to be a sustained environment.

## Assumptions

- Just over one day at every level, and that is the finding. Fails when: a long exceedance is assumed. Measured, the mean run is 1.271845 days at the G1 bound, 1.206897 at G2 and 1.142857 at G3, and the maxima are 5, 3 and 2 days. The higher the bound the shorter the run, which is what a threshold cutting further into a peaked distribution must do. So at G3 the vehicle is above its design Ap for about one day at a time, at most two — long enough to matter to an attitude budget or a drag transient, not long enough to be a sustained environment. The three numbers are so close together that switching G level changes how OFTEN far more than how LONG
- The mean is over 7 events at G3, and one of them is half the information. Fails when: the third decimal place is believed. At G3 the runs are six of one day and one of two, so the mean of 1.142857 is exactly 8 divided by 7. Remove the single two-day event and it is 1.000. The number is a mean over a sample small enough to write out, and it is published as a mean because that is what the study's field holds — but a designer should read it as 'one day, occasionally two'
- A one-day dip below the bound ends the exceedance. Fails when: a storm rides just under the threshold for a day and comes back. Runs break on the first day that fails the test, so a disturbed week with one quieter day in the middle counts as two exceedances rather than one, shortening the mean and raising the event count in sw_exceedance_rate. Allowing a one-day bridge would be as defensible and would give a different pair of numbers. The rule is stated rather than the number being presented as unique — and it is the same rule sw_event_duration uses for F10.7, so the two are at least consistent with each other
- Daily means, so a violent six hours and a disturbed day look the same. Fails when: the exceedance is short and sharp. Ap is the mean of eight three-hourly slots, so a storm that peaks for six hours and subsides can fail to lift the daily mean above the bound at all, and one that sits moderately high all day can pass it. This row therefore measures days on which the DAILY average exceeded the design value, which is a coarser event than the one a spacecraft feels. The record's three-hourly Kp is in observed_daily.csv and nothing in this group reads it yet

## Validity

From 1 to 5 Day. Below: a run is at least one day by construction. A value below 1 means the run-finding is broken rather than that exceedances are brief. Above: the longest run at any design level in 29 years is 5 days, at the G1 bound. A mean above that would exceed every single event the record contains, which no averaging can produce.

Read it as 'one day, occasionally two', not as 1.142857. At the G3 bound the runs are six of one day and one of two, so the mean is exactly 8 over 7 and removing the single two-day event makes it 1.000. The third decimal place is arithmetic on a sample small enough to write out in full. Note also that it is measured on DAILY means: a storm that peaks for six hours and subsides may never lift a daily mean above the bound, so the event counted here is coarser than the one a spacecraft feels.
