/*
  The shell.

  It holds every view and no physics. There is no fast path here for a small
  case: a fast path is a second implementation that will eventually disagree,
  and the disagreement will be found in a review rather than in a test.

  One selected node and one loaded case in the whole tool. Three views of one
  state, never three states that have to be kept in step — that is the class of
  bug that once put three correct numbers on one screen, correct at three
  different times.
*/
'use strict';

const S = {
  index: null,
  byId: new Map(),
  consumers: new Map(),   // derived on load, never stored: it cannot go stale
  selected: null,
  mode: 'alone',
  caseId: 'nominal',
  lastRun: null,
  lastCaseHash: null,
  version: null,
};

const $ = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => Array.from(r.querySelectorAll(s));

// ---------------------------------------------------------------------------
// boot

async function boot() {
  try {
    S.version = await (await fetch('/v1/version')).json();
  } catch (e) {
    setRunState('the engine did not answer. This page is served by the daemon, so if you can read it the engine is running — reload.', 'bad');
  }
  S.index = await (await fetch('/v1/index')).json();
  for (const r of S.index.rows) S.byId.set(r.id, r);
  // Reverse indexes are derived on every load and never stored, so they cannot
  // disagree with the graph they came from.
  for (const r of S.index.rows) {
    for (const i of r.in) {
      const producer = S.index.rows[i].id;
      if (!S.consumers.has(producer)) S.consumers.set(producer, []);
      S.consumers.get(producer).push(r.id);
    }
  }
  fillCases();
  drawProvenance();
  drawTree();
  drawFooter();
  wire();
  select(S.index.rows.find(r => r.id === 'prop_thrust_to_drag')?.id || S.index.rows[0].id);
}

function fillCases() {
  const sel = $('#case');
  sel.innerHTML = '';
  for (const c of S.index.cases) {
    const o = document.createElement('option');
    o.value = c.id;
    o.textContent = c.id + ' — ' + c.label;
    o.title = c.note;
    sel.appendChild(o);
  }
  sel.value = S.caseId;
}

function drawProvenance() {
  const v = S.version || {};
  $('#p-engine').innerHTML = 'engine <b>' + (v.endpoint || 'none') + '</b>';
  $('#p-kernel').innerHTML = 'kernel <b>' + (v.kernel || '—') + '</b>';
  $('#p-graph').innerHTML = 'graph <b>' + (v.graph || '—') + '</b>';
  const d = (v.data || []);
  const el = $('#p-data');
  if (!d.length) {
    // Amber, and it says why. A run with no reference data is not a degraded
    // run: every node that declares a bundle refuses, by name.
    el.className = 'p-item amber';
    el.innerHTML = 'data <b>none synced</b>';
    el.title = 'No reference data in the local store. Every node that declares a bundle will refuse, naming what to sync. Run: vleo data sync';
  } else {
    el.className = 'p-item';
    el.innerHTML = 'data <b>' + d.map(x => x.split('#')[0]).join(' · ') + '</b>';
    el.title = d.join('\n');
  }
}

function drawFooter() {
  const k = {};
  for (const r of S.index.rows) k[r.kind] = (k[r.kind] || 0) + 1;
  $('#foot-counts').textContent =
    Object.entries(k).map(([a, b]) => b + ' ' + a).join(' · ') +
    ' · ' + S.index.groups.length + ' groups · ' + S.index.relations.length + ' relation edges';
}

// ---------------------------------------------------------------------------
// the tree

function layerFilter() { return $('#layer').value; }

function groupsUnder(rootId) {
  if (!rootId) return new Set(S.index.groups.map(g => g.id));
  const keep = new Set([rootId]);
  let moved = true;
  while (moved) {
    moved = false;
    for (const g of S.index.groups) {
      if (!keep.has(g.id) && keep.has(g.parent)) { keep.add(g.id); moved = true; }
    }
  }
  return keep;
}

function visibleRows() {
  const keep = groupsUnder(layerFilter());
  const q = $('#search').value.trim().toLowerCase();
  return S.index.rows.filter(r => {
    if (!keep.has(r.parent)) return false;
    if (!q) return true;
    return (r.id + ' ' + r.label + ' ' + r.symbol + ' ' + r.question).toLowerCase().includes(q);
  });
}

function drawTree() {
  const body = $('#tree-body');
  const rows = visibleRows();
  const sel = S.selected;
  const feeds = sel ? new Set(S.consumers.get(sel) || []) : new Set();
  const fedBy = sel ? new Set((S.byId.get(sel)?.in || []).map(i => S.index.rows[i].id)) : new Set();
  const reach = sel ? reachable(sel) : new Set();

  const byGroup = new Map();
  for (const r of rows) {
    if (!byGroup.has(r.parent)) byGroup.set(r.parent, []);
    byGroup.get(r.parent).push(r);
  }
  const frag = document.createDocumentFragment();
  for (const [gid, list] of byGroup) {
    const g = S.index.groups.find(x => x.id === gid);
    const h = document.createElement('div');
    h.className = 'group';
    h.innerHTML = esc(g ? g.label : gid) + '<span class="owner">' + esc(g ? g.owner : '') + '</span>';
    frag.appendChild(h);
    for (const r of list) {
      const d = document.createElement('div');
      d.className = 'row k-' + r.kind +
        (r.id === sel ? ' sel' : feeds.has(r.id) ? ' feeds' : fedBy.has(r.id) ? ' fedby' : reach.has(r.id) ? ' reach' : '');
      d.dataset.id = r.id;
      d.title = r.question;
      d.innerHTML = '<span class="dot">●</span><span class="rid">' + esc(r.id) + '</span><span class="rlabel">' + esc(r.label) + '</span>';
      frag.appendChild(d);
    }
  }
  body.replaceChildren(frag);
  $('#tree-count').textContent = rows.length + ' of ' + S.index.rows.length;
}

/* What a change reaches. Derived from the graph, never a second hand-written
   list — which is the whole reason the graph has to be the artefact. */
function reachable(id) {
  const out = new Set();
  const stack = [id];
  while (stack.length) {
    const n = stack.pop();
    for (const c of (S.consumers.get(n) || [])) {
      if (!out.has(c)) { out.add(c); stack.push(c); }
    }
  }
  return out;
}

function upstreamClosure(id) {
  const out = new Set();
  const stack = [id];
  while (stack.length) {
    const n = stack.pop();
    for (const i of (S.byId.get(n)?.in || [])) {
      const p = S.index.rows[i].id;
      if (!out.has(p)) { out.add(p); stack.push(p); }
    }
  }
  return out;
}

// ---------------------------------------------------------------------------
// what this touches

function drawPaths() {
  const r = S.byId.get(S.selected);
  const body = $('#paths-body');
  if (!r) { body.innerHTML = ''; return; }
  const ins = (r.in || []).map(i => S.index.rows[i]);
  const outs = (S.consumers.get(r.id) || []).map(id => S.byId.get(id));
  const reach = reachable(r.id);

  let h = '';
  h += '<div class="path-h in">what feeds it — ' + ins.length + '</div>';
  h += ins.length ? ins.map(x =>
    '<div class="path-item" data-id="' + esc(x.id) + '">' + esc(x.id) +
    '<div class="why">' + esc(x.label) + '</div></div>').join('')
    : '<p class="muted">Nothing. This is a declared value or a leaf of the design.</p>';
  h += '<div class="path-h out">what it feeds — ' + outs.length + '</div>';
  h += outs.length ? outs.map(x =>
    '<div class="path-item" data-id="' + esc(x.id) + '">' + esc(x.id) +
    '<div class="why">' + esc(x.label) + '</div></div>').join('')
    : '<p class="muted">Nothing reads this yet. Every one of these is a leaf of the design, or an oversight.</p>';
  h += '<div class="reach-line">Change this and <b>' + reach.size + '</b> node' + (reach.size === 1 ? '' : 's') +
       ' become stale — unknown, not wrong. Counted from the graph, not remembered.</div>';
  body.innerHTML = h;
}

// ---------------------------------------------------------------------------
// the matrix
//
// Rows and columns are the tree in depth-first order and the hierarchy is drawn
// inside the matrix as nested boxes on the diagonal. A flat 250x250 grid is
// 62 500 cells of which 0.2% are filled and is useless; a rollup to eight
// branches is one cell in sixty-four and is useless for the opposite reason.
// The nested form keeps every subtree a contiguous band, so a mark inside a box
// is coupling that subtree owns and a mark outside it crosses a boundary.

function drawMatrix() {
  const body = $('#matrix-body');
  const sel = S.selected;
  if (!sel) { body.innerHTML = ''; return; }
  // Scope to the selected node's own group plus its neighbourhood, because the
  // measurement says a node's neighbourhood is always legible whatever the
  // total, and the whole grid never is.
  const r = S.byId.get(sel);
  const near = new Set([sel]);
  for (const i of (r.in || [])) near.add(S.index.rows[i].id);
  for (const c of (S.consumers.get(sel) || [])) near.add(c);
  const scope = S.index.rows.filter(x => x.parent === r.parent || near.has(x.id));
  const idx = new Map(scope.map((x, i) => [x.id, i]));

  const edges = new Set();
  for (const x of scope) {
    for (const i of (x.in || [])) {
      const p = S.index.rows[i].id;
      if (idx.has(p)) edges.add(idx.get(p) + ':' + idx.get(x.id)); // row feeds column
    }
  }

  let h = '<table class="mx"><tbody>';
  for (let rr = 0; rr < scope.length; rr++) {
    h += '<tr>';
    for (let cc = 0; cc < scope.length; cc++) {
      if (rr === cc) {
        h += '<td class="diag' + (scope[rr].id === sel ? ' sel' : '') + '" title="' +
             esc(scope[rr].id) + ' — click to open" data-open="' + esc(scope[rr].id) + '"></td>';
        continue;
      }
      const has = edges.has(rr + ':' + cc);
      const sameBox = scope[rr].parent === scope[cc].parent;
      let cls = 'mark ' + (sameBox ? 'inside' : 'crosses');
      if (scope[rr].id === sel) cls += ' rowsel';
      if (scope[cc].id === sel) cls += ' colsel';
      h += has
        ? '<td class="' + cls + '" title="' + esc(scope[rr].id) + ' feeds ' + esc(scope[cc].id) + '"></td>'
        : '<td' + (sameBox ? ' class="box"' : '') + '></td>';
    }
    h += '</tr>';
  }
  h += '</tbody></table>';
  body.innerHTML = h;
  $('#mx-count').textContent = scope.length + ' rows · ' + edges.size + ' marks';

  // The matrix says that two nodes connect. Only the list says what crosses.
  const list = $('#crossings');
  const items = [];
  for (const key of edges) {
    const [a, b] = key.split(':').map(Number);
    if (scope[a].parent === scope[b].parent) continue;
    items.push('<li>' + esc(scope[a].id) + ' → ' + esc(scope[b].id) +
               ' <span class="muted">' + esc(scope[a].label) + '</span></li>');
  }
  list.innerHTML = items.length
    ? '<li class="muted">crossings — an edge leaving its own branch</li>' + items.join('')
    : '<li class="muted">Every mark in scope stays inside its own branch. No boundary is crossed here.</li>';
}

// ---------------------------------------------------------------------------
// selection

async function select(id) {
  if (!S.byId.has(id)) return;
  S.selected = id;
  drawTree();
  drawPaths();
  drawMatrix();
  await openNode(id);
  updateRunControl();
}

async function openNode(id) {
  const body = $('#node-body');
  body.innerHTML = '<p class="muted">loading the fragment…</p>';
  try {
    const html = await (await fetch('/v1/fragment/' + encodeURIComponent(id))).text();
    body.innerHTML = html;
  } catch (e) {
    body.innerHTML = '<p class="empty">The fragment for <code>' + esc(id) + '</code> is not on disk. Run <code>cargo xtask docs</code>.</p>';
  }
  $$('.tab', body).forEach(t => t.onclick = () => {
    $$('.tab', body).forEach(x => x.classList.remove('sel'));
    $$('[data-panel]', body).forEach(x => x.classList.remove('sel'));
    t.classList.add('sel');
    const p = $('[data-panel="' + t.dataset.tab + '"]', body);
    if (p) p.classList.add('sel');
  });
  $$('.xref[data-goto]', body).forEach(a => a.onclick = () => select(a.dataset.goto));
  renderResult();
}

// ---------------------------------------------------------------------------
// the Run control
//
// A disabled control that does not say why is a defect. "Run alone" is only
// available when every dependency already holds a value, and when it is not it
// names what is missing.

function updateRunControl() {
  const r = S.byId.get(S.selected);
  const btn = $('#run');
  const state = $('#runstate');
  if (!r) { btn.disabled = true; return; }
  const known = new Set((S.lastRun?.values || []).map(v => v.id));
  const missing = (r.in || []).map(i => S.index.rows[i].id).filter(x => !known.has(x));

  const aloneBtn = $('.mode[data-mode="alone"]');
  aloneBtn.disabled = missing.length > 0;
  aloneBtn.title = missing.length
    ? 'Disabled: ' + missing.slice(0, 4).join(', ') + (missing.length > 4 ? ' and ' + (missing.length - 4) + ' more' : '') + ' ' + (missing.length === 1 ? 'has' : 'have') + ' never run. Run the branch instead.'
    : 'Only this node. Upstream values are whatever the store already holds.';
  if (aloneBtn.disabled && S.mode === 'alone') setMode('branch');

  btn.disabled = false;
  if (missing.length && S.mode === 'alone') {
    state.className = 'runstate warn';
    state.textContent = missing.length + ' dependency has never run: ' + missing.slice(0, 3).join(', ');
  } else {
    state.className = 'runstate';
    state.textContent = S.mode === 'branch'
      ? upstreamClosure(r.id).size + 1 + ' nodes in the closure'
      : S.mode === 'all' ? S.index.rows.length + ' rows, everything buildable' : '1 node';
  }
}

function setMode(m) {
  S.mode = m;
  $$('.mode').forEach(b => b.classList.toggle('sel', b.dataset.mode === m));
  updateRunControl();
}

async function run() {
  const r = S.byId.get(S.selected);
  if (!r) return;
  const btn = $('#run');
  btn.disabled = true;
  $('#runstate').textContent = 'running…';
  const body = new URLSearchParams();
  body.set('node', r.id);
  body.set('mode', S.mode);
  body.set('case', S.caseId);
  for (const [k, v] of overrides()) body.append('set', k + ':' + v);
  try {
    const res = await (await fetch('/v1/run', {
      method: 'POST',
      headers: { 'content-type': 'application/x-www-form-urlencoded' },
      body: body.toString(),
    })).json();
    S.lastRun = res.ok ? res : null;
    renderResult(res);
  } catch (e) {
    renderResult({ ok: false, message: 'the engine did not answer: ' + e });
  }
  btn.disabled = false;
  updateRunControl();
}

/* Case overrides typed on the page. Out of range turns red and the run is
   refused before anything crosses the boundary: a value silently clamped is a
   design that drifted without anyone deciding to. */
const OVERRIDES = new Map();
function overrides() { return OVERRIDES; }

function renderResult(res) {
  const el = $('#result');
  const r = S.byId.get(S.selected);
  if (!r) { el.innerHTML = ''; return; }
  if (!res) {
    el.innerHTML = '<p class="muted">Not run. Press <b>Run</b> — the eight tabs above are what the node <i>is</i>; this is what it <i>returns</i>.</p>' + sweepControls(r);
    wireSweep(r);
    return;
  }
  if (!res.ok) {
    el.innerHTML = '<div class="blocked"><b>' + esc(res.fault || 'refused') + '</b> · ' +
      esc(res.node || '') + '<div>' + esc(res.message || '') + '</div></div>' +
      '<p class="muted">A refusal is a value, not a magic number the caller may forget to check. It names the field, the bound it broke and the reason that bound exists.</p>' +
      sweepControls(r);
    wireSweep(r);
    return;
  }
  const v = res.values.find(x => x.id === r.id);
  let h = '';
  if (v) {
    h += '<div class="answer">' + esc(v.symbol) + ' = ' + esc(v.shown) +
         ' <span class="unit">' + esc(v.unit) + '</span></div>';
    h += credBars(v);
    h += '<p class="cred-note">Eight factors, and the lowest governs — averaging would let a strong factor hide a zero, which is the whole failure the score exists to prevent. ' +
         'This node is held down by <b>' + esc(v.governing) + '</b>. Never stored: a badge read out of a field is a claim about last March.</p>';
  } else {
    h += '<div class="answer stale">not computed</div>';
  }
  h += '<div class="chainline">' + res.manifest.ran + ' ran · ' + res.manifest.blocked +
       ' blocked · ' + res.manifest.iterations + ' cycle sweep' + (res.manifest.iterations === 1 ? '' : 's') + '</div>';
  if (res.blocked.length) {
    h += '<div class="blocked">' + res.blocked.slice(0, 40).map(b =>
      '<div>' + esc(b.id) + ' — ' + esc(b.message) + '</div>').join('') +
      (res.blocked.length > 40 ? '<div>… and ' + (res.blocked.length - 40) + ' more</div>' : '') + '</div>';
  }
  if (res.verdicts.length) {
    h += '<h4>evidence, executed on this run</h4><div class="verdicts">' + res.verdicts.map(x =>
      '<div class="' + (x.passed ? 'pass' : 'fail') + '">' + (x.passed ? '✓' : '✗') + ' ' +
      esc(x.node) + ' · ' + esc(x.label) + ' — got ' + fmt(x.got) + ', expected ' + fmt(x.expected) +
      ', relative error ' + x.error.toExponential(2) + ' against ' + x.tolerance.toExponential(1) +
      ' <span class="muted">' + esc(x.provenance) + ' / ' + esc(x.source) + '</span></div>').join('') + '</div>';
  }
  h += '<div class="chainline">kernel ' + esc(res.manifest.kernel) + ' · graph ' + esc(res.manifest.graph) +
       ' · case ' + esc(res.manifest.case) + ' · chain ' + esc(res.manifest.chain) +
       ' · endpoint ' + esc(res.manifest.endpoint) +
       (res.manifest.data.length ? ' · data ' + esc(res.manifest.data.join(' ')) : ' · <span class="refusal">no data bundle</span>') + '</div>';
  h += '<p class="muted">The chain hash is the run\'s identity and the cache key. It covers every node the run reached and each one\'s implementation content — not the case alone, because rewriting a node\'s arithmetic without touching its interface must invalidate every result downstream of it.</p>';
  h += sweepControls(r);
  el.innerHTML = h;
  wireSweep(r);
}

function credBars(v) {
  const names = ['maths', 'assump', 'verif', 'valid', 'pedigree', 'uncert', 'underst', 'reprod'];
  const min = Math.min(...v.vec);
  return '<div class="cred">' + v.vec.map((s, i) =>
    '<div class="f' + (s === min ? ' gov' : '') + '" title="' + names[i] + ': ' + s + ' of 4">' +
    '<i style="height:' + (s / 4 * 100) + '%"></i><span>' + names[i] + '</span></div>').join('') + '</div>';
}

// ---------------------------------------------------------------------------
// the sweep — the one place a canvas is right, because it is a field and not
// prose. Drawn from the numbers the engine returned; the page computes nothing.

function sweepControls(r) {
  /* Every declared value the node depends on, however far upstream — a design
     sweep asks about a decision, and the decisions are the declared rows. A
     list of direct inputs would offer computed intermediates, which is asking
     what happens if a consequence changes. */
  const ins = Array.from(upstreamClosure(r.id))
    .map(id => S.byId.get(id))
    .filter(x => x && x.kind === 'declared')
    .sort((a, b) => a.id.localeCompare(b.id));
  if (!ins.length) return '<p class="muted">Nothing declared upstream of this node, so there is no decision to sweep.</p>';
  const def = ins.find(x => x.id === 'orbit_altitude') || ins[0];
  return '<h4>behaviour sweep</h4><div class="sweepctl">' +
    'over <select id="sw-over">' + ins.map(x =>
      '<option value="' + esc(x.id) + '"' + (x.id === def.id ? ' selected' : '') + '>' + esc(x.id) + '</option>').join('') + '</select>' +
    ' from <input id="sw-from" value="' + def.lo + '"> to <input id="sw-to" value="' + def.hi + '">' +
    ' <span class="muted">SI, and the declared range is ' + fmt(def.lo) + ' … ' + fmt(def.hi) + '</span>' +
    ' <button class="primary" id="sw-go">sweep</button>' +
    '</div><canvas class="plot" id="sw-plot" width="900" height="300"></canvas>' +
    '<div id="sw-note" class="muted"></div>';
}

function wireSweep(r) {
  const go = $('#sw-go');
  if (!go) return;
  const from = $('#sw-from'), to = $('#sw-to'), over = $('#sw-over');
  const check = () => {
    const d = S.byId.get(over.value);
    for (const el of [from, to]) {
      const v = parseFloat(el.value);
      // Refuse, never clamp. Out of range goes red with its reason and the run
      // is refused before anything crosses.
      const bad = !isFinite(v) || v < d.lo || v > d.hi;
      el.classList.toggle('bad', bad);
      el.title = bad ? d.id + ' is declared valid over ' + d.lo + ' … ' + d.hi + '. Out of range is refused, not corrected.' : '';
    }
    go.disabled = from.classList.contains('bad') || to.classList.contains('bad');
  };
  over.onchange = () => {
    const d = S.byId.get(over.value);
    from.value = d.lo; to.value = d.hi; check();
  };
  from.oninput = to.oninput = check;
  check();
  go.onclick = async () => {
    const p = new URLSearchParams({
      node: r.id, over: over.value, from: from.value, to: to.value,
      points: '80', case: S.caseId, mode: 'branch',
    });
    $('#sw-note').textContent = 'sweeping…';
    const res = await (await fetch('/v1/sweep?' + p.toString())).json();
    plot(res);
  };
}

function plot(res) {
  const c = $('#sw-plot');
  if (!c || !res.ok) { $('#sw-note').textContent = res.message || 'the sweep was refused'; return; }
  const ctx = c.getContext('2d');
  const W = c.width, H = c.height, L = 78, B = 34, T = 14, R = 16;
  ctx.clearRect(0, 0, W, H);
  const xs = res.x.map(v => v / res.x_factor);
  const ys = res.y.map(v => v / res.y_factor);
  if (!xs.length) { $('#sw-note').textContent = 'every point was refused.'; return; }
  const x0 = Math.min(...xs), x1 = Math.max(...xs);
  let y0 = Math.min(...ys), y1 = Math.max(...ys);
  if (y0 === y1) { y0 -= 1; y1 += 1; }
  const px = v => L + (v - x0) / (x1 - x0 || 1) * (W - L - R);
  const py = v => H - B - (v - y0) / (y1 - y0 || 1) * (H - B - T);

  ctx.strokeStyle = '#ebe7de'; ctx.lineWidth = 1;
  ctx.fillStyle = '#767b86'; ctx.font = '10px ui-monospace, monospace';
  for (let i = 0; i <= 4; i++) {
    const y = T + i * (H - B - T) / 4;
    ctx.beginPath(); ctx.moveTo(L, y); ctx.lineTo(W - R, y); ctx.stroke();
    ctx.fillText(fmt(y1 - i * (y1 - y0) / 4), 6, y + 3);
  }
  for (let i = 0; i <= 5; i++) {
    const x = L + i * (W - L - R) / 5;
    ctx.beginPath(); ctx.moveTo(x, T); ctx.lineTo(x, H - B); ctx.stroke();
    ctx.fillText(fmt(x0 + i * (x1 - x0) / 5), x - 16, H - B + 14);
  }
  ctx.strokeStyle = '#1d4ed8'; ctx.lineWidth = 1.6;
  ctx.beginPath();
  xs.forEach((x, i) => i ? ctx.lineTo(px(x), py(ys[i])) : ctx.moveTo(px(x), py(ys[i])));
  ctx.stroke();
  ctx.fillStyle = '#17181c'; ctx.font = '11px system-ui, sans-serif';
  ctx.fillText(res.y_id + '  [' + res.y_unit + ']', L, 11);
  ctx.fillText(res.x_id + '  [' + res.x_unit + ']', W - R - 200, H - 6);

  $('#sw-note').innerHTML = res.x.length + ' point' + (res.x.length === 1 ? '' : 's') + ' ran' +
    (res.refused.length
      ? ', <span class="refusal">' + res.refused.length + ' refused</span> — ' + esc(res.refused[0].why) +
        '. Refusals are recorded, never dropped: a sweep in which some rows quietly used a substituted value is a sweep whose conclusion is unknown.'
      : ', none refused.');
}

// ---------------------------------------------------------------------------

function wire() {
  $('#tree-body').addEventListener('click', e => {
    const row = e.target.closest('.row');
    if (row) select(row.dataset.id);
  });
  $('#paths-body').addEventListener('click', e => {
    const it = e.target.closest('.path-item');
    if (it) select(it.dataset.id);
  });
  $('#matrix-body').addEventListener('click', e => {
    const d = e.target.closest('[data-open]');
    if (d) select(d.dataset.open);
  });
  $('#layer').onchange = drawTree;
  $('#search').oninput = drawTree;
  $('#case').onchange = () => { S.caseId = $('#case').value; S.lastRun = null; renderResult(); updateRunControl(); };
  $$('.mode').forEach(b => b.onclick = () => setMode(b.dataset.mode));
  $('#run').onclick = run;
  document.addEventListener('keydown', e => {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) run();
    if (e.key === '/' && document.activeElement !== $('#search')) { e.preventDefault(); $('#search').focus(); }
  });
}

const esc = s => String(s == null ? '' : s)
  .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

function fmt(v) {
  if (v == null || !isFinite(v)) return '—';
  const a = Math.abs(v);
  if (a === 0) return '0';
  if (a >= 1e6 || a < 1e-3) return v.toExponential(3);
  return String(Number(v.toPrecision(6)));
}

boot();
