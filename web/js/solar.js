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
import { solarRecord, bundleFile, parityFile, engineValues, engineSweep, engineAt,
  engineProbe, probeSweep,
  engineLevers, centredMean, corr, quantile, num, daysSince2000 } from './record.js';
import { drawChart, attachHover, tableFor, tableTsv, viewSpec, viewIsOn,
  watchScheme, sizeCanvas, cssSize, INK } from './chart.js';

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

/**
 * The autocorrelation of a series at every lag to `maxLag`, and the pair count
 * behind each.
 *
 * Written out rather than calling corr() on two slices, and the reason is the
 * COUNT rather than the speed. Bartlett's band needs n at each lag, and corr()
 * returns a correlation and drops how many pairs it used — so the band would
 * have had to guess at the very number that sets its width.
 *
 * Pairing is res[i] against res[i+L], over the i where both are present, which
 * is what corr(res.slice(0, n-L), res.slice(L)) did. A null is skipped rather
 * than treated as zero: the record is missing 273 days, and counting them as no
 * departure from trend would pull every correlation toward the mean.
 */
function laggedCorr(res, maxLag) {
  const lags = [], r = [], n = [];
  for (let L = 1; L <= maxLag; L++) {
    let sa = 0, sb = 0, k = 0;
    for (let i = 0; i + L < res.length; i++) {
      const a = res[i], b = res[i + L];
      if (a === null || b === null) continue;
      sa += a; sb += b; k++;
    }
    lags.push(L);
    if (k < 3) { r.push(null); n.push(k); continue; }
    const ma = sa / k, mb = sb / k;
    let sab = 0, saa = 0, sbb = 0;
    for (let i = 0; i + L < res.length; i++) {
      const a = res[i], b = res[i + L];
      if (a === null || b === null) continue;
      const da = a - ma, db = b - mb;
      sab += da * db; saa += da * da; sbb += db * db;
    }
    r.push(saa && sbb ? sab / Math.sqrt(saa * sbb) : null);
    n.push(k);
  }
  return { lags, r, n };
}

/**
 * The five closures, as the panel beneath needs to name them.
 *
 * `q` is the ACHIEVED QUANTITY's row, which is not the closure row: since §20 an
 * achieved row publishes the signed margin and reads the quantity as an input, so
 * a picture of "what the record gives" has to sweep the input rather than the row
 * that is named for it. Written out once here because getting that wrong would
 * draw a margin on an axis labelled sfu and look entirely plausible.
 */
const QUANTITY_OF = {
  '01': 'sw_f107_design_long',
  '02': 'sw_f107_design_short',
  '03': 'sw_storm_return_level',
  '04': 'sw_ap_design_long',
  '05': 'sw_ap_design_short',
};

const PAIR_LABEL = {
  '01': { q: 'sustained F10.7 to design to', u: 'sfu', text:
    'The F10.7 level the mission must sustain, against the level the record gives it. Both sides ' +
    'are built the same way — a central expectation with a band on it — so this closure is closer ' +
    'to a consistency check than to a test, and it passing means less than the Ap pairs passing.' },
  '02': { q: 'single-day F10.7 to design to', u: 'sfu', text:
    'The worst single day of F10.7, against the requirement for one. The daily side rides on the ' +
    'sustained side with a level-conditioned within-rotation spread on top, which is the ' +
    'construction §20 replaced a fixed-window percentile with.' },
  '03': { q: 'daily Ap the record expects once in that time', u: '-', text:
    'The storm that recurs once in a mission of this length, against the G4 threshold the vehicle ' +
    'must survive. THIS IS THE ONE REAL TEST of the five: the requirement is a published threshold ' +
    'on the G scale and the achieved value is a tail quantile of a 28-year record, so nothing ties ' +
    'them and there is no reason for them to agree. Its top end rests on two observations in 28.2 ' +
    'years, and a margin computed from it carries none of that.' },
  '04': { q: 'sustained Ap to design to', u: '-', text:
    'The Ap level the mission must operate through, against the level the record gives it. The ' +
    'requirement is a published G-scale threshold and the achieved value comes from the record, so ' +
    'like the survival pair the two are independent.' },
  '05': { q: 'single-day Ap to design to', u: '-', text:
    'The worst single day of Ap, against the requirement for one. The daily spread is measured AT ' +
    'the sustained level it applies to, which is why this number and the return level in the ' +
    'survival pair disagree by a factor of 1.75 without either being wrong.' },
};

// One sweep per pair, kept for as long as the page is open. The five pairs do not
// change while a reader clicks between them, and re-asking the engine for a sweep
// it has already answered is four questions nobody is looking at.
const CLOSURE_CACHE = {};


/**
 * The five scenarios the solar subsystem publishes, cold to hot.
 *
 * The same ladder the `drivers` panel draws along, and for the same reason: in
 * this order every solar quantity is monotone, so a picture is checked by whether
 * it rises. `l3_solar_interface`'s own answer is f107_hotmean — the one cell that
 * is not a published member — which is why that one is reached by the node id.
 */
//
// A STEP OF THE RAMP, NOT A COLOUR. This list is module-level and the scheme is
// not: a hex frozen here would still be the light one on a dark page. `colour`
// is a getter so the ladder reads whichever ramp is in force at the moment the
// picture is drawn.
const SCEN = [
  { k: 'coldday',  shown: 'quietest day',   ci: 0 },
  { k: 'coldmean', shown: 'cold sustained', ci: 1 },
  { k: 'nominal',  shown: 'nominal',        ci: 2 },
  { k: 'hotmean',  shown: 'hot sustained',  ci: 3 },
  { k: 'hotday',   shown: 'worst day',      ci: 4 },
].map(sc => ({ ...sc, get colour() { return INK.ramp5[this.ci]; } }));

/** One scenario's four drivers, in SI, out of one run of the crossing. */
function driversOf(eng, k) {
  const at = id => {
    const v = eng['l3_solar_interface.' + id];
    return v && isFinite(v.si) ? v.si : null;
  };
  const own = eng.l3_solar_interface;
  return {
    f107: k === 'hotmean' ? (own && isFinite(own.si) ? own.si : null) : at('f107_' + k),
    f107a: at('f107bar_' + k),
    kpMean: at('kp_mean_' + k),
    kpPeak: at('kp_peak_' + k),
  };
}

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
      series.push({ name: 'mean of the complete cycles', kind: 'line', x: xs, y: meanCycle, colour: INK.text, width: 2.4 });
      rec.cycles.forEach((c, i) => {
        series.push({
          name: 'cycle ' + c.n + (c.n === 25 ? ' (incomplete)' : ''),
          kind: 'line', x: xs, y: xs.map((_, k) => mean(per.get(c.n)[k])),
          colour: INK.series[i % INK.series.length], width: 1.4,
          dash: c.n === 25 ? [4, 3] : null,
          // CONTEXT. §34.1 named this frame: the mean cycle and the individual
          // ones carried the same contrast, so five curves arrived at once and
          // a reader had to be told in prose which one the panel is about. The
          // mean is the row the panel is named for; the cycles are what makes
          // it legible AS a mean, and they stay readable enough that "cycle 23
          // peaks higher than 24" is still something to look at rather than
          // read.
          context: true,
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
      // How far apart the two complete cycles run, on the rise and after it.
      // Measured, because "they converge" is the kind of clause that is written
      // once from one picture and then carried through every variable.
      const gapIn = (lo, hi) => {
        const d = [];
        for (let i = 0; i < xs.length; i++) {
          if (xs[i] >= lo && xs[i] < hi && a[i] !== null && b[i] !== null) d.push(a[i] - b[i]);
        }
        return d.length ? d.reduce((q, c) => q + c, 0) / d.length : null;
      };
      // The peak disagreement is measured for the variable on screen. The figure
      // used to be 28 per cent under all three, which is F10.7's.
      const pk = n => Math.max(...xs.map((_, i) => mean(per.get(n)[i])).filter(v => v !== null));
      const p23 = pk(23), p24 = pk(24);
      const vname = key === 'f107' ? 'F10.7' : key === 'ap' ? 'Ap' : 'the sunspot number';
      return {
        // THE CORRELATION IS THE ANSWER AND THE RATIO IS WHY IT IS NOT ENOUGH.
        // Both, in one line, because quoting the correlation alone beside a
        // question about repeatability is the reading this panel exists to stop.
        answer: r === null
          ? { value: '\u2014', of: 'no phase bin both complete cycles populate' }
          : { value: r.toFixed(3),
              of: 'how well cycle 24 repeats 23\u2019s ' + vname + ' SHAPE \u2014 while its peak is '
                + (p24 / p23).toFixed(2) + ' of 23\u2019s' },
        spec: {
          // THE FINDING IS NOT THE ANSWER. The answer above the chart is the
          // correlation, which is a number about the two cycles; this is a
          // relation between two curves a reader can check by looking at which
          // one is on top and over how much of the axis.
          finding: 'cycle 23 runs above cycle 24 in ' +
            a.filter((v, i) => v !== null && b[i] !== null && v > b[i]).length +
            ' of the ' + usable + ' phase bins both fill, by ' + sig(gapIn(0, 0.6)) +
            ' on the rise and ' + sig(gapIn(0.6, 1)) + ' after phase 0.6',
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
    // THE DETREND WINDOW AND THE MAXIMUM LAG WERE CONTROLS AND ARE NOW THE
    // PICTURE. Nine settings of two knobs drew nine curves that differ in
    // exactly the way a reader wants to compare, and a control is the one
    // presentation that makes comparison impossible: it shows one at a time.
    // The three windows are three lines, 365 heavy because that is the window
    // sw_recurrence_lag and sw_recurrence_strength were measured under, and the
    // axis simply runs to the longest lag that was on offer. 19 combinations
    // become 3 views, and every one of them says more than the nine did.
    controls: [
      { k: 'view', label: 'view', opts: [['acf', 'recurrence and decay'], ['spikes', 'spikes: size and timing']] },
      // The spike view is spikes(rec) and reads nothing below it.
      { k: 'v', label: 'variable', when: o => o.view === 'acf',
        opts: [['f107', 'F10.7'], ['ap', 'Ap']] },
    ],
    build(rec, o) {
      if (o.view === 'spikes') return spikes(rec);
      const key = o.v;
      const MAXLAG = 200;
      const v = rec.days.map(d => d[key]);
      // 0.6, the same completeness rule sw_recurrence_lag and
      // sw_recurrence_strength were measured under. A panel that illustrates a
      // row and quotes a different number for it is worse than no panel.
      const acfFor = (W) => {
        const trend = centredMean(v, W, 0.6);
        const res = v.map((x, i) => (x === null || trend[i] === null ? null : x - trend[i]));
        return laggedCorr(res, MAXLAG);
      };
      // 365 FIRST, so it takes the first hue — the one that means "the record"
      // everywhere in this tool — and so the peaks below are read off the curve
      // the rows were measured on.
      const A = acfFor(365), Ashort = acfFor(181), Along = acfFor(731);
      const xs = A.lags;

      // THE BAND, AND WHY IT IS BARTLETT'S AND NOT 2/sqrt(n).
      //
      // The naive band tests each correlation against the hypothesis that the
      // whole series is white noise. This series is emphatically not white — it
      // decays from 0.94 at lag 1 — so that test is passed by everything and
      // says nothing. The question a reader actually has at lag 26 is whether
      // that bump is more than the decay below it would already produce, and
      // Bartlett's large-lag standard error is the one that asks it: the
      // variance of r_k grows with the correlations at every shorter lag.
      //
      //   se(r_k) = sqrt( (1 + 2 * sum_{j<k} r_j^2) / n )
      //
      // So the band WIDENS with lag, which is the honest shape: a correlation
      // far out has to be bigger to mean the same thing. Computed on the 365
      // curve, because that is the one it is drawn against.
      const band = [];
      let acc = 0, outside = 0;
      for (let k = 0; k < xs.length; k++) {
        const n = A.n[k];
        const b = n > 2 ? 1.96 * Math.sqrt((1 + 2 * acc) / n) : null;
        band.push(b);
        const r = A.r[k];
        if (r !== null && b !== null && Math.abs(r) > b) outside++;
        if (r !== null) acc += r * r;
      }

      // The first bump's peak and its harmonics, which are what say what the
      // period is. Measured on the 365 curve for the same reason the band is.
      const peakIn = (lo, hi) => {
        let best = null, at = null;
        for (let L = lo; L <= hi && L <= MAXLAG; L++) {
          const r = A.r[L - 1];
          if (r !== null && (best === null || r > best)) { best = r; at = L; }
        }
        return { at, r: best };
      };
      const p1 = peakIn(18, 36), p2 = peakIn(45, 65), p3 = peakIn(72, 95);
      // THREE NOTES WHERE THERE WERE THREE FULL-HEIGHT RULES, and each now says
      // what its peak IMPLIES rather than only where it is. The harmonics are
      // the whole argument — the first bump rides on the decay from lag 1, which
      // pulls its apparent peak toward zero, and the far harmonics are clear of
      // it — and the division that shows it was a sentence underneath. Three
      // dashed lines down a frame of oscillating curves also cost more ink than
      // any of them was worth.
      //
      // THE THIRD PEAK WAS COMPUTED, QUOTED IN THE PROSE AND NEVER DRAWN until
      // §26; it is the one that settles the period, so leaving it to the prose
      // asked a reader to take the most important of the three on trust.
      const marks = [];
      const notes = [];
      if (p1.at) notes.push({ x: p1.at, y: p1.r, text: 'first peak, lag ' + p1.at });
      if (p2.at) {
        notes.push({ x: p2.at, y: p2.r,
          text: 'second, ' + p2.at + ' → ' + (p2.at / 2).toFixed(1) + ' d per cycle',
          colour: INK.series[1] });
      }
      if (p3.at) {
        notes.push({ x: p3.at, y: p3.r,
          text: 'third, ' + p3.at + ' → ' + (p3.at / 3).toFixed(1) + ' d per cycle',
          colour: INK.series[2] });
      }

      // The claim panels/pattern.toml makes about this picture, counted rather
      // than asserted: a longer detrend window calls less of the record trend,
      // so more low-frequency signal survives and every correlation is higher.
      let over = 0, pairs = 0;
      for (let k = 0; k < xs.length; k++) {
        const lo = A.r[k], hi = Along.r[k];
        if (lo === null || hi === null || !isFinite(lo) || !isFinite(hi)) continue;
        pairs++; if (hi > lo) over++;
      }
      return {
        spec: {
          finding: 'the 731-day curve sits above the 365-day one at ' + over + ' of the ' +
            pairs + ' lags, and the 365 curve is outside the band at ' + outside + ' of them',
          x: { label: 'lag  [days]', min: 1, max: MAXLAG },
          y: { label: 'autocorrelation of the detrended series  [-]' },
          series: [
            { kind: 'band', x: xs, y: band,
              y0: band.map(b => (b === null ? null : -b)), colour: INK.muted, alpha: 0.16 },
            { name: 'detrended over 365 d', kind: 'line', x: xs, y: A.r, width: 2.2 },
            // CONTEXT. 365 is the window sw_recurrence_lag and sw_recurrence_
            // strength were measured under, and it is the curve the band and
            // the three peak marks are computed on. The other two are here to
            // show that the published number is a CHOICE and how much it moves
            // — which is a job that wants them legible and not equal.
            { name: '181 d', kind: 'line', x: xs, y: Ashort.r, width: 1.3, context: true },
            { name: '731 d', kind: 'line', x: xs, y: Along.r, width: 1.3, context: true },
            // INSIDE THE BAND IS "SHAPE, NOT FINDING", which is a region and was
            // drawn as its two edges. The edges stay — they are where the band
            // ends and a reader reads a value off them — and the wash between
            // them is what makes "is this peak real" answerable by looking.
            { name: '95 % band, Bartlett', kind: 'line', x: xs, y: band,
              colour: INK.muted, width: 1, dash: [3, 3], aside: true },
            { name: '', kind: 'line', x: xs, y: band.map(b => (b === null ? null : -b)),
              colour: INK.muted, width: 1, dash: [3, 3], aside: true },
          ],
          marks,
          notes,
        },
        answer: p1.at
          ? { value: p1.at + ' d', of: 'the lag the rotation signal peaks at, r = ' +
              p1.r.toFixed(3) + ' — outside the band at ' + outside + ' of ' + MAXLAG + ' lags' }
          : { value: 'none', of: 'no peak in the rotation band at this setting' },
        note: p1.at
          ? 'The first peak is at lag ' + p1.at + ' with r = ' + p1.r.toFixed(6) +
            (p2.at && p3.at
              ? ', and the harmonics say the period is not that number: the second peak at ' + p2.at +
                ' and the third at ' + p3.at + ' imply ' + (p2.at / 2).toFixed(1) + ' and ' +
                (p3.at / 3).toFixed(1) + ' days per cycle. The first bump rides on the tail of the ' +
                'steep decay from lag 1, which pulls its apparent peak toward zero; the far ' +
                'harmonics are clear of it — which is why the third is marked and not merely ' +
                'mentioned.'
              : '.') +
            '\n\nThe three lines are three detrend windows, and the comparison is the point: ' +
            'the window sets what counts as the trend and therefore what is left to correlate. ' +
            'At 731 days more low-frequency signal survives, so the whole curve sits higher and ' +
            'the rotation bump is a smaller share of it. At 181 days the window is approaching ' +
            'the rotation itself and starts removing what it is meant to leave. 365 is drawn ' +
            'heavy because it is the window sw_recurrence_lag and sw_recurrence_strength were ' +
            'measured under; the other two are here so a reader can see that the published ' +
            'number is a choice and how much it moves.' +
            '\n\nThe dashed band is the 95 per cent interval under Bartlett\u2019s large-lag ' +
            'standard error, which widens with lag because the variance of a correlation grows ' +
            'with every correlation below it. The naive \u00b12/\u221an band — ' +
            (A.n[0] ? '\u00b1' + (2 / Math.sqrt(A.n[0])).toFixed(4) : 'a flat line') +
            ' — tests whether the series is white noise, which it obviously is not, and would ' +
            'pass everything drawn here. Bartlett\u2019s asks the question a reader actually ' +
            'has: is this bump more than the decay beneath it already produces. The 365 curve ' +
            'is outside the band at ' + outside + ' of the ' + MAXLAG + ' lags; where it is ' +
            'inside, the wiggle is shape rather than finding.'
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
      const pc = i => (100 * share[i] / vals.length);
      const TOP = names.length - 1;
      return {
        answer: { value: pc(TOP).toFixed(1) + '%',
          of: 'of the record is ' + names[TOP] + ' \u2014 the tail a drag design is sized by; '
            + names[0] + ' holds ' + pc(0).toFixed(1) + '%' },
        spec: {
          // The fact that made the shaded region wrong (§35.3), said as a
          // sentence instead: the band a design is sized by is a sliver of the
          // record and most of the axis.
          finding: 'the ' + names[TOP] + ' band holds ' + pc(TOP).toFixed(1) +
            '% of the days and runs from ' + sig(cuts[cuts.length - 1]) + ' to ' + sig(hi) +
            ', which is most of the axis',
          x: { label: (key === 'ap' ? 'daily Ap' : 'F10.7  [sfu]') + '  [bin ' + bw + ']', min: 0 },
          y: { label: o.scale === 'log' ? 'days in bin  [log10]' : 'days in bin', min: 0 },
          series: [{ name: '', kind: 'bars', x: xs, y: ys, colour: INK.series[0] }],
          // NO REGION HERE, AND THAT IS A MEASUREMENT RATHER THAN A PREFERENCE.
          // Shading the top band is the obvious move and it is wrong on this
          // axis: storm is Ap 26 and above, the axis runs to 400, so the band
          // holding 2.3 per cent of the DAYS is 94 per cent of the WIDTH. It
          // washed 69.5 per cent of the frame's pixels and told a reader the
          // opposite of the share it was drawn to show. The share is in the
          // answer line instead, where it is a number and not an area.
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
    // THE PERCENTILE AND THE SPAN WERE CONTROLS AND ARE NOW THE PICTURE.
    //
    // Four percentiles behind a knob is four pictures of one quantity that
    // nobody can see at once, and the question the knob was standing in for —
    // how much does this answer depend on which percentile you take — is
    // answerable only by seeing them together. They are an ORDERED set, so they
    // are drawn as an ordinal ramp in one hue rather than in four categorical
    // ones: a rainbow across 50, 90, 95, 99 would say they are four unrelated
    // things. The 95th is drawn heavy because it is the percentile
    // sw_uncertainty_growth publishes.
    //
    // The span was three truncations of one curve, which is not a comparison at
    // all — it is the same picture with less of it. The axis simply runs to the
    // longest of them. 48 combinations become 4 views.
    controls: [
      { k: 'v', label: 'variable', opts: [['f107', 'F10.7'], ['ap', 'Ap']] },
      { k: 'by', label: 'split', opts: [['all', 'the whole record'], ['cycle', 'by cycle']] },
    ],
    build(rec, o) {
      const key = o.v, maxL = 5478;
      // The percentiles, lightest to darkest, and the one the row publishes.
      // Validated as an ordinal ramp: monotone lightness, every adjacent gap
      // clear, and the light end at 2.11:1 against the surface — re-run when
      // INK.surface was corrected to the white card the canvas actually sits
      // on, against a floor of 2. Never the
      // categorical hues — those say "unrelated", and these are a ladder.
      // FROM THE RAMP, NOT FROM FOUR HEXES TYPED HERE. A hue a panel spells out
      // is a hue that cannot follow the scheme, and on a dark surface an ordinal
      // ramp runs the other way: dim at the unemphasised end, bright at the one
      // the panel is about. Both directions are validated in chart.js; this
      // reads whichever is in force.
      const QS = [
        { q: 0.50, name: '50th', colour: INK.ramp4[0], width: 1.4 },
        { q: 0.90, name: '90th', colour: INK.ramp4[1], width: 1.4 },
        { q: 0.95, name: '95th — the published one', colour: INK.ramp4[2], width: 2.4 },
        { q: 0.99, name: '99th', colour: INK.ramp4[3], width: 1.4 },
      ];
      if (o.by === 'cycle') return growthByCycle(rec, key, 0.95, maxL);
      const byDay = new Map();
      for (const d of rec.days) if (d[key] !== null) byDay.set(d.t, d[key]);
      const leads = [];
      for (let L = 30; L <= maxL; L = Math.round(L * 1.35)) leads.push(L);
      const xs = [], ns = [], ys = QS.map(() => []);
      for (const L of leads) {
        const ch = [];
        for (const [t, v] of byDay) {
          const w = byDay.get(t + L);
          if (w !== undefined) ch.push(w - v);
        }
        // SORTED ONCE FOR ALL FOUR. The percentiles differ only in where they
        // read the same sorted sample, and sorting it four times would be four
        // chances for them to disagree about what the sample was.
        ch.sort((a, b) => a - b);
        xs.push(L / 365.25); ns.push(ch.length);
        QS.forEach((Q, i) => ys[i].push(quantile(ch, Q.q)));
      }
      const name = key === 'f107' ? 'F10.7' : 'Ap';
      const unit = key === 'f107' ? 'sfu' : '';
      const P95 = 2;
      const y95 = ys[P95];
      // Whether the eleven-year cycle is visible is a property of the curve
      // rather than of a setting, now that the axis always runs the whole way.
      const mid = y95.filter((y, i) => y !== null && xs[i] >= 3 && xs[i] <= 6);
      const late = y95.filter((y, i) => y !== null && xs[i] >= 9 && xs[i] <= 12);
      const humped = mid.length && late.length && Math.max(...mid) > Math.max(...late);
      // How much the answer depends on which percentile is taken, at the lead
      // the row itself is read at. Measured rather than asserted.
      const atYear = xs.reduce((b, x, i) => (Math.abs(x - 1) < Math.abs(xs[b] - 1) ? i : b), 0);
      const spread = QS.map((Q, i) => ys[i][atYear]).filter(v => v !== null && isFinite(v));
      const at1 = y95[atYear];
      // THE TWO FEATURES THE PROSE POINTS AT, POINTED AT. The hump and the dip
      // are the eleven-year cycle showing through a statistic that was never
      // told about it, and the sentence saying so sat four paragraphs below the
      // place it is about. Found the same way the prose finds them — the
      // largest of the mid-lead points and the smallest of the late ones — so
      // the label cannot drift from the curve under it.
      const pick = (lo, hi, want) => {
        let bi = -1;
        for (let i = 0; i < xs.length; i++) {
          const v = y95[i];
          if (v === null || !isFinite(v) || xs[i] < lo || xs[i] > hi) continue;
          if (bi < 0 || (want === 'max' ? v > y95[bi] : v < y95[bi])) bi = i;
        }
        return bi;
      };
      const iHump = pick(3, 6, 'max'), iDip = pick(9, 12, 'min');
      // How wide the fan is at one lead, for the case where there is no hump to
      // name. Measured off the two edges the fill is drawn between.
      const fanAt = i => {
        const a = ys[3][i], b = ys[0][i];
        return (a === null || b === null || !isFinite(a) || !isFinite(b)) ? null : a - b;
      };
      const notes = [];
      if (humped && iHump >= 0) {
        notes.push({ x: xs[iHump], y: y95[iHump],
          text: 'half a cycle — the lead most likely to land on the opposite phase' });
      }
      if (humped && iDip >= 0) {
        notes.push({ x: xs[iDip], y: y95[iDip], text: 'about a full cycle, back to a similar one' });
      }
      return {
        answer: at1 === null || !isFinite(at1)
          ? { value: '\u2014', of: 'no lead near a year has enough pairs to read' }
          : { value: (at1 > 0 ? '+' : '') + sig(at1) + (unit ? ' ' + unit : ''),
              of: 'the 95th-percentile change in ' + name + ' over a lead of '
                + xs[atYear].toFixed(2) + ' yr \u2014 the percentile sw_uncertainty_growth publishes' },
        spec: {
          // The shape of the published percentile, off the published percentile.
          // The answer above the chart is its value at a year; this is what the
          // curve DOES, which is the thing a monotone-looking fan hides.
          finding: (iHump >= 0 && iDip >= 0 && humped)
            ? 'the 95th rises to ' + sig(y95[iHump]) + (unit ? ' ' + unit : '') + ' at ' +
              xs[iHump].toFixed(1) + ' yr, falls to ' + sig(y95[iDip]) + ' at ' +
              xs[iDip].toFixed(1) + ', and rises again \u2014 that is the eleven-year cycle'
            : 'the fan between the 50th and the 99th opens from ' + sig(fanAt(0)) +
              (unit ? ' ' + unit : '') + ' at the shortest lead to ' + sig(fanAt(xs.length - 1)) +
              ' at the longest',
          x: { label: 'lead  [years]', min: 0 },
          y: { label: 'change in ' + name + ' at a percentile  [' + (unit || '-') + ']' },
          // `ns` was counted here and thrown away. Handing it to the chart is what
          // makes the far end of these curves look as thin as they are — and it
          // goes on ALL FOUR, because the thinning is a property of the lead and
          // fading only the published one would say the others rest on more.
          notes,
          series: [
            // THE FAN, FILLED, AND THE FOUR LINES STILL ON IT. The note says the
            // gap between the 50th and the 99th is the whole of what a band buys
            // and costs — and it said it about four curves that sit inside two
            // pixels of each other at a short lead, so the thing described was
            // thinner than the ink describing it. Filled, the cost of choosing a
            // percentile is an area that visibly opens with the lead. It goes
            // FIRST so the lines sit on top, carries no name so the legend stays
            // four entries, and wears the published percentile's own hue: it is
            // that family's spread and not a fifth quantity.
            { kind: 'band', x: xs, y: ys[3], y0: ys[0], colour: QS[P95].colour, alpha: 0.07 },
          ].concat(QS.map((Q, i) => ({
            name: Q.name, kind: 'line', x: xs, y: ys[i], n: ns,
            colour: Q.colour, width: Q.width,
          }))),
        },
        note: 'The SIGNED change in ' + name + ' over a lead — not the absolute change, because ' +
          'the unsafe direction for a drag design is the driver arriving higher than planned. ' +
          'Four percentiles at once, because the choice of percentile is a design decision and ' +
          'a picture of one of them hides its cost: at a lead of one year they run from ' +
          (spread.length ? sig(Math.min(...spread)) + ' to ' + sig(Math.max(...spread)) +
            (unit ? ' ' + unit : '') : 'nothing drawn') +
          '. sw_uncertainty_growth publishes the 95th, drawn heavy.\n\n' +
          shape(xs, y95, unit, 'yr', 'lead') +
          ' At the median there is almost no tail: half of all changes are above that line and ' +
          'half below, so a value near zero says the driver has no trend over these leads, which ' +
          'is what a cyclic quantity looks like when the lead is not tied to its phase. The gap ' +
          'between the 50th and the 99th is the whole of what a band buys and costs.' +
          (humped
            ? ' The hump and the dip are the eleven-year cycle rather than noise: a lead of about ' +
              'half a cycle is the lead most likely to land on the opposite phase, and a lead of ' +
              'about a full cycle returns to a similar one.'
            : '') +
          '\n\nEvery line fades as its sample thins — the far end of a fifteen-year curve rests ' +
          'on ' + ns[ns.length - 1] + ' pairs against ' + ns[0] + ' at the near end. And pairs at ' +
          'a given lead overlap almost completely, so even ' + ns[0] + ' is nothing like that many ' +
          'independent observations.',
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
      // THE METRIC AND THE BASELINE WERE CONTROLS AND ARE NOW THE PICTURE, and
      // this is the panel the stacked frame was built for.
      //
      // Three metrics of one forecast against one lead, and they cannot share a
      // y axis: skill is a dimensionless ratio, bias is signed sfu and RMS error
      // is positive sfu on a different scale. A second y axis would let whoever
      // drew it choose where the curves cross, which is the most reliable way to
      // make a chart say something the data did not. So the answer was a control
      // — and a control shows one at a time, which is exactly what makes "is the
      // outlook biased where its skill collapses" unanswerable. Three frames in a
      // column on one lead axis answers it by looking.
      //
      // The baseline is two lines in the skill frame, which is better than a
      // control was: the strict and the leaky score are now computed over their
      // own pairs and drawn together, so the reader sees the size of the leak
      // rather than having to remember the other picture. 13 combinations
      // become 3 views.
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
      // BOTH BASELINES, BUILT ONCE. `leaky` is allowed to use the issue date
      // itself; `strict` is what a forecaster actually had. Two lookups from one
      // function rather than two functions, because two functions is two places
      // for the one-day difference between them to stop being one day.
      const persistFrom = (issue, first) => {
        const t = tOf.get(issue);
        if (t === undefined) return null;
        for (let b = first; b <= 15; b++) {
          const v = byDay.get(t - b);
          if (v !== undefined) return v;
        }
        return null;
      };
      const pcS = new Map(), pcL = new Map();
      const strict = i => { if (!pcS.has(i)) pcS.set(i, persistFrom(i, 1)); return pcS.get(i); };
      const leaky = i => { if (!pcL.has(i)) pcL.set(i, persistFrom(i, 0)); return pcL.get(i); };

      if (o.view === 'year') return byIssueYear(fc, byDay, tOf, strict, leaky);

      // EACH METRIC OVER THE PAIRS ITS OWN DEFINITION COVERS, and that is a
      // correction rather than a nicety. Bias and RMS error use no baseline —
      // sw_forecast_bias is the mean of (forecast − observed) at a lead, full
      // stop — but this panel used to drop every row whose persistence lookup
      // came back empty before computing them, so it quoted the row's quantity
      // over a subset the row does not take. The note carried the claim that the
      // baseline "changes nothing on this metric", which was very nearly true
      // and not exactly, which is the worst kind.
      const xs = [], nObs = [], nStr = [], nLk = [];
      const skS = [], skL = [], bias = [], rmse = [];
      for (let L = 1; L <= 27; L++) {
        let e2 = 0, se = 0, n = 0;
        let e2s = 0, p2s = 0, ns = 0, e2l = 0, p2l = 0, nl = 0;
        for (const r of fc) {
          if (+r.lead_days !== L || r.f107 === null) continue;
          const tt = tOf.get(r.target_date);
          const obs = tt === undefined ? undefined : byDay.get(tt);
          if (obs === undefined) continue;
          const e = +r.f107 - obs;
          e2 += e * e; se += e; n++;
          const ps = strict(r.issue_date);
          if (ps !== null) { e2s += e * e; p2s += (ps - obs) * (ps - obs); ns++; }
          const pl = leaky(r.issue_date);
          if (pl !== null) { e2l += e * e; p2l += (pl - obs) * (pl - obs); nl++; }
        }
        if (!n) continue;
        xs.push(L); nObs.push(n); nStr.push(ns); nLk.push(nl);
        bias.push(se / n); rmse.push(Math.sqrt(e2 / n));
        skS.push(ns && p2s ? 1 - (e2s / ns) / (p2s / ns) : null);
        skL.push(nl && p2l ? 1 - (e2l / nl) / (p2l / nl) : null);
      }
      const last = xs[xs.length - 1];
      const at = (arr, L) => { const k = xs.indexOf(L); return k < 0 ? null : arr[k]; };
      // WHERE THE TWO SAMPLES ACTUALLY DIFFER, named rather than assumed. The
      // first example this note reached for was lead 27, where they happen to be
      // equal — a sentence about a correction, illustrated with the one case the
      // correction does not touch.
      let gapAt = -1, gapBy = 0;
      for (let k = 0; k < xs.length; k++) {
        if (nObs[k] - nStr[k] > gapBy) { gapBy = nObs[k] - nStr[k]; gapAt = xs[k]; }
      }
      const sig2 = v => (v === null ? '—' : v.toFixed(3));
      const runOf = arr => arr.filter(v => v !== null && isFinite(v) && v > 0).length;
      const best = skS.reduce((b, v, i) => (v !== null && (b < 0 || v > skS[b]) ? i : b), -1);
      return {
        answer: best < 0
          ? { value: '—', of: 'no lead scored' }
          : { value: (skS[best] >= 0 ? '+' : '') + skS[best].toFixed(3),
              of: 'peak skill against persistence, at lead ' + xs[best] +
                ' — above zero the outlook beats assuming nothing changes' },
        spec: {
          // The sentence panels/forecast.toml asks a reader to check, counted:
          // the strict baseline is beaten over a run of leads and the leaky one
          // is not, and where each stops is the leak.
          finding: 'the strict line is above zero at ' + runOf(skS) + ' of the ' + xs.length +
            ' leads and the leaky one at ' + runOf(skL) + '; the bias frame is below zero at ' +
            bias.filter(v => v !== null && isFinite(v) && v < 0).length + ' of them',
          x: { label: 'lead  [days]', min: 1, max: 27 },
          panes: [
            {
              y: { label: 'skill against persistence  [-]' },
              series: [
                { name: 'vs. last obs BEFORE issue', kind: 'line', x: xs, y: skS, n: nStr },
                // CONTEXT. The strict baseline is the score; the leaky one is
                // here to show what the leak is worth, and drawn at equal
                // weight it competed with the number the panel publishes.
                { name: 'vs. obs ON the issue date (leaks)', kind: 'line', x: xs, y: skL,
                  n: nLk, colour: INK.series[1], dash: [5, 3], context: true },
              ],
              marks: scoreBaseline('skill'),
            },
            {
              y: { label: 'mean signed error, forecast − observed  [sfu]' },
              series: [{ name: '', kind: 'line', x: xs, y: bias, n: nObs }],
              marks: scoreBaseline('bias'),
            },
            {
              // ZERO ON THE AXIS, AND THE AREA FILLED. RMS error is a magnitude
              // measured from perfect: zero is not a baseline to beat but the
              // origin the quantity is a size from, so an axis that started at
              // 9 was drawing the VARIATION in the error and calling it the
              // error. With zero on it the frame says the error roughly triples
              // across the lead range, which is the claim, and the fill is what
              // makes that a size rather than a height to read off gridlines.
              y: { label: 'RMS error  [sfu]', min: 0 },
              series: [{ name: '', kind: 'line', x: xs, y: rmse, n: nObs, fill: true }],
              marks: scoreBaseline('rmse'),
            },
          ],
        },
        note: 'Three scores of one outlook against one lead axis, stacked rather than offered as a ' +
          'control, because the question is how they move TOGETHER: skill collapses toward the far ' +
          'leads while RMS error grows, and the bias frame says which direction the misses go. They ' +
          'cannot share a y axis — a ratio, signed sfu and positive sfu on a different scale — and ' +
          'a second y axis would let whoever drew it choose where the curves cross.\n\n' +
          'Skill is one minus the ratio of mean squared errors, so zero is the red line: no better ' +
          'than assuming nothing changes, and negative is worse than not bothering. THE DASHED LINE ' +
          'IS A BASELINE THAT LEAKS. 719 of the 1281 issues index their rows from lead 0, so the ' +
          'issue date is itself a forecast target for most of the record, and handing it to ' +
          'persistence gives the baseline a number the forecaster did not have. At lead 1 the two ' +
          'read ' + sig2(at(skS, 1)) + ' strict against ' + sig2(at(skL, 1)) + ' leaky; the gap is ' +
          'the leak, and on the strict baseline the outlook beats persistence from lead 1.\n\n' +
          'Bias is signed, forecast minus observed, so below the line means the outlook came in LOW ' +
          'and a design reading it gets a thinner atmosphere than it will fly. RMS error is accuracy ' +
          'rather than skill: it says nothing about beating a baseline, only how far the outlook ' +
          'misses, and it grows with the lead whatever the skill does. Neither uses a baseline, so ' +
          'neither has a second line — and both are now computed over every pair with an ' +
          'observation rather than over the pairs the baseline happened to cover. The two samples ' +
          'are not the same: they differ most at lead ' + gapAt + ', where ' + at(nObs, gapAt) +
          ' pairs have an observation and the strict baseline reaches ' + at(nStr, gapAt) +
          ' of them. Tying a baseline-free metric to a baseline is a small error and it was a ' +
          'real one.\n\n' +
          'Every line fades as its sample thins. Lead ' + last + ' draws on ' +
          nObs[nObs.length - 1] + ' pairs against ' + nObs[0] + ' at lead 1: lead_days is indexed ' +
          'two ways in one column, and only the 1-based minority reaches 27, which is why ' +
          'sw_outlook_lead declares 26.',
      };
    },
  },

  // -------------------------------------------------------------------------
  // THE DRIVER SET, AND THE LEGACY RUN BESIDE IT.
  //
  // Thirteen rows had no figure, and §23 was right that this was not an
  // omission: the legacy tool published the driver set as a TABLE, not a plot,
  // so there was never a picture here to port. But "the legacy tool had no
  // figure" is a reason not to have ported one, not a reason not to have one.
  //
  // What makes it worth drawing is that this port and the study DISAGREE, and
  // the disagreement is the finding. Five quantities, five scenarios, twenty-five
  // numbers — and they agree to a hundredth of a per cent on the sustained Ap and
  // Kp, and diverge by a factor of two on F10.7 and on every single-day value.
  // Both differences are deliberate and both are explained in §20; a table of
  // fifty numbers makes that invisible and a picture makes it the first thing
  // anybody sees.
  //
  // So this is a standing parity check a person looks at rather than one buried
  // in a test, and it is the only figure in the subsystem whose subject is the
  // port itself.
  {
    id: 'drivers',
    rows: ['l3_solar_interface',
      'sw_ap_central_expectation', 'sw_ap_cold_long', 'sw_ap_cold_short',
      'sw_ap_daily_band_drop', 'sw_ap_daily_band_spread',
      'sw_ap_design_long', 'sw_ap_design_short', 'sw_ap_mean_band_spread',
      'sw_daily_band_drop', 'sw_daily_band_spread', 'sw_kp_scenarios',
      'sw_mean_band_spread'],
    // One run returns the crossing and all twenty-four of its members, so the
    // whole table costs one question.
    engine: ['l3_solar_interface'],
    label: 'Drivers',
    draws: 'The five design scenarios this subsystem publishes, against the legacy run’s own.',
    asks: 'What does this subsystem hand upward, and does it agree with the study it ports?',
    controls: [
      { k: 'view', label: 'view', opts: [['set', 'one quantity across the scenarios'],
        ['parity', 'all twenty-five, against the legacy run']] },
      { k: 'q', label: 'quantity', when: o => o.view === 'set', opts: [
        ['f107', 'F10.7'], ['f107bar', '81-day mean F10.7'], ['ap', 'daily Ap'],
        ['kp_mean', 'Kp, mean slot'], ['kp_peak', 'Kp, peak slot'],
      ] },
    ],
    async data() {
      return { legacy: await parityFile('mission_drivers.csv') };
    },
    build(rec, o, extra, eng) {
      // COLD TO HOT, which is an ordering and not an alphabet. The five
      // scenarios are a ladder — the quietest single day, the cold sustained
      // level, where the mission sits, the hot sustained level, the worst single
      // day — and drawn in that order every quantity here is monotone, so a
      // reader checks the picture by whether it rises. In the order the legacy
      // CSV happens to list them it is a zigzag that says nothing.
      const ORDER = ['coldday', 'coldmean', 'nominal', 'hotmean', 'hotday'];
      const SHOWN = ['quietest day', 'cold sustained', 'nominal', 'hot sustained', 'worst day'];
      const QS = [
        { k: 'f107', label: 'F10.7', unit: 'sfu' },
        { k: 'f107bar', label: '81-day mean F10.7', unit: 'sfu' },
        { k: 'ap', label: 'daily Ap', unit: '-' },
        { k: 'kp_mean', label: 'Kp, mean slot', unit: '-' },
        { k: 'kp_peak', label: 'Kp, peak slot', unit: '-' },
      ];
      // OURS, FROM THE ENGINE, THROUGH THE DOT. The crossing publishes a set and
      // a member is `<node>.<member>`; the node's own answer is f107_hotmean,
      // which is the one cell that is not a member, so it is reached by the node
      // id. Written as a lookup rather than a table of literals for the reason
      // §21 gives: three copied numbers in `design` went stale without anything
      // noticing.
      const ours = (q, sc) => {
        const id = q === 'f107' && sc === 'hotmean'
          ? 'l3_solar_interface'
          : 'l3_solar_interface.' + q + '_' + sc;
        const v = eng[id];
        return v && v.si !== undefined && isFinite(v.si) ? v.si : null;
      };
      const legacyRows = new Map((extra.legacy.rows || []).map(r => [r.scenario, r]));
      const theirs = (q, sc) => {
        const r = legacyRows.get(sc);
        const v = r ? num(r[q]) : null;
        return v === null || !isFinite(v) ? null : v;
      };
      const xs = ORDER.map((_, i) => i);
      const fmtX = v => SHOWN[Math.round(v)] || '';
      // A LITTLE ROOM AT BOTH ENDS. With the extent exactly 0 to 4 the first and
      // last scenarios sit on the frame's own edges: the quietest day's marker is
      // half outside the axis and the worst day's tick label is centred on the
      // right margin and gets cut by the canvas. A categorical axis is not a
      // range that happens to run 0 to 4 — the outer categories need the same
      // room as the inner ones.
      // NO VERTICAL RULES. The axis is five named scenarios, and a gridline is
      // an invitation to read a value off the axis where it falls. Between
      // "cold sustained" and "nominal" there is nothing to read, so the rules
      // were ink separating categories the tick labels had already separated.
      // The horizontal grid stays: the y axis IS a scale.
      const XPAD = { min: -0.3, max: 4.3, ticks: 5, fmt: fmtX, grid: false };

      if (o.view === 'parity') {
        // THE RATIO, ON A LOG AXIS, because agreement is 1 and the two kinds of
        // disagreement here are a factor of about two in each direction. On a
        // linear axis "twice" and "half" are 1 and 0.5 and look nothing like the
        // same size of error; in log space they are the same distance from the
        // line, which is what they are.
        const series = QS.map((Q, i) => ({
          name: Q.label, kind: 'line', x: xs,
          y: ORDER.map(sc => {
            const a = ours(Q.k, sc), b = theirs(Q.k, sc);
            return a === null || b === null || b === 0 ? null : a / b;
          }),
          colour: INK.series[i % INK.series.length],
        }));
        const all = series.flatMap(s => s.y).filter(v => v !== null);
        const worst = all.length
          ? all.reduce((m, v) => (Math.abs(Math.log(v)) > Math.abs(Math.log(m)) ? v : m), 1)
          : null;
        const near = all.filter(v => Math.abs(v - 1) < 0.001).length;
        const off = worst === null ? null : (worst > 1 ? worst : 1 / worst);
        return {
          answer: { value: near + ' of ' + all.length,
            of: 'cells agree with the legacy run to a tenth of a per cent'
              + (off === null ? '' : '; the furthest is out by a factor of ' + off.toFixed(2))
              + ', and both families of disagreement are deliberate' },
          spec: {
            aspect: 1.5,
            finding: (() => {
              const lowTop = all.filter(v => v < 0.999).length;
              return near + ' of the ' + all.length + ' points sit on the line, ' + lowTop +
                ' below it and ' + (all.length - near - lowTop) + ' above \u2014 the ' +
                'disagreement is not scattered, it is two families';
            })(),
            x: { label: 'scenario', ...XPAD },
            y: { label: 'this tree ÷ the legacy run  [-]', log: true },
            series,
            marks: [{ axis: 'y', at: 1, label: 'exact agreement', colour: INK.bound }],
          },
          note: 'Every one of the twenty-five numbers this subsystem publishes, divided by what the ' +
            'legacy tool wrote for the same cell. One is agreement, and the axis is logarithmic so ' +
            'that twice and half sit the same distance from it — on a linear axis they are 1.0 and ' +
            '0.5 apart, which makes the two directions of error look like different sizes.\n\n' +
            near + ' of ' + all.length + ' cells agree to within a tenth of a per cent, and they are ' +
            'the SUSTAINED Ap and Kp scenarios. The rest disagree, the furthest by a factor of ' +
            (worst === null ? '—' : (worst > 1 ? worst.toFixed(2) : (1 / worst).toFixed(2))) +
            ', and BOTH families of disagreement are deliberate.\n\n' +
            'F10.7 is low across every scenario because the two tools centre the window differently. ' +
            'The study holds its last 27-day rotation forecast flat and gets 158.33 sfu for its own ' +
            '2027 window; sw_central_expectation reads the cycle analogue at the declared epoch and ' +
            'gets 86.85. Neither is arithmetic — it is a choice about what a window beyond the ' +
            'record should be centred on, and §20 took the second.\n\n' +
            'The SINGLE-DAY scenarios disagree because the within-rotation departure is not one ' +
            'number. The study reads one percentile of it over the 2001 days before the window and ' +
            'applies it everywhere; measured against the level each day sits in, the 95th runs from ' +
            '4.63 sfu at a rotation of 70 to 46.93 at 210, a factor of ten. A fixed-window ' +
            'percentile is that statistic mixed over whatever levels fell in its own sample, right ' +
            'near their mean and wrong at both ends. The four daily rows are level-conditioned here, ' +
            'which is why the worst day and the quietest day move in opposite directions from the ' +
            'study’s.',
        };
      }

      const Q = QS.find(x => x.k === o.q) || QS[0];
      const mine = ORDER.map(sc => ours(Q.k, sc));
      const theirsY = ORDER.map(sc => theirs(Q.k, sc));
      const gap = mine.map((v, i) => (v === null || theirsY[i] === null ? null : v - theirsY[i]));
      const worstI = gap.reduce((b, v, i) =>
        (v !== null && (b < 0 || Math.abs(v) > Math.abs(gap[b])) ? i : b), -1);
      const u = Q.unit === '-' ? '' : ' ' + Q.unit;
      return {
        answer: worstI < 0
          ? { value: '\u2014', of: 'nothing to compare at this setting' }
          : { value: sig(Math.abs(gap[worstI])) + u,
              of: 'the widest gap to the legacy run, at the ' + SHOWN[worstI] + ' scenario \u2014 '
                + sig(mine[worstI]) + u + ' here against ' + sig(theirsY[worstI]) + u },
        spec: {
          // FIVE CATEGORIES DO NOT NEED 1180 PIXELS. See render(): width is what
          // is wasted on this axis, not height, and stretching five points
          // across a full frame flattens the ladder the view is drawn to show
          // rising.
          aspect: 1.5,
          finding: (() => {
            const both = ORDER.map((_, i) => [mine[i], theirsY[i]])
              .filter(([m2, t]) => m2 !== null && t !== null);
            if (!both.length) return 'nothing to compare at this setting';
            const below = both.filter(([m2, t]) => m2 < t).length;
            return (below === both.length
              ? 'this tree is below the legacy run at all ' + both.length + ' scenarios'
              : below === 0
                ? 'this tree is above the legacy run at all ' + both.length + ' scenarios'
                : 'this tree is below the legacy run at ' + below + ' of the ' + both.length +
                  ' scenarios and above it at ' + (both.length - below)) +
              ', and this tree\u2019s own line ' + (mine.every((v, i) =>
                i === 0 || v === null || mine[i - 1] === null || v >= mine[i - 1])
                ? 'rises across all five'
                : 'does not rise all the way \u2014 the ladder turns back somewhere');
          })(),
          x: { label: 'scenario', ...XPAD },
          y: { label: Q.label + '  [' + Q.unit + ']' },
          series: [
            { name: 'this tree', kind: 'line', x: xs, y: mine, width: 2.2,
              row: 'l3_solar_interface' },
            // Dots rather than a second line: the legacy run is five separate
            // answers and joining them would claim it interpolates between
            // scenarios, which is not a thing a scenario set does.
            { name: 'the legacy run', kind: 'dots', x: xs, y: theirsY,
              colour: INK.series[1], width: 5, alpha: 1 },
          ],
          // WHERE THE TWO METHODS PART, SAID AT THE POINT IT HAPPENS. The gap is
          // widest at whichever scenario the data says, and on the single-day
          // ones the reason is one sentence: the study reads one percentile of
          // the within-rotation departure over a fixed window and applies it at
          // every level. That sentence is four paragraphs below the dot.
          notes: worstI < 0 ? [] : [{
            x: xs[worstI], y: theirsY[worstI],
            text: /day$/.test(ORDER[worstI])
              ? 'one percentile, applied at every level'
              : 'the two windows are centred differently',
            colour: INK.series[1],
          }],
        },
        note: 'The five scenarios this subsystem hands to sys_space_environment, in the order a ' +
          'design reads them: the quietest single day, the cold sustained level, where the mission ' +
          'is expected to sit, the hot sustained level, and the worst single day. An array is sized ' +
          'on a sustained level and a thermal transient on a day, which is why the tool publishes ' +
          'all five rather than choosing one and calling it governing.\n\n' +
          'The dots are the legacy tool’s own answers for the same five cells, from ' +
          'matlab/reference/mission_drivers.csv — a record of what a DIFFERENT PROGRAM computed, ' +
          'not a measurement, which is exactly what makes it usable as a second opinion. ' +
          (worstI < 0
            ? 'Nothing to compare at this setting.'
            : 'The two differ most at the ' + SHOWN[worstI] + ' scenario: ' +
              sig(mine[worstI]) + u + ' here against ' + sig(theirsY[worstI]) + u +
              ', a gap of ' + sig(Math.abs(gap[worstI])) + u + '.') +
          ' Switch to the parity view for all twenty-five at once and why they differ.',
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
      'sw_f107_cold_short', 'l3_solar_req_01',
      // §26 D4's leftover: the one row in the subsystem that no figure cited.
      'sw_window_peak_level'],
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
      const [ret, gmap, fl, fs, cl, cs, pk] = await Promise.all([
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
        // THE LAST ROW IN THE SUBSYSTEM WITH NO FIGURE, and it belongs here.
        //
        // sw_window_peak_level is the maximum of the same cycle analogue the four
        // design rows are built on a MEAN of, over the same window, against the
        // same axis. It was the one row §26's D4 left over — nothing read it and
        // no panel cited it — and the reason it was worth keeping rather than
        // removing is visible only when it is drawn beside them: it rises
        // monotonically with mission length, because a longer window can only
        // contain more of the cycle, while every design curve follows the window
        // MEAN and wanders with where the window ends. Past about eight years the
        // analogue's own peak is above the hot single-day design value, which is
        // a design sized on a mean sitting under the thing it averages.
        engineSweep('sw_window_peak_level', 'sys_mission_requirements_mission_duration',
          dur.lo, dur.hi, 60),
      ]);
      return { ret, gmap, win: { fl, fs, cl, cs, pk } };
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
      const hitsAt = cross(bound);
      return {
        answer: !isFinite(hitsAt)
          ? { value: 'never', of: 'the design level is not reached inside the declared range' }
          : { value: sig(hitsAt) + ' yr', of: 'before the record expects a storm above the ' +
              'G' + o.g + ' design level of Ap ' + bound.toFixed(0) },
        spec: {
          finding: !isFinite(hitsAt)
            ? 'the curve stays under the design level across the whole declared range'
            : 'the curve enters the shaded region at ' + hitsAt.toFixed(2) +
              ' yr and never leaves it; ' + (isFinite(cross(req))
                ? 'it reaches the requirement at ' + cross(req).toFixed(2) + ' yr'
                : 'the requirement is not reached inside the declared range'),
          x: { label: 'mission length  [years]', min: xs[0], max: xs[xs.length - 1] },
          y: { label: 'daily Ap the record expects once in that time  [-]' },
          // `row` MAKES THE PICTURE NAVIGABLE. A curve that IS a row opens it
          // on a click. Most curves in this face are computed from the record
          // by the panel and have no row behind them; they carry no `row`, and
          // that absence is correct rather than missing.
          series: [{ name: 'sw_storm_return_level', kind: 'line', x: xs, y: ys,
            row: 'sw_storm_return_level' }],
          marks: [
            // THE REGION, NOT ITS EDGE. Everything above the design level is
            // outside what the vehicle was built for, and a wash says that where
            // a dashed rule asked the reader to work out which side they were on.
            //
            // IN THE DESIGN LEVEL'S OWN INK, not the requirement's. It was drawn
            // in the pink this face uses for a bound a value is measured against
            // — which is the colour of the OTHER rule in this frame, the one at
            // Ap 207 — so the wash started at one line and was coloured like the
            // other. A region belongs to the edge it opens at.
            { axis: 'y', from: bound, label: '', colour: INK.mark, alpha: 0.05 },
            { axis: 'y', at: bound, label: 'designed for G' + o.g + ' = Ap ' + bound.toFixed(0) +
              '  (sw_ap_design)', row: 'sw_ap_design' },
            { axis: 'y', at: req, label: 'required ≤ ' + req.toFixed(0) + '  (' + o.req + ')',
              colour: INK.bound, row: o.req },
          ],
          // A NOTE, NOT A RULE. This was a dashed line the full height of the
          // frame carrying "exceeds the design at 2.62 yr" at the top, which is
          // a fact about a POINT ON THE CURVE told at the ceiling. It now points
          // where it happens, and the frame is one long dashed line lighter for
          // it.
          notes: !isFinite(hitsAt) ? [] : [{
            x: hitsAt, y: bound,
            text: 'exceeded here — ' + hitsAt.toFixed(2) + ' yr',
          }],
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


  // -------------------------------------------------------------------------
  // THE ONLY PICTURE OF THE QUESTION THE TOOL EXISTS TO ANSWER.
  //
  // Ten rows — five requirements and the five closures against them — had no
  // figure, and until §20 they had no closure either: the achieved rows read one
  // input and returned it unchanged, and nothing in the tree compared a
  // requirement against anything. l3_solar_req_03's own sheet said so.
  //
  // Now each pair publishes a signed fractional margin, and this draws the thing
  // a designer actually acts on: where the margin is, which decision spends it,
  // and how far that decision can move before it is gone. The sweep axis is NOT
  // chosen here — it is whatever /v1/levers reports as spending this margin
  // fastest, which is why that endpoint exists. A panel that picked its own axis
  // would be the author guessing, and the sweep control on the old face offered
  // a decision worth exactly zero because somebody did.
  //
  // Two frames, because the bound and the achieved value share a unit and the
  // margin does not. The top frame is where the two cross; the bottom is the
  // same fact as a number, signed, with zero drawn.
  {
    id: 'closure',
    rows: ['l3_solar_req_01', 'l3_solar_req_02', 'l3_solar_req_03',
      'l3_solar_req_04', 'l3_solar_req_05',
      'l3_solar_ach_01', 'l3_solar_ach_02', 'l3_solar_ach_03',
      'l3_solar_ach_04', 'l3_solar_ach_05'],
    // Both sides of all five, so the panel never computes a margin itself: the
    // achieved rows ARE the margins and the requirement rows are the bounds.
    engine: ['l3_solar_ach_01', 'l3_solar_ach_02', 'l3_solar_ach_03',
      'l3_solar_ach_04', 'l3_solar_ach_05',
      'l3_solar_req_01', 'l3_solar_req_02', 'l3_solar_req_03',
      'l3_solar_req_04', 'l3_solar_req_05'],
    label: 'Closure',
    draws: 'Each requirement against what the record gives it, and the margin between them.',
    asks: 'Does the design close against the sky, and what spends the margin fastest?',
    controls: [
      { k: 'pair', label: 'closure', opts: [
        ['01', 'F10.7 — sustained'],
        ['02', 'F10.7 — single day'],
        ['03', 'Ap — survival'],
        ['04', 'Ap — sustained'],
        ['05', 'Ap — single day'],
      ] },
    ],
    // Opened from l3_solar_ach_03 or l3_solar_req_03, show the survival pair.
    // The ten rows this panel lives on are five pairs, and each row belongs to
    // exactly one of them.
    openAt(rowId, o) {
      const m = /^l3_solar_(?:ach|req)_(0[1-5])$/.exec(rowId);
      if (m) o.pair = m[1];
    },
    async data(o) {
      const n = (o && o.pair) || '01';
      const ach = 'l3_solar_ach_' + n, req = 'l3_solar_req_' + n;
      const key = ach;
      CLOSURE_CACHE.want = CLOSURE_CACHE.want || new Map();
      if (CLOSURE_CACHE.want.has(key)) return CLOSURE_CACHE.want.get(key);
      const pr = (async () => {
        // THE AXIS IS THE ENGINE'S ANSWER, NOT THE AUTHOR'S. Levers come back
        // ordered by how much they move this row, and the requirement is always
        // near the top of that list because a margin is a fraction OF it —
        // moving the bound moves the margin by construction and says nothing
        // about the sky. What a designer wants is the decision that spends the
        // margin they have, so the bound is skipped and the next one taken.
        const levers = await engineLevers(ach);
        const lv = levers.find(l => l.id !== req && l.span > 0) || null;
        if (!lv) return { lv: null };
        const quantity = QUANTITY_OF[n];
        const [mar, qty] = await Promise.all([
          engineSweep(ach, lv.id, lv.lower, lv.upper, 90),
          engineSweep(quantity, lv.id, lv.lower, lv.upper, 90),
        ]);
        return { lv, quantity, mar, qty };
      })();
      CLOSURE_CACHE.want.set(key, pr);
      return pr;
    },
    build(rec, o, extra, eng) {
      const n = o.pair || '01';
      const ach = 'l3_solar_ach_' + n, req = 'l3_solar_req_' + n;
      const title = PAIR_LABEL[n];
      if (!extra || !extra.lv) {
        throw new Error('no decision upstream of ' + ach + ' moves its margin, so there is ' +
          'nothing to sweep it over — which is itself worth knowing and not worth drawing');
      }
      const { lv, quantity, mar, qty } = extra;
      // SI out of the engine, divided back by the factor the sweep reports, the
      // same boundary rule every other panel here obeys.
      const x = mar.x.map(v => v / mar.x_factor);
      const margin = mar.y.map(v => v / mar.y_factor);
      const value = qty.y.map(v => v / qty.y_factor);
      // The requirement is a declared number: it does not move when an
      // environmental decision does, and drawing it as a flat line is the whole
      // point — it is what the achieved curve has to stay under. Divided by the
      // sweep's own factor rather than read off `shown`, which is a formatted
      // string for a person.
      const rq = eng[req];
      const reqV = rq && isFinite(rq.si) ? rq.si / (qty.y_factor || 1) : null;
      // Where the two cross, which is where the margin goes through zero.
      let cross = null;
      for (let k = 1; k < margin.length; k++) {
        const a = margin[k - 1], b = margin[k];
        if (a === null || b === null || !isFinite(a) || !isFinite(b)) continue;
        if ((a > 0) !== (b > 0)) { cross = x[k - 1] + (x[k] - x[k - 1]) * a / (a - b); break; }
      }
      // THE TOP FRAME'S CROSSING, COMPUTED FROM THE TOP FRAME. `cross` above is
      // where the MARGIN goes through zero; this is where the achieved curve
      // meets the requirement line. panels/closure.toml says the two must be the
      // same x, and until now that was a check a person made by looking — which
      // is exactly the kind of agreement worth measuring, because the two come
      // from different arrays and a panel where they disagreed would still draw.
      let crossQ = null;
      if (reqV !== null) {
        for (let k = 1; k < value.length; k++) {
          const a2 = value[k - 1] - reqV, b2 = value[k] - reqV;
          if (a2 === null || b2 === null || !isFinite(a2) || !isFinite(b2)) continue;
          if ((a2 > 0) !== (b2 > 0)) {
            crossQ = x[k - 1] + (x[k] - x[k - 1]) * a2 / (a2 - b2);
            break;
          }
        }
      }
      const agree = cross !== null && crossQ !== null &&
        Math.abs(cross - crossQ) <= Math.abs(x[x.length - 1] - x[0]) * 1e-6;
      const now = eng[ach] && isFinite(eng[ach].si) ? eng[ach].si : null;
      const marks = cross === null ? [] : [{ axis: 'x', at: cross,
        label: 'the margin runs out here' }];
      // The achieved curve on one side of zero margin, cut exactly where the
      // margin changes sign. `want` true is the side with margin left.
      const bandSide = want => {
        const xx = [], yy = [];
        const sgn = k => (margin[k] === null || !isFinite(margin[k]) ? null : margin[k] >= 0);
        for (let k = 0; k < x.length; k++) {
          const sk = sgn(k), pk = k > 0 ? sgn(k - 1) : null;
          if (pk !== null && sk !== null && pk !== sk) {
            const a = margin[k - 1], b = margin[k], f = a / (a - b);
            const xc = x[k - 1] + (x[k] - x[k - 1]) * f;
            const vc = value[k - 1] + (value[k] - value[k - 1]) * f;
            xx.push(xc); yy.push(vc);
            if (sk !== want) { xx.push(xc); yy.push(null); }
          }
          if (sk === want) { xx.push(x[k]); yy.push(value[k]); }
        }
        return { x: xx, y: yy };
      };
      return {
        answer: now === null
          ? { value: 'no answer', of: 'the engine did not return a margin for this pair' }
          : { value: (now >= 0 ? '+' : '') + now.toFixed(4),
              of: now < 0
                ? 'margin — NEGATIVE, the requirement is exceeded by ' +
                  (-now * 100).toFixed(1) + ' per cent of its own value'
                : 'margin — ' + (now * 100).toFixed(1) + ' per cent of ' + req + ' is unspent' },
        spec: {
          // THE CHECK panels/closure.toml ASKS FOR, MADE BY THE PANEL. "Wherever
          // the swept curve crosses the requirement in the top frame, the margin
          // crosses zero in the bottom one, at the same x" — the two crossings
          // are computed from different arrays, so their agreement is worth
          // stating rather than assuming.
          finding: cross === null && crossQ === null
            ? 'neither frame crosses anywhere in this decision\u2019s range'
            : agree
              ? 'the two lines cross at ' + sig(crossQ) + ' and the margin reaches zero at the ' +
                'same point; past it the fill is the requirement being exceeded'
              : 'THE TWO FRAMES DISAGREE: the lines cross at ' +
                (crossQ === null ? 'no point drawn' : sig(crossQ)) +
                ' and the margin reaches zero at ' + (cross === null ? 'no point drawn' : sig(cross)),
          x: { label: lv.label + '  [' + lv.unit + ']' },
          panes: [
            {
              // THE UNIT IS THE FACE'S, NOT THE ENGINE'S, and only here. F10.7 is
              // modelled as a dimensionless Ratio throughout this tree — sfu is
              // not in the unit system — so the sweep reports '-' for it, and a
              // frame labelled "sustained F10.7 to design to [-]" is true and
              // unreadable. Every other panel in this subsystem writes sfu in
              // its axis title for the same reason; this one now does too, and
              // says so rather than looking like an oversight.
              y: { label: title.q + '  [' + title.u + ']' },
              series: [
                // THE AREA BETWEEN THE TWO IS THE MARGIN. It is the one thing
                // this panel is named for and it was the only thing not drawn:
                // a reader had two lines and had to hold the distance between
                // them in their head. First in the list, so both lines sit on
                // top of it.
                // AND SPLIT AT THE CROSSING, because the area left of it and the
                // area right of it are opposite facts. One fill made "this much
                // margin is unspent" and "this much requirement is exceeded" the
                // same wedge in the same ink, which is the one distinction the
                // panel exists to draw. Unspent wears the achieved curve's own
                // hue; exceeded wears the pink every bound in this face is drawn
                // in. Both end exactly at the interpolated crossing rather than
                // at the nearest swept point, so neither claims a sample it does
                // not have.
                ...[[true, INK.series[0], 0.10], [false, INK.bound, 0.13]].map(([w, c, a]) => {
                  const b = bandSide(w);
                  return { kind: 'band', x: b.x, y: b.y, y0: b.x.map(() => reqV),
                    colour: c, alpha: a };
                }),
                { name: 'what the record gives — ' + quantity, kind: 'line',
                  x, y: value, width: 2.2, row: ach },
                // THE SAME COLOUR THE ZERO RULE BELOW IS DRAWN IN, deliberately.
                // It is not a series competing with the curve — it is the line
                // the curve is measured against, which is the job every mark in
                // this face uses this colour for. The two frames then say the
                // same thing in the same ink: crossing the pink line above is
                // crossing the pink line below.
                { name: 'the requirement — ' + req, kind: 'line',
                  x, y: x.map(() => reqV), colour: INK.bound, dash: [6, 4], row: req },
              ],
              marks,
            },
            {
              y: { label: 'margin, signed fraction of the requirement  [-]' },
              series: [{ name: '', kind: 'line', x, y: margin, colour: INK.series[2], row: ach }],
              marks: [
                // Below zero the requirement is not met, whichever way it binds:
                // the margin is signed, so its sign is the verdict and the
                // region is the verdict drawn.
                { axis: 'y', to: 0, label: '', colour: INK.bound, alpha: 0.05 },
                { axis: 'y', at: 0, label: 'the requirement is exactly met',
                  colour: INK.bound },
              ].concat(marks),
            },
          ],
        },
        note: title.text + '\n\n' +
          'The horizontal axis is not a choice made here. /v1/levers measures every declared ' +
          'decision upstream of this closure at both ends of its own range and reports which moves ' +
          'the margin most; ' + lv.label + ' is that decision, and it spans ' +
          sig(lv.span) + ' of margin across its declared range. The requirement itself is skipped, ' +
          'and not because it is small — it is usually the largest — but because a margin is a ' +
          'fraction OF the bound, so moving the bound moves it by construction and says nothing ' +
          'about the sky.\n\n' +
          // Fixed places rather than sig(), which rounded 0.604 to "0.6" and
          // 60.4 per cent to "60". A margin is the number a review argues over
          // and it is read to the decimal.
          (now === null
            ? 'The engine did not return a margin for this pair.'
            : 'As the tree stands the margin is ' + now.toFixed(4) + ' — ' +
              (now < 0
                ? 'NEGATIVE, so this requirement is exceeded by ' +
                  (-now * 100).toFixed(1) + ' per cent of its own value.'
                : (now * 100).toFixed(1) + ' per cent of the requirement is unspent.')) +
          ' ' +
          (cross === null
            ? 'The margin does not reach zero anywhere in ' + lv.label + '’s declared range, so ' +
              'this decision cannot spend it on its own.'
            : 'It runs out at ' + sig(cross) + ' ' + (lv.unit === '-' ? '' : lv.unit) +
              ', which is where the two lines above cross. That is the number to carry into a ' +
              'review: past it the design is outside what it was built for.') +
          '\n\nThe top frame and the bottom one are the same fact twice, and both are here ' +
          'because they answer different questions. The crossing says WHERE; the signed fraction ' +
          'says HOW MUCH, in a unit that compares across the five closures — the requirements are ' +
          'in sfu and in Ap and cannot be set beside each other, and their margins can.',
      };
    },
  },

  // -------------------------------------------------------------------------
  // WHERE THE SOLAR DRIVERS FIRST BECOME PHYSICS.
  //
  // env_exospheric_temperature is the first row downstream of the sky that is a
  // quantity rather than a description of one, and everything this tree computes
  // about density, drag, decay and aero heating is downstream of it in turn. It
  // is also where §29.2's open question lands: the solar subsystem publishes a
  // daily MEAN Kp and a daily PEAK Kp per scenario, Jacchia 1971 takes one Kp,
  // and nothing in the tree says which. The third view is that question.
  //
  // EVERY NUMBER HERE COMES FROM THE ENGINE, including the shapes. No coefficient
  // of the relation appears in this file: the slopes in the first view are
  // MEASURED off three sweeps rather than read off the sheet, which is the defect
  // §21 found in `design` — a figure carrying a row's constants is a second copy
  // of that row and goes stale silently.
  {
    id: 'thermosphere',
    rows: ['env_exospheric_temperature', 'env_f107', 'env_f107a', 'env_kp'],
    engine: ['l3_solar_interface', 'env_f107', 'env_f107a', 'env_kp',
      'env_exospheric_temperature'],
    label: 'Thermosphere',
    draws: 'The exospheric temperature the drivers produce, and what moves it.',
    asks: 'How hot does the sky this subsystem publishes make the upper thermosphere?',
    controls: [
      { k: 'view', label: 'view', opts: [
        ['solar', 'against the flux'],
        ['kp', 'against Kp, one curve per scenario'],
        ['slot', 'the two Kp readings, side by side'],
        ['shape', 'where the geomagnetic term stops being linear'],
      ] },
    ],
    async data(o) {
      const v = (o && o.view) || 'solar';
      if (THERMO_CACHE[v]) return THERMO_CACHE[v];
      const pr = (async () => {
        if (v === 'solar') {
          // THREE WAYS TO RAISE THE FLUX, and they are different quantities.
          // Moving F10.7 alone is a single day departing from its own 81-day
          // mean; moving F10.7A alone is the mean moving under a fixed day;
          // moving both together is a sustained level. The relation treats the
          // three differently and the three slopes are what says so — measured
          // here, never stated.
          //
          // PROBED, NOT RUN, and the difference is the subject of §49. These
          // three slopes are facts about Jacchia 1971 — true at any design
          // point — and the flux stopped being a thing the design lets you move
          // when env_f107 began reading the solar subsystem. A run holding a
          // computed driver is now refused, correctly; a probe asks the
          // relation directly, which is what this view was always asking.
          //
          // The others are held at the point the design currently sits at, so
          // the curves pass through the marks drawn beside them.
          const at = await engineValues(['env_f107', 'env_f107a', 'env_kp']);
          const si = id => (at[id] && isFinite(at[id].si) ? at[id].si : null);
          const [f0, fa0, kp0] = [si('env_f107'), si('env_f107a'), si('env_kp')];
          if (f0 === null || fa0 === null || kp0 === null) return { refused: true };

          const N = 24;
          const both = [];
          for (let i = 0; i < N; i++) {
            const x = 60 + (400 - 60) * i / (N - 1);
            both.push(engineProbe('env_exospheric_temperature',
              { env_f107: x, env_f107a: x, env_kp: kp0 }).then(r => ({ x, y: r.si })));
          }
          const [fast, slow, sust] = await Promise.all([
            probeSweep('env_exospheric_temperature', 'env_f107', 60, 400, 60,
              { env_f107a: fa0, env_kp: kp0 }),
            probeSweep('env_exospheric_temperature', 'env_f107a', 60, 400, 60,
              { env_f107: f0, env_kp: kp0 }),
            Promise.all(both),
          ]);
          return { fast, slow, sust };
        }
        // THE SCENARIO VIEWS ASK THE CROSSING THEMSELVES. `data` runs beside
        // engineValues rather than after it, so it cannot see what the panel
        // declared in `engine`; one more run of the crossing is cheaper than
        // sequencing the two.
        const set = await engineValues(['l3_solar_interface']);
        if (v === 'kp') {
          // One Kp sweep per scenario, each taken AT that scenario's own flux.
          // A single sweep at the declared flux would draw one curve where the
          // subsystem publishes five places to stand.
          const curves = await Promise.all(SCEN.map(async sc => {
            const d = driversOf(set, sc.k);
            if (d.f107 === null || d.f107a === null) return { sc, d, sweep: null };
            // THE TWO DOTS ARE RUNS, NOT SWEPT POINTS, and that is not fussiness.
            // Reading them off the 46-point sweep put the worst-day gap at
            // 141.1 K while the next view, which runs the engine at the exact Kp,
            // put it at 140.6 — two views of one fact disagreeing in the first
            // decimal because one of them had interpolated. Both now run.
            const [sweep, m, k] = await Promise.all([
              probeSweep('env_exospheric_temperature', 'env_kp', 0, 9, 46,
                { env_f107: d.f107, env_f107a: d.f107a }),
              engineProbe('env_exospheric_temperature',
                { env_f107: d.f107, env_f107a: d.f107a, env_kp: d.kpMean }),
              engineProbe('env_exospheric_temperature',
                { env_f107: d.f107, env_f107a: d.f107a, env_kp: d.kpPeak }),
            ]);
            return { sc, d, sweep, tMean: m.si, tPeak: k.si };
          }));
          return { curves };
        }
        if (v === 'slot') {
          // Ten runs: each scenario at the day's mean Kp and at its worst slot,
          // with that scenario's own flux. The gap between them is the question.
          const pts = await Promise.all(SCEN.map(async sc => {
            const d = driversOf(set, sc.k);
            if (d.f107 === null || d.kpMean === null || d.kpPeak === null) return { sc, d };
            const [m, k] = await Promise.all([
              engineProbe('env_exospheric_temperature',
                { env_f107: d.f107, env_f107a: d.f107a, env_kp: d.kpMean }),
              engineProbe('env_exospheric_temperature',
                { env_f107: d.f107, env_f107a: d.f107a, env_kp: d.kpPeak }),
            ]);
            return { sc, d, tMean: m.si, tPeak: k.si };
          }));
          return { pts };
        }
        // shape: one sweep at the point the design sits at is enough — the
        // geomagnetic term does not depend on the flux, and showing that it
        // does not is part of what this view says.
        const now = await engineValues(['env_f107', 'env_f107a']);
        const pick = id => (now[id] && isFinite(now[id].si) ? now[id].si : null);
        if (pick('env_f107') === null || pick('env_f107a') === null) return { refused: true };
        return {
          kp: await probeSweep('env_exospheric_temperature', 'env_kp', 0, 9, 46,
            { env_f107: pick('env_f107'), env_f107a: pick('env_f107a') }),
        };
      })();
      THERMO_CACHE[v] = pr;
      return pr;
    },
    build(rec, o, extra, eng) {
      const v = o.view || 'solar';
      const dec = id => (eng[id] && isFinite(eng[id].si) ? eng[id].si : null);
      const decF = dec('env_f107'), decFa = dec('env_f107a'), decKp = dec('env_kp');
      const now = dec('env_exospheric_temperature');

      if (v === 'solar') return thermoSolar(extra, eng, decF, decFa, decKp, now);
      if (v === 'slot') return thermoSlot(extra, eng);
      if (v === 'shape') return thermoShape(extra, eng, decF, decFa, decKp);
      return thermoKp(extra, eng, decKp);
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
        marks.push({ axis: 'x', at: 80, label: 'March equinox', colour: INK.series[1] });
        marks.push({ axis: 'x', at: 266, label: 'September equinox', colour: INK.series[1] });
      }
      // HOISTED OUT OF THE NOTE, because the answer and the prose have to be the
      // same measurement. They were one expression inside the caption's closure,
      // which is fine until a second reader needs them and copies them.
      const vname = key === 'f107' ? 'F10.7' : key === 'ap' ? 'Ap' : 'the sunspot number';
      const unit = key === 'f107' ? 'sfu' : '';
      const hi = Math.max(...ys), lo = Math.min(...ys);
      const atHi = xs[ys.indexOf(hi)], atLo = xs[ys.indexOf(lo)];
      const gname = o.by === 'doy' ? '5-day bin' : o.by === 'year' ? 'year' : 'month';
      // THE GAP, POINTED AT. "The 2017 gap is 273 consecutive days and shows
      // here as a year drawn from nine months" is the last sentence of a
      // paragraph under a chart where the bar it is about looks like every other
      // bar. Counted from the record rather than asserted, and only drawn where
      // the record actually holds a gap that year — a hard-coded 273 would be a
      // second copy of a fact the bundle owns.
      const notes = [];
      if (o.by === 'year') {
        const gi = ks.indexOf(2017);
        if (gi >= 0) {
          const held = rec.days.filter(d => d.year === 2017 && d[key] !== null).length;
          if (held < 330) {
            notes.push({ x: xs[gi], y: ys[gi],
              text: 'a year drawn from ' + held + ' days, not 365' });
          }
        }
      }
      return {
        answer: !isFinite(hi / lo) || lo === 0
          ? { value: '\u2014', of: 'the record holds nothing to compare at this setting' }
          : { value: '\u00d7' + (hi / lo).toFixed(2),
              of: 'between the quietest ' + gname + ' of the record and the busiest \u2014 a mission '
                + 'is sized against wherever in that range it falls' },
        spec: {
          finding: ys.filter(v => v < overall).length + ' of the ' + ys.length + ' ' + gname +
            's sit below the record mean of ' + overall.toFixed(1) + ', and the highest is ' +
            (hi / lo).toFixed(1) + ' times the lowest',
          notes,
          x: { label: o.by === 'year' ? 'year' : o.by === 'doy' ? 'day of year  [5-day bins]' : 'year' },
          y: { label: (key === 'f107' ? 'F10.7  [sfu]' : key === 'ap' ? 'Ap  [-]' : 'sunspot number  [-]') + ', mean' },
          series: [{ name: '', kind: o.by === 'year' ? 'bars' : 'line', x: xs, y: ys }],
          marks,
        },
        note: (() => {
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
      // How much of the cloud sits below BOTH medians. Two independent drivers
      // put a quarter of their days there; the distance from a quarter is the
      // dependence, and it is the one thing a scatter of ten thousand dots does
      // not tell a reader by looking.
      const mf = quantile(withBoth.map(d => d.f107).sort((x, y) => x - y), 0.5);
      const ma = quantile(withBoth.map(d => d.ap).sort((x, y) => x - y), 0.5);
      const quad = 100 * withBoth.filter(d => d.f107 < mf && d.ap < ma).length / withBoth.length;
      // And whether the tail really is everywhere along the flux axis, counted
      // by decile rather than seen in a cloud of ten thousand dots.
      const fSorted = withBoth.map(d => d.f107).sort((x, y) => x - y);
      const edge = k => quantile(fSorted, k / 10);
      let stormDecs = 0;
      for (let k = 0; k < 10; k++) {
        const lo = edge(k), hi2 = k === 9 ? Infinity : edge(k + 1);
        if (withBoth.some(d => d.f107 >= lo && d.f107 < hi2 && d.ap >= 26)) stormDecs++;
      }
      return {
        // THE ANSWER IS A REFUSAL, AND IT SAYS SO. Rule 5: a refusal is never a
        // substitution. Putting the correlation in this slot would answer "what
        // are these drivers worth as density" with a number about something
        // else, which is precisely the reading the panel is here to prevent.
        answer: { value: 'no answer',
          of: 'nothing computes sys_space_environment_atmospheric_density \u2014 what is drawn is '
            + 'the precondition, not the density' },
        spec: {
          x: { label: 'F10.7  [sfu]' },
          // Ap FLOORS AT ZERO and this is the one panel whose data reaches it:
          // a quiet day really is Ap 0, so the 6 per cent padding the chart adds
          // below an undeclared minimum ran the axis down to -16.4 — a region of
          // an index that has no negative values. Five other panels already
          // declare this floor; this one did not, which is why it was the only
          // chart in the repository drawing space that cannot exist.
          y: { label: 'daily Ap  [-]', min: 0 },
          finding: quad.toFixed(1) + '% of the days sit below both medians, against the 25% two ' +
            'independent drivers would give, and storm-level Ap appears in ' + stormDecs +
            ' of the 10 flux deciles',
          series: [{ name: '', kind: 'dots', x: withBoth.map(d => d.f107), y: withBoth.map(d => d.ap), width: 1.1, alpha: 0.18 }],
        },
        note: 'NOTHING COMPUTES THIS ROW AND THIS FIGURE IS NOT IT — but the reason written here ' +
          'was wrong, and §28.3 of docs/MATLAB_PORT_PLAN.md corrects it. This note said the ' +
          'study’s density tab needs an atmosphere model belonging to a subsystem with nothing ' +
          'written in it. That was true once and is not now: env_mass_density is published and ' +
          'answers 6.63e-11 kg/m³, off env_exospheric_temperature, and every sweep the five legacy ' +
          'views need already works. What is still empty is THIS row — the layer-2 one, which ' +
          'nothing computes and three rows read — so the honest gap is a seam, not a model.\n\n' +
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
  // How uneven "unevenly" is, at the mildest level, measured off the bars drawn.
  const g1 = (series[0].y || []).filter(v => v !== null && isFinite(v) && v > 0);
  const evenness = g1.length > 1 ? Math.max(...g1) / Math.min(...g1) : null;
  // Which cycle is busiest, at each level, off the bars themselves.
  const topAt = series.map(sr => {
    let bi = -1;
    sr.y.forEach((v, i) => { if (v !== null && isFinite(v) && (bi < 0 || v > sr.y[bi])) bi = i; });
    return bi;
  });
  const oneWinner = topAt.length && topAt.every(i => i >= 0 && i === topAt[0]);
  return {
    answer: evenness === null
      ? { value: '\u2014', of: 'not enough cycles reach G1 to compare' }
      : { value: '\u00d7' + evenness.toFixed(1),
          of: 'between the busiest cycle and the quietest, in days a year at G1 or above' },
    spec: {
      finding: oneWinner
        ? 'cycle ' + per[topAt[0]].c.n + ' has the most days a year at all three levels'
        : 'the busiest cycle is not the same at every level: ' +
          LV.map(([, n], i) => n + ' \u2192 cycle ' +
            (topAt[i] < 0 ? '\u2014' : per[topAt[i]].c.n)).join(', '),
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
  const rate = byPhase.map((c, i) => (allPhase[i] ? 1000 * c / allPhase[i] : null));
  const drawnRates = rate.filter(v => v !== null && isFinite(v));
  const rHi = drawnRates.length ? Math.max(...drawnRates) : null;
  const rLo = drawnRates.length ? Math.min(...drawnRates) : null;
  return {
    answer: { value: n + ' days',
      of: 'F10.7 spikes over the record, in ' + runs + ' separate bursts \u2014 a spike being a day '
        + 'above its own 81-day mean by 2.5 of the record\u2019s own scatter' },
    spec: {
      finding: rHi === null
        ? 'no phase bin holds a spike'
        : 'the busiest phase bin is at ' + xs[rate.indexOf(rHi)].toFixed(2) +
          ' and holds ' + rHi.toFixed(0) + ' spike days per thousand, against ' +
          rLo.toFixed(0) + ' at the quietest',
      x: { label: 'cycle phase', min: 0, max: 1 },
      y: { label: 'spike days per 1000 days at that phase', min: 0 },
      series: [{ name: '', kind: 'bars', x: xs, y: rate }],
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
  const stPc = st.map((c, i) => (tot[i] ? 100 * c / tot[i] : null));
  const qtPc = qt.map((c, i) => (tot[i] ? 100 * c / tot[i] : null));
  // Where the storm share peaks, off the curve rather than from the prose.
  const drawn = stPc.filter(v => v !== null && isFinite(v));
  const pk = drawn.length ? Math.max(...drawn) : null;
  const atPk = pk === null ? null : xs[stPc.indexOf(pk)];
  const qDrawn = qtPc.filter(v => v !== null && isFinite(v));
  const qMin = qDrawn.length ? Math.min(...qDrawn) : null;
  const qMinAt = qMin === null ? null : xs[qtPc.indexOf(qMin)];
  const mirror = corr(stPc, qtPc);
  return {
    answer: pk === null
      ? { value: '\u2014', of: 'no phase bin holds a day with an Ap' }
      : { value: pk.toFixed(1) + '%',
          of: 'the peak storm share, at phase ' + atPk.toFixed(2)
            + ' \u2014 past maximum, on the declining side, and the declared epoch sits in it' },
    spec: {
      // Whether the two curves really are mirrors is a correlation, and it is
      // the one thing a reader would otherwise take from the shape by eye.
      finding: pk === null
        ? 'no phase bin holds a day with an Ap'
        : 'the storm share peaks at phase ' + atPk.toFixed(2) + ' and the quiet share bottoms at ' +
          (qMinAt === null ? '\u2014' : qMinAt.toFixed(2)) + '; the two move opposite at r = ' +
          (mirror === null ? '\u2014' : mirror.toFixed(2)),
      x: { label: 'cycle phase', min: 0, max: 1 },
      y: { label: 'share of days at that phase  [%]', min: 0 },
      series: [
        { name: 'storm (Ap \u2265 26)', kind: 'line', x: xs, y: stPc, colour: INK.bound },
        // CONTEXT. The view is called "where in a cycle a storm is likely" and
        // the quiet share is the mirror that makes the storm curve mean
        // something; two curves at one weight made it a pair of equals.
        { name: 'quiet (Ap \u2264 6)', kind: 'line', x: xs, y: qtPc, colour: INK.series[1],
          context: true },
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
    answer: !spread
      ? { value: '\u2014', of: 'no lead is reached by all of the cycles \u2014 shorten it until they overlap' }
      : { value: sig(spread[0]) + ' to ' + sig(spread[1]) + unit,
          of: 'how far the cycles disagree at ' + shared.toFixed(2)
            + ' yr, the longest lead all of them reach \u2014 sw_uncertainty_growth pools exactly this' },
    spec: {
      finding: !spread
        ? 'no lead is reached by all of the cycles'
        : 'at ' + shared.toFixed(2) + ' yr the highest cycle is ' + sig(spread[1]) + unit +
          ' and the lowest ' + sig(spread[0]) + ' \u2014 the curves do not share a shape',
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
function byIssueYear(fc, byDay, tOf, strict, leaky) {
  const LO = 1, HI = 14;
  //: A year with few pairs produces a skill that is arithmetic rather than
  //: evidence — 2010 has 25 of them and scores -11.8. Dropped, and counted in
  //: the note, on the same principle the Repeatability panel drops thin bins.
  const MIN = 200;
  const acc = new Map();
  for (const r of fc) {
    const L = +r.lead_days;
    if (!(L >= LO && L <= HI) || r.f107 === null) continue;
    const tt = tOf.get(r.target_date);
    const obs = tt === undefined ? undefined : byDay.get(tt);
    if (obs === undefined) continue;
    const y = +String(r.issue_date).slice(0, 4);
    if (!isFinite(y)) continue;
    if (!acc.has(y)) acc.set(y, { e2: 0, se: 0, n: 0, e2s: 0, p2s: 0, ns: 0, e2l: 0, p2l: 0, nl: 0 });
    const a = acc.get(y), e = +r.f107 - obs;
    // Bias and RMS error over every pair; each skill over the pairs its own
    // baseline reaches. Same rule as the by-lead view, for the same reason.
    a.e2 += e * e; a.se += e; a.n++;
    const ps = strict(r.issue_date);
    if (ps !== null) { a.e2s += e * e; a.p2s += (ps - obs) * (ps - obs); a.ns++; }
    const pl = leaky(r.issue_date);
    if (pl !== null) { a.e2l += e * e; a.p2l += (pl - obs) * (pl - obs); a.nl++; }
  }
  const years = [...acc.keys()].sort((a, b) => a - b);
  const thin = years.filter(y => acc.get(y).n < MIN);
  //: A dropped year is a HOLE, not an absence. Filtering the thin years out of
  //: the series entirely leaves the line joining 2008 straight to 2011, and
  //: that segment reads as two years of evidence rather than as the gap it is.
  //: A null breaks the line here the same way it does everywhere else.
  const span = [];
  for (let y = years[0]; y <= years[years.length - 1]; y++) span.push(y);
  const ok = y => acc.has(y) && acc.get(y).n >= MIN;
  const col = f => span.map(y => (ok(y) ? f(acc.get(y)) : null));
  const skS = col(a => (a.ns && a.p2s ? 1 - (a.e2s / a.ns) / (a.p2s / a.ns) : null));
  const skL = col(a => (a.nl && a.p2l ? 1 - (a.e2l / a.nl) / (a.p2l / a.nl) : null));
  const bias = col(a => a.se / a.n);
  const rmse = col(a => Math.sqrt(a.e2 / a.n));
  const nObs = col(a => a.n), nStr = col(a => a.ns), nLk = col(a => a.nl);
  const kept = span.filter((y, i) => skS[i] !== null);
  const keptS = kept.map(y => {
    const a = acc.get(y);
    return a.ns && a.p2s ? 1 - (a.e2s / a.ns) / (a.p2s / a.ns) : null;
  });
  const worst = kept.length ? kept[keptS.indexOf(Math.min(...keptS))] : null;
  const best = kept.length ? kept[keptS.indexOf(Math.max(...keptS))] : null;
  // The worst annual bias, measured off the middle frame. Not the skill: the
  // year-to-year skill swing is mostly the record's difficulty, and the bias is
  // the one line here a design is read off wrongly.
  const low = bias.reduce((b, v, i) =>
    (v !== null && isFinite(v) && (b < 0 || v < bias[b]) ? i : b), -1);
  return {
    answer: low < 0
      ? { value: '\u2014', of: 'no year holds enough usable pairs to score' }
      : { value: (bias[low] > 0 ? '+' : '') + bias[low].toFixed(1) + ' sfu',
          of: 'the worst annual bias, in ' + span[low] + ' \u2014 the outlook came in LOW, which is '
            + 'the direction that costs propellant' },
    spec: {
      finding: 'of the ' + kept.length + ' years with enough pairs to score, ' +
        keptS.filter(v => v !== null && v > 0).length + ' beat persistence and ' +
        bias.filter(v => v !== null && isFinite(v) && v < 0).length +
        ' came in low; the RMS frame tracks the level, not the skill',
      x: { label: 'the calendar year the outlook was issued in', fmt: v => String(Math.round(v)) },
      panes: [
        {
          y: { label: 'skill against persistence  [-]' },
          series: [
            { name: 'vs. last obs BEFORE issue', kind: 'line', x: span, y: skS, n: nStr },
            { name: 'vs. obs ON the issue date (leaks)', kind: 'line', x: span, y: skL,
              n: nLk, colour: INK.series[1], dash: [5, 3], context: true },
          ],
          marks: scoreBaseline('skill'),
        },
        {
          y: { label: 'mean signed error, forecast − observed  [sfu]' },
          series: [{ name: '', kind: 'line', x: span, y: bias, n: nObs }],
          marks: scoreBaseline('bias'),
        },
        {
          // Zero on the axis and the area filled, for the reason the by-lead
          // view carries: this is a size measured from perfect, not a score
          // against a baseline.
          y: { label: 'RMS error  [sfu]', min: 0 },
          series: [{ name: '', kind: 'line', x: span, y: rmse, n: nObs, fill: true }],
          marks: scoreBaseline('rmse'),
        },
      ],
    },
    note: 'One point per calendar year, scored on leads ' + LO + ' to ' + HI + ' ONLY. That ' +
      'restriction is what makes the years comparable and it is not cosmetic: the archive is not ' +
      'uniform, and 2004 and 2007 carry no row past lead 14 at all while 2011 onward carry a ' +
      'balanced mix. Scoring every lead together would draw the history of the archive and label ' +
      'it the skill of the forecaster. The year is the year of ISSUE, so a December outlook is ' +
      'counted against December even where it verifies into January.\n\n' +
      (thin.length
        ? thin.length + ' year(s) are dropped for holding fewer than ' + MIN + ' usable pairs — ' +
          thin.join(', ') + '. They are not quiet years, they are thin ones, and the arithmetic on ' +
          'them is violent: 2010 holds 25 pairs and scores -11.8. The line BREAKS at a dropped ' +
          'year rather than stepping over it.\n\n'
        : '') +
      (worst !== null
        ? 'Skill is worst in ' + worst + ' and best in ' + best + '. A year at solar minimum is the ' +
          'hard case for a forecaster and the easy one for persistence — when the flux is flat, ' +
          'assuming nothing changes is very nearly right — so 2008 scoring below the red line is ' +
          'the record behaving, not the outlook failing. Read it against the frame below: its RMS ' +
          'error that year is about 3.5 sfu, the smallest in the series, which is the whole point ' +
          'of stacking them.\n\n'
        : '') +
      'Bias runs strongly negative through 2022 to 2024, the rise of cycle 25: the outlook came in ' +
      'LOW by 7 to 9 sfu a year while activity was climbing. A design reading it there gets a ' +
      'thinner atmosphere than it will fly, which is the direction that costs propellant rather ' +
      'than the one that wastes it. RMS error tracks the LEVEL rather than the difficulty — ' +
      'smallest at the 2008 and 2018 minima, largest through solar maximum — because a bigger ' +
      'number has bigger errors.',
  };
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
 * SKILL GETS A SHADED SIDE AND BIAS DOES NOT, and the difference is what zero
 * MEANS on each. On the skill frame zero is a verdict — below it the outlook is
 * worse than not bothering — so the side is worth a wash that says so without
 * being read, and the region came in on the by-lead view alone, which is
 * exactly how the two views start to differ, so it lives here with its rule.
 * On the bias frame zero is a direction rather than a verdict: an outlook that
 * reads low is not a failing outlook, it is a fact a design has to carry.
 * Shading it also fails the test every region here has to pass, which is that
 * it be a REGION — the bias is negative at all 27 leads, so the wash covers the
 * whole frame, and a frame that is entirely shaded has said nothing. The label
 * on the rule carries the direction instead.
 *
 * One function rather than one expression per view, because the by-lead and
 * by-year views ask the same question of the same metric and two copies is two
 * places for them to start answering it differently.
 */
function scoreBaseline(metric) {
  if (metric === 'skill') {
    return [
      { axis: 'y', to: 0, label: '', colour: INK.bound, alpha: 0.05 },
      { axis: 'y', at: 0, label: 'no better than persistence', colour: INK.bound },
    ];
  }
  if (metric === 'bias') {
    return [{ axis: 'y', at: 0, label: 'unbiased — above is high, below is LOW',
      colour: INK.bound }];
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
    answer: { value: quantile(gaps, 0.5) + ' days',
      of: 'the median gap between one outlook and the next \u2014 how stale the newest one already '
        + 'is on a typical day, which no lead_days column says' },
    spec: {
      finding: 'the tallest bar is at ' + xs[xs.map(k => hist.get(k))
        .indexOf(Math.max(...xs.map(k => hist.get(k))))] + ' days, and ' +
        (100 * gaps.filter(g => g > 1).length / gaps.length).toFixed(0) +
        '% of the gaps are longer than one day',
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
  const analogue = w.pk.y;
  const peak = Math.max(...hot);
  // How wide the filled window actually is, along its length — the fill's own
  // two edges, so the sentence and the area are the same measurement.
  const band = xs.map((_, k) => hot[k] - cold[k]).filter(v => v !== null && isFinite(v));
  // Where the analogue's own maximum climbs above the hot single-day design
  // value, which is the finding this fifth line is here for.
  let over = null;
  for (let k = 0; k < xs.length; k++) {
    if (analogue[k] !== null && hot[k] !== null && analogue[k] > hot[k]) { over = xs[k]; break; }
  }
  return {
    answer: { value: (REQ - peak >= 0 ? '+' : '') + (REQ - peak).toFixed(1) + ' sfu',
      of: 'margin against ' + o.reqf + '\u2019s ' + REQ.toFixed(0)
        + ' \u2014 the worst single day any declared window reaches is ' + peak.toFixed(1) },
    spec: {
      finding: 'the window is ' + sig(Math.max(...band)) + ' sfu wide at its widest and ' +
        sig(Math.min(...band)) + ' at its narrowest, and the dashed peak ' +
        (over === null ? 'stays under the hot single-day curve throughout'
          : 'crosses above it at ' + over.toFixed(1) + ' yr'),
      x: { label: 'mission length  [years]', min: xs[0], max: xs[xs.length - 1] },
      y: { label: 'F10.7 the window is designed to  [sfu]' },
      series: [
        // THE DESIGN WINDOW IS AN AREA AND THE NOTE ALREADY CALLED IT ONE: "the
        // band between the outer two is the design window". It was four lines
        // and a reader was asked to hold the outermost two in mind and subtract.
        // Filled between the hot single day and the cold single day, the window
        // is the thing on the canvas and the four rows are its edges.
        { kind: 'band', x: xs, y: hot, y0: cold, colour: INK.series[0], alpha: 0.10 },
        { name: 'sw_f107_design_short — hot, single day', kind: 'line', x: xs, y: hot,
          row: 'sw_f107_design_short' },
        { name: 'sw_f107_design_long — hot, sustained', kind: 'line', x: xs, y: hotLong,
          colour: INK.series[1], row: 'sw_f107_design_long' },
        { name: 'sw_f107_cold_long — cold, sustained', kind: 'line', x: xs, y: coldLong,
          colour: INK.series[2], row: 'sw_f107_cold_long' },
        { name: 'sw_f107_cold_short — cold, single day', kind: 'line', x: xs, y: cold,
          colour: INK.series[3], row: 'sw_f107_cold_short' },
        // Dashed, because it is not a design value: it is the analogue's own
        // ceiling, the thing the four solid curves are built from a mean of.
        // CONTEXT, and the note has always said why: it is NOT a design value.
        // Four design curves and a fifth line at the same weight read as five
        // design curves, which is the one misreading this line can cause.
        { name: 'sw_window_peak_level — the analogue’s own peak', kind: 'line',
          x: xs, y: analogue, colour: INK.series[5], dash: [7, 4], context: true,
          row: 'sw_window_peak_level' },
      ],
      marks: [
        // Above the bound is the side that fails, and the bound binds one way:
        // the requirement's sense is `<=`, so the region is everything over it.
        { axis: 'y', from: REQ, label: '', colour: INK.bound, alpha: 0.06 },
        { axis: 'y', at: REQ, label: 'required \u2264 ' + REQ.toFixed(0) + '  (' + o.reqf + ')',
          colour: INK.bound, row: o.reqf },
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
        + peak.toFixed(1) + '.' : 'exceeded inside the declared range, at ' + peak.toFixed(1) + '.') +
      '\n\nThe dashed line is not a design value. sw_window_peak_level is the MAXIMUM of the same ' +
      'cycle analogue these four are built on a mean of, over the same window, and it rises ' +
      'monotonically because a longer window can only contain more of the cycle. The design curves ' +
      'wander instead, because a window mean depends on where the window ends. ' +
      (over === null
        ? 'It stays under the hot single-day value across the whole declared range.'
        : 'They cross at about ' + over.toFixed(1) + ' years: past there the analogue\u2019s own ' +
          'peak is ABOVE the hot single-day design value, which is a design sized on a mean sitting ' +
          'under the thing it averages. That is worth a reviewer\u2019s attention and it is the ' +
          'reason this row is kept rather than removed.'),
  };
}


// ---------------------------------------------------------------------------
// the thermosphere views

const THERMO_CACHE = {};

/** The x axis the two scenario views share: five categories, cold to hot. */
const SCEN_X = {
  // `grid: false` for the reason XPAD carries: this axis is five names, and a
  // vertical rule between two of them marks a place no value can be read at.
  min: -0.3, max: 4.3, ticks: 5, grid: false,
  fmt: v => (SCEN[Math.round(v)] || {}).shown || '',
};

/** The slope of a swept curve, measured off its own ends. */
function slopeOf(sw) {
  const xs = sw.x.map(v => v / sw.x_factor), ys = sw.y.map(v => v / sw.y_factor);
  const ok = xs.map((x, i) => [x, ys[i]]).filter(p => p[1] !== null && isFinite(p[1]));
  if (ok.length < 2) return null;
  const a = ok[0], b = ok[ok.length - 1];
  return b[0] === a[0] ? null : (b[1] - a[1]) / (b[0] - a[0]);
}

/**
 * Thermosphere · against the flux, three ways, and the three slopes are the
 * finding.
 *
 * Raising F10.7 alone is one day departing from its own 81-day mean. Raising
 * F10.7A alone is the mean moving under a fixed day. Raising both together is a
 * sustained level. The relation answers differently for each, and how
 * differently is measured here off the three curves rather than read off the
 * sheet — this file contains no coefficient of it.
 */
function thermoSolar(extra, eng, decF, decFa, decKp, now) {
  const fx = sw => sw.x.map(v => v / sw.x_factor);
  const fy = sw => sw.y.map(v => v / sw.y_factor);
  const sust = extra.sust.filter(p => p.y !== null && isFinite(p.y));
  const mSust = sust.length > 1
    ? (sust[sust.length - 1].y - sust[0].y) / (sust[sust.length - 1].x - sust[0].x) : null;
  const mFast = slopeOf(extra.fast), mSlow = slopeOf(extra.slow);
  const k = n => (n === null ? '—' : n.toFixed(2));
  const si = eng.l3_solar_interface && isFinite(eng.l3_solar_interface.si)
    ? eng.l3_solar_interface.si : null;
  return {
    answer: now === null
      ? { value: '\u2014', of: 'env_exospheric_temperature did not answer' }
      : { value: now.toFixed(0) + ' K',
          // The caption used to argue §28.2's gap — a declared 150 here and a
          // computed 104 two panels along, with nothing comparing them. The
          // gap is closed: env_f107 reads the crossing now. So the caption
          // says whether the two AGREE, measured on the draw, and it will
          // start pointing at a discrepancy again the moment one appears.
          of: 'the sky the subsystem computes, at env_f107 = '
            + (decF === null ? '—' : decF.toFixed(1)) + ' \u2014 every density below it is downstream'
            + (si === null || decF === null ? ''
               : Math.abs(si - decF) < 0.05
                 ? ', which is what the crossing publishes'
                 : ', and the crossing publishes ' + si.toFixed(1) + ' sfu \u2014 they should agree') },
    spec: {
      finding: 'the three lines meet only at ' + (decF === null ? '\u2014' : decF.toFixed(0)) +
        ' sfu, where the day and its mean are declared equal, and the sustained slope is ' +
        k(mSust) + ' K per sfu against ' + k(mFast) + ' for a single day',
      x: { label: 'F10.7  [sfu]' },
      y: { label: 'exospheric temperature  [K]' },
      series: [
        { name: 'sustained — the day and its 81-day mean together', kind: 'line',
          x: sust.map(p => p.x), y: sust.map(p => p.y), width: 2.4,
          row: 'env_exospheric_temperature' },
        { name: 'one day alone, its 81-day mean held', kind: 'line',
          x: fx(extra.fast), y: fy(extra.fast), colour: INK.series[1],
          row: 'env_exospheric_temperature' },
        { name: 'the 81-day mean alone, the day held', kind: 'line',
          x: fx(extra.slow), y: fy(extra.slow), colour: INK.series[2],
          row: 'env_exospheric_temperature' },
      ],
      // THE TWO SKIES, ON THE AXIS, BESIDE EACH OTHER. The declared constant the
      // density chain is actually sized on, and what the solar subsystem computes
      // for its hot sustained scenario. §28.2 is the distance between these two
      // marks, and putting the second on the canvas rather than only in the prose
      // is also what lets check 2b see that this panel reads the crossing at all.
      marks: [
        // §28.2 IS THE DISTANCE BETWEEN THE TWO MARKS, so it is drawn as the
        // distance rather than left as two ticks a reader subtracts. Open on
        // neither side: this region has two measured ends and both are named.
        (decF === null || !(eng.l3_solar_interface && isFinite(eng.l3_solar_interface.si)))
          ? null
          : { axis: 'x',
            from: Math.min(decF, eng.l3_solar_interface.si),
            to: Math.max(decF, eng.l3_solar_interface.si),
            label: '', colour: INK.bound, alpha: 0.05 },
        decF === null ? null : { axis: 'x', at: decF,
          label: 'env_f107 = ' + decF.toFixed(1) + ', computed from the crossing',
          colour: INK.bound, row: 'env_f107' },
        // DRAWN ONLY WHERE IT IS A SECOND PLACE. Since §49 wired env_f107 to
        // read the crossing the two marks land on the same sfu, and a second
        // tick labelled "the solar subsystem says 104.1" beside "env_f107 =
        // 104" reads as a disagreement to a reader who does not know they are
        // one number. It reappears the moment they part, which is the only
        // time it says anything.
        (!(eng.l3_solar_interface && isFinite(eng.l3_solar_interface.si))
         || decF === null || Math.abs(eng.l3_solar_interface.si - decF) < 0.05) ? null
          // INK.mark, not a series slot. Slot 2 is the green the "81-day mean
          // alone" line is drawn in three inches to the right, and a mark
          // wearing a series' colour invites the reader to pair the two.
          : { axis: 'x', at: eng.l3_solar_interface.si,
            label: 'the solar subsystem says ' + eng.l3_solar_interface.si.toFixed(1),
            colour: INK.mark, row: 'l3_solar_interface' },
      ].filter(Boolean),
    },
    note: 'Three ways to raise the flux, and the relation answers differently for each — which ' +
      'is the point of drawing all three rather than one. Every slope below is MEASURED off ' +
      'these curves, not taken from the sheet: a figure carrying a row\u2019s constants is a ' +
      'second copy of that row, and §21 found three of those going stale in the design panel.\n\n' +
      'A SUSTAINED rise, where the day and its 81-day mean move together, is worth ' + k(mSust) +
      ' K per sfu. A SINGLE DAY departing from a fixed mean is worth only ' + k(mFast) +
      ' K per sfu, because the thermosphere cannot fully respond to one day. And the 81-day ' +
      'MEAN moving under a fixed day is worth ' + k(mSlow) + ' — less than the sustained slope, ' +
      'because raising the mean while holding the day shrinks the departure at the same time.\n\n' +
      'The three cross where the design sits: env_f107 and env_f107a are both ' +
      (decF === null ? '—' : decF.toFixed(1)) + ', so there is no departure there and all three ' +
      'agree. THAT NUMBER IS COMPUTED, not declared \u2014 it is what the solar subsystem ' +
      'publishes for its hot sustained scenario, ' +
      (eng.l3_solar_interface && isFinite(eng.l3_solar_interface.si)
        ? eng.l3_solar_interface.si.toFixed(2) : '—') +
      ' sfu, carried inward through layer 2. §28.2 was the gap where a declared 150 sat here ' +
      'instead and nothing compared the two; §49 is the wiring that closed it, and this is ' +
      'where it is felt, because every density in this tree is downstream of the temperature ' +
      'this curve gives.' +
      (now === null ? '' : ' As the tree stands the answer is ' + now.toFixed(1) + ' K.'),
  };
}

/**
 * Thermosphere · against Kp, one curve per scenario, with both readings marked.
 *
 * This is §29.2 as a picture. Each curve is swept at its own scenario's flux, so
 * the five are five places to stand rather than one curve relabelled; the two
 * dots on each are the day's mean Kp and the day's worst slot, and the vertical
 * distance between them is what choosing one over the other costs.
 */
function thermoKp(extra, eng, decKp) {
  const series = [];
  let worst = null;
  const lines = [];
  for (const c of extra.curves) {
    if (!c.sweep) continue;
    const xs = c.sweep.x.map(v => v / c.sweep.x_factor);
    const ys = c.sweep.y.map(v => v / c.sweep.y_factor);
    series.push({ name: c.sc.shown, kind: 'line', x: xs, y: ys, colour: c.sc.colour });
    lines.push(ys);
    const tm = (c.tMean === undefined || c.tMean === null) ? null : c.tMean;
    const tp = (c.tPeak === undefined || c.tPeak === null) ? null : c.tPeak;
    if (tm !== null && tp !== null) {
      // The two readings as dots ON the curve they belong to. Unnamed so the
      // legend stays five entries rather than ten, and `aside` because they are
      // marks and not a sixth measurement: every value in them is a point the
      // curve beneath already carries. Without it the table grew five columns
      // all headed "exospheric temperature [K]", which is what an unnamed series
      // falls back to, and the readout announced each value twice.
      series.push({ name: '', kind: 'dots', x: [c.d.kpMean, c.d.kpPeak], y: [tm, tp],
        colour: c.sc.colour, width: 5, alpha: 1, aside: true });
      if (worst === null || tp - tm > worst.d) {
        worst = { sc: c.sc, d: tp - tm, tm, tp, km: c.d.kpMean, kp: c.d.kpPeak };
      }
    }
  }
  // How far apart the two closest curves ever get, measured rather than eyeballed.
  let close = null;
  if (lines.length > 1) {
    close = 0;
    for (let i = 0; i < lines[0].length; i++) {
      const a = lines[0][i], b = lines[1][i];
      if (a !== null && b !== null) close = Math.max(close, Math.abs(b - a));
    }
  }
  return {
    answer: worst === null
      ? { value: '\u2014', of: 'no scenario answered at both of its Kp' }
      : { value: worst.d.toFixed(1) + ' K',
          of: 'the widest gap between a scenario\u2019s two Kp readings, at the ' + worst.sc.shown
            + ' \u2014 ' + (100 * worst.d / worst.tm).toFixed(1) + ' per cent of the temperature, '
            + 'and nothing in this tree says which slot to take' },
    spec: {
      finding: 'all ' + lines.length + ' curves bend upward, and the two lightest stay within ' +
        (close === null ? '\u2014' : close.toFixed(1) + ' K') +
        ' of each other across the whole Kp range',
      x: { label: 'Kp  [-]', min: 0, max: 9 },
      y: { label: 'exospheric temperature  [K]' },
      series,
      marks: decKp === null ? [] : [{ axis: 'x', at: decKp,
        label: 'env_kp = ' + decKp.toFixed(0), colour: INK.bound, row: 'env_kp' }],
    },
    note: 'One curve per scenario, each swept at that scenario\u2019s OWN flux — so these are ' +
      'five places to stand and not one curve drawn five times. The two dots on each are the ' +
      'two Kp the solar subsystem publishes for it: the day\u2019s MEAN slot and the day\u2019s ' +
      'WORST three-hour slot.\n\n' +
      'THE VERTICAL GAP BETWEEN A PAIR OF DOTS IS AN UNANSWERED QUESTION. Jacchia 1971 takes one ' +
      'Kp and nothing in this tree says which of the two it should be. ' +
      (worst === null ? '' :
        'It costs most at the ' + worst.sc.shown + ' scenario: Kp ' + worst.km.toFixed(2) +
        ' gives ' + worst.tm.toFixed(1) + ' K and Kp ' + worst.kp.toFixed(2) + ' gives ' +
        worst.tp.toFixed(1) + ' K, a difference of ' + worst.d.toFixed(1) + ' K — ' +
        (100 * worst.d / worst.tm).toFixed(1) + ' per cent of the temperature every density in ' +
        'this tree is built from, and more than that in the density itself.') +
      '\n\nTHE TWO LIGHTEST CURVES VERY NEARLY COINCIDE, and that is the data rather than the ' +
      'drawing: the quietest day and the cold sustained level differ by ' +
      (close === null ? '\u2014' : close.toFixed(1) + ' K') + ' across the whole Kp range, ' +
      'because the cold day rides on the cold mean and barely departs from it. Five curves are ' +
      'drawn and four are easy to separate.' +
      '\n\nThe curves bend upward because Kp is a quasi-logarithmic index and the relation ' +
      'carries an exponential to match it. The fourth view is that bend on its own.',
  };
}

/**
 * Thermosphere · the two Kp readings side by side.
 *
 * The same fact as the dots in the third view, with the curves taken away so the
 * only thing left is the gap. Ten runs of the engine, each at a scenario's own
 * three drivers.
 */
function thermoSlot(extra, eng) {
  const xs = SCEN.map((_, i) => i);
  const tm = extra.pts.map(p => (p.tMean === undefined ? null : p.tMean));
  const tp = extra.pts.map(p => (p.tPeak === undefined ? null : p.tPeak));
  const gaps = tm.map((v, i) => (v === null || tp[i] === null ? null : tp[i] - v));
  const ok = gaps.filter(g => g !== null);
  const wi = gaps.reduce((b, g, i) => (g !== null && (b < 0 || g > gaps[b]) ? i : b), -1);
  return {
    answer: !ok.length
      ? { value: '—', of: 'no scenario answered' }
      : { value: Math.max(...ok).toFixed(1) + ' K',
          of: 'the most the choice of Kp slot is worth — at the ' +
            (wi < 0 ? 'worst scenario' : SCEN[wi].shown) + ', and nothing says which slot to use' },
    spec: {
      aspect: 1.5,
      finding: !ok.length
        ? 'no scenario answered at both of its Kp'
        : 'the worst-slot line is above the mean-slot one at all ' + ok.length +
          ' scenarios, and the gap grows from ' + Math.min(...ok).toFixed(1) + ' K to ' +
          Math.max(...ok).toFixed(1) + ' K across them',
      x: { label: 'scenario', ...SCEN_X },
      y: { label: 'exospheric temperature  [K]' },
      series: [
        // THE GAP IS THE QUESTION. §29.2 asks which of the two Kp the design is
        // driven by; the answer's size is the area between these lines, and it
        // was the one thing the picture made the reader measure by eye.
        { kind: 'band', x: xs, y: tp, y0: tm, colour: INK.series[3], alpha: 0.12 },
        { name: 'at the day\u2019s mean Kp', kind: 'line', x: xs, y: tm, width: 2.2 },
        // NOT INK.series[4]. That slot is #c2185b, which this face uses for the
        // line a value is MEASURED AGAINST — every zero rule, every requirement,
        // every "exact agreement" mark, and the env_kp mark in this panel's own
        // previous view. A data series wearing it would read as a threshold.
        { name: 'at the day\u2019s worst three-hour slot', kind: 'line', x: xs, y: tp,
          colour: INK.series[3], dash: [6, 4], width: 2.2 },
      ],
    },
    note: 'The same fact as the dots in the previous view with the curves taken away, so that the ' +
      'only thing left is the gap. Each point is a run of the engine at one scenario\u2019s own ' +
      'three drivers — its F10.7, its 81-day mean, and one of its two Kp — so nothing here is ' +
      'interpolated off a curve.\n\n' +
      'The gap runs from ' + (ok.length ? Math.min(...ok).toFixed(1) : '—') + ' K at the quiet end ' +
      'to ' + (ok.length ? Math.max(...ok).toFixed(1) : '—') + ' K' +
      (wi < 0 ? '' : ' at the ' + SCEN[wi].shown) + '. It widens toward the hot end because the ' +
      'two slots diverge there AND because the relation\u2019s geomagnetic term is exponential: ' +
      'the same difference in Kp buys more temperature the higher up the scale it sits.\n\n' +
      'NEITHER LINE IS MORE CORRECT THAN THE OTHER. The mean is what a daily-averaged density ' +
      'model wants and what the legacy run used — its header says Kp slot \u2018mean\u2019. The ' +
      'peak is what a vehicle actually meets on the worst pass of the day. Both are defensible ' +
      'and they are different designs, which is why §30 B1 makes it a declared row somebody signs ' +
      'rather than a column a consumer happens to pick up.',
  };
}

/**
 * Thermosphere · where the geomagnetic term stops being linear.
 *
 * Nothing here knows the relation. The curve is T(Kp) minus T(0) off one sweep,
 * and the straight line is drawn through that curve's own first two usable
 * points — so the departure is measured against the measurement rather than
 * against a coefficient copied out of the sheet.
 */
function thermoShape(extra, eng, decF, decFa, decKp) {
  const sw = extra.kp;
  const xs = sw.x.map(v => v / sw.x_factor);
  const ys = sw.y.map(v => v / sw.y_factor);
  const i0 = ys.findIndex(v => v !== null && isFinite(v));
  if (i0 < 0) throw new Error('the Kp sweep returned nothing to draw');
  const base = ys[i0];
  const d = ys.map(v => (v === null || !isFinite(v) ? null : v - base));
  // The straight line the low end sets, through the first usable point and the
  // one nearest Kp 2 — low enough that the exponential is still negligible.
  let iRef = i0;
  for (let i = 0; i < xs.length; i++) {
    if (d[i] !== null && Math.abs(xs[i] - 2) < Math.abs(xs[iRef] - 2)) iRef = i;
  }
  const m = xs[iRef] === xs[i0] ? 0 : (d[iRef] - d[i0]) / (xs[iRef] - xs[i0]);
  const lin = xs.map(x => m * (x - xs[i0]));
  const last = d.length - 1;
  const excess = d[last] === null ? null : d[last] - lin[last];
  // Where the measured curve first stands clear of the straight line by more
  // than a fiftieth of its final departure — "they separate above Kp 4" said as
  // a measurement rather than as an impression of the drawing.
  let splitAt = null;
  if (excess !== null && excess > 0) {
    for (let i = 0; i <= last; i++) {
      if (d[i] === null) continue;
      if (d[i] - lin[i] > excess / 50) { splitAt = xs[i]; break; }
    }
  }
  return {
    answer: excess === null
      ? { value: '\u2014', of: 'the sweep returned nothing at the top of the Kp range' }
      : { value: excess.toFixed(1) + ' K',
          of: 'how far the measured curve stands above the straight line its own quiet end sets, '
            + 'by Kp ' + xs[last].toFixed(0) + ' \u2014 that is the exponential, measured' },
    spec: {
      finding: 'the two lie on top of each other up to Kp ' + (splitAt === null ? '\u2014'
        : splitAt.toFixed(1)) + ' and separate above it, reaching ' +
        (excess === null ? '\u2014' : excess.toFixed(0) + ' K') + ' apart at the top',
      x: { label: 'Kp  [-]', min: 0, max: 9 },
      y: { label: 'temperature the geomagnetic term adds  [K]', min: 0 },
      series: [
        // FILLED TO ZERO. This quantity is a temperature the geomagnetic term
        // ADDS, measured from its own value at Kp 0, so the area under the
        // curve is the thing being reported and the axis already starts at
        // zero. A lone stroke left that size to be read off the gridlines.
        { name: 'measured', kind: 'line', x: xs, y: d, width: 2.4, fill: true,
          row: 'env_exospheric_temperature' },
        { name: 'the straight line the quiet end sets', kind: 'line', x: xs, y: lin,
          colour: INK.muted, width: 1.2, dash: [5, 4], context: true },
      ],
    },
    note: 'How much of the exospheric temperature the geomagnetic term is responsible for, against ' +
      'Kp, measured as the swept answer minus its own value at Kp 0. The dashed line is drawn ' +
      'through this curve\u2019s own quiet end — the point nearest Kp 2, where the exponential is ' +
      'still small — so the departure above is measured against the measurement and not against a ' +
      'coefficient copied out of the sheet. Nothing in this file knows the relation.\n\n' +
      'THE TWO AGREE AT THE QUIET END AND SEPARATE AT THE TOP. By Kp ' + xs[last].toFixed(0) +
      ' the measured curve is ' + (excess === null ? '—' : excess.toFixed(1)) + ' K above the ' +
      'straight line. That is the exponential, and it is why the relation has one: Kp is a ' +
      'quasi-logarithmic index, so equal steps up the scale are not equal amounts of energy, and ' +
      'a purely linear geomagnetic term would be right at quiet levels and would under-read ' +
      'every storm.\n\n' +
      'The sweep is taken at the declared flux, and that is not a simplification: the ' +
      'geomagnetic term of this relation does not depend on the flux at all, so this curve is ' +
      'the same shape at every scenario. The previous view is where the scenarios differ.',
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
  // How often the raw line actually cuts the smoother, and how far it ever
  // strays — the two things a reader takes from this picture by eye.
  let crossings = 0, maxDev = 0, prevSign = 0;
  for (const [a2, b2] of both) {
    const dv = a2 - b2;
    if (Math.abs(dv) > maxDev) maxDev = Math.abs(dv);
    const sgn = dv > 0 ? 1 : dv < 0 ? -1 : 0;
    if (sgn && prevSign && sgn !== prevSign) crossings++;
    if (sgn) prevSign = sgn;
  }
  return {
    answer: { value: sig(resid) + unit,
      of: 'the rms a month keeps once the cycle is smoothed out of it \u2014 the part of '
        + vname + ' a design cannot plan around' },
    spec: {
      finding: 'the monthly line crosses the smoother ' + crossings +
        ' times, and is never further from it than ' + sig(maxDev) + unit,
      x: { label: 'year' },
      y: { label: (key === 'f107' ? 'F10.7  [sfu]' : key === 'ap' ? 'Ap  [-]' : 'sunspot number  [-]') + ', monthly' },
      series: [
        // SLOT 0, NOT A GOLD OF ITS OWN. The palette is six hues assigned in a
        // fixed order and never extended; this line was the one place in the
        // face wearing a seventh, and a seventh hue is also a hue with no dark
        // counterpart validated for it.
        { name: 'monthly mean', kind: 'line', x: xs, y: raw, width: 1.1,
          colour: INK.series[0] },
        { name: '13-month smoother', kind: 'line', x: xs, y: sm, width: 2.2, colour: INK.text },
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
  const i7 = ks.indexOf(7);
  const t7 = tableAt(7), m7 = i7 < 0 ? null : med[i7];
  let aboveMed = 0, above90 = 0;
  ks.forEach((kp, i) => {
    const t = tableAt(kp);
    if (t === null) return;
    if (med[i] !== null && t > med[i]) aboveMed++;
    if (p90[i] !== null && t > p90[i]) above90++;
  });
  return {
    answer: m7 === null || !m7 || t7 === null
      ? { value: '\u2014', of: 'the record holds no day whose worst slot reached Kp 7' }
      : { value: '\u00d7' + (t7 / m7).toFixed(1),
          of: 'the published table\u2019s ap over the median day\u2019s, at Kp 7 \u2014 the bias '
            + 'sw_kp_slot_bias exists for, and why the table is read as a ceiling' },
    spec: {
      // The claim panels/climate.toml makes about this view, counted at every Kp
      // the record holds rather than read off the drawing at one of them.
      finding: 'the published table is above the median day at ' + aboveMed + ' of the ' +
        ks.length + ' Kp the record holds, and above the 90th percentile at ' + above90,
      x: { label: 'Kp reached that day  [worst three-hourly slot]', min: 0, max: 9 },
      y: { label: 'daily Ap  [-], log scale', log: true },
      series: [
        // THE SPREAD, FILLED. Two thin grey lines is two more curves to tell
        // apart in a frame that already has three; the 10th to 90th is one
        // quantity — how wide a day's Ap can be for a given worst slot — and
        // filling it says so, and says it behind the median rather than beside
        // it. The edges stay: an area with no boundary invites reading its top
        // as a maximum, and the 90th is not one.
        { kind: 'band', x: ks, y: p90, y0: p10, colour: INK.muted, alpha: 0.13 },
        { name: 'median daily Ap', kind: 'line', x: ks, y: med },
        { name: '10th and 90th percentile', kind: 'line', x: ks, y: p10, colour: INK.muted,
          width: 1, context: true },
        { name: '', kind: 'line', x: ks, y: p90, colour: INK.muted, width: 1, context: true },
        { name: 'published ap at that Kp', kind: 'line', x: ks, y: ks.map(tableAt), colour: INK.bound, dash: [5, 4] },
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
 * WHAT A READER HAS DONE TO A FIGURE, kept per host rather than per panel.
 *
 * One node page can hold one figure, but the solar face can hold several, and a
 * view is a property of the picture somebody is looking at rather than of the
 * panel definition. Keyed on the host element, so navigating away drops it: a
 * zoom that survived a change of row would be a frame showing two years of a
 * quantity that no longer has years.
 *
 * Reset when the controls change, for the same reason. The window a reader
 * chose on a lead axis means nothing on a phase axis.
 */
const VIEWS = new WeakMap();
const viewOf = host => {
  let v = VIEWS.get(host);
  if (!v) { v = { zoom: null, hidden: new Set(), pinned: null }; VIEWS.set(host, v); }
  return v;
};

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

function optsFor(p, rowId) {
  if (!state.opts[p.id]) {
    state.opts[p.id] = Object.fromEntries(p.controls.map(c => [c.k, c.opts[0][0]]));
  }
  // A FIGURE OPENED FROM A ROW SHOWS THAT ROW.
  //
  // `closure` draws one required/achieved pair at a time and lives on all ten of
  // their rows, so without this a reader arriving at l3_solar_ach_03 — having
  // clicked it precisely because they want the survival closure — met a picture
  // of the F10.7 one and had to find the control. The panel says which of its
  // views belongs to which row and the page says which row it is; neither knows
  // alone.
  //
  // Only on a call that names a row. `redraw` passes none, so a reader's own
  // choice of control survives every redraw and is overridden only by their
  // navigating to a different row — which is them asking for that row.
  if (rowId && p.openAt) p.openAt(rowId, state.opts[p.id]);
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
export function drawRowFigure(host, panelId, rowId) {
  const p = PANELS.find(x => x.id === panelId);
  if (!host || !p) return;
  const o = optsFor(p, rowId);
  host.innerHTML = panelBody(p, o);
  wirePanel(host, p, o, () => drawRowFigure(host, panelId));
}

/** The caption, the controls and the surface. Shared by both callers. */
function panelBody(p, o) {
  return '<p class="caption"><b>' + esc(p.asks) + '</b></p>' +
    // THE ANSWER, WHERE THE QUESTION IS.
    //
    // A panel has always shown its question twice — `asks` in bold and `draws`
    // in grey — and never its answer. A reader met a question, a line, and then
    // a paragraph, and the paragraph was where the work was: §34 measured the
    // median view at 179 words of prose against 37 characters of text inside the
    // frame. The number is not new; every build already computes it for that
    // paragraph. Putting it here means a reader has the answer before they start
    // decoding the picture, and the picture then shows WHY rather than having to
    // be read first.
    '<p class="caption sw-answer"></p>' +
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
    '<canvas class="plot sw-panel" width="980" height="420" ' +
      'title="point at it, or focus it and use the arrow keys"></canvas>' +
    // Every value, for anyone who wants them all — or wants to copy one into a
    // document without screenshotting a picture of it. Closed by default
    // because the figure is the point; present always because a tooltip that is
    // the only way to reach a number gates the data behind a mouse.
    // WHAT THE POINTER CAN DO, AS BUTTONS. Drag-to-zoom and click-to-mute are
    // invisible and unreachable without a mouse, so every one of them has a
    // control here: the strip names the state the figure is in and offers the
    // way out of it, and `pin` and `copy` are only reachable from here at all.
    // A hint line that says what the pointer can do lives in the same strip,
    // because a discoverable interaction nobody can find is not one.
    '<div class="sw-view"></div>' +
    '<details class="sw-table"><summary>the numbers behind this picture</summary>' +
    '<div class="sw-table-copy"><button class="ctl sw-copy" type="button">' +
    'copy as TSV</button><span class="sw-copied"></span></div>' +
    '<div class="sw-table-body"></div></details>' +
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

// A CANVAS DOES NOT RESTYLE ITSELF. The shell follows `prefers-color-scheme`
// for free because it is CSS; the figures are pixels, and without this a reader
// flipping their system theme would be left with the light palette on a dark
// page — the one combination neither scheme was ever validated in. The same
// registry the resize listener uses, for the same reason: a redraw must not
// keep a detached figure alive.
watchScheme(() => {
  for (const fn of [...LIVE]) {
    if (fn._host && fn._host.isConnected) fn();
    else LIVE.delete(fn);
  }
});

function wirePanel(host, p, o, redraw) {
  host.querySelectorAll('.sw-opt').forEach(sel => {
    sel.onchange = () => {
      // A CONTROL CHANGE DROPS THE VIEW. The window a reader brushed on a lead
      // axis means nothing on a phase axis, and a hidden series named "cycle
      // 23" is not the series named "cycle 23" in another variable. Carrying
      // either across would show a figure nobody asked for and give no sign
      // that it had happened.
      const v = viewOf(host);
      v.zoom = null; v.hidden.clear(); v.pinned = null;
      o[sel.dataset.k] = sel.value;
      redraw();
    };
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

/**
 * The strip under a figure: what has been done to it, and how to undo it.
 *
 * Rebuilt on every render rather than patched, because it is a statement about
 * the view and the view is recomposed on every render. It carries nothing when
 * the figure is as the panel built it, except the one line saying what the
 * pointer can do — which is there because an interaction nobody can discover is
 * an interaction nobody has.
 */
function viewStrip(host, p, o, view, shown) {
  const el = $('.sw-view', host);
  if (!el) return;
  const fx = (shown.x && shown.x.fmt) || (v => (Number.isInteger(v) ? String(v) : sig(v)));
  const bits = [];
  const z = view.zoom || {};
  const ys = Object.entries(z.y || {});
  if (z.x || ys.length) {
    const said = [];
    if (z.x) said.push('showing ' + esc(fx(z.x[0])) + ' to ' + esc(fx(z.x[1])));
    // Named by the frame's own quantity rather than by an index, because "pane
    // 1" is a fact about the code and the reader is looking at a picture.
    for (const [i, r] of ys) {
      const pane = (shown.panes || [shown])[+i] || {};
      const lab = ((pane.y || {}).label || '').split('  [')[0];
      said.push((lab ? lab + ' ' : '') + 'from ' + esc(sig(r[0])) + ' to ' + esc(sig(r[1])));
    }
    bits.push('<span class="sw-vs">' + said.join(' · ') + '</span>' +
      '<button class="ctl sw-unzoom" type="button">the whole frame</button>');
  }
  if (view.hidden.size) {
    bits.push('<span class="sw-vs">' + view.hidden.size + ' series hidden</span>' +
      '<button class="ctl sw-unhide" type="button">show all</button>');
  }
  bits.push(view.pinned
    ? '<span class="sw-vs">a pinned view is overlaid</span>' +
      '<button class="ctl sw-unpin" type="button">drop it</button>'
    : '<button class="ctl sw-pin" type="button">pin this view</button>');
  bits.push('<span class="sw-hint muted">drag a box on the plot to zoom — wide for a span, ' +
    'tall for a band — double-click or Escape to undo, click a key entry to hide its ' +
    'line</span>');
  el.innerHTML = bits.join(' ');

  const again = () => render(host, p, o);
  const on = (sel, fn) => { const b = $(sel, el); if (b) b.onclick = () => { fn(); again(); }; };
  on('.sw-unzoom', () => { view.zoom = null; });
  on('.sw-unhide', () => view.hidden.clear());
  on('.sw-unpin', () => { view.pinned = null; });
  // PINNED WITHOUT THE PIN. `_built` is the panel's own spec, before the view
  // transform — pinning the shown spec would pin the previous pin as well, and
  // two changes of control later the figure would be carrying a stack of
  // overlays nobody asked for.
  on('.sw-pin', () => {
    const cv = $('.sw-panel', host);
    view.pinned = cv && cv._built ? cv._built : null;
  });

  const cp = $('.sw-copy', host), said = $('.sw-copied', host);
  if (cp) {
    cp.onclick = async () => {
      const text = tableTsv(shown);
      let okay = true;
      try {
        await navigator.clipboard.writeText(text);
      } catch (e) {
        // A clipboard is not always there to write to — an insecure origin, a
        // permission refused. Falling back to a selection means the reader can
        // still take the numbers with one more keystroke instead of none.
        okay = false;
        const ta = document.createElement('textarea');
        ta.value = text;
        ta.style.cssText = 'position:fixed;left:-9999px';
        document.body.appendChild(ta);
        ta.select();
        try { okay = document.execCommand('copy'); } catch (e2) { okay = false; }
        ta.remove();
      }
      if (said) {
        said.textContent = okay ? 'copied' : 'could not reach the clipboard';
        setTimeout(() => { said.textContent = ''; }, 2000);
      }
    };
  }
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
  // ASKED FOR IN CSS PIXELS. `sizeCanvas` gives the backing store the display's
  // own resolution and pins the box to this; every size read back out of the
  // canvas from here on is `cssSize`, so the layout arithmetic is unchanged.
  const want = w > 320 ? Math.min(1180, w) : cssSize(cv).w;
  sizeCanvas(cv, want, fitHeight(want));
}

/** One picture's height at a given width. A stack asks for more; see render. */
function fitHeight(w) {
  return Math.round(Math.max(340, Math.min(560, w / 2.2)));
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
      // `o` because the closure panel's sweep axis is chosen per pair, and a
      // panel that fetched all five pairs to draw one would ask the engine for
      // four answers nobody is looking at. Every other panel ignores it.
      p.data ? p.data(o) : null,
      engineValues(p.engine || []),
    ]);
    const out = p.build(rec, o, extra, eng);
    // A MISPLACED KEY IS A FEATURE THAT SILENTLY DOES NOTHING. `answer` belongs
    // to the build and `finding` belongs to the spec, and the two returns sit
    // one line apart — the drivers parity view had its finding beside `answer`,
    // where nothing reads it, and the only symptom was a figure that looked
    // exactly like a figure nobody had written a finding for.
    if (out.finding !== undefined) {
      throw new Error('`finding` belongs in spec, beside `series` — it is on the build');
    }
    if (out.spec && out.spec.answer !== undefined) {
      throw new Error('`answer` belongs on the build, beside `spec` — it is inside the spec');
    }
    // `answer` is optional and a build that has no single number should not
    // invent one — a headline that is a guess is worse than no headline.
    const ans = $('.sw-answer', host);
    if (ans) {
      ans.innerHTML = out.answer
        ? '<b>' + esc(out.answer.value) + '</b> <span>' + esc(out.answer.of) + '</span>'
        : '';
    }
    const cv = $('.sw-panel', host);
    // A STACK NEEDS THE ROOM ITS FRAMES NEED, and the picture is the only thing
    // that knows how many there are — fitCanvas runs before the build and sizes
    // for one. Three frames squeezed into one picture's height would give each a
    // third of the vertical resolution a slope is read from, which is the aspect
    // argument from B6 applied three times over and in the wrong direction.
    const nPanes = (out.spec.panes || []).length;
    // SIZED FROM THE HOST, EVERY TIME. This read cv.width, which is fine on the
    // first draw and compounding on every one after it: a panel asking for
    // `aspect` narrowed itself, the next render took that narrower width as its
    // base and narrowed again, and `drivers` walked 1180 → 806 → 512 across
    // three redraws. It never showed up through the controls, because a control
    // change rebuilds the body and fitCanvas runs; it showed up the moment a
    // zoom redrew in place. Check four found it on its first full pass.
    fitCanvas(host);
    const H1 = fitHeight(cssSize(cv).w);
    // A NARROWER FRAME WHERE THE WIDTH HAS NOTHING TO SPEND ITSELF ON. §34.3
    // asked for a SHORTER frame "where the curve is monotone", and measuring the
    // fourteen references says there is no height to reclaim: every canvas panel
    // puts ink in 99.4 per cent of its rows, because the y axis scales to its own
    // data. What is actually wasted is WIDTH, and only where the x axis is five
    // named scenarios: 1180 pixels across five positions stretches the picture
    // into a shallow diagonal, which is the banking argument fitCanvas was
    // written for, running the other way. `aspect` is width ÷ height; leaving it
    // out keeps the frame the host gives, so a panel that does not ask is
    // untouched to the pixel.
    const wantW = out.spec.aspect
      ? Math.max(320, Math.min(cssSize(cv).w, Math.round(H1 * out.spec.aspect)))
      : cssSize(cv).w;
    sizeCanvas(cv, wantW, nPanes > 1
      ? Math.round(H1 * (1 + 0.42 * (nPanes - 1)))
      : H1);
    // WHAT THE READER ASKED FOR, AS A SPEC. Zoom, mute and pin are a transform
    // from the spec the panel built to the spec that gets drawn, so the
    // picture, the readout, the arrow-key ladder and the table below are all
    // reading one thing. `out.spec` is kept beside it because the view is
    // composed afresh on every redraw and the panel's own spec is what it is
    // composed from.
    const view = viewOf(host);
    const shown = viewSpec(out.spec, view);
    cv._built = out.spec;
    drawChart(cv, shown);
    const again = () => render(host, p, o);
    attachHover(cv, {
      onBrush: win => {
        const next = { ...(view.zoom || {}) };
        if (win.x) {
          // A window holding fewer than two drawn points is not a view of the
          // data, it is a view of the gap between two of them.
          const pts = (shown.panes || [shown]).flatMap(q => (q.series || []))
            .flatMap(q => q.x || []).filter(v => v >= win.x[0] && v <= win.x[1]);
          if (new Set(pts).size < 2) return;
          next.x = win.x;
        }
        // A Y WINDOW IS NOT CHECKED THE SAME WAY, and that is deliberate. It
        // clips rather than filters, so an empty one shows an empty frame
        // rather than a broken picture — and on a scatter the reader zooming
        // into a sparse band is doing exactly what the box is for.
        if (win.y) next.y = { ...(next.y || {}), ...win.y };
        view.zoom = next;
        again();
      },
      onReset: () => {
        if (!viewIsOn(view)) return;
        view.zoom = null; view.hidden.clear(); view.pinned = null;
        again();
      },
      onIsolate: name => {
        if (view.hidden.has(name)) view.hidden.delete(name);
        else view.hidden.add(name);
        // Never all of them. A frame with nothing in it is not an isolation,
        // it is a panel that looks broken.
        const named = (out.spec.panes || [out.spec]).flatMap(q => (q.series || []))
          .filter(q => q.name).map(q => q.name);
        if (named.length && named.every(n => view.hidden.has(n))) view.hidden.delete(name);
        again();
      },
      onOpenRow: row => { if (window.openNode) window.openNode(row); },
    });
    // From the same spec the chart was drawn from, so the two cannot disagree.
    const tb = $('.sw-table-body', host);
    if (tb) tb.innerHTML = tableFor(shown);
    viewStrip(host, p, o, view, shown);
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
