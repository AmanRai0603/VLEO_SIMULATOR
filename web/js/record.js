/*
  The reference record, as the bundle publishes it.

  The daemon serves each bundle file byte for byte, so this module parses the
  same CSV the engine reads. That is the point: a face that draws a shaped
  summary served from somewhere else is a second description of the data, and
  the first time it disagrees with the file the disagreement is invisible.

  Nothing here is remembered across a reload and nothing is computed twice — a
  file is fetched once and the derived series are built once, because the
  record does not change while the page is open.

  MISSING IS NOT ZERO. Every column that can be absent is parsed to null and
  stays null. The 273 days the record is missing are absent rows, not zeros, and
  a mean that quietly counted them as zero would be wrong in the direction that
  matters.
*/
'use strict';

const cache = new Map();

export async function bundleFile(name, file) {
  const key = name + '/' + file;
  if (cache.has(key)) return cache.get(key);
  const p = (async () => {
    const r = await fetch('/v1/bundle/' + key);
    if (!r.ok) throw new Error(await r.text());
    return parseCsv(await r.text());
  })();
  cache.set(key, p);
  return p;
}

// A comma parser, not a CSV parser. Quoted fields appear in exactly one column
// of one file in this bundle — the flags on forecast_issues — and nothing here
// reads it. A general parser would be more code than the thing it protects.
export function parseCsv(text) {
  const lines = text.split('\n').filter(l => l.length && l[0] !== '#');
  if (!lines.length) return { cols: [], rows: [] };
  const cols = lines[0].split(',').map(s => s.trim());
  const rows = [];
  for (let i = 1; i < lines.length; i++) {
    const f = lines[i].split(',');
    if (f.length < cols.length) continue;
    const o = {};
    for (let k = 0; k < cols.length; k++) {
      const v = f[k].trim();
      o[cols[k]] = v === '' || v === 'NaN' ? null : v;
    }
    rows.push(o);
  }
  return { cols, rows };
}

export const num = v => (v === null || v === undefined ? null : Number(v));

// Days since 2000-01-01, the epoch the engine's tables use. Written out rather
// than taken from Date, so a browser's timezone cannot move a day.
export function daysSince2000(s) {
  const y = +s.slice(0, 4), m = +s.slice(5, 7), d = +s.slice(8, 10);
  const yy = m <= 2 ? y - 1 : y;
  const era = Math.floor(yy / 400);
  const yoe = yy - era * 400;
  const doy = Math.floor((153 * (m + (m > 2 ? -3 : 9)) + 2) / 5) + d - 1;
  const doe = yoe * 365 + Math.floor(yoe / 4) - Math.floor(yoe / 100) + doy;
  return era * 146097 + doe - 730425;
}

export function dayOfYear(s) {
  return daysSince2000(s) - daysSince2000(s.slice(0, 4) + '-01-01') + 1;
}

let recordP = null;

/** The daily record and the cycle boundaries, parsed once. */
export function solarRecord() {
  if (recordP) return recordP;
  recordP = (async () => {
    const [obs, cyc] = await Promise.all([
      bundleFile('solar-weather', 'observed_daily.csv'),
      bundleFile('solar-weather', 'solar_cycles.csv'),
    ]);
    const days = obs.rows.map(r => ({
      date: r.date,
      t: daysSince2000(r.date),
      year: +r.date.slice(0, 4),
      doy: dayOfYear(r.date),
      f107: num(r.f107),
      ap: num(r.ap_planetary),
      kp: num(r.kp_max),
      ssn: num(r.ssn_sesc),
    }));
    const cycles = cyc.rows.map(r => ({
      n: +r.cycle,
      start: daysSince2000(r.start),
      end: daysSince2000(r.end),
      peak: num(r.peak),
    }));
    // Cycle phase, 0 at the opening minimum to 1 at the next. A day outside
    // every listed cycle gets null rather than a phase folded from the nearest
    // one, because the record starts mid-cycle 23 and ends inside 25.
    for (const d of days) {
      d.cycle = null; d.phase = null;
      for (const c of cycles) {
        if (d.t >= c.start && d.t < c.end) {
          d.cycle = c.n;
          d.phase = (d.t - c.start) / (c.end - c.start);
          break;
        }
      }
    }
    // The gap is a property worth carrying rather than rediscovering: the span
    // is 10592 days and there are 10319 rows.
    const span = days.length ? days[days.length - 1].t - days[0].t + 1 : 0;
    return { days, cycles, span, missing: span - days.length };
  })();
  return recordP;
}

/**
 * A centred moving mean, and the reason it is centred.
 *
 * A trailing mean sits below the series during a rise and above it during a
 * fall, so a ratio against it is not symmetric and a threshold on that ratio
 * catches rises and misses falls. Centred costs the ability to compute it in
 * real time, which a descriptive statistic does not need.
 *
 * A window with fewer than `minFrac` of its days present returns null rather
 * than a mean of whatever was there — across the 2017 gap that would otherwise
 * be a mean of one side.
 */
export function centredMean(vals, w, minFrac = 0.7) {
  const half = Math.floor(w / 2), n = vals.length, out = new Array(n).fill(null);
  for (let i = 0; i < n; i++) {
    let s = 0, k = 0;
    for (let j = Math.max(0, i - half); j < Math.min(n, i + half + 1); j++) {
      if (vals[j] !== null) { s += vals[j]; k++; }
    }
    if (k > w * minFrac) out[i] = s / k;
  }
  return out;
}

/** Pearson correlation over the pairs where both are present. */
export function corr(a, b) {
  const xs = [], ys = [];
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== null && b[i] !== null) { xs.push(a[i]); ys.push(b[i]); }
  }
  if (xs.length < 3) return null;
  const ma = xs.reduce((p, c) => p + c, 0) / xs.length;
  const mb = ys.reduce((p, c) => p + c, 0) / ys.length;
  let sab = 0, sa = 0, sb = 0;
  for (let i = 0; i < xs.length; i++) {
    const da = xs[i] - ma, db = ys[i] - mb;
    sab += da * db; sa += da * da; sb += db * db;
  }
  return sa && sb ? sab / Math.sqrt(sa * sb) : null;
}

export function quantile(sorted, q) {
  if (!sorted.length) return null;
  const i = Math.min(sorted.length - 1, Math.max(0, Math.floor(q * sorted.length)));
  return sorted[i];
}
