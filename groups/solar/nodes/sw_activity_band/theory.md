## Equations

```
band(F107) = 1 + count(edges <= F107),  edges = 90, 130, 170 sfu
```

## Derivation

A flux in sfu means nothing to most readers of a design. 228 could be ordinary or extraordinary and there is no way to tell from the number. A band answers that in one symbol, and the bands used are not this repository's invention: they are the standard four NOAA F10.7 activity levels, which the study's prf_segment applies unchanged. So the row is a lookup against a convention — there is nothing fitted in it to be wrong about this record, and equally nothing in it that adapts to this record.

1. The four standard levels are defined by three edges, so the relation is a count of edges reached rather than a chain of comparisons. Low is below 90 sfu, moderate 90 to 129, elevated 130 to 169, and high 170 and above. `band(F107) = 1 + #{ e in (90, 130, 170) : F107 >= e }`
2. The comparison is 'at or above', so a flux sitting exactly on an edge belongs to the band the edge OPENS — 90.0 sfu is moderate, not low. Every off-by-one here produces a label that looks entirely plausible, which is why the fixtures beside this row sit on the edges rather than in the middle of the bands.
3. The top band is unbounded in flux, which is the one asymmetry in the scheme. Band 4 holds everything from 170 sfu up, including the record's largest day at 343, so the label stops discriminating exactly where the design cares most.
4. A different scheme was available and is deliberately not this row. prf_segment also offers data-driven terciles of the same quantity, which cut the archive into equal thirds and land on different edges. Those adapt to the record and these do not; a published convention is comparable between studies and a tercile is comparable only within one.

## Assumptions

- The bands are a published convention and this row is a lookup, not a measurement. Fails when: the four levels — low below 90, moderate 90 to 129, elevated 130 to 169, high 170 and above — are the standard NOAA F10.7 activity levels and prf_segment applies exactly these. Nothing here is fitted, so there is nothing in it to be wrong about this record, and equally nothing in it that adapts to this record: prf_segment ALSO offers data-driven terciles of the same quantity, which cut the archive into equal thirds and land in different places. Those are a different row and this is not it.
- A band is an ordinal label carried as a number, and arithmetic on it is meaningless. Fails when: the answer is 1, 2, 3 or 4 and the gaps between them are not equal in sfu — band 1 spans 26 sfu of observed record, band 4 spans 173. Averaging bands, interpolating between them, or treating band 4 as twice band 2 are all errors this row cannot prevent, because the tree carries one scalar per row and a label has to arrive as one. A consumer that wants a flux wants env_f107 or sw_f107_design.
- It bands a single day's flux, and a mission does not live on one day. Fails when: F10.7 moves through every band over any mission longer than a few months — the record spends 39.8% of its days in band 1 and 12.8% in band 4 — so banding the design value says which band the DESIGN POINT sits in and not which band the mission will experience. Reading it as the latter would be reading a design percentile as a forecast.

## Validity

From 1 to 4 One. Below: there are four bands and the lowest is 1. A zero or negative band means the counting started in the wrong place, which would shift every label by one and still look like a valid answer. Above: there are four bands and the highest is 4, unbounded above in flux — band 4 holds everything from 170 sfu upward, including the record's largest day at 343. A fifth band means an edge was added without the range being updated.

An ordinal label carried as a number, and arithmetic on it is meaningless: the gaps are not equal in flux, with band 1 spanning 26 sfu of observed record and band 4 spanning 173. Averaging bands or treating band 4 as twice band 2 are errors the tree cannot prevent, because one row carries one scalar. It also bands a SINGLE day's flux: F10.7 moves through every band over any mission longer than a few months — 39.8 per cent of the record's days are in band 1 and 12.8 per cent in band 4 — so banding a design value says where the design point sits, not what the mission will experience.
