/*
  The solar weather tabs.

  Eight panels, one per tab of the study this subsystem was ported from. Each
  one is an argument about the record, and each is drawn from the record itself
  — the bundle's own CSV, served byte for byte — rather than from a summary
  somebody prepared. A figure computed from a different copy of the data than
  the engine reads is a figure that can quietly stop agreeing with the answers.

  EVERY PANEL IS RECOMPUTED ON EVERY CHANGE. Nothing is memoised except the
  files themselves. A control that redraws a cached picture is a control that
  eventually lies about what it is showing, and these are fast enough that the
  honest thing is also the simple one.

  What the panels do NOT do is compute the tree's answers. Where a number is a
  row, it is fetched by running the row — the rows are the claims, and a face
  that recomputed them would be a second implementation nobody reviews.
*/
'use strict';

import { $, esc } from './dom.js';
import { S } from './state.js';
import { solarRecord, bundleFile, engineValues, engineSweep, centredMean, corr, quantile, num, daysSince2000 } from './record.js';
import { drawChart, attachHover, INK } from './chart.js';

// ---------------------------------------------------------------------------
// describing the picture that was actually drawn

/**
 * What this curve does, measured from the curve.
 *
 * A panel with four selectors draws dozens of different pictures, and a caption
 * written for one of them is telling every other reader a story about a picture
 * they are not looking at. An audit of this view found 53 of its 95 distinct
 * pictures sharing a caption with a different picture — the Predict panel alone
 * had 40 pictures and 7 captions.
 *
 * So the part of a caption that describes the shape is computed here from the
 * series in hand. It is deliberately dull: where it starts, where it ends, where
 * the extremes are, and whether it moves at all. A flat line is the one a reader
 * most needs told about, because a flat line and a broken panel look identical.
 */
/**
 * A number as it should appear inside a SENTENCE.
 *
 * fmt is tuned for the value a chart reports, where six figures are a readout.
 * A caption is prose: three significant figures, with an integer left as an
 * integer — "1 d to 27 d" rather than "1.00 d to 27.0 d", and 0.0685 rather
 * than 0.0685159.
 */
function sig(v) {
  if (v === null || !isFinite(v)) return String(v);
  if (Number.isInteger(v)) return String(v);
  const s = Math.abs(v) >= 1000 ? v.toFixed(0) : v.toPrecision(3);
  return s.indexOf('.') >= 0 ? s.replace(/0+$/, '').replace(/\.$/, '') : s;
}

function shape(xs, ys, unit, xunit, xname) {
  const pts = xs.map((x, i) => [x, ys[i]]).filter(([, y]) => y !== null && isFinite(y));
  if (pts.length < 2) return 'Too few points to describe a shape.';
  const u = unit ? ' ' + unit : '', xu = xunit ? ' ' + xunit : '';
  const vals = pts.map(([, y]) => y);
  const lo = Math.min(...vals), hi = Math.max(...vals);
  const at = v => pts.find(([, y]) => y === v)[0];
  if (hi - lo <= Math.max(1e-9, 1e-9 * Math.abs(hi))) {
    return 'It is FLAT at ' + sig(hi) + u + ' at every ' + (xname || 'point') +
      ' drawn — which is a finding about the quantity, not a panel that failed to load.';
  }
  const first = pts[0], last = pts[pts.length - 1];
  const rising = last[1] > first[1];
  let s = 'It runs from ' + sig(first[1]) + u + ' at ' + sig(first[0]) + xu +
    ' to ' + sig(last[1]) + u + ' at ' + sig(last[0]) + xu + '. ';
  // WHETHER A CURVE IS "NOT MONOTONE" IS THE WRONG QUESTION TO ANSWER
  // LITERALLY. Almost no measured curve is monotone to the last decimal, and
  // saying so of a curve that rises all the way with one small wiggle in it
  // tells a reader the opposite of what the picture shows. What matters is
  // whether the extreme is INSIDE the range — a genuine hump or dip — which is
  // the thing a straight line through the curve would miss.
  const iHi = pts.findIndex(([, y]) => y === hi), iLo = pts.findIndex(([, y]) => y === lo);
  const interior = i => i > 0 && i < pts.length - 1;
  const turns = interior(iHi) || interior(iLo);
  if (turns) {
    s += 'It turns inside the range: the highest point is ' + sig(hi) + u + ' at ' + sig(at(hi)) + xu +
      ' and the lowest ' + sig(lo) + u + ' at ' + sig(at(lo)) + xu +
      ', so a straight line fitted through it would be describing neither end.';
  } else {
    // How far it backs up against its own direction, as a share of the range —
    // reported only when it is large enough for a reader to see.
    let back = 0;
    for (let i = 1; i < pts.length; i++) {
      const d = (rising ? pts[i - 1][1] - pts[i][1] : pts[i][1] - pts[i - 1][1]);
      if (d > back) back = d;
    }
    s += 'It ' + (rising ? 'rises' : 'falls') + ' across the whole range' +
      (back > (hi - lo) * 0.05
        ? ', with a reversal of ' + sig(back) + u + ' along the way.'
        : ' without turning back.');
  }
  return s;
}

// ---------------------------------------------------------------------------
// the panels

const PANELS = [
  {
    id: 'repeatability',
    rows: ['sw_cycle_phase', 'sw_mean_cycle_level', 'sw_cycle_repeatability'],
    label: 'Repeatability',
    draws: 'The mean cycle against every cycle the record holds, stacked on phase.',
    asks: 'How much of F10.7 does the cycle explain — and does one cycle repeat the last?',
    controls: [
      { k: 'view', label: 'view', opts: [['stack', 'the mean cycle'], ['storm', 'storm scale by cycle']] },
      // The storm-scale view is stormScale(rec): it takes the record and
      // nothing else, so neither of these reaches it. 8 of this panel's 18
      // combinations were the same picture.
      { k: 'v', label: 'variable', when: o => o.view === 'stack',
        opts: [['f107', 'F10.7'], ['ap', 'Ap'], ['ssn', 'sunspot number']] },
      { k: 'bins', label: 'phase bins', when: o => o.view === 'stack',
        opts: [['20', '20'], ['10', '10'], ['40', '40']] },
    ],
    build(rec, o) {
      if (o.view === 'storm') return stormScale(rec);
      const nb = +o.bins, key = o.v;
      const per = new Map();
      for (const c of rec.cycles) per.set(c.n, Array.from({ length: nb }, () => []));
      for (const d of rec.days) {
        if (d.phase === null || d[key] === null) continue;
        const b = Math.min(nb - 1, Math.floor(d.phase * nb));
        per.get(d.cycle)[b].push(d[key]);
      }
      const xs = Array.from({ length: nb }, (_, i) => (i + 0.5) / nb);
      const mean = a => (a.length ? a.reduce((p, c) => p + c, 0) / a.length : null);
      const series = [];
      const complete = rec.cycles.filter(c => c.n !== 25);
      const meanCycle = xs.map((_, i) => {
        const v = complete.map(c => mean(per.get(c.n)[i])).filter(x => x !== null);
        return v.length ? v.reduce((p, c) => p + c, 0) / v.length : null;
      });
      series.push({ name: 'mean of the complete cycles', kind: 'line', x: xs, y: meanCycle, colour: '#1a1a1a', width: 2.4 });
      rec.cycles.forEach((c, i) => {
        series.push({
          name: 'cycle ' + c.n + (c.n === 25 ? ' (incomplete)' : ''),
          kind: 'line', x: xs, y: xs.map((_, k) => mean(per.get(c.n)[k])),
          colour: INK.series[i % INK.series.length], width: 1.4,
          dash: c.n === 25 ? [4, 3] : null,
        });
      });
      // A bin thinned by the 2017 gap is dropped, not averaged. sw_cycle_repeatability
      // is measured the same way: a bin holding 141 days against the usual 201 is a
      // mean of a different thing, and including it moves the correlation by 0.006.
      const MIN = 150;
      const ok = i => per.get(23)[i].length >= MIN && per.get(24)[i].length >= MIN;
      const a = xs.map((_, i) => (ok(i) ? mean(per.get(23)[i]) : null));
      const b = xs.map((_, i) => (ok(i) ? mean(per.get(24)[i]) : null));
      const r = corr(a, b);
      const usable = a.filter((v, i) => v !== null && b[i] !== null).length;
      // The peak disagreement is measured for the variable on screen. The figure
      // used to be 28 per cent under all three, which is F10.7's.
      const pk = n => Math.max(...xs.map((_, i) => mean(per.get(n)[i])).filter(v => v !== null));
      const p23 = pk(23), p24 = pk(24);
      const vname = key === 'f107' ? 'F10.7' : key === 'ap' ? 'Ap' : 'the sunspot number';
      return {
        spec: {
          x: { label: 'cycle phase  [0 = minimum, 1 = the next]', min: 0, max: 1 },
          y: { label: key === 'f107' ? 'F10.7  [sfu]' : key === 'ap' ? 'Ap  [-]' : 'sunspot number  [-]' },
          series,
        },
        note: 'Cycles 23 and 24 are the only complete ones, and their ' + vname + ' shapes correlate at ' +
          (r === null ? '—' : r.toFixed(4)) + ' over the ' + usable + ' of ' + nb +
          ' bins both populate with at least ' + MIN + ' days. A correlation is scale-free, so it is ' +
          'blind to the thing a drag design cares about, and here that blindness costs: the stacked ' +
          'peaks are ' + sig(p23) + ' for cycle 23 against ' + sig(p24) + ' for cycle 24, a ratio of ' +
          (p24 / p23).toFixed(3) + '. The shape repeats and the size does not. Cycle 25 is dashed and ' +
          'stops part way because it is still running: its end in solar_cycles.csv is the record’s ' +
          'end rather than a minimum, so its phase is folded against the mean length of the ' +
          'complete cycles — the same fold sw_cycle_phase uses — and the record simply has no days ' +
          'past that point to draw.',
      };
    },
  },

  {
    id: 'pattern',
    rows: ['sw_recurrence_lag', 'sw_recurrence_strength', 'sw_spike_threshold', 'sw_event_duration'],
    label: 'Pattern',
    draws: 'The autocorrelation of the detrended record against lag, and its harmonics.',
    asks: 'At what lag does the solar rotation come back, and how strongly?',
    controls: [
      { k: 'view', label: 'view', opts: [['acf', 'recurrence and decay'], ['spikes', 'spikes: size and timing']] },
      // The spike view is spikes(rec) and reads none of these — 17 of this
      // panel's 36 combinations drew one picture.
      { k: 'v', label: 'variable', when: o => o.view === 'acf',
        opts: [['f107', 'F10.7'], ['ap', 'Ap']] },
      { k: 'w', label: 'detrend window', when: o => o.view === 'acf',
        opts: [['365', '365 d'], ['181', '181 d'], ['731', '731 d']] },
      { k: 'lag', label: 'max lag', when: o => o.view === 'acf',
        opts: [['120', '120 d'], ['60', '60 d'], ['200', '200 d']] },
    ],
    build(rec, o) {
      if (o.view === 'spikes') return spikes(rec);
      const key = o.v, W = +o.w, maxLag = +o.lag;
      const v = rec.days.map(d => d[key]);
      // 0.6, the same completeness rule sw_recurrence_lag and
      // sw_recurrence_strength were measured under. A panel that illustrates a
      // row and quotes a different number for it is worse than no panel.
      const trend = centredMean(v, W, 0.6);
      const res = v.map((x, i) => (x === null || trend[i] === null ? null : x - trend[i]));
      const xs = [], ys = [];
      for (let L = 1; L <= maxLag; L++) {
        xs.push(L);
        ys.push(corr(res.slice(0, res.length - L), res.slice(L)));
      }
      // The first bump's peak, and the harmonics that say what the period is.
      const peakIn = (lo, hi) => {
        let best = null, at = null;
        for (let L = lo; L <= hi && L <= maxLag; L++) {
          const r = ys[L - 1];
          if (r !== null && (best === null || r > best)) { best = r; at = L; }
        }
        return { at, r: best };
      };
      const p1 = peakIn(18, 36), p2 = peakIn(45, 65), p3 = peakIn(72, 95);
      const marks = [];
      if (p1.at) marks.push({ axis: 'x', at: p1.at, label: 'first peak, lag ' + p1.at });
      if (p2.at && maxLag >= 65) marks.push({ axis: 'x', at: p2.at, label: 'second, ' + p2.at, colour: '#2f6fa8' });
      return {
        spec: {
          x: { label: 'lag  [days]', min: 1, max: maxLag },
          y: { label: 'autocorrelation of the detrended series  [-]' },
          series: [{ name: '', kind: 'line', x: xs, y: ys }],
          marks,
        },
        note: p1.at
          ? 'The first peak is at lag ' + p1.at + ' with r = ' + p1.r.toFixed(6) +
            (p2.at && p3.at
              ? ', and the harmonics say the period is not that number: the second peak at ' + p2.at +
                ' and the third at ' + p3.at + ' imply ' + (p2.at / 2).toFixed(1) + ' and ' +
                (p3.at / 3).toFixed(1) + ' days per cycle. The first bump rides on the tail of the ' +
                'steep decay from lag 1, which pulls its apparent peak toward zero; the far ' +
                'harmonics are clear of it.'
              : '. Raise the max lag to see the harmonics, which are what settle the period.') +
            ' Drawn out to lag ' + maxLag + ', where the correlation is ' +
            (ys[maxLag - 1] === null ? 'undefined' : ys[maxLag - 1].toFixed(4)) + '. ' +
            'Shorten the detrend window toward the rotation itself and the signal disappears, ' +
            'because the window removes what it is meant to leave.'
          : 'No peak in the rotation band at this setting.',
      };
    },
  },

  {
    id: 'segmentation',
    rows: ['sw_regime', 'sw_activity_band'],
    label: 'Segmentation',
    draws: 'Where the record actually sits, and the boundaries the study cut it at.',
    asks: 'Quiet, active or storm — and how much of the record is each?',
    controls: [
      { k: 'view', label: 'view', opts: [['hist', 'where the record sits'], ['phase', 'regime against cycle phase']] },
      // regimeByPhase(rec) reads neither of these.
      { k: 'v', label: 'variable', when: o => o.view === 'hist',
        opts: [['ap', 'Ap — regime'], ['f107', 'F10.7 — activity band']] },
      { k: 'scale', label: 'count axis', when: o => o.view === 'hist',
        opts: [['log', 'log'], ['lin', 'linear']] },
    ],
    build(rec, o) {
      if (o.view === 'phase') return regimeByPhase(rec);
      const key = o.v;
      // THE ROW'S OWN EDGES, AND ITS OWN COMPARISON. sw_activity_band cuts F10.7
      // at 90, 130 and 170 with `>=`, so a flux sitting exactly on an edge
      // belongs to the band the edge OPENS. This panel drew 90/120/180 with `>`
      // and banded 1010 days — 9.8 per cent of the record — differently from the
      // row it exists to illustrate, reporting the top band at 8.9 per cent where
      // the row says 12.8. A panel that contradicts its own node is worse than no
      // panel, so the edges are taken from the node rather than restated.
      //
      // Ap is an integer, so its marks sit BETWEEN the bands at 6.5 and 25.5;
      // F10.7 is continuous and its marks sit on the edges themselves.
      const cuts = key === 'ap' ? [6.5, 25.5] : [90, 130, 170];
      const names = key === 'ap' ? ['quiet', 'active', 'storm'] : ['low', 'moderate', 'elevated', 'high'];
      const vals = rec.days.map(d => d[key]).filter(x => x !== null);
      const hi = Math.max(...vals);
      const bw = key === 'ap' ? 2 : 5;
      const nb = Math.ceil(hi / bw) + 1;
      const counts = new Array(nb).fill(0);
      for (const x of vals) counts[Math.floor(x / bw)]++;
      const xs = counts.map((_, i) => (i + 0.5) * bw);
      const ys = counts.map(c => (o.scale === 'log' ? (c ? Math.log10(c) : null) : c));
      const share = new Array(cuts.length + 1).fill(0);
      for (const x of vals) {
        let k = 0; while (k < cuts.length && x >= cuts[k]) k++;
        share[k]++;
      }
      return {
        spec: {
          x: { label: (key === 'ap' ? 'daily Ap' : 'F10.7  [sfu]') + '  [bin ' + bw + ']', min: 0 },
          y: { label: o.scale === 'log' ? 'days in bin  [log10]' : 'days in bin', min: 0 },
          series: [{ name: '', kind: 'bars', x: xs, y: ys, colour: '#b5731a' }],
          marks: cuts.map((c, i) => ({ axis: 'x', at: c, label: names[i] + ' | ' + names[i + 1] })),
        },
        note: 'Over ' + vals.length + ' days: ' +
          share.map((n, i) => names[i] + ' ' + (100 * n / vals.length).toFixed(1) + '%').join(' · ') +
          (key === 'ap'
            ? '. These are the study’s own mixture boundaries, and they fell on integers: quiet ' +
              'holds Ap 0 to 6, active 7 to 25, storm 26 and above. sw_regime reproduces ' +
              'daily_regime.csv exactly on 10128 of its 10299 days. The 171 that differ all have ' +
              'Ap 0 or 1 and are labelled storm in the published table — the broad storm component ' +
              'winning at the low tail — and this repository calls them quiet.'
            : '. The edges are sw_activity_band’s own — 90, 130 and 170 sfu — and so is the rule ' +
              'that a flux sitting exactly on an edge belongs to the band the edge opens. This ' +
              'panel used to draw 90/120/180 with a strict comparison, which banded 1010 days ' +
              'differently from the row it illustrates and reported the top band at 8.9 per cent ' +
              'where the row says 12.8.') +
          (o.scale === 'log'
            ? ' The count axis is logarithmic, so a bar half as tall holds a tenth as many days — ' +
              'which is the only way the tail a design is sized by is visible at all.'
            : ' On this linear count axis the tail that matters to a design is a row of pixels one ' +
              'high; the log axis is what makes it readable.'),
      };
    },
  },

  {
    id: 'predict',
    rows: ['sw_uncertainty_growth', 'sw_band_coverage', 'sw_horizon_persistence', 'sw_horizon_climatology'],
    label: 'Predict',
    draws: 'How far the flux moves over a lead, at a percentile.',
    asks: 'How far ahead is F10.7 knowable, and what does the band cost?',
    controls: [
      { k: 'v', label: 'variable', opts: [['f107', 'F10.7'], ['ap', 'Ap']] },
      { k: 'q', label: 'percentile', opts: [['0.95', '95th'], ['0.5', '50th'], ['0.9', '90th'], ['0.99', '99th']] },
      { k: 'span', label: 'lead out to', opts: [['1826', '5 years'], ['365', '1 year'], ['5478', '15 years']] },
      { k: 'by', label: 'split', opts: [['all', 'the whole record'], ['cycle', 'by cycle']] },
    ],
    build(rec, o) {
      const q = +o.q, maxL = +o.span, key = o.v;
      if (o.by === 'cycle') return growthByCycle(rec, key, q, maxL);
      const byDay = new Map();
      for (const d of rec.days) if (d[key] !== null) byDay.set(d.t, d[key]);
      const leads = [];
      for (let L = 30; L <= maxL; L = Math.round(L * 1.35)) leads.push(L);
      const xs = [], ys = [], ns = [];
      for (const L of leads) {
        const ch = [];
        for (const [t, v] of byDay) {
          const w = byDay.get(t + L);
          if (w !== undefined) ch.push(w - v);
        }
        ch.sort((a, b) => a - b);
        xs.push(L / 365.25); ys.push(quantile(ch, q)); ns.push(ch.length);
      }
      const name = key === 'f107' ? 'F10.7' : 'Ap';
      const unit = key === 'f107' ? 'sfu' : '';
      const pct = (q * 100).toFixed(q * 100 % 1 ? 1 : 0);
      // Whether the eleven-year cycle is visible depends on how far out the lead
      // goes, so the sentence about it is earned by the picture rather than
      // attached to every one of them.
      // Keyed on what the reader SELECTED, not on where the geometric lead ladder
      // happened to stop: choosing five years lands the last lead at 4.08, so a
      // test on the curve told a reader who had already extended it to extend it.
      const far = maxL > 400;
      const drawn = ys.filter(y => y !== null && isFinite(y));
      const flat = drawn.length > 1 && Math.max(...drawn) - Math.min(...drawn) <= 1e-9;
      const mid = ys.filter((y, i) => y !== null && xs[i] >= 3 && xs[i] <= 6);
      const late = ys.filter((y, i) => y !== null && xs[i] >= 9 && xs[i] <= 12);
      const humped = far && mid.length && late.length &&
        Math.max(...mid) > Math.max(...late);
      return {
        spec: {
          x: { label: 'lead  [years]', min: 0 },
          y: { label: 'change in ' + name + ' at the ' + pct + 'th percentile  [' + (unit || '-') + ']' },
          // `ns` was counted here and thrown away. Handing it to the chart is what
          // makes the far end of this curve look as thin as it is.
          series: [{ name: '', kind: 'line', x: xs, y: ys, n: ns }],
        },
        note: 'The ' + pct + 'th percentile of the SIGNED change in ' + name + ' over a lead — not the ' +
          'absolute change, because the unsafe direction for a drag design is the driver arriving ' +
          'higher than planned. ' + shape(xs, ys, unit, 'yr', 'lead') +
          (q <= 0.5
            ? ' At the median there is no tail to speak of: half of all changes are above this line ' +
              'and half below, so a value near zero says the driver has no trend over these leads, ' +
              'which is what a cyclic quantity looks like when the lead is not tied to its phase.'
            : '') +
          (humped
            ? ' The hump and the dip are the eleven-year cycle rather than noise: a lead of about ' +
              'half a cycle is the lead most likely to land on the opposite phase, and a lead of ' +
              'about a full cycle returns to a similar one.'
            : far || flat
              ? ''
              : ' At a lead of a year the cycle is invisible: extend it to five or fifteen years to ' +
                'see what the solar cycle does to this curve.') +
          ' Pairs at a given lead overlap almost completely, so the ' + ns[0] + ' to ' +
          ns[ns.length - 1] + ' counted here are nothing like that many independent observations.',
      };
    },
  },

  {
    id: 'forecast',
    rows: ['sw_outlook_lead', 'sw_forecast_skill', 'sw_forecast_bias'],
    label: 'Forecast',
    draws: 'The issued 27-day outlook, scored against what arrived.',
    asks: 'Is the published forecast worth more than assuming nothing changes?',
    needs: ['forecast_issued.csv'],
    controls: [
      // 'by calendar year' was offered here for a while and read nowhere, so
      // choosing it silently redrew the against-lead picture. Declaring this
      // panel in panels/ found it — check two drives every option a panel
      // claims to read and demands the canvas change — and it was removed
      // rather than left advertised. It is now built, and its [[input]] in
      // panels/forecast.toml is what keeps it built.
      { k: 'view', label: 'view', opts: [['lead', 'against lead'], ['year', 'by calendar year'], ['age', 'issue age']] },
      // issueAge(idx) is the issue-age view and reads neither metric nor
      // baseline — 5 of this panel's 18 combinations were one picture.
      { k: 'm', label: 'metric', when: o => o.view !== 'age',
        opts: [['skill', 'skill vs persistence'], ['bias', 'bias'], ['rmse', 'RMS error']] },
      { k: 'base', label: 'persistence baseline', when: o => o.view !== 'age',
        opts: [['strict', 'last obs BEFORE issue'], ['leaky', 'obs ON the issue date']] },
    ],
    async data() {
      const [fc, idx] = await Promise.all([
        bundleFile('solar-weather', 'forecast_issued.csv'),
        bundleFile('solar-weather', 'forecast_issues.csv'),
      ]);
      return { fc, idx };
    },
    build(rec, o, extra) {
      const fc = extra.fc.rows;
      if (o.view === 'age') return issueAge(extra.idx.rows);
      const byDay = new Map();
      for (const d of rec.days) if (d.f107 !== null) byDay.set(d.t, d.f107);
      const tOf = new Map();
      for (const d of rec.days) tOf.set(d.date, d.t);
      const persist = (issue) => {
        const t = tOf.get(issue);
        if (t === undefined) return null;
        for (let b = o.base === 'leaky' ? 0 : 1; b <= 15; b++) {
          const v = byDay.get(t - b);
          if (v !== undefined) return v;
        }
        return null;
      };
      // The year view needs the same observations and the same baseline, so it
      // is dispatched here rather than at the top: a second copy of `persist`
      // is a second place for the leaky/strict choice to stop agreeing.
      if (o.view === 'year') {
        const y = byIssueYear(fc, byDay, tOf, persist, o.m);
        const marks = scoreBaseline(o.m);
        const worst = y.kept.length ? y.kept[y.keptYs.indexOf(Math.min(...y.keptYs))] : null;
        const best = y.kept.length ? y.kept[y.keptYs.indexOf(Math.max(...y.keptYs))] : null;
        return {
          spec: {
            x: { label: 'the calendar year the outlook was issued in' },
            y: { label: o.m === 'skill' ? 'skill against persistence  [-]'
              : o.m === 'bias' ? 'mean signed error, forecast − observed  [sfu]' : 'RMS error  [sfu]' },
            series: [{ name: '', kind: 'line', x: y.years, y: y.ys }],
            marks,
          },
          note: 'One point per calendar year, scored on leads ' + y.LO + ' to ' + y.HI + ' ONLY. ' +
            'That restriction is what makes the years comparable and it is not cosmetic: the archive ' +
            'is not uniform, and 2004 and 2007 carry no row past lead 14 at all while 2011 onward ' +
            'carry a balanced mix. Scoring every lead together would draw the history of the archive ' +
            'and label it the skill of the forecaster. The year is the year of ISSUE, so a December ' +
            'outlook is counted against December even where it verifies into January.\n\n' +
            (y.thin.length
              ? y.thin.length + ' year(s) are dropped for holding fewer than ' + y.MIN + ' usable ' +
                'pairs — ' + y.thin.join(', ') + '. They are not quiet years, they are thin ones, and ' +
                'the arithmetic on them is violent: 2010 holds 25 pairs and scores -11.8. '
              : '') +
            (o.m === 'skill' && worst !== null
              ? 'Skill is worst in ' + worst + ' and best in ' + best + '. A year at solar minimum ' +
                'is the hard case for a forecaster and the easy one for persistence — when the flux ' +
                'is flat, assuming nothing changes is very nearly right — so 2008 scoring below the ' +
                'red line is the record behaving, not the outlook failing. Its RMS error that year ' +
                'is about 3.5 sfu, the smallest in the series.'
              : o.m === 'bias'
                ? 'Bias runs strongly negative through 2022 to 2024, the rise of cycle 25: the ' +
                  'outlook came in LOW by 7 to 9 sfu a year while activity was climbing. A design ' +
                  'reading it there gets a thinner atmosphere than it will fly, which is the ' +
                  'direction that costs propellant rather than the one that wastes it.'
                : 'RMS error tracks the level rather than the difficulty — it is smallest at the ' +
                  '2008 and 2018 minima and largest through solar maximum, because a bigger number ' +
                  'has bigger errors. It says nothing about beating a baseline; the skill metric ' +
                  'does.') +
            ' ' + shape(y.kept, y.keptYs, o.m === 'skill' ? '' : 'sfu', '', 'year'),
        };
      }

      const pc = new Map();
      const xs = [], ys = [], ns = [];
      for (let L = 1; L <= 27; L++) {
        let e2 = 0, p2 = 0, se = 0, n = 0;
        for (const r of fc) {
          if (+r.lead_days !== L || r.f107 === null) continue;
          const tt = tOf.get(r.target_date);
          const obs = tt === undefined ? undefined : byDay.get(tt);
          if (obs === undefined) continue;
          if (!pc.has(r.issue_date)) pc.set(r.issue_date, persist(r.issue_date));
          const p = pc.get(r.issue_date);
          if (p === null) continue;
          const e = +r.f107 - obs;
          e2 += e * e; p2 += (p - obs) * (p - obs); se += e; n++;
        }
        if (!n) continue;
        xs.push(L); ns.push(n);
        ys.push(o.m === 'skill' ? 1 - (e2 / n) / (p2 / n) : o.m === 'bias' ? se / n : Math.sqrt(e2 / n));
      }
      const marks = scoreBaseline(o.m);
      const last = xs[xs.length - 1];
      return {
        spec: {
          x: { label: 'lead  [days]', min: 1, max: 27 },
          y: { label: o.m === 'skill' ? 'skill against persistence  [-]' : o.m === 'bias' ? 'mean signed error, forecast − observed  [sfu]' : 'RMS error  [sfu]' },
          // `ns` was counted here and thrown away. Handing it to the chart is what
          // makes the far end of this curve look as thin as it is.
          series: [{ name: '', kind: 'line', x: xs, y: ys, n: ns }],
          marks,
        },
        note: (o.m === 'skill'
          ? 'Skill is one minus the ratio of mean squared errors, so zero is the red line — no ' +
            'better than assuming nothing changes — and negative is worse than not bothering. '
          : o.m === 'bias'
            ? 'Signed error, forecast minus observed, so a negative value means the outlook came in ' +
              'LOW and a design reading it gets a thinner atmosphere than it will fly. '
            : 'Root mean square error in sfu, which is accuracy rather than skill: it says nothing ' +
              'about whether the outlook beats a baseline, only how far it misses. ') +
          shape(xs, ys, o.m === 'skill' ? '' : 'sfu', 'd', 'lead') + ' ' +
          (o.base === 'leaky'
            ? 'THIS BASELINE LEAKS. 719 of the 1281 issues index their rows from lead 0, so the issue ' +
              'date is itself a forecast target for most of the record, and handing it to persistence ' +
              'gives the baseline a number the forecaster did not have. Switch to the strict baseline ' +
              'and the sign of the short-lead answer changes.' +
              (o.m !== 'skill' ? ' It changes nothing on this metric, which does not use a baseline.' : '')
            : 'Persistence is the last observation strictly BEFORE the issue date — what a ' +
              'forecaster actually had.' +
              (o.m === 'skill' ? ' On this baseline the outlook beats it from lead 1.' : '')) +
          ' Lead ' + last + ' draws on ' + ns[ns.length - 1] + ' pairs against ' + ns[0] + ' at lead 1: ' +
          'lead_days is indexed two ways in one column, and only the 1-based minority reaches 27, ' +
          'which is why sw_outlook_lead declares 26.',
      };
    },
  },

  {
    id: 'design',
    rows: ['sw_storm_return_level', 'sw_storm_design_level', 'sw_ap_design',
      'sw_design_safe_duration', 'sw_exceedance_rate', 'sw_exceedance_duration',
      'sw_exceedance_phase', 'l3_solar_req_03', 'l3_solar_ach_03',
      // The six §20 built for the F10.7 half. The panel's own label is "the
      // design window for F10.7 and for Ap" and it cited none of them, because
      // it was still drawing the method they replaced.
      'sw_f107_design_long', 'sw_f107_design_short', 'sw_f107_cold_long',
      'sw_f107_cold_short', 'l3_solar_req_01'],
    // WHAT THIS PANEL ASKS THE ENGINE FOR, rather than carrying a copy of.
    // `rows` says what the picture argues about; `engine` says what it reads,
    // and panel_check holds it to the second. The literals these replace had
    // gone stale by two revisions — §21.1 lists them.
    engine: ['l3_solar_req_01', 'l3_solar_req_03',
      'l3_solar_req_04', 'l3_solar_req_05', 'l3_solar_req_02'],
    label: 'Design',
    draws: 'The design window: what the record expects against what the vehicle is built for.',
    asks: 'Will the design be exceeded, and if so beyond what mission length?',
    controls: [
      { k: 'v', label: 'driver', opts: [['ap', 'Ap — return period'], ['f107', 'F10.7 — lead and confidence']] },
      // THE G SCALE IS GEOMAGNETIC. G1 to G3 is a storm scale and F10.7 has no
      // place on it; offering it there was not a repeat to be merged but a
      // control that could never have meant anything. §24.1.
      { k: 'g', label: 'designed for', when: o => o.v === 'ap',
        opts: [['3', 'G3 strong'], ['2', 'G2 moderate'], ['1', 'G1 minor']] },
      // THE REQUIREMENTS ARE ROWS, NOT NUMBERS TYPED HERE. This offered Ap 150,
      // 132 and 200; no row has ever held 150 or 200, and the requirement a
      // person settled on 2026-09-16 is 207. The options name rows and the
      // values come from the engine, so the list cannot drift from the tree
      // again — and an Ap bound can no longer be drawn on an F10.7 axis,
      // because the two drivers carry their own control.
      { k: 'req', label: 'Ap requirement', when: o => o.v === 'ap', opts: [
        ['l3_solar_req_03', 'survival — req_03'],
        ['l3_solar_req_04', 'sustained — req_04'],
        ['l3_solar_req_05', 'single day — req_05'],
      ] },
      { k: 'reqf', label: 'F10.7 requirement', when: o => o.v === 'f107', opts: [
        ['l3_solar_req_01', 'sustained — req_01'],
        ['l3_solar_req_02', 'single day — req_02'],
      ] },
    ],
    // THE TWO RELATIONS THIS PANEL DRAWS, ASKED OF THE ENGINE RATHER THAN
    // COPIED. `A = 92.515531, B = 40.926516` were sw_storm_return_level's fit
    // constants written out here, and `{1: 48, 2: 80, 3: 132}` was sw_ap_design's
    // G-to-Ap conversion written out here. A figure carrying a row's constants
    // is a second copy of that row, and it goes stale silently: the centre this
    // panel drew was two revisions old before anything noticed. Both are swept
    // from the rows now, so the picture is the relation the engine computes.
    //
    // Neither sweep depends on a control, so both are fetched once here rather
    // than on every redraw.
    async data() {
      const dur = S.byId.get('sys_mission_requirements_mission_duration');
      const glv = S.byId.get('sw_storm_design_level');
      // The F10.7 half is the four rows §20 built for it — hot and cold, each
      // sustained and single-day. That IS the design window, and it is built
      // with the within-rotation spread conditioned on the level it applies at.
      // The panel used to compute `centre + p95` here, which is the relation of
      // sw_f107_design — the row §20 DEPRECATED, and the method the record puts
      // 74 per cent high. Drawing a retired method beside live rows is how a
      // figure tells a reader something the tree has stopped believing.
      const [ret, gmap, fl, fs, cl, cs] = await Promise.all([
        engineSweep('sw_storm_return_level', 'sys_mission_requirements_mission_duration',
          dur.lo, dur.hi, 120),
        engineSweep('sw_ap_design', 'sw_storm_design_level', glv.lo, glv.hi, 3),
        engineSweep('sw_f107_design_long', 'sys_mission_requirements_mission_duration',
          dur.lo, dur.hi, 60),
        engineSweep('sw_f107_design_short', 'sys_mission_requirements_mission_duration',
          dur.lo, dur.hi, 60),
        engineSweep('sw_f107_cold_long', 'sys_mission_requirements_mission_duration',
          dur.lo, dur.hi, 60),
        engineSweep('sw_f107_cold_short', 'sys_mission_requirements_mission_duration',
          dur.lo, dur.hi, 60),
      ]);
      return { ret, gmap, win: { fl, fs, cl, cs } };
    },
    build(rec, o, extra, eng) {
      if (o.v === 'f107') return f107Window(extra, o, eng);
      // The relation as the ENGINE computes it, swept from sw_storm_return_level
      // rather than re-stated from its constants. A figure that carries a row's
      // coefficients is a second copy of that row.
      const YR = 31557600;
      const xs = extra.ret.x.map(v => v / YR);
      const ys = extra.ret.y.slice();
      // sw_ap_design's own G-to-Ap conversion, swept over the G level. The
      // sweep returns the three points in the order of the level, so the bound
      // is read off by index rather than from a table written out here.
      const gi = extra.gmap.x.indexOf(+o.g);
      if (gi < 0 || extra.gmap.y[gi] === undefined) {
        throw new Error('sw_ap_design did not answer at G' + o.g);
      }
      const bound = extra.gmap.y[gi];
      const rq = eng[o.req];
      if (!rq || rq.si === undefined) {
        throw new Error(o.req + ' did not answer: ' + ((rq && rq.refused) || 'not asked'));
      }
      const req = rq.si;
      // Where the curve crosses a level, read off the swept points by
      // interpolation. The closed form needed the fit constants; this needs
      // only the curve, which is the thing actually drawn.
      const cross = (ap) => {
        for (let i = 1; i < ys.length; i++) {
          if ((ys[i - 1] - ap) * (ys[i] - ap) <= 0 && ys[i] !== ys[i - 1]) {
            const f = (ap - ys[i - 1]) / (ys[i] - ys[i - 1]);
            return xs[i - 1] + f * (xs[i] - xs[i - 1]);
          }
        }
        return ap <= ys[0] ? xs[0] : NaN;
      };
      // What the record actually did above the bound, counted here from the
      // record rather than taken from the rows, because the panel must be able
      // to answer for a level the rows are not set to.
      const days = rec.days.filter(d => d.ap !== null);
      const years = days.length / 365.25;
      const above = days.filter(d => d.ap >= bound);
      let runs = 0, prev = -99;
      for (const d of above) { if (d.t !== prev + 1) runs++; prev = d.t; }
      const rate = above.length / years;
      return {
        spec: {
          x: { label: 'mission length  [years]', min: xs[0], max: xs[xs.length - 1] },
          y: { label: 'daily Ap the record expects once in that time  [-]' },
          series: [{ name: 'sw_storm_return_level', kind: 'line', x: xs, y: ys }],
          marks: [
            { axis: 'y', at: bound, label: 'designed for G' + o.g + ' = Ap ' + bound.toFixed(0) +
              '  (sw_ap_design)' },
            { axis: 'y', at: req, label: 'required ≤ ' + req.toFixed(0) + '  (' + o.req + ')',
              colour: '#c2185b' },
            { axis: 'x', at: cross(bound), label: 'exceeds the design at ' + cross(bound).toFixed(2) + ' yr' },
          ],
        },
        note: 'The design bound is exceeded beyond a ' + cross(bound).toFixed(2) + '-year mission and the ' +
          o.req + '\u2019s ' + req.toFixed(0) + ' beyond ' + cross(req).toFixed(2) +
          '. Over a 5-year mission the record holds ' +
          (5 * rate).toFixed(2) + ' days above Ap ' + bound.toFixed(0) + ', in about ' + (5 * runs / years).toFixed(2) +
          ' separate events — ' + above.length + ' days in ' + runs + ' events across ' +
          years.toFixed(2) + ' years of record. That the exceedance is brief and rare is what makes ' +
          'the bound acceptable rather than failed, and it is only knowable because it is counted.',
      };
    },
  },

  {
    id: 'climate',
    rows: ['sw_central_expectation', 'sw_semiannual_amplitude', 'sw_f107a_ratio', 'sw_mean_cycle_level', 'sw_kp_from_ap', 'sw_kp_slot_bias'],
    label: 'Climate',
    draws: 'The long run: the record by year, and the season inside the year.',
    asks: 'What is the context a single mission sits inside?',
    controls: [
      // kpAgainstAp(rec) plots Kp against ap and has no variable to pick.
      { k: 'v', label: 'variable', when: o => o.by !== 'kpap',
        opts: [['f107', 'F10.7'], ['ap', 'Ap'], ['ssn', 'sunspot number']] },
      { k: 'by', label: 'aggregate', opts: [['year', 'by year'], ['doy', 'by day of year'], ['month', 'by month'], ['smooth', 'the 13-month smoother'], ['kpap', 'Kp against ap']] },
    ],
    async data() { return bundleFile('solar-weather', 'monthly_means.csv'); },
    build(rec, o, extra) {
      if (o.by === 'smooth') return smoothed(extra.rows, o.v);
      if (o.by === 'kpap') return kpAgainstAp(rec);
      const key = o.v;
      const grp = new Map();
      for (const d of rec.days) {
        if (d[key] === null) continue;
        const g = o.by === 'year' ? d.year : o.by === 'doy' ? Math.ceil(d.doy / 5) * 5 : (d.year * 12 + +d.date.slice(5, 7));
        if (!grp.has(g)) grp.set(g, []);
        grp.get(g).push(d[key]);
      }
      const ks = [...grp.keys()].sort((a, b) => a - b);
      const xs = o.by === 'month' ? ks.map(k => Math.floor(k / 12) + (k % 12) / 12) : ks;
      const ys = ks.map(k => grp.get(k).reduce((p, c) => p + c, 0) / grp.get(k).length);
      // THE RECORD'S MEAN IS OVER DAYS, NOT OVER GROUPS. Averaging the yearly
      // means weights 1997 — which the record joins in January and holds 357
      // days of — the same as a full year, and the answer then disagrees with
      // sw_central_expectation's climatology, which is the day mean. For F10.7
      // the two are 113.78 and 114.84.
      const allDays = rec.days.map(d => d[key]).filter(v => v !== null);
      const overall = allDays.reduce((p, c) => p + c, 0) / allDays.length;
      const marks = [{ axis: 'y', at: overall,
        label: 'mean over the record’s ' + allDays.length + ' days = ' + overall.toFixed(2) }];
      if (o.by === 'doy') {
        marks.push({ axis: 'x', at: 80, label: 'March equinox', colour: '#2f6fa8' });
        marks.push({ axis: 'x', at: 266, label: 'September equinox', colour: '#2f6fa8' });
      }
      return {
        spec: {
          x: { label: o.by === 'year' ? 'year' : o.by === 'doy' ? 'day of year  [5-day bins]' : 'year' },
          y: { label: (key === 'f107' ? 'F10.7  [sfu]' : key === 'ap' ? 'Ap  [-]' : 'sunspot number  [-]') + ', mean' },
          series: [{ name: '', kind: o.by === 'year' ? 'bars' : 'line', x: xs, y: ys }],
          marks,
        },
        note: (() => {
          const vname = key === 'f107' ? 'F10.7' : key === 'ap' ? 'Ap' : 'the sunspot number';
          const unit = key === 'f107' ? 'sfu' : '';
          const hi = Math.max(...ys), lo = Math.min(...ys);
          const atHi = xs[ys.indexOf(hi)], atLo = xs[ys.indexOf(lo)];
          const spread = 'The ' + (o.by === 'doy' ? '5-day bins' : o.by === 'year' ? 'yearly means' : 'monthly means') +
            ' run from ' + sig(lo) + (unit ? ' ' + unit : '') + ' at ' + sig(atLo) + ' to ' + sig(hi) +
            (unit ? ' ' + unit : '') + ' at ' + sig(atHi) + ', a ratio of ' + (hi / lo).toFixed(2) +
            ' about a mean of ' + overall.toFixed(2) + ' over the record’s ' + allDays.length +
            ' days. ';
          if (o.by === 'doy' && key === 'ap') {
            return spread + 'That swing is the equinoctial effect and its SIZE is the point: the ' +
              'fitted semiannual amplitude is 1.278 on an offset of 10.494, about 12 per cent, and ' +
              'it accounts for 0.63 per cent of the DAILY variance. It moves a monthly budget and ' +
              'says almost nothing about a given day.';
          }
          if (o.by === 'doy') {
            return spread + vname + ' has no seasonal term — it is a property of the Sun, not of the ' +
              'Earth’s tilt — so this spread is the cycle landing unevenly across the calendar ' +
              'rather than a season. Switch to Ap, where the two equinox marks line up with real peaks.';
          }
          return spread + 'The eleven-year cycle is the whole of that range: a mission is sized ' +
            'against wherever in it the mission falls. The 2017 gap is 273 consecutive days and ' +
            'shows here as a year drawn from nine months — nothing is interpolated across it.';
        })(),
      };
    },
  },

  {
    id: 'density',
    // The one figure with no row that ANSWERS it, hung on the row that should.
    // sys_space_environment_atmospheric_density is seeded: declared at layer 2,
    // computed by nothing. Opening it and finding the two drivers and no
    // density curve is the point stated where it bites, instead of on a tab of
    // its own where it read as a gap in the tool rather than a gap in the tree.
    rows: ['sys_space_environment_atmospheric_density'],
    label: 'Density',
    draws: 'Nothing yet, and the reason is worth a panel.',
    asks: 'What are these drivers worth as atmospheric density?',
    controls: [],
    build(rec) {
      const withBoth = rec.days.filter(d => d.f107 !== null && d.ap !== null);
      // "Close to independent" was an assertion. It is now a measurement, because
      // it is the claim this panel rests on: if the two drivers carried the same
      // information a density model would not need both.
      const r = corr(withBoth.map(d => d.f107), withBoth.map(d => d.ap));
      return {
        spec: {
          x: { label: 'F10.7  [sfu]' },
          // Ap FLOORS AT ZERO and this is the one panel whose data reaches it:
          // a quiet day really is Ap 0, so the 6 per cent padding the chart adds
          // below an undeclared minimum ran the axis down to -16.4 — a region of
          // an index that has no negative values. Five other panels already
          // declare this floor; this one did not, which is why it was the only
          // chart in the repository drawing space that cannot exist.
          y: { label: 'daily Ap  [-]', min: 0 },
          series: [{ name: '', kind: 'dots', x: withBoth.map(d => d.f107), y: withBoth.map(d => d.ap), width: 1.1, alpha: 0.18 }],
        },
        note: 'NOTHING COMPUTES THIS ROW AND THIS FIGURE IS NOT IT. The study’s density tab — profile, ' +
          'spread, sensitivity, by driver, by altitude — needs an atmosphere model, and that ' +
          'belongs to a different subsystem which has nothing written in it. Drawing a density ' +
          'curve here would mean this face carrying a model no row owns and no reviewer signed.\n\n' +
          'What is drawn instead is the honest precondition: the two drivers a density model takes, ' +
          'against each other, over ' + withBoth.length + ' days. They correlate at ' +
          (r === null ? '—' : r.toFixed(4)) + ', which is ' +
          (r === null ? '' : (100 * r * r).toFixed(1) + ' per cent of the variance shared') +
          ' — close to independent, and that is why a density model asks for both and why neither ' +
          'substitutes for the other. A flux and a disturbance are different physics: EUV heats the ' +
          'thermosphere from above and a geomagnetic storm deposits energy into it at high latitude.',
      };
    },
  },
];


// ---------------------------------------------------------------------------
// the views the study's tabs name and the first pass did not draw

/** Repeatability · storm scale. How unevenly the storms fall across cycles. */
function stormScale(rec) {
  const LV = [[48, 'G1'], [80, 'G2'], [132, 'G3']];
  const per = rec.cycles.map(c => {
    const d = rec.days.filter(x => x.cycle === c.n && x.ap !== null);
    return { c, n: d.length, days: d };
  });
  const series = LV.map(([thr, name], i) => ({
    name: name + ' (Ap \u2265 ' + thr + ')',
    kind: 'bars',
    x: per.map(p => p.c.n),
    // Per YEAR of the cycle, not per cycle: cycle 25 is six years long in this
    // record and the other two are eleven, so raw counts would say more about
    // how much of each cycle the record holds than about the Sun.
    y: per.map(p => (p.n ? p.days.filter(d => d.ap >= thr).length / (p.n / 365.25) : null)),
    colour: INK.series[i],
  }));
  const worst = per.map(p => ({ n: p.c.n, max: Math.max(...p.days.map(d => d.ap)) }));
  return {
    spec: {
      x: { label: 'solar cycle', ticks: 2 },
      y: { label: 'days a year at or above the level', min: 0 },
      series,
    },
    note: 'Storms are not shared out evenly between cycles. Largest daily Ap by cycle: ' +
      worst.map(w => w.n + ' \u2192 ' + w.max).join(', ') +
      '. Counted per year of each cycle rather than per cycle, because this record holds all of 23 ' +
      'and 24 and only six years of 25. A design sized on the average cycle is sized for neither the ' +
      'cycle it will fly through nor the worst one here.',
  };
}

/** Pattern · spikes. What counts as one, how big, and when they fall. */
function spikes(rec) {
  const v = rec.days.map(d => d.f107);
  const base = centredMean(v, 81, 0.7);
  const ratio = rec.days.map((d, i) => (v[i] === null || base[i] === null ? null : v[i] / base[i]));
  const ok = ratio.filter(x => x !== null);
  const mu = ok.reduce((p, c) => p + c, 0) / ok.length;
  const sd = Math.sqrt(ok.reduce((p, c) => p + (c - mu) * (c - mu), 0) / ok.length);
  const thr = mu + 2.5 * sd;
  const nb = 20, byPhase = new Array(nb).fill(0), allPhase = new Array(nb).fill(0);
  let n = 0, runs = 0, prev = -99;
  rec.days.forEach((d, i) => {
    if (ratio[i] === null || d.phase === null) return;
    const b = Math.min(nb - 1, Math.floor(d.phase * nb));
    allPhase[b]++;
    if (ratio[i] >= thr) {
      byPhase[b]++; n++;
      if (d.t !== prev + 1) runs++;
      prev = d.t;
    }
  });
  const xs = byPhase.map((_, i) => (i + 0.5) / nb);
  return {
    spec: {
      x: { label: 'cycle phase', min: 0, max: 1 },
      y: { label: 'spike days per 1000 days at that phase', min: 0 },
      series: [{ name: '', kind: 'bars', x: xs, y: byPhase.map((c, i) => (allPhase[i] ? 1000 * c / allPhase[i] : null)) }],
    },
    note: 'A spike is a day whose F10.7 exceeds its own 81-day centred mean by the record\u2019s own ' +
      'scatter: mean + 2.5 sd = ' + thr.toFixed(6) + ', which sw_spike_threshold declares. That ' +
      'catches ' + n + ' days in ' + runs + ' separate bursts. Normalised per thousand days at each ' +
      'phase, so a phase the record simply holds more of does not look spikier. They cluster before ' +
      'and around maximum \u2014 which is a different answer from where the Ap exceedances fall, and ' +
      'the two should not be confused: flux spikes and geomagnetic storms are not the same event.',
  };
}

/** Segmentation · regime against cycle phase. Where in a cycle a storm is likely. */
function regimeByPhase(rec) {
  const nb = 20;
  const tot = new Array(nb).fill(0), st = new Array(nb).fill(0), qt = new Array(nb).fill(0);
  for (const d of rec.days) {
    if (d.phase === null || d.ap === null) continue;
    const b = Math.min(nb - 1, Math.floor(d.phase * nb));
    tot[b]++;
    if (d.ap >= 26) st[b]++;
    else if (d.ap <= 6) qt[b]++;
  }
  const xs = tot.map((_, i) => (i + 0.5) / nb);
  return {
    spec: {
      x: { label: 'cycle phase', min: 0, max: 1 },
      y: { label: 'share of days at that phase  [%]', min: 0 },
      series: [
        { name: 'storm (Ap \u2265 26)', kind: 'line', x: xs, y: st.map((c, i) => (tot[i] ? 100 * c / tot[i] : null)), colour: '#c2185b' },
        { name: 'quiet (Ap \u2264 6)', kind: 'line', x: xs, y: qt.map((c, i) => (tot[i] ? 100 * c / tot[i] : null)), colour: '#2f6fa8' },
      ],
      marks: [{ axis: 'x', at: 0.6193669438, label: 'the declared epoch, phase 0.619' }],
    },
    note: 'The storm share peaks on the DECLINING side, past maximum, where coronal holes dominate ' +
      'rather than active regions \u2014 and the declared epoch sits inside that band. This is the ' +
      'same shape sw_exceedance_phase reports as a median of 0.603 for the days above the design Ap; ' +
      'here it is the whole distribution rather than its middle.',
  };
}

/** Predict · by cycle. The same growth curve, computed inside each cycle. */
function growthByCycle(rec, key, q, maxL) {
  const leads = [];
  for (let L = 30; L <= Math.min(maxL, 1826); L = Math.round(L * 1.5)) leads.push(L);
  const series = rec.cycles.map((c, i) => {
    const by = new Map();
    for (const d of rec.days) if (d.cycle === c.n && d[key] !== null) by.set(d.t, d[key]);
    const ys = leads.map(L => {
      const ch = [];
      for (const [t, v] of by) { const w = by.get(t + L); if (w !== undefined) ch.push(w - v); }
      ch.sort((a, b) => a - b);
      return ch.length > 30 ? quantile(ch, q) : null;
    });
    return { name: 'cycle ' + c.n, kind: 'line', x: leads.map(L => L / 365.25), y: ys, colour: INK.series[i] };
  });
  // How far apart the cycles actually are, at the longest lead all of them reach.
  // "They disagree" was the claim and it was never measured; this measures it.
  const name = key === 'f107' ? 'F10.7' : 'Ap';
  const unit = key === 'f107' ? ' sfu' : '';
  let shared = -1, spread = null;
  for (let i = leads.length - 1; i >= 0; i--) {
    const vs = series.map(s => s.y[i]).filter(v => v !== null && isFinite(v));
    if (vs.length === series.length) {
      shared = leads[i] / 365.25;
      spread = [Math.min(...vs), Math.max(...vs)];
      break;
    }
  }
  return {
    spec: {
      x: { label: 'lead  [years]', min: 0 },
      y: { label: 'change in ' + name + ' at the ' + (q * 100) + 'th percentile  [' + (unit.trim() || '-') + ']' },
      series,
    },
    note: 'The ' + (q * 100) + 'th percentile of the change in ' + name + ', one curve per cycle. ' +
      'Pairs are taken only WITHIN a cycle, so a lead cannot straddle a minimum and the curve is ' +
      'about the cycle rather than about the boundary. Cycle 25 stops early because this record holds ' +
      'six years of it, and a lead with fewer than thirty pairs is left undrawn rather than computed ' +
      'from a handful. ' +
      (spread
        ? 'At the longest lead all three reach, ' + shared.toFixed(2) + ' yr, they run from ' +
          sig(spread[0]) + unit + ' to ' + sig(spread[1]) + unit +
          (spread[0] === 0 ? '' : ', a ratio of ' + (spread[1] / spread[0]).toFixed(2)) +
          ' — and sw_uncertainty_growth pools exactly that disagreement into one number.'
        : 'No lead here is reached by all three cycles, so there is no like-for-like comparison to ' +
          'make: shorten the lead until the curves overlap.'),
  };
}

/** Forecast · issue age. How stale the newest outlook is on an average day. */
/**
 * Forecast · the score by the CALENDAR YEAR the outlook was issued in.
 *
 * The plan's §12 lists "rolling windows" as one of this tab's four views and it
 * was never built — the option string sat in the control list and nothing read
 * it, which declaring this panel in panels/ is what found.
 *
 * TWO CHOICES MAKE THIS COMPARABLE YEAR TO YEAR AND BOTH ARE FORCED BY THE DATA.
 *
 * The lead band is fixed at 1 to 14. The issues are not uniform: 2004 and 2007
 * carry NO row past lead 14 at all, 2002 and 2003 carry a handful, and 2011
 * onward carry a balanced 728 short against 624 long. Scoring every lead
 * together would make the short-lead years look better for no reason but their
 * composition, and the shape of the resulting line would be the archive's
 * history rather than the forecaster's. 1 to 14 is the band every year
 * populates.
 *
 * The year is the year the forecast was ISSUED, not the year it verified into,
 * because the question is how good the forecasts MADE in a year were. A
 * December issue reaches into January and is counted against December's year;
 * at fourteen days that is about 4 per cent of a year's rows landing one year
 * late, which moves nothing and is stated rather than corrected for.
 */
function byIssueYear(fc, byDay, tOf, persist, metric) {
  const LO = 1, HI = 14;
  //: A year with few pairs produces a skill that is arithmetic rather than
  //: evidence — 2010 has 25 of them and scores -11.8. Dropped, and counted in
  //: the note, on the same principle the Repeatability panel drops thin bins.
  const MIN = 200;
  const acc = new Map();
  const pc = new Map();
  for (const r of fc) {
    const L = +r.lead_days;
    if (!(L >= LO && L <= HI) || r.f107 === null) continue;
    const tt = tOf.get(r.target_date);
    const obs = tt === undefined ? undefined : byDay.get(tt);
    if (obs === undefined) continue;
    if (!pc.has(r.issue_date)) pc.set(r.issue_date, persist(r.issue_date));
    const p = pc.get(r.issue_date);
    if (p === null) continue;
    const y = +String(r.issue_date).slice(0, 4);
    if (!isFinite(y)) continue;
    if (!acc.has(y)) acc.set(y, { e2: 0, p2: 0, se: 0, n: 0 });
    const a = acc.get(y), e = +r.f107 - obs;
    a.e2 += e * e; a.p2 += (p - obs) * (p - obs); a.se += e; a.n++;
  }
  const years = [...acc.keys()].sort((a, b) => a - b);
  const thin = years.filter(y => acc.get(y).n < MIN);
  const score = (y) => {
    const a = acc.get(y);
    return metric === 'skill' ? 1 - (a.e2 / a.n) / (a.p2 / a.n)
      : metric === 'bias' ? a.se / a.n : Math.sqrt(a.e2 / a.n);
  };
  //: A dropped year is a HOLE, not an absence. Filtering the thin years out of
  //: the series entirely leaves the line joining 2008 straight to 2011, and
  //: that segment reads as two years of evidence rather than as the gap it is.
  //: A null breaks the line here the same way it does everywhere else.
  const span = [];
  for (let y = years[0]; y <= years[years.length - 1]; y++) span.push(y);
  const ys = span.map(y => (acc.has(y) && acc.get(y).n >= MIN ? score(y) : null));
  const kept = span.filter((y, i) => ys[i] !== null);
  return { years: span, ys, kept, keptYs: kept.map(score), thin, acc, LO, HI, MIN };
}

/**
 * Forecast · the line a score is read against, for whichever metric is on.
 *
 * Skill and bias are both differences, and a difference is unreadable without
 * the value that means "no difference" drawn on it. Skill had that line from
 * the start; bias did not, so a reader met a series running mostly below zero
 * and had to find zero on the axis to learn that it meant the outlook comes in
 * LOW — which is the one thing that view exists to say.
 *
 * RMS error gets no line. It is a magnitude, it cannot be negative, and zero is
 * perfection rather than a baseline: a line along the floor of the axis would
 * be decoration that the other two have earned and this one has not.
 *
 * One function rather than one expression per view, because the by-lead and
 * by-year views ask the same question of the same metric and two copies is two
 * places for them to start answering it differently.
 */
function scoreBaseline(metric) {
  if (metric === 'skill') {
    return [{ axis: 'y', at: 0, label: 'no better than persistence', colour: '#c2185b' }];
  }
  if (metric === 'bias') {
    return [{ axis: 'y', at: 0, label: 'unbiased — above is high, below is LOW', colour: '#c2185b' }];
  }
  return [];
}

function issueAge(idx) {
  const ds = idx.map(r => daysSince2000(r.issue_date)).filter(x => isFinite(x)).sort((a, b) => a - b);
  const gaps = [];
  for (let i = 1; i < ds.length; i++) { const g = ds[i] - ds[i - 1]; if (g > 0) gaps.push(g); }
  const hist = new Map();
  for (const g of gaps) hist.set(Math.min(g, 30), (hist.get(Math.min(g, 30)) || 0) + 1);
  const xs = [...hist.keys()].sort((a, b) => a - b);
  const mean = gaps.reduce((p, c) => p + c, 0) / gaps.length;
  gaps.sort((a, b) => a - b);
  return {
    spec: {
      x: { label: 'days between one issue and the next  [30 = 30 or more]', min: 0 },
      y: { label: 'number of gaps', min: 0 },
      series: [{ name: '', kind: 'bars', x: xs, y: xs.map(k => hist.get(k)) }],
    },
    note: 'From forecast_issues.csv, the index of ' + idx.length + ' issues \u2014 the one table in ' +
      'this bundle nothing else reads. The outlook is not published daily: the gap between issues is ' +
      'a median of ' + quantile(gaps, 0.5) + ' days and a mean of ' + mean.toFixed(2) +
      ', so on a typical day the newest outlook is already that old and its nominal lead understates ' +
      'the real one. A verification keyed on lead_days alone, as the rows here are, measures the ' +
      'forecast and not the staleness a user actually meets.',
  };
}

/** Design · the F10.7 window, which is the other half of what flows out.
 *
 * THE METHOD CHANGED HERE, AND THAT IS THE POINT OF THE PANEL.
 *
 * This used to compute `central expectation + 95th percentile growth` from the
 * record. That is sw_f107_design's relation — the row §20 DEPRECATED — and it
 * is `prf_design`'s method: the legacy tool freezes its last rotation forecast
 * and holds it flat, giving 158.33 sfu for its own 2027 window. Measured
 * against the two completed cycles scaled onto cycle 25's amplitude, the record
 * puts that 74 per cent high.
 *
 * What replaced it is four rows, and they are what this draws now: the
 * sustained and single-day levels on the hot side and on the cold side, each
 * built from a within-rotation spread conditioned on the level it applies at
 * rather than one spread used at every level. Together they ARE the design
 * window, which is what this panel's own label has always claimed to draw.
 */
function f107Window(extra, o, eng) {
  const rq = eng && eng[o.reqf];
  if (!rq || rq.si === undefined) {
    throw new Error((o.reqf || 'the F10.7 requirement') + ' did not answer: ' +
      ((rq && rq.refused) || 'the engine was not asked'));
  }
  const REQ = rq.si;
  // sw_central_expectation is NOT read here any more, and the panel no longer
  // declares it. The four design-window rows carry the centre inside
  // themselves; guarding on a value this figure does not draw would be a
  // declaration the panel cannot honour, and 2b says so — it caught exactly
  // that within a minute of this rewrite.
  const YR = 31557600;
  const w = extra.win;
  const xs = w.fl.x.map(v => v / YR);
  const hot = w.fs.y, hotLong = w.fl.y, coldLong = w.cl.y, cold = w.cs.y;
  const peak = Math.max(...hot);
  return {
    spec: {
      x: { label: 'mission length  [years]', min: xs[0], max: xs[xs.length - 1] },
      y: { label: 'F10.7 the window is designed to  [sfu]' },
      series: [
        { name: 'sw_f107_design_short — hot, single day', kind: 'line', x: xs, y: hot },
        { name: 'sw_f107_design_long — hot, sustained', kind: 'line', x: xs, y: hotLong,
          colour: '#2f6fa8' },
        { name: 'sw_f107_cold_long — cold, sustained', kind: 'line', x: xs, y: coldLong,
          colour: '#2e7d55' },
        { name: 'sw_f107_cold_short — cold, single day', kind: 'line', x: xs, y: cold,
          colour: '#8f43e0' },
      ],
      marks: [
        { axis: 'y', at: REQ, label: 'required \u2264 ' + REQ.toFixed(0) + '  (' + o.reqf + ')',
          colour: '#c2185b' },
      ],
    },
    note: 'The F10.7 half of what crosses to the system, as the four rows §20 built for it compute ' +
      'it: the sustained and single-day levels on the hot side and the cold side. The band between ' +
      'the outer two is the design window, and it is built from a within-rotation spread conditioned ' +
      'on the level it applies at.\n\n' +
      'This panel used to draw central expectation plus the 95th percentile growth, which is ' +
      'sw_f107_design\u2019s relation and prf_design\u2019s method \u2014 the legacy tool freezes ' +
      'its last rotation forecast flat and answers 158.33 sfu for its own window, and the record puts ' +
      'that 74 per cent high. That row is deprecated and this figure no longer draws it.\n\n' +
      'Every curve moves with mission length because the window mean rises and falls with where the ' +
      'window ends in the cycle \u2014 the cycle showing through a statistic that was never told ' +
      'about it. ' + o.reqf + '\u2019s ' + REQ.toFixed(0) + ' sfu is ' +
      (peak <= REQ ? 'met across the whole declared range; the worst single day the window reaches is '
        + peak.toFixed(1) + '.' : 'exceeded inside the declared range, at ' + peak.toFixed(1) + '.'),
  };
}

/** Climate · the 13-month smoother, the one view monthly_means.csv exists for. */
function smoothed(rows, key) {
  const col = key === 'f107' ? 'f107' : key === 'ap' ? 'ap' : 'ssn';
  const xs = [], raw = [], sm = [];
  for (const r of rows) {
    const t = daysSince2000(r.month) / 365.25 + 2000;
    xs.push(t);
    raw.push(num(r[col + '_mean']));
    sm.push(num(r[col + '_smooth']));
  }
  const missing = sm.filter(v => v === null).length;
  // How much the smoother actually removes, for the variable on screen. The
  // caption used to be the same sentence under all three.
  const both = xs.map((_, i) => [raw[i], sm[i]]).filter(([a, b]) => a !== null && b !== null);
  const amp = a => Math.max(...a) - Math.min(...a);
  const rawAmp = both.length ? amp(both.map(b => b[0])) : 0;
  const smAmp = both.length ? amp(both.map(b => b[1])) : 0;
  const resid = both.length
    ? Math.sqrt(both.reduce((p, [a, b]) => p + (a - b) * (a - b), 0) / both.length)
    : 0;
  const vname = key === 'f107' ? 'F10.7' : key === 'ap' ? 'Ap' : 'the sunspot number';
  const unit = key === 'f107' ? ' sfu' : '';
  return {
    spec: {
      x: { label: 'year' },
      y: { label: (key === 'f107' ? 'F10.7  [sfu]' : key === 'ap' ? 'Ap  [-]' : 'sunspot number  [-]') + ', monthly' },
      series: [
        { name: 'monthly mean', kind: 'line', x: xs, y: raw, width: 1.1, colour: '#c9a227' },
        { name: '13-month smoother', kind: 'line', x: xs, y: sm, width: 2.2, colour: '#1a1a1a' },
      ],
    },
    note: 'monthly_means.csv, which until the audit that added this view nothing in this repository ' +
      'read \u2014 no row and no other panel. The 13-month box smoother is the curve solar cycles are ' +
      'conventionally counted on. For ' + vname + ' it takes a monthly swing of ' + sig(rawAmp) + unit +
      ' down to ' + sig(smAmp) + unit + ', removing an rms of ' + sig(resid) + unit +
      ' \u2014 which is what is left of a month once the cycle is taken out, and is the part a ' +
      'design cannot plan around. ' +
      'It is undefined for ' + missing + ' of ' + rows.length + ' months at the two ends of ' +
      'the record, left empty rather than extrapolated: the line breaks there rather than being drawn ' +
      'across, because a smoother that runs to the edge of a record is claiming to know half a window ' +
      'it does not have.',
  };
}

/** Climate · Kp against ap, which is what the two conversion rows are about. */
function kpAgainstAp(rec) {
  const per = new Map();
  for (const d of rec.days) {
    if (d.ap === null || d.kp === null) continue;
    if (!per.has(d.kp)) per.set(d.kp, []);
    per.get(d.kp).push(d.ap);
  }
  const ks = [...per.keys()].sort((a, b) => a - b);
  const med = ks.map(k => { const v = per.get(k).sort((a, b) => a - b); return quantile(v, 0.5); });
  const p90 = ks.map(k => quantile(per.get(k).sort((a, b) => a - b), 0.9));
  const p10 = ks.map(k => quantile(per.get(k).sort((a, b) => a - b), 0.1));
  // The published three-hourly equivalent amplitude, all 28 points — the same
  // pairs sw_kp_from_ap carries in its parity grid.
  //
  // THIRTY-EIGHT, NOT TEN. Kp is reported in thirds, and the record's kp_max
  // takes values like 1.33 and 6.67. A lookup on whole Kp returns nothing for
  // two values in three, and because a null breaks a line rather than being
  // skipped, the series drew as no line at all — present in the legend and
  // absent from the picture. That is the failure this view was built to expose
  // in the DATA, arriving first in the code that draws it.
  const AP_AT_KP = [
    0, 2, 3, 4, 5, 6, 7, 9, 12, 15, 18, 22, 27, 32, 39, 48,
    56, 67, 80, 94, 111, 132, 154, 179, 207, 236, 300, 400,
  ];
  const tableAt = kp => {
    const i = Math.round(kp * 3);
    return i >= 0 && i < AP_AT_KP.length ? AP_AT_KP[i] : null;
  };
  return {
    spec: {
      x: { label: 'Kp reached that day  [worst three-hourly slot]', min: 0, max: 9 },
      y: { label: 'daily Ap  [-], log scale', log: true },
      series: [
        { name: 'median daily Ap', kind: 'line', x: ks, y: med },
        { name: '10th and 90th percentile', kind: 'line', x: ks, y: p10, colour: '#8a8880', width: 1 },
        { name: '', kind: 'line', x: ks, y: p90, colour: '#8a8880', width: 1 },
        { name: 'published ap at that Kp', kind: 'line', x: ks, y: ks.map(tableAt), colour: '#c2185b', dash: [5, 4] },
      ],
    },
    note: 'The published table converts a THREE-HOURLY Kp to a three-hourly ap; the record\u2019s daily ' +
      'Ap is the mean of eight such slots, and a day is labelled by its worst. So the two curves must ' +
      'diverge and the gap between them is the whole reason sw_kp_slot_bias exists \u2014 at Kp 7 the ' +
      'table says ' + tableAt(7) + ' and the median day says ' + med[ks.indexOf(7)] + '. Reading the dashed line as ' +
      'what a disturbed day looks like is the mistake this view is drawn to prevent, and it is also ' +
      'why sw_ap_design takes the table value as a design CEILING rather than as a typical day.',
  };
}

// ---------------------------------------------------------------------------
// the view

const state = { panel: 'design', opts: {} };

/**
 * Which version of a bundle the engine is actually serving.
 *
 * This line used to read `S.index.data_versions[0]`, a field the index has never
 * carried, so the header rendered `solar-weather@` with nothing after it — a
 * provenance claim with the provenance missing, on the one view whose whole
 * argument is that it reads the same bytes the engine reads. The versions live
 * on /v1/version as `name@date#hash`; the hash is dropped because the header is
 * an identification, not a checksum.
 */
function bundleVersion(name) {
  const all = (S.version && S.version.data) || [];
  const hit = all.find(d => d.startsWith(name + '@'));
  return hit ? hit.split('#')[0] : name + '@(the engine did not say)';
}

/**
 * A row named in a caption is a claim the reader can go and check.
 *
 * Until now they were plain text: a panel would say `sw_activity_band` and leave
 * the reader to find it. The tree's own index decides which tokens are rows, so a
 * name that is not a row — `daily_regime.csv`, `lead_days` — stays plain rather
 * than becoming a link that leads nowhere.
 *
 * Applied AFTER escaping, and the pattern needs an underscore, so it cannot
 * match the only markup the caption carries.
 */
function linkRows(html) {
  return html.replace(/\b[a-z][a-z0-9]*(?:_[a-z0-9]+)+\b/g, m =>
    (S.byId && S.byId.has(m)) ? '<a class="xref" data-goto="' + m + '">' + m + '</a>' : m);
}

/** The controls this branch reads. A control with no `when` always applies. */
function visibleControls(p, o) {
  return p.controls.filter(c => !c.when || c.when(o));
}

function optsFor(p) {
  if (!state.opts[p.id]) {
    state.opts[p.id] = Object.fromEntries(p.controls.map(c => [c.k, c.opts[0][0]]));
  }
  return state.opts[p.id];
}

/**
 * Which figures argue about a row.
 *
 * The map already existed, one way round: every panel declares the rows it is
 * an argument about. Inverting it is what lets a figure live on the row it
 * argues about instead of in a place of its own — which is where the port
 * plan's §18 put it: "a figure is not a row, and this repository already has
 * the better home for one".
 */
export function figuresForRow(id) {
  return PANELS.filter(p => (p.rows || []).includes(id));
}

/** One figure, drawn into whatever host is given. */
export function drawRowFigure(host, panelId) {
  const p = PANELS.find(x => x.id === panelId);
  if (!host || !p) return;
  const o = optsFor(p);
  host.innerHTML = panelBody(p, o);
  wirePanel(host, p, o, () => drawRowFigure(host, panelId));
}

/** The caption, the controls and the surface. Shared by both callers. */
function panelBody(p, o) {
  return '<p class="caption"><b>' + esc(p.asks) + '</b></p>' +
    '<p class="caption muted">' + esc(p.draws) + '</p>' +
    // Which rows this picture is an argument about. A panel that illustrates a
    // claim and does not say which claim leaves the reader to guess, and the
    // guess is the place the picture and the row quietly stop agreeing.
    (p.rows && p.rows.length
      ? '<p class="caption rows muted">the rows this argues about: ' +
        p.rows.map(r => '<a class="xref" data-goto="' + esc(r) + '">' + esc(r) + '</a>').join(' · ') +
        '</p>'
      : '<p class="caption rows muted">no row in this tree answers this tab, which is the panel’s ' +
        'whole point.</p>') +
    // ONLY THE CONTROLS THE CURRENT BRANCH ACTUALLY READS.
    //
    // Every panel's build() has branches, and a branch routinely ignores a
    // control that stayed on screen offering a choice. pattern's spike view
    // ignores the variable, the detrend window and the maximum lag; design's
    // F10.7 view ignores the G scale, which is geomagnetic and has no F10.7
    // meaning at all. Across the eight panels that was 35 of 162 combinations
    // reaching a picture already reachable another way.
    //
    // A knob that changes nothing is worse than no knob: the reader turns it,
    // sees no change, and stops trusting every other knob on the page. That is
    // the same defect the behaviour sweep had when it offered a decision worth
    // exactly zero.
    //
    // `when` is declared rather than inferred, because a branch that stops
    // reading a control should have to say so.
    (visibleControls(p, o).length
      ? '<div class="sweepctl">' + visibleControls(p, o).map(c =>
          '<span class="lbl">' + esc(c.label) + '</span><select class="ctl sw-opt" data-k="' + c.k + '">' +
          c.opts.map(([v, t]) => '<option value="' + esc(v) + '"' + (o[c.k] === v ? ' selected' : '') +
            '>' + esc(t) + '</option>').join('') + '</select>').join(' ') + '</div>'
      : '') +
    '<canvas class="plot sw-panel" width="980" height="420" title="point at the chart to read a value"></canvas>' +
    '<div class="sw-panel-note muted">reading the record…</div>';
}

/**
 * The controls, the sizing and the draw.
 *
 * `redraw` rather than a hard call to one view: a figure now lives wherever the
 * row it argues about is open, so the thing to redraw on a control change is
 * whichever host asked, not a view that may not be on screen.
 */
const LIVE = new Set();

function wirePanel(host, p, o, redraw) {
  host.querySelectorAll('.sw-opt').forEach(sel => {
    sel.onchange = () => { o[sel.dataset.k] = sel.value; redraw(); };
  });

  // The canvas takes the width it is given rather than a width chosen once. The
  // panels that matter most — a return curve against mission length, a skill
  // score across 26 leads — are long and thin, and a third of the page left
  // blank is a third of the resolution thrown away.
  fitCanvas(host);

  // Redraw on resize only while the host is still on the page. A figure opened
  // on one row and then navigated away from must not keep redrawing, and must
  // not keep the detached node alive by being referenced from a listener.
  LIVE.add(redraw);
  redraw._host = host;
  if (!wirePanel._bound) {
    wirePanel._bound = true;
    window.addEventListener('resize', debounce(() => {
      for (const fn of [...LIVE]) {
        if (fn._host && fn._host.isConnected) fn();
        else LIVE.delete(fn);
      }
    }, 180));
  }

  render(host, p, o);
}

/** The drawing surface, sized to the space there actually is. */
/**
 * The drawing surface, sized to the space there actually is — and no wider
 * than the picture can use.
 *
 * The canvas took the full host width at a fixed 0.40 of it, which on a wide
 * screen is nearly three to one. A gentle rise across three units of width and
 * one of height is a slope of about 18 degrees, and a reader compares slopes by
 * their ANGLE: flattened like that, the difference between two curves stops
 * being visible before it stops being real. Banking the principal slope toward
 * 45 degrees is the classical answer and needs the data; capping the aspect is
 * the part that can be done without it, and it is most of the benefit.
 *
 * 2.2 : 1 rather than 2.8 : 1, with the width capped so a very wide window adds
 * height instead of stretching the picture further. The panels that genuinely
 * want length — a skill score across 26 leads — still get it; what they stop
 * getting is a third of the angle thrown away.
 */
function fitCanvas(host) {
  const cv = $('.sw-panel', host);
  if (!cv) return;
  const w = Math.round(host.getBoundingClientRect().width);
  if (w > 320) cv.width = Math.min(1180, w);
  cv.height = Math.round(Math.max(340, Math.min(560, cv.width / 2.2)));
}

function debounce(fn, ms) {
  let h = null;
  return (...a) => { clearTimeout(h); h = setTimeout(() => fn(...a), ms); };
}

async function render(host, p, o) {
  const note = $('.sw-panel-note', host);
  try {
    // THE RECORD AND THE ENGINE ARE TWO DIFFERENT SOURCES AND A PANEL MAY NEED
    // BOTH. `rec` and `data()` are the reference bundle — what was observed.
    // `eng` is what this tree COMPUTES, fetched for exactly the rows the panel
    // declares in `engine`. Before this a panel needing a computed number
    // carried a copy of it, and three of those copies were stale; see §21.
    const [rec, extra, eng] = await Promise.all([
      solarRecord(),
      p.data ? p.data() : null,
      engineValues(p.engine || []),
    ]);
    const out = p.build(rec, o, extra, eng);
    const cv = $('.sw-panel', host);
    drawChart(cv, out.spec);
    attachHover(cv);
    note.innerHTML = linkRows(esc(out.note).replace(/\n\n/g, '<br><br>'));
    // SAY SO WHEN IT WORKED, so that a checker can tell a redraw from a
    // collapse. A failed render blanks the canvas, and a blank canvas has a
    // different signature from a drawn one — so "did the pixels change" is
    // satisfied by the panel BREAKING, and panel_check's second check passed a
    // deliberately broken panel until this existed. The state goes on the mount
    // rather than in the note, because the note is prose for a reader and this
    // is a fact for a machine.
    delete cv.dataset.failed;
  } catch (e) {
    const cv = $('.sw-panel', host);
    if (cv) cv.dataset.failed = String(e && e.message ? e.message : e);
    note.textContent = 'the record could not be read: ' + e;
  }
}
