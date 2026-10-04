## Equations

```
T_safe(Ap_design) = exp((Ap_design - 92.515531) / 40.926516)
```

## Derivation

The three exceedance rows answer how often, how long and when. This one answers the question a programme manager asks instead of all three: how long a mission could be before the record expects to exceed the design at all. It is sw_storm_return_level read backwards — that row maps a mission length to a storm level, and this one inverts the same fit — so it says nothing new about the sky and everything about how to read the design decision.

1. sw_storm_return_level fits the level a mission of length T should expect. The question here is the inverse: given a level the design can take, what T makes that level the expectation? Same fit, solved the other way. `Ap = a + b*ln(T) => T = exp((Ap - a)/b)`
2. Substituting the fitted coefficients gives a closed form, which is why this row is exact where the three exceedance tables are interpolations. `T_safe(Ap) = exp((Ap - 92.515531) / 40.926516)`
3. At the declared G3 bound of Ap 132 that is 2.62 years, against a declared mission of 5. So the design is exceeded at a little over half the mission length, and the mission is 1.9 times longer than its design bound survives. `T_safe(132) = exp(39.4845 / 40.9265) = 2.6242 yr`
4. Every level below G4 is exceeded well inside a five-year mission, which is the same conclusion the rate row reaches from the other direction. `G1: 0.3370 yr G2: 0.7365 yr G3: 2.6242 yr`
5. A design that must NOT be exceeded at five years needs Ap 158.4, which is G4 territory — and sw_storm_design_level refuses G4 on purpose, so the refusal and this number are the same decision seen twice.
6. The row inherits the whole fit including the part that rests on two storms, so where the answer lands inside the fitted range of 0.5035 to 14.0986 years is what decides whether to trust it. At G3 the answer of 2.62 years sits comfortably inside; a design bound near Ap 200 would put it near the top, where the curve is supported by two observations.

## Assumptions

- It is exceeded, and the number is when rather than whether. Fails when: the row is read as a safety margin. At G3 the answer is 2.6242 years and orbit_mission_duration is declared at 5, so the mission is 1.9 times longer than the design bound survives. At G2 it is 0.7365 years and at G1 0.3370 — every level the G scale offers below G4 is exceeded well inside a five-year mission. A design that must not be exceeded at five years needs Ap 158.4, which is G4 territory, and sw_storm_design_level refuses G4 on purpose
- It inherits the whole fit, including the part of it that rests on two storms. Fails when: the answer lands near the top of the fitted range. This is sw_storm_return_level's relation read backwards, so every limitation of that row applies here unchanged: the fit is log-linear through ranks 2 to 56 of a 28.197-year sample, the fitted domain is 0.5035 to 14.0986 years, and the top of it rests on two observations. At the G3 bound the answer of 2.62 years sits comfortably inside the fitted range, which is the one thing that makes this row trustworthy at the declared level and would not survive a design bound near Ap 200
- A return period is not a countdown and a mission is not guaranteed its share. Fails when: 2.62 years is read as time before failure. A once-per-2.62-years storm can arrive in the first month or not in ten years; what the number means is that the expected count of such days over a 2.62-year window is one. Over the declared five-year mission the expected count is 1.42 days, which sw_exceedance_rate measures directly and which is the figure to plan with. This row is the threshold where that count passes one, not a date on which anything happens
- Closed form, so it is exact where the table rows are not. Fails when: it is compared against an interpolated row and they disagree. The three exceedance rows read a three-point table and interpolate between anchors, which their sheets declare is wrong between them. This row evaluates the fit itself at any Ap the producer can supply, so it is exact at every point including the ones between G levels. If the two ever disagree about a value between anchors, this row is right and the tables are the approximation

## Validity

From 0.3 to 3 Year. Below: the shortest this relation can return is 0.3370 years, at the G1 bound of Ap 48. A bound at 0.3 sits just under it. A value below would mean a design level under Ap 46, which is beneath anything the G scale calls a storm. Above: the longest is 2.6242 years, at the G3 bound of Ap 132, which is the top of the declared G range. A bound at 3 sits just above it. It is NOT a claim that no design survives longer: Ap 207 at G4 would reach 16.4 years, and sw_storm_design_level refuses G4 deliberately rather than this guard forbidding it.

A return period is not a countdown and a mission is not guaranteed its share. A once-per-2.62-years storm can arrive in the first month or not in ten years; what 2.62 means is that the expected COUNT of such days over a 2.62-year window is one. Over the declared five-year mission the expected count is 1.42 days, which sw_exceedance_rate measures directly and which is the figure to plan with. This row is the threshold where that count passes one, not a date on which anything happens.
