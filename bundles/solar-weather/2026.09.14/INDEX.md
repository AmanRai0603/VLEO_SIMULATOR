# What is in this bundle

Seven tables, one file each, every column named and given a unit in its own
header. Open any of them in a spreadsheet; nothing here needs a library.

| file | rows | cols | span | kind |
|---|---:|---:|---|---|
| `alerts.csv` | 17,971 | 8 | 1997-06-16 .. 2025-12-28 | as issued |
| `daily_regime.csv` | 10,299 | 6 | 1997-01-09 .. 2025-12-31 | derived |
| `forecast_issued.csv` | 30,047 | 7 | 1997-08-12 .. 2025-12-29 | as issued |
| `forecast_issues.csv` | 1,437 | 10 | 1997-06-10 .. 2025-12-29 | as issued |
| `monthly_means.csv` | 339 | 7 | 1997-01-15 .. 2025-12-15 | derived |
| `observed_daily.csv` | 10,319 | 19 | 1997-01-01 .. 2025-12-31 | measured |
| `solar_cycles.csv` | 3 | 6 | 1997-01-15 .. 2019-12-01 | derived |

## What each one is

**`alerts.csv`** — every alert, warning and watch, with the threshold that triggered it.

**`daily_regime.csv`** — a Gaussian mixture's per-day regime label, with its own confidence.

**`forecast_issued.csv`** — every 27-day outlook, one row per forecast day; pair on target_date to score it.

**`forecast_issues.csv`** — the index of issues, with the parser flags and the archive path each came from.

**`monthly_means.csv`** — monthly means and their 13-month smoothers.

**`observed_daily.csv`** — the daily record — F10.7, sunspot number and area, flare counts, Ap, the eight three-hourly Kp, the College and Fredericksburg indices.

**`solar_cycles.csv`** — the three cycles the record spans, with boundaries and peaks.

## measured · as issued · derived

The distinction is the point, and it is not decoration.

**measured** is an observation. `observed_daily.csv` is the only one.

**as issued** is a record of what was *said at the time* — a forecast, an
alert. It is evidence about the forecaster, not about the sky. Scoring a
forecast means joining it to the measured record, which is why both are here.

**derived** is computed from the measured record by a model. `daily_regime.csv`
is a three-component Gaussian mixture's opinion and carries its own confidence
per day: a row labelled `storm` is a day the mixture put there.

## The hole in 2017

`observed_daily.csv` has 10,319 rows across a 10,592-day span. **2017-01-01 to
2017-09-30 is missing** — 273 days, one continuous run, the only gap in
twenty-nine years.

The source's own metadata calls the daily table "gap-free". It is not, and this
line exists because that claim was believed once already.

A window overlapping 2017 comes back short rather than wrong.
`vleo_data::days_missing(rows, from, to)` returns how many days are absent, and
a design sized on an incomplete window should be refused rather than averaged.

## One caveat the source itself states

`ap_planetary` is the SWPC **estimated** planetary A, not the GFZ definitive
value. Anyone reconciling against GFZ will find differences, and they are not
transcription errors.

## And one this repository found

Thirteen of the 1,437 rows in `forecast_issues.csv` carry a `url_path` that
disagrees with the row's own date and issue number — 1998-08-25 carries issue
1199 and was fetched from `1998/09/prf1200.pdf`. Those are facts about the
archive. The path is stored rather than derived from date and number precisely
so they survive.

## Total

70,415 rows, 3.7 MB of text.

