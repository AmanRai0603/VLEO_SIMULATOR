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

/**
 * What the ENGINE says a row answers, for a figure that needs it.
 *
 * A panel draws the record, which is what `bundleFile` is for. Where it also
 * needs a number this tree COMPUTES — a design bound, a requirement, the centre
 * of a window — it has until now carried that number as a literal copied in when
 * the panel was written. Three of those copies went stale without anything
 * noticing, because a panel declared the bundle as what it reads and the check
 * that a panel moves when its inputs move therefore never asked the engine
 * anything. §21 of docs/MATLAB_PORT_PLAN.md has the three.
 *
 * So: one call, every row a panel names, values in SI as everything crossing
 * this boundary is. Refusals are kept rather than dropped — a panel drawing a
 * line for a row that refused would be drawing a number nobody computed.
 */
export async function engineValues(ids) {
  const want = [...new Set(ids)].filter(Boolean);
  if (!want.length) return {};
  const take = v => ({ si: v.si, shown: v.shown, unit: v.unit, symbol: v.symbol, label: v.label });
  const runs = await Promise.all(want.map(async (id) => {
    try {
      const r = await fetch('/v1/run?node=' + encodeURIComponent(id));
      const d = await r.json();
      if (!d.ok) return { id, refused: d.message || d.fault || 'refused' };
      return { id, values: d.values || [] };
    } catch (e) {
      return { id, refused: String(e) };
    }
  }));

  // WHICH RUN'S ANSWER WINS, DECIDED RATHER THAN RACED.
  //
  // A run returns every value on the path, not only the row asked for — which is
  // what lets a panel naming an interface reach its published members by name.
  // But a row asked for directly is also usually ON another asked-for row's path:
  // `closure` asks for l3_solar_ach_04 and l3_solar_req_04, and the requirement
  // rides inside the achieved row's run.
  //
  // This used to write every value of every response into one object as each
  // arrived, so a row reached both ways took whichever request resolved LAST.
  // panel_check found it, intermittently, which is the worst way to find
  // anything: check 2b serves one row a different answer by bending the reply to
  // `node=<that row>`, and whether the bend survived depended on promise
  // ordering. The same race would let two runs under different cases disagree
  // and hand the panel whichever landed second, with nothing to show for it.
  //
  // So: a row that was asked for takes its OWN run's answer, always. Everything
  // else on a path fills in only where nothing asked for it, and the first run to
  // carry it wins rather than the last — an order that is the caller's list
  // rather than the network's.
  const out = {};
  for (const r of runs) {
    for (const v of (r.values || [])) {
      if (v.id !== r.id && !out[v.id]) out[v.id] = take(v);
    }
  }
  for (const r of runs) {
    if (r.refused) { out[r.id] = { refused: r.refused }; continue; }
    const own = r.values.find(v => v.id === r.id);
    out[r.id] = own ? take(own) : { refused: 'the run did not return this row' };
  }
  return out;
}

/**
 * One row swept across another's declared range, for a figure that draws a
 * relation rather than a point. Same boundary rule: SI in, SI out, refusals
 * recorded with their reason.
 */
export async function engineSweep(node, over, from, to, points = 80, sets = null) {
  const q = new URLSearchParams({ node, over, from: String(from), to: String(to),
    points: String(points) });
  // OTHER DECLARED VALUES HELD SOMEWHERE ELSE, for the length of the sweep.
  //
  // A sweep moves one decision and leaves every other at what the tree declares,
  // which answers "what does this row do as X moves" and not "what does it do at
  // a scenario the tree does not sit at". The thermosphere panel needs the
  // second: the five solar scenarios are five different places to stand, and a
  // Kp sweep taken at the declared flux would draw one curve where there are
  // five. `sets` is {id: value in SI}, and it is the same override the run
  // endpoint takes, so a swept point and a run at the same place agree.
  for (const [k, v] of Object.entries(sets || {})) q.append('set', k + ':' + v);
  const r = await fetch('/v1/sweep?' + q.toString());
  const d = await r.json();
  if (!d.ok) throw new Error(d.message || 'the sweep was refused');
  return d;
}

/**
 * One row's answer with some declared values held elsewhere.
 *
 * `engineValues` asks what the tree says as it stands. This asks what it would
 * say somewhere else, which is what a scenario IS — and it is a question the
 * engine answers, so the face never has to evaluate a relation to find out.
 *
 * Returns the row's own SI value, or null with the reason, because a refusal at
 * a scenario is a fact about that scenario rather than an error in the figure.
 */
export async function engineAt(node, sets) {
  const q = new URLSearchParams({ node });
  for (const [k, v] of Object.entries(sets || {})) q.append('set', k + ':' + v);
  try {
    const r = await fetch('/v1/run?' + q.toString());
    const d = await r.json();
    if (!d.ok) return { si: null, refused: d.message || d.fault || 'refused' };
    const own = (d.values || []).find(v => v.id === node);
    return own ? { si: own.si, unit: own.unit } : { si: null, refused: 'the run did not return it' };
  } catch (e) {
    return { si: null, refused: String(e) };
  }
}

/**
 * Which declared decisions actually move a row's answer, most first. The engine
 * measures it; see the levers endpoint. A figure uses this to choose what to put
 * on an axis rather than guessing, which is how the sweep control came to offer
 * a decision worth exactly nothing.
 */
export async function engineLevers(node) {
  const r = await fetch('/v1/levers?node=' + encodeURIComponent(node));
  const d = await r.json();
  return d.ok ? (d.levers || []) : [];
}

/**
 * Another implementation's saved answers, for a figure that checks this one.
 *
 * Kept apart from `bundleFile` on purpose, and the separation is the point. A
 * bundle is verified reference data — what was OBSERVED, with a provenance and a
 * licence. `matlab/reference/mission_drivers.csv` is what a DIFFERENT PROGRAM
 * computed, saved by its author. Drawing the two with one function would be the
 * first step toward a figure that presents a second implementation's output as
 * evidence about the sky.
 */
export async function parityFile(file) {
  const key = 'parity/' + file;
  if (cache.has(key)) return cache.get(key);
  const p = (async () => {
    const r = await fetch('/v1/parity/' + file);
    if (!r.ok) throw new Error(await r.text());
    return parseCsv(await r.text());
  })();
  cache.set(key, p);
  return p;
}

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
    //
    // THE INCOMPLETE CYCLE IS FOLDED WITH THE MEAN LENGTH, NOT ITS OWN. A cycle
    // whose end nothing else opens is still running, and the "end" beside it in
    // solar_cycles.csv is where the RECORD stops rather than where the cycle
    // stops. Dividing by that length stretched cycle 25 across the whole phase
    // axis by a factor of 1.894, putting the record's last day at phase 1.007
    // where sw_cycle_phase puts it at 0.532 — so every phase-indexed panel drew
    // the running cycle at nearly double the phase the engine assigns it, and
    // the Repeatability panel's whole argument is a comparison of shapes ON that
    // axis. The mean of the complete cycles is what sw_cycle_phase folds with,
    // and it is what the face folds with now.
    // A cycle is complete when another one opens at or after its end; the last
    // in the table has no successor and is the one still running.
    const done = cycles.filter(c => cycles.some(o => o.start >= c.end));
    const meanLen = done.length
      ? done.reduce((p, c) => p + (c.end - c.start), 0) / done.length
      : null;
    const lengthOf = new Map(cycles.map(c =>
      [c.n, done.includes(c) ? c.end - c.start : meanLen]));
    for (const d of days) {
      d.cycle = null; d.phase = null;
      for (const c of cycles) {
        if (d.t >= c.start && d.t < c.end) {
          d.cycle = c.n;
          const len = lengthOf.get(c.n);
          const ph = len ? (d.t - c.start) / len : null;
          // A day more than one mean length into a cycle nobody has seen the end
          // of belongs to a cycle this record cannot name — exactly what the
          // engine's declared upper bound of 1 says — so it gets no phase rather
          // than one wrapped round.
          d.phase = ph !== null && ph <= 1 ? ph : null;
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

/**
 * A percentile of a sorted sample, interpolated between order statistics.
 *
 * THE CONVENTION IS THE ONE THE ROWS WERE MEASURED UNDER, and it is not the only
 * defensible one. A nearest-rank percentile — take the element at floor(q*n) —
 * was what this used to do, and it disagreed with sw_uncertainty_growth by up to
 * 0.7 sfu: that row's table holds 68.3 and 113.3 at one- and five-year leads, and
 * neither is a value the record contains, because they sit between two adjacent
 * observations. Linear interpolation at h = (n-1)q reproduces both exactly. A
 * panel that illustrates a row must compute the row's quantity the row's way, or
 * the picture and the claim drift apart in the fourth figure and nobody notices.
 */
export function quantile(sorted, q) {
  if (!sorted.length) return null;
  if (sorted.length === 1) return sorted[0];
  const h = (sorted.length - 1) * q;
  const lo = Math.floor(h), hi = Math.min(sorted.length - 1, lo + 1);
  return sorted[lo] + (h - lo) * (sorted[hi] - sorted[lo]);
}
