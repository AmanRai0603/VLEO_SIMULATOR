/*
  The figure: the layer view.

  It composes the three drawings and the prose that surrounds them. Everything
  it prints is counted from the graph on this draw and never remembered — a
  number on a legend that was true last time the page was built is worse than
  no number.
*/
'use strict';

import { $, esc, plural } from './dom.js';
import { S, LAYERS, CAPTIONS, subtreeNodes, isUndefined } from './state.js';
import { buildDisplay, relations } from './display.js';
import { drawTree } from './tree.js';
import { drawPaths } from './paths.js';
import { drawMatrix } from './matrix.js';

export function drawFigure() {
  const disp = buildDisplay();
  S.disp = disp;
  S.dispIndex = new Map(disp.map((d, i) => [d.id, i]));
  if (!S.dispIndex.has(S.selected)) {
    const first = disp.find(d => d.kind === 'node') || disp[1] || disp[0];
    S.selected = first ? first.id : null;
  }

  const rel = relations(disp);
  drawTree(disp, rel);
  drawPaths(disp, rel);
  drawMatrix(disp, rel);
  drawReach(rel);
  drawNotes();
  return disp;
}

export function drawCaptions() {
  const cap = CAPTIONS[S.layer];
  $('#caption-a').textContent = cap[0];
  $('#caption-b').textContent = cap[1];
}

export function drawStepper() {
  const at = S.view === 'arch' ? 0 : S.view === 'run' ? 4 : S.view === 'node' ? 3 : 2;
  const steps = ['pick a layer', 'read the tree', 'open a node', 'run it'];
  $('#stepper').innerHTML = steps.map((t, i) =>
    '<span class="step' + (i + 1 === at ? ' on' : '') + '"><span class="n">' + (i + 1) + '</span>' +
    esc(t) + '</span>' + (i < 3 ? '<span class="dash">—</span>' : '')).join('') +
    '<span class="aside">' + S.rows.length + ' rows · ' + S.index.groups.length + ' groups · ' +
    S.index.relations.length + ' declared relations · ' +
    // How many of them ANSWER. Counted from the graph on this draw, like every
    // other number in this strip, and put beside the row count because the two
    // together are the fact — 1395 rows of which 182 answer is a different
    // programme from 1395 rows.
    S.rows.filter(x => x.state !== 'empty' && !isUndefined(x)).length + ' answer</span>';
}

function drawReach(rel) {
  const d = rel.sel;
  if (!d) { $('#reach').textContent = ''; return; }
  $('#reach').innerHTML =
    'Change <b>' + esc(d.label) + '</b> and <b>' + rel.rOut.size + '</b> node' +
    (rel.rOut.size === 1 ? '' : 's') + ' become stale — unknown, not wrong. It stands on <b>' +
    rel.rIn.size + '</b> node' + (rel.rIn.size === 1 ? '' : 's') +
    ' upstream. Both counted from the graph on this draw, never remembered.';
}

function drawNotes() {
  const st = S.matrixStats;
  const seeded = S.rows.filter(r => r.state === 'empty').length;
  const inactive = S.rows.filter(r => isUndefined(r)).length;
  const rels = S.index.relations.filter(r => S.dispIndex.has(r.from) || S.dispIndex.has(r.to));
  const put = [
    ['a row', 'One small question, one answer, one folder, one row. The variable id <b>is</b> the node id.'],
    ['a box', 'A group of rows with an owner. Closed, it is one row and every edge inside it rolls up onto that row — summarised, never hidden.'],
    ['a mark', 'The row feeds the column. <b>' + st.marks + '</b> marks are drawn here; <b>' + st.crossing + '</b> of them leave their own box and so need an interface.'],
    ['against tree order', '<b>' + st.below + '</b> marks sit below the diagonal — the row feeds something earlier in the tree. That is either a declared cycle or a decomposition that has the order wrong.'],
    ['crossing upward', 'Exactly one row in each subsystem layer carries that layer to the system. It is marked with a thin teal outline.'],
    ['a case', 'A case selects which boxes are in scope. It is never a copy of the tree: a cloned architecture is two architectures that will disagree.'],
    ['seeded', '<b>' + seeded + '</b> of ' + S.rows.length + ' rows are seeded — the folder, the sheet and the row exist, and nothing is specified in them. Running one returns <code>NotRun</code>, by name.'],
    ['inactive', '<b>' + inactive + '</b> rows are written, generated and compiling, and still do not answer: their relation is stated and never derived. Not the same state as seeded, and a different piece of work — a seeded row needs somebody to decide what it is, an inactive one needs somebody to say where its relation came from.'],
    ['a relation', '<b>' + rels.length + '</b> of ' + S.index.relations.length + ' declared group relations touch this layer. A relation is stated by the layer file, never inferred from the marks.'],
  ];
  $('#notes').innerHTML = put.map(([k, v]) => '<dt>' + k + '</dt><dd>' + v + '</dd>').join('');
}

export function drawStatus(disp) {
  const st = S.matrixStats;
  if (S.view === 'arch') {
    $('#status').textContent = 'Architecture · the pattern every one of ' + S.rows.length +
      ' folders repeats';
    $('#caserow-note').textContent = '';
    return;
  }
  if (S.view === 'node') {
    const r = S.byId.get(S.selected);
    const g = r ? S.G.get(r.parent) : null;
    $('#status').textContent = ['Layer ' + (r ? r.layer : S.layer), g ? g.label : '',
      S.caseSel.toUpperCase(), r ? r.id : '', r ? r.state : ''].filter(Boolean).join(' · ');
    $('#caserow-note').textContent = r ? 'owner ' + r.owner : '';
    return;
  }
  if (S.view === 'run') {
    const r = S.byId.get(S.runTarget);
    $('#status').textContent = ['Layer 4', 'The run', r ? r.id : '—', S.engineCase, S.mode]
      .filter(Boolean).join(' · ');
    $('#caserow-note').textContent = '';
    return;
  }
  // Every view that is not the layer view returns above. Without this guard
  // one that forgets to fell through to the crumb below and described the
  // LAST LAYER SELECTION instead of itself: the solar view printed
  // "Layer 3 · Solar weather — addition · C1 · sw_recurrence_lag · 0 rows ·
  // 1 box · 0 crossing · 5 against tree order" over the mean-cycle chart,
  // every figure of it stale, including a tree-order count from a matrix that
  // was no longer on screen. A status line that describes something else is
  // worse than none, because it is read as a caption.
  if (S.view !== 'layer') {
    $('#status').textContent = 'a view with no status line of its own';
    $('#caserow-note').textContent = '';
    return;
  }
  const bits = ['Layer ' + S.layer, LAYERS[S.layer].name];
  if (S.layer === 3 && S.G.has(S.subsys)) bits[1] = S.G.get(S.subsys).label;
  bits.push(S.caseSel.toUpperCase(), S.selected || '—',
    plural(disp.length, 'row'), plural(st.boxes, 'box', 'boxes'),
    st.crossing + ' crossing', st.below + ' against tree order');
  $('#status').textContent = bits.join(' · ');
  const g = S.G.get(S.layer === 3 ? S.subsys : LAYERS[S.layer].root);
  $('#caserow-note').textContent = S.layer === 1
    ? 'the same architecture, read as ' + (S.concept === 'single' ? 'one satellite' : 'a constellation')
    : g ? 'owner ' + g.owner : '';
}

export function drawFoot() {
  const v = S.version || {};
  const k = {};
  for (const r of S.rows) k[r.kind] = (k[r.kind] || 0) + 1;
  const data = v.data || [];
  $('#foot').innerHTML =
    'kernel <b>' + esc(v.kernel || '—') + '</b> · graph <b>' + esc(v.graph || '—') +
    '</b> · engine <b>' + esc(v.endpoint || 'none') + '</b> · ' +
    Object.entries(k).map(([a, b]) => b + ' ' + a).join(' · ') +
    ' · ' + (data.length
      ? 'data <b>' + esc(data.map(x => x.split('#')[0]).join(' · ')) + '</b>'
      : 'data <b>none synced</b> — every node that declares a bundle will refuse, naming what to sync');
}

/** Used by the subsystem picker and the architecture view alike. */
export const rowsUnder = gid => subtreeNodes(gid).length;
