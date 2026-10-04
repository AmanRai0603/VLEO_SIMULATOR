## Equations

```
Kp_slot(s) = kp_from_ap(Ap_s) + dKp_slot(Ap_s), for each of the five scenarios and each of the two slots
```

## Derivation

Three relations, composed, at five points.

The published IAGA scale turns a daily Ap into a Kp. It is a lookup and not a formula, and it is concave — ap grows roughly geometrically with Kp — which is the whole reason the slot corrections exist. Run on a DAILY MEAN Ap the scale returns a value that sits above the mean of the day's eight three-hourly Kp and far below its peak, by Jensen's inequality, and both offsets are measurable on the record rather than arguable from the shape.

sw_kp_mean_bias and sw_kp_slot_bias measure them, in nine bins of Ap, as the median of mean_8(Kp) - table(Ap) and of max_8(Kp) - table(Ap). Both tables now live in vleo-core beside the scale, because more than one caller reads each: those two rows at one Ap, and this row at five.

What this row adds is nothing but the composition and the five points. That is deliberately thin — a row that did more would be doing physics its own constituents already own — and the value of it is that the ten numbers exist as variables the crossing can relay, rather than as a relation nothing downstream can evaluate.

1. Take each scenario's daily Ap. Five of them, from the five rows that compute them. `Ap_s for s in {nominal, hotmean, coldmean, hotday, coldday}`
2. Convert on the published 28-point IAGA scale, which is a lookup rather than a formula and is clamped at both ends rather than extrapolated. `base_s = kp_from_ap(Ap_s)`
3. Add the measured offset for the slot. The scale is fed a daily MEAN and Kp(ap) is concave, so it reads above the mean of the eight three-hourly slots and far below the peak; both offsets are medians of the record in nine bins of Ap. `Kp_mean_s = base_s + dKp_mean(Ap_s) · Kp_peak_s = base_s + dKp_peak(Ap_s)`
4. Ten results, published as a set, because the crossing carries the driver product as one conclusion and these are two of its five columns.

## Assumptions

- The offsets are medians over nine bins of Ap and nothing else. Fails when: the day is unusual in a way Ap does not capture. The correction knows the daily mean and the bin it falls in; it does not know whether the day was one long storm or eight quiet slots and one severe one, and those have the same Ap and very different peaks. The median is the middle of that spread, so half of the days in any bin exceed the peak this row publishes
- The tables are clamped at both ends rather than extrapolated. Fails when: a scenario's Ap falls outside them. The scale runs to ap 400 and the bias bins to a centre of 255, so a hot day above that reads the last bin's offset. This tree's hot Ap day is 90.55, inside both; the study's driver sets never exceeded 42. A scenario multiplier applied to a disturbed day would reach the clamp silently
- Both slot corrections are measured on the record and belong to a bundle version. Fails when: the bundle moves. They are medians over bundles/solar-weather@2026.09.14 and must be re-measured when it does. They live in vleo-core, which is the second and third pieces of measured data in a kernel whose own comment used to say SOLAR_CYCLE_SHAPE was the only one — that claim is now wrong and the kernel says so
- This row composes and does not measure, so its errors are its constituents' errors. Fails when: a reader looks here for the physics. The scale is sw_kp_from_ap's, both offsets are sw_kp_mean_bias's and sw_kp_slot_bias's, and the five Ap values are the design rows'. What this row owns is the composition and the choice of five points, and its parity grid is the only place all three are checked together

## Validity

From 0 to 9 One. Below: Kp is defined on 0 to 9 and the scale's first point is Kp 0 at ap 0; a negative index is a sign error, not a quiet sky. Above: Kp is defined on 0 to 9 and the scale's last point is Kp 9 at ap 400. This member is the worst slot of the worst day, so it is the one of the ten most likely to reach it.

FED THE STUDY'S OWN FIVE Ap VALUES THIS ROW REPRODUCES ALL TEN OF ITS Kp NUMBERS TO FOUR DECIMALS — every scenario, both slots, which is what the parity grid beside this sheet holds. That is a strong result for a chain of three relations and two measured tables, and it says the port of prf_ap2kp is right.

On this tree's own Ap values the three *mean scenarios still agree closely with the study — 3.5483 against 3.5481 in the mean slot at nominal — because this tree's sustained Ap values are within half a per cent of the study's. The two *day scenarios do not, and they are not meant to: this tree's hot Ap day is 90.55 against the study's 41.54, because the within-rotation band is conditioned on the rotation level. So the hot day carries Kp 5.80 in the mean slot and 8.00 in the peak, against the study's 4.50 and 6.14.

Kp 8.0 in the peak slot is a severe geomagnetic storm, G4. That follows from the Ap correction rather than from anything here, and it is the same finding l3_solar_req_05 is now failing on.
