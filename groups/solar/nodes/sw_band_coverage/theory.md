## Equations

```
C_band = fraction of pairs with F107(t+L) - F107(t) <= dF107_p95(L), pooled over the seventeen leads
```

## Derivation

sw_f107_design sells a 95 per cent band and every margin downstream is built on that claim. A stated confidence that nobody ever counted is a decoration, and the cheapest way for it to be wrong is not a modelling error but an implementation one — a lead off by one, a percentile taken on the absolute change instead of the signed one, a table read at the wrong index. This row counts. Its value is not that it comes back near 0.95 but that it could have come back somewhere else.

1. The claim to be tested is a conditional one: at each lead, the change in F10.7 should stay below the percentile that lead publishes, in 95 per cent of cases. `claim: P( F107(t+L) - F107(t) <= dF107_p95(L) ) = 0.95`
2. So count it directly. Over the same seventeen leads sw_uncertainty_growth is tabulated on, take every pair of observed days and ask whether the change stayed inside the band. `C_band = 0.9509267569 over 128135 pairs`
3. The test is SIGNED, not two-sided: it asks how often F10.7 rose by more than the stated amount, and a large fall counts as inside the band. For a drag design that is the right test, because the unsafe direction is flux arriving higher than planned. A mission exposed to flux being LOWER than planned — a power budget — is not checked by this row at all.
4. The pooled figure could hide a bad lead, so the per-lead coverages are the part that is actually evidence. They run from 0.949970 at 365 and 1826 days to 0.952425 at 2557 days — a spread of 0.0025 across leads from half a year to fifteen years.
5. Uniformity is what distinguishes a correct table from a coincidence. An implementation right at short leads and wrong at long ones would show as a drift across that list, and there is none — so the pooled 0.9509 is a fair summary rather than an average over disagreeing parts.
6. The small excess over 0.95 is in the safe direction: the band is very slightly conservative, containing the truth marginally more often than it claims.

## Assumptions

- It is an in-sample check and cannot be anything else on this record. Fails when: 0.9509 is read as out-of-sample validation. The percentiles in sw_uncertainty_growth were measured on these same pairs, so a coverage near 0.95 is close to arithmetic rather than evidence — it confirms the percentile was computed correctly, not that it will hold. What makes the number worth publishing is that it could have come back wrong: an off-by-one in the lead, a percentile taken on the absolute change rather than the signed one, or a table read at the wrong index would all show here. It is a check on the implementation, and it is honest about being only that
- The coverage is uniform across leads, which is the part that is evidence. Fails when: the pooled figure hides a bad lead. It does not: per-lead coverage runs from 0.949970 at 365 and 1826 days to 0.952425 at 2557 days, a spread of 0.0025 across seventeen leads spanning half a year to fifteen years. A table that was right at short leads and wrong at long ones would show as a drift and there is none. The pooled 0.9509 is therefore a fair summary rather than an average over disagreeing parts
- Signed, not absolute — this counts only the band being exceeded UPWARD. Fails when: a two-sided band is wanted. The growth percentile is the 95th of the SIGNED change, so this row asks how often F10.7 rose by more than the stated amount, and a large fall counts as inside the band. For a drag design that is the right test, because the unsafe direction is flux arriving higher than planned. A mission exposed to F10.7 being LOWER than planned — a power budget, for instance — is not checked by this row at all
- Pairs are massively overlapping, so the sample is far smaller than 128135. Fails when: the count is read as independent evidence. Consecutive pairs at a given lead share all but one day, and at a lead of fifteen years two pairs a day apart are nearly the same measurement. The effective sample is closer to the number of independent intervals — the record divided by the lead, which at the longest leads is five or six — than to the 4838 to 9947 pairs each lead contributes. Nothing here is a confidence interval, and the third decimal place of 0.9509 means nothing

## Validity

From 0.9 to 1 One. Below: below 0.9 a band sold as 95 per cent would be missing the truth twice as often as it claims, and every margin built on sw_f107_design would be smaller than it reads. That is a defect in the percentile table, not a property of the sky, and it should stop a run. Above: a coverage cannot exceed 1. A value at exactly 1 would mean the band was never exceeded in 29 years, which for a 95th percentile would mean the table is far too wide and the design is paying for margin it does not need.

An in-sample check, and it cannot be anything else on this record: the percentiles were measured on these same pairs, so coverage near 0.95 is close to arithmetic rather than evidence about the future. What it confirms is that the percentile was computed and wired correctly. The pairs are also massively overlapping — consecutive pairs at a lead share all but one day — so the effective sample is closer to the record divided by the lead, which at the longest leads is five or six intervals, than to the 128135 pairs counted. Nothing here is a confidence interval and the third decimal place means nothing.
