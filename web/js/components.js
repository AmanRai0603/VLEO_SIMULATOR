/*
  The component library: every part a page is built from, by name.

  NOBODY WRITES HTML FOR A PAGE (docs/ARCHITECTURE.html, section 5). A lesson
  is content — text, claims, equations, widgets declared as rows, questions —
  and each part is drawn here, once, the same way everywhere. A page that
  needs something new gets it added here, and from then on any page can use it
  by name. The gate refuses a lesson carrying markup, so nothing reaches a
  page except through one of these.

    answerFirst      the point, before anything else          (dom.js)
    claimTag         what a claim rests on: sourced, derived, declared, illustrative
    station          one step of the explaining order: simply, real, breaks, story
    equation         a relation as a reader sees it, with its source
    widget           "try it": sliders for input rows, the engine's answer for
                     output rows, and the sweep as the engine describes it
    checkYourself    a question, its options, and why the answer is right
    references       where the claims come from
    figure           any figure the engine describes             (figures.js)

  DEPTH. Blocks carry the depth classes depth.js reads: `d-lr` for the plain
  words (dropped for an expert), `d-l` for what only a learner is shown.

  A WIDGET COMPUTES NOTHING ITSELF. It sends the reader's value to the engine
  as a supplied input and shows what the engine returns — so a lesson cannot
  disagree with the tool, because it is the tool.
*/
'use strict';

import { esc } from './dom.js';
export { answerFirst } from './dom.js';
import { S } from './state.js';
import { fromSI, toSI, unitOf } from './inputs.js';
import { drawFigureInto } from './figures.js';

const CLAIM_SAYS = {
  sourced: 'stated in the cited work',
  derived: 'worked here from sourced relations',
  declared: 'a number a person chose, and put their name against',
  illustrative: 'an example, not data',
};

/** What a claim rests on, on the claim (docs/EXPLAINING.md E3). */
export function claimTag(kind, whose = '') {
  return '<span class="claim claim-' + esc(kind) + '" title="' + esc(CLAIM_SAYS[kind] || '') + '">' +
    esc(kind) + (whose ? ' · ' + esc(whose) : '') + '</span>';
}

const STATION = {
  simply: ['Said simply', 'd-lr'],
  real: ['The real thing', ''],
  breaks: ['Where it breaks', ''],
  story: ['A story', 'd-lr'],
};

/** One step of the explaining order, with its claim. */
export function station(s) {
  const [name, depth] = STATION[s.kind] || [s.kind, ''];
  return '<section class="ls-station ls-' + esc(s.kind) + (depth ? ' ' + depth : '') + '">' +
    '<h4>' + esc(s.title || name) + '</h4>' +
    s.text.split(/\n\s*\n/).map(p => '<p>' + esc(p.trim()) + '</p>').join('') +
    '<p class="ls-claim">' + claimTag(s.claim, s.source) + '</p></section>';
}

/** A relation as a reader sees it, and what it says. */
export function equation(e) {
  return '<figure class="ls-eq"><code>' + esc(e.text) + '</code><figcaption>' + esc(e.says) + ' ' +
    claimTag(e.claim, e.source) + '</figcaption></figure>';
}

/** A question, answered by clicking; the right answer says why. */
export function checkYourself(q, i) {
  return '<section class="ls-check d-lr" data-answer="' + q.answer + '"><h4>Check yourself' +
    (i != null ? ' · ' + (i + 1) : '') + '</h4><p>' + esc(q.question) + '</p><div class="runbar">' +
    q.options.map((o, k) => '<button class="ctl ls-opt" type="button" data-k="' + (k + 1) + '">' + esc(o) +
      '</button>').join('') + '</div><p class="ls-why" hidden data-why="' + esc(q.why) + '"></p></section>';
}

/** Where the claims come from. */
export function references(list) {
  return list.length ? '<section class="ls-refs"><h4>References</h4><ol>' +
    list.map(r => '<li>' + esc(r) + '</li>').join('') + '</ol></section>' : '';
}

/** A "try it" widget's frame; `mountWidget` makes it live. */
export function widget(w, i) {
  return '<section class="ls-widget" data-w="' + i + '"><h4>' + esc(w.title || 'Try it') + '</h4>' +
    '<div class="ls-sliders"></div><div class="ls-out"></div><div class="ls-fig"></div></section>';
}

const POST = { method: 'POST', headers: { 'content-type': 'application/x-www-form-urlencoded' } };

/**
 * WHERE A WIDGET'S ANSWERS COME FROM. The running engine, by default; a page
 * read without it — the readers' docs folder — sets the engine compiled for
 * the browser (`xtask readers`, crates/vleo-kernel-wasm). Either way the
 * answer is the engine's, in the shape `/v1/run` and `/v1/sweep` give it.
 */
let ENGINE = {
  run: async p => (await fetch('/v1/run', { ...POST, body: p.toString() })).json(),
  sweep: async q => (await fetch('/v1/sweep?' + q.toString())).json(),
};
export function useEngine(e) { ENGINE = e; }

/**
 * Make a widget live: a slider per input row, in the row's own unit and range,
 * starting where the engine says the row stands — the case in the tool, the
 * declared value in a page read without it; each change runs the engine with
 * those values supplied and shows the output rows it returns; with `sweep`,
 * the first output across that input's range, as the engine describes it.
 */
export async function mountWidget(el, w) {
  const rows = w.inputs.map(id => S.byId.get(id)).filter(Boolean);
  const ask = async p => {
    try { return await ENGINE.run(p); } catch (e) { return { ok: false, message: 'the engine did not answer: ' + e }; }
  };
  const start = await ask(new URLSearchParams({ node: w.outputs[0] || '', mode: 'branch' }));
  const at = id => ((start && start.values) || []).find(v => v.id === id);
  const vals = new Map(rows.map(r => [r.id, at(r.id) ? at(r.id).si : (r.lo + r.hi) / 2]));
  el.querySelector('.ls-sliders').innerHTML = rows.map(r =>
    '<label class="ls-slider"><span>' + esc(r.label) + ' <code>' + esc(r.id) + '</code></span>' +
    '<input type="range" data-id="' + esc(r.id) + '" min="' + fromSI(r, r.lo) + '" max="' + fromSI(r, r.hi) +
    '" step="' + (fromSI(r, r.hi) - fromSI(r, r.lo)) / 200 + '" value="' + fromSI(r, vals.get(r.id)) + '">' +
    '<b class="ls-v"></b> ' + esc(unitOf(r.unit)) + '</label>').join('');
  const out = el.querySelector('.ls-out');
  let seq = 0;
  const run = async () => {
    const mine = ++seq;
    // The value the engine is given, not the slider's: a range input snaps to
    // its step, and the row's own value is rarely on one.
    el.querySelectorAll('.ls-slider').forEach(l => {
      const id = l.querySelector('input').dataset.id;
      l.querySelector('.ls-v').textContent = (+fromSI(S.byId.get(id), vals.get(id))).toPrecision(4);
    });
    const shown = new Map();
    for (const target of w.outputs) {
      if (shown.has(target)) continue;
      const p = new URLSearchParams({ node: target, mode: 'branch' });
      for (const [id, si] of vals) p.append('set', id + ':' + si);
      const r = await ask(p);
      if (mine !== seq) return;
      if (!r.ok) { shown.set(target, { refused: r.message || r.fault || 'refused' }); continue; }
      for (const v of r.values || []) if (w.outputs.includes(v.id) && !shown.has(v.id)) shown.set(v.id, v);
      // A row the engine would not answer is named with the engine's reason —
      // never left blank, never drawn as a number (rule 5).
      for (const b of r.blocked || []) if (w.outputs.includes(b.id) && !shown.has(b.id)) {
        shown.set(b.id, { refused: b.message || b.kind || 'blocked' });
      }
      if (!shown.has(target)) shown.set(target, { refused: 'the run did not return it' });
    }
    out.innerHTML = '<table class="fx"><tbody>' + w.outputs.map(id => {
      const v = shown.get(id) || {}, r = S.byId.get(id) || {};
      return '<tr><td>' + esc(r.label || id) + ' <code>' + esc(id) + '</code></td><td><b>' +
        (v.refused ? 'refused — ' + esc(v.refused) : esc(v.shown != null ? v.shown : v.si)) + '</b></td></tr>';
    }).join('') + '</tbody></table><p class="muted">computed by the engine on these inputs, now; ' +
      'nothing on this page works it out</p>';
  };
  el.querySelectorAll('.ls-slider input').forEach(inp => {
    inp.oninput = () => { vals.set(inp.dataset.id, toSI(S.byId.get(inp.dataset.id), +inp.value)); run(); };
  });
  run();
  if (w.sweep && S.byId.get(w.sweep)) {
    const r = S.byId.get(w.sweep);
    const q = new URLSearchParams({ node: w.outputs[0], mode: 'branch', over: w.sweep, from: r.lo, to: r.hi, points: 25 });
    for (const [id, si] of vals) if (id !== w.sweep) q.append('set', id + ':' + si);
    Promise.resolve().then(() => ENGINE.sweep(q)).then(sw => {
      const host = el.querySelector('.ls-fig');
      if (!sw.ok || !sw.figure) { host.innerHTML = '<p class="muted">the sweep did not run: ' + esc(sw.message || '') + '</p>'; return; }
      drawFigureInto(host, sw.figure, w.outputs[0] + '_across_' + w.sweep);
    }).catch(() => {});
  }
}

/**
 * A lesson, drawn from its content: answer first, the stations in order with
 * the equations after the real thing, the widgets, the checks, the references.
 */
export function renderLesson(host, l) {
  const real = l.stations.findIndex(s => s.kind === 'real');
  const parts = l.stations.map((s, i) => station(s) + (i === real ? l.equations.map(equation).join('') : ''));
  if (real < 0) parts.push(l.equations.map(equation).join(''));
  host.innerHTML = '<article class="lesson">' +
    '<section class="answer-first view-af"><p class="af-k">Answer first <span class="dx dx-' + esc(l.kind) + '">' +
      esc(l.kind) + '</span></p><p class="af-a">' + esc(l.answer) + '</p></section>' +
    '<p class="muted ls-by">' + esc(l.title) + ' · written by ' + esc(l.by) + '</p>' +
    parts.join('') + l.widgets.map(widget).join('') + l.checks.map(checkYourself).join('') +
    references(l.references) + '</article>';
  host.querySelectorAll('.ls-widget').forEach(el => mountWidget(el, l.widgets[+el.dataset.w]));
  host.querySelectorAll('.ls-check').forEach(el => {
    el.querySelectorAll('.ls-opt').forEach(b => {
      b.onclick = () => {
        const right = b.dataset.k === el.dataset.answer;
        el.querySelectorAll('.ls-opt').forEach(x => x.classList.remove('ls-right', 'ls-wrong'));
        b.classList.add(right ? 'ls-right' : 'ls-wrong');
        const why = el.querySelector('.ls-why');
        why.hidden = false;
        why.textContent = (right ? 'Yes. ' : 'Not quite. ') + why.dataset.why;
      };
    });
  });
}
