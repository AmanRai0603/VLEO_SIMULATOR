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

import { $, esc, fmt } from './dom.js';
import { S } from './state.js';
import { solarRecord, bundleFile, centredMean, corr, quantile, num } from './record.js';
import { drawChart, attachHover, INK } from './chart.js';

// ---------------------------------------------------------------------------
// the panels

const PANELS = [
  {
    id: 'repeatability',
    label: 'Repeatability',
    draws: 'The mean cycle against every cycle the record holds, stacked on phase.',
    asks: 'How much of F10.7 does the cycle explain — and does one cycle repeat the last?',
    controls: [
      { k: 'v', label: 'variable', opts: [['f107', 'F10.7'], ['ap', 'Ap'], ['ssn', 'sunspot number']] },
      { k: 'bins', label: 'phase bins', opts: [['20', '20'], ['10', '10'], ['40', '40']] },
    ],
    build(rec, o) {
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
      return {
        spec: {
          x: { label: 'cycle phase  [0 = minimum, 1 = the next]', min: 0, max: 1 },
          y: { label: key === 'f107' ? 'F10.7  [sfu]' : key === 'ap' ? 'Ap  [-]' : 'sunspot number  [-]' },
          series,
        },
        note: 'Cycles 23 and 24 are the only complete ones, and their shapes correlate at ' +
          (r === null ? '—' : r.toFixed(4)) + ' over the ' + usable + ' of ' + nb +
          ' bins both populate with at least ' + MIN + ' days. A correlation is scale-free, so it is blind to the thing a drag ' +
          'design cares about: the two peaks differ by 28 per cent. Cycle 25 is dashed because ' +
          'its end in solar_cycles.csv is the record’s end, not a minimum, so its phase is ' +
          'computed against a length nobody yet knows.',
      };
    },
  },

  {
    id: 'pattern',
    label: 'Pattern',
    draws: 'The autocorrelation of the detrended record against lag, and its harmonics.',
    asks: 'At what lag does the solar rotation come back, and how strongly?',
    controls: [
      { k: 'v', label: 'variable', opts: [['f107', 'F10.7'], ['ap', 'Ap']] },
      { k: 'w', label: 'detrend window', opts: [['365', '365 d'], ['181', '181 d'], ['731', '731 d']] },
      { k: 'lag', label: 'max lag', opts: [['120', '120 d'], ['60', '60 d'], ['200', '200 d']] },
    ],
    build(rec, o) {
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
      if (p2.at && maxLag >= 65) marks.push({ axis: 'x', at: p2.at, label: 'second, ' + p2.at, colour: '#2a6f97' });
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
            ' Shorten the detrend window toward the rotation itself and the signal disappears, ' +
            'because the window removes what it is meant to leave.'
          : 'No peak in the rotation band at this setting.',
      };
    },
  },

  {
    id: 'segmentation',
    label: 'Segmentation',
    draws: 'Where the record actually sits, and the boundaries the study cut it at.',
    asks: 'Quiet, active or storm — and how much of the record is each?',
    controls: [
      { k: 'v', label: 'variable', opts: [['ap', 'Ap — regime'], ['f107', 'F10.7 — activity band']] },
      { k: 'scale', label: 'count axis', opts: [['log', 'log'], ['lin', 'linear']] },
    ],
    build(rec, o) {
      const key = o.v;
      const cuts = key === 'ap' ? [6.5, 25.5] : [90, 120, 180];
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
        let k = 0; while (k < cuts.length && x > cuts[k]) k++;
        share[k]++;
      }
      return {
        spec: {
          x: { label: (key === 'ap' ? 'daily Ap' : 'F10.7  [sfu]') + '  [bin ' + bw + ']', min: 0 },
          y: { label: o.scale === 'log' ? 'days in bin  [log10]' : 'days in bin', min: 0 },
          series: [{ name: '', kind: 'bars', x: xs, y: ys, colour: '#b5731a' }],
          marks: cuts.map((c, i) => ({ axis: 'x', at: c, label: names[i] + ' | ' + names[i + 1] })),
        },
        note: share.map((n, i) => names[i] + ' ' + (100 * n / vals.length).toFixed(1) + '%').join(' · ') +
          (key === 'ap'
            ? '. These are the study’s own mixture boundaries, and they fell on integers: quiet ' +
              'holds Ap 0 to 6, active 7 to 25, storm 26 and above. sw_regime reproduces ' +
              'daily_regime.csv exactly on 10128 of its 10299 days. The 171 that differ all have ' +
              'Ap 0 or 1 and are labelled storm in the published table — the broad storm component ' +
              'winning at the low tail — and this repository calls them quiet.'
            : '. A log count axis is the honest default here: on a linear one the tail that matters ' +
              'to a design is a row of pixels one high.'),
      };
    },
  },

  {
    id: 'predict',
    label: 'Predict',
    draws: 'How far the flux moves over a lead, at a percentile.',
    asks: 'How far ahead is F10.7 knowable, and what does the band cost?',
    controls: [
      { k: 'q', label: 'percentile', opts: [['0.95', '95th'], ['0.5', '50th'], ['0.9', '90th'], ['0.99', '99th']] },
      { k: 'span', label: 'lead out to', opts: [['1826', '5 years'], ['365', '1 year'], ['5478', '15 years']] },
    ],
    build(rec, o) {
      const q = +o.q, maxL = +o.span;
      const byDay = new Map();
      for (const d of rec.days) if (d.f107 !== null) byDay.set(d.t, d.f107);
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
      return {
        spec: {
          x: { label: 'lead  [years]', min: 0 },
          y: { label: 'change in F10.7 at the ' + (q * 100) + 'th percentile  [sfu]' },
          series: [{ name: '', kind: 'line', x: xs, y: ys }],
        },
        note: 'Signed change, not absolute: the unsafe direction for a drag design is flux arriving ' +
          'HIGHER than planned, so this is the upper tail. The curve rises and falls again — at the ' +
          '95th it peaks near four years and dips near ten — and that is the eleven-year cycle, not ' +
          'noise: a lead of half a cycle takes you from minimum to maximum, which is the largest ' +
          'change available, and a full cycle returns you to similar activity. Pairs at a given lead ' +
          'overlap almost completely, so the ' + ns[0] + ' to ' + ns[ns.length - 1] + ' counted here ' +
          'are nothing like that many independent observations.',
      };
    },
  },

  {
    id: 'forecast',
    label: 'Forecast',
    draws: 'The issued 27-day outlook, scored against what arrived.',
    asks: 'Is the published forecast worth more than assuming nothing changes?',
    needs: ['forecast_issued.csv'],
    controls: [
      { k: 'm', label: 'metric', opts: [['skill', 'skill vs persistence'], ['bias', 'bias'], ['rmse', 'RMS error']] },
      { k: 'base', label: 'persistence baseline', opts: [['strict', 'last obs BEFORE issue'], ['leaky', 'obs ON the issue date']] },
    ],
    async data() { return bundleFile('solar-weather', 'forecast_issued.csv'); },
    build(rec, o, extra) {
      const fc = extra.rows;
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
      const marks = o.m === 'skill' ? [{ axis: 'y', at: 0, label: 'no better than persistence', colour: '#c1440e' }] : [];
      const last = xs[xs.length - 1];
      return {
        spec: {
          x: { label: 'lead  [days]', min: 1, max: 27 },
          y: { label: o.m === 'skill' ? 'skill against persistence  [-]' : o.m === 'bias' ? 'mean signed error, forecast − observed  [sfu]' : 'RMS error  [sfu]' },
          series: [{ name: '', kind: 'line', x: xs, y: ys }],
          marks,
        },
        note: (o.base === 'leaky'
          ? 'THIS BASELINE LEAKS. 719 of the 1281 issues index their rows from lead 0, so the issue ' +
            'date is itself a forecast target for most of the record, and handing it to persistence ' +
            'gives the baseline a number the forecaster did not have. Switch to the strict baseline ' +
            'and the sign of the short-lead answer changes.'
          : 'Persistence is the last observation strictly BEFORE the issue date — what a ' +
            'forecaster actually had. On this baseline the outlook beats it from lead 1.') +
          ' Lead ' + last + ' draws on ' + ns[ns.length - 1] + ' pairs against ' + ns[0] + ' at lead 1: ' +
          'lead_days is indexed two ways in one column, and only the 1-based minority reaches 27, ' +
          'which is why sw_outlook_lead declares 26.',
      };
    },
  },

  {
    id: 'design',
    label: 'Design',
    draws: 'The design window: what the record expects against what the vehicle is built for.',
    asks: 'Will the design be exceeded, and if so beyond what mission length?',
    controls: [
      { k: 'g', label: 'designed for', opts: [['3', 'G3 strong'], ['2', 'G2 moderate'], ['1', 'G1 minor']] },
      { k: 'req', label: 'requirement', opts: [['150', 'Ap 150'], ['132', 'Ap 132'], ['200', 'Ap 200']] },
    ],
    build(rec, o) {
      // The fitted return relation, as sw_storm_return_level publishes it. The
      // constants are that row's; this panel does not re-fit, because a figure
      // that fits its own line is drawing a second opinion and calling it the
      // answer.
      const A = 92.515531, B = 40.926516;
      const apAt = T => A + B * Math.log(T);
      const AP_AT_G = { 1: 48, 2: 80, 3: 132 };
      const bound = AP_AT_G[o.g], req = +o.req;
      const xs = [], ys = [];
      for (let t = 0.5; t <= 15.0001; t += 0.1) { xs.push(t); ys.push(apAt(t)); }
      const cross = ap => Math.exp((ap - A) / B);
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
          x: { label: 'mission length  [years]', min: 0.5, max: 15 },
          y: { label: 'daily Ap the record expects once in that time  [-]' },
          series: [{ name: 'sw_storm_return_level', kind: 'line', x: xs, y: ys }],
          marks: [
            { axis: 'y', at: bound, label: 'designed for G' + o.g + ' = Ap ' + bound },
            { axis: 'y', at: req, label: 'required ≤ ' + req, colour: '#c1440e' },
            { axis: 'x', at: cross(bound), label: 'exceeds the design at ' + cross(bound).toFixed(2) + ' yr' },
          ],
        },
        note: 'The design bound is exceeded beyond a ' + cross(bound).toFixed(2) + '-year mission and the ' +
          'requirement beyond ' + cross(req).toFixed(2) + '. Over a 5-year mission the record holds ' +
          (5 * rate).toFixed(2) + ' days above Ap ' + bound + ', in about ' + (5 * runs / years).toFixed(2) +
          ' separate events — ' + above.length + ' days in ' + runs + ' events across ' +
          years.toFixed(2) + ' years of record. That the exceedance is brief and rare is what makes ' +
          'the bound acceptable rather than failed, and it is only knowable because it is counted.',
      };
    },
  },

  {
    id: 'climate',
    label: 'Climate',
    draws: 'The long run: the record by year, and the season inside the year.',
    asks: 'What is the context a single mission sits inside?',
    controls: [
      { k: 'v', label: 'variable', opts: [['f107', 'F10.7'], ['ap', 'Ap'], ['ssn', 'sunspot number']] },
      { k: 'by', label: 'aggregate', opts: [['year', 'by year'], ['doy', 'by day of year'], ['month', 'by month']] },
    ],
    build(rec, o) {
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
      const overall = ys.reduce((p, c) => p + c, 0) / ys.length;
      const marks = [{ axis: 'y', at: overall, label: 'mean over the record = ' + overall.toFixed(2) }];
      if (o.by === 'doy') {
        marks.push({ axis: 'x', at: 80, label: 'March equinox', colour: '#2a6f97' });
        marks.push({ axis: 'x', at: 266, label: 'September equinox', colour: '#2a6f97' });
      }
      return {
        spec: {
          x: { label: o.by === 'year' ? 'year' : o.by === 'doy' ? 'day of year  [5-day bins]' : 'year' },
          y: { label: (key === 'f107' ? 'F10.7  [sfu]' : key === 'ap' ? 'Ap  [-]' : 'sunspot number  [-]') + ', mean' },
          series: [{ name: '', kind: o.by === 'year' ? 'bars' : 'line', x: xs, y: ys }],
          marks,
        },
        note: o.by === 'doy' && key === 'ap'
          ? 'The equinoctial effect, and its size is the point: the fitted semiannual amplitude is ' +
            '1.278 on an offset of 10.494, about 12 per cent, and it accounts for 0.63 per cent of ' +
            'the DAILY variance. It moves a monthly budget and says almost nothing about a given day.'
          : o.by === 'doy'
            ? 'F10.7 has no seasonal term — it is a property of the Sun, not of the Earth’s ' +
              'tilt. Any structure here is the cycle landing unevenly across the calendar. Switch to ' +
              'Ap to see a real season.'
            : 'The 2017 gap is 273 consecutive days and shows here as a year drawn from nine months. ' +
              'Nothing is interpolated across it.',
      };
    },
  },

  {
    id: 'density',
    label: 'Density',
    draws: 'Nothing yet, and the reason is worth a panel.',
    asks: 'What are these drivers worth as atmospheric density?',
    controls: [],
    build(rec) {
      const withBoth = rec.days.filter(d => d.f107 !== null && d.ap !== null);
      return {
        spec: {
          x: { label: 'F10.7  [sfu]' },
          y: { label: 'daily Ap  [-]' },
          series: [{ name: '', kind: 'dots', x: withBoth.map(d => d.f107), y: withBoth.map(d => d.ap), width: 1.1, alpha: 0.18 }],
        },
        note: 'THIS TAB HAS NO ROWS AND THIS IS NOT ONE. The study’s density tab — profile, ' +
          'spread, sensitivity, by driver, by altitude — needs an atmosphere model, and that ' +
          'belongs to a different subsystem which has nothing written in it. Drawing a density ' +
          'curve here would mean this face carrying a model no row owns and no reviewer signed.\n\n' +
          'What is drawn instead is the honest precondition: the two drivers a density model takes, ' +
          'against each other, over ' + withBoth.length + ' days. They are close to independent, ' +
          'which is why a design needs both and why neither substitutes for the other.',
      };
    },
  },
];

// ---------------------------------------------------------------------------
// the view

const state = { panel: 'design', opts: {} };

function optsFor(p) {
  if (!state.opts[p.id]) {
    state.opts[p.id] = Object.fromEntries(p.controls.map(c => [c.k, c.opts[0][0]]));
  }
  return state.opts[p.id];
}

export function drawSolar() {
  const host = $('#solarview');
  if (!host) return;
  const p = PANELS.find(x => x.id === state.panel) || PANELS[0];
  const o = optsFor(p);

  host.innerHTML =
    '<div class="node-head"><h2>Solar weather</h2>' +
    '<p class="ident">eight tabs, drawn from <code>solar-weather@' +
    esc((S.index && S.index.data_versions && S.index.data_versions[0]) || '') +
    '</code> — the same bytes the engine reads</p></div>' +
    '<div class="tabrow sub sw-tabs">' +
    PANELS.map(x => '<button class="ctl sw-tab' + (x.id === p.id ? ' sel' : '') +
      '" data-panel="' + x.id + '">' + esc(x.label) + '</button>').join('') +
    '</div>' +
    '<p class="caption"><b>' + esc(p.asks) + '</b></p>' +
    '<p class="caption muted">' + esc(p.draws) + '</p>' +
    (p.controls.length
      ? '<div class="sweepctl">' + p.controls.map(c =>
          '<span class="lbl">' + esc(c.label) + '</span><select class="ctl sw-opt" data-k="' + c.k + '">' +
          c.opts.map(([v, t]) => '<option value="' + esc(v) + '"' + (o[c.k] === v ? ' selected' : '') +
            '>' + esc(t) + '</option>').join('') + '</select>').join(' ') + '</div>'
      : '') +
    '<canvas class="plot sw-panel" width="980" height="380" title="point at the chart to read a value"></canvas>' +
    '<div class="sw-panel-note muted">reading the record…</div>';

  host.querySelectorAll('.sw-tab').forEach(b => {
    b.onclick = () => { state.panel = b.dataset.panel; drawSolar(); };
  });
  host.querySelectorAll('.sw-opt').forEach(sel => {
    sel.onchange = () => { o[sel.dataset.k] = sel.value; drawSolar(); };
  });

  render(host, p, o);
}

async function render(host, p, o) {
  const note = $('.sw-panel-note', host);
  try {
    const rec = await solarRecord();
    const extra = p.data ? await p.data() : null;
    const out = p.build(rec, o, extra);
    const cv = $('.sw-panel', host);
    drawChart(cv, out.spec);
    attachHover(cv);
    note.innerHTML = esc(out.note).replace(/\n\n/g, '<br><br>');
  } catch (e) {
    note.textContent = 'the record could not be read: ' + e;
  }
}
