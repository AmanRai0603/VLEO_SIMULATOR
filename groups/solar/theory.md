## Equations

The group adds no relation of its own; each row's relation is on its own page. The chain they form, as the sheets state it:

- sustained hot: `F107_long = F107_central + 1.28 * sigma_total`, and `Ap_long = Ap_central + 1.28 * sigma_ap`
- hottest day: `F107_short = F107_long + dF107_day`, and `Ap_short = Ap_long + dAp_day`
- sustained quiet: `F107_cold_long = F107_central - 1.28 * sigma_total`, and `Ap_cold_long = Ap_central - 1.28 * sigma_ap`
- quietest day: `F107_cold_short = F107_cold_long - dF107_day_low`, and `Ap_cold_short = Ap_cold_long - dAp_day_low`
- each closure: `M = closure(required, achieved, AtMost).margin`, the signed fractional margin (required − achieved) / required
- the crossing: `driver_set = {nominal, hotmean, coldmean, hotday, coldday} x {f107, f107bar, ap}`, each member relayed from the row that computed it.

## Derivation

Nothing is derived at the group's own level; the derivation of each number is on the row that computes it. The group's shape follows from three choices the sheets argue for: a centre needs a band before it has edges, so each driver gets a centre, a sustained band either side, and a single day stacked on each band; the environment is not delivered, so its five requirement and achieved pairs close when achieved stays under required; and a subsystem is reached through exactly one node, so `l3_solar_interface` relays the whole set of five scenarios to `sys_space_environment` without computing anything.

## Assumptions

- The sustained bands are 1.28 standard deviations either side of the centre, which is the 90th percentile, though the run is labelled 95 per cent.
- The daily excursions are read at the level the rotation sits at, not as one number for every level.
- The F10.7 centre is a cycle analogue at the mission's own dates; the Ap centre is the last rotation forecast, held flat across the window.
- Geomagnetic activity has no usable long-term forecast, so the storm side designs to a return period rather than a percentile.
- The requirements are declared ceilings on indices, anchored in the record or in the published storm scale, not on what the spacecraft feels.

## Validity

It holds for the declared mission window and the satellite-era record behind it, 28.2 years long. It stops holding where that record does: cycle 19 in the late 1950s ran higher than anything in it, the top of the Ap daily table rests on only 300 days, and the Ap storm return level's top end rests on two observations. The margins are a property of the epoch rather than of the design, and a window at cycle maximum would narrow them sharply.
