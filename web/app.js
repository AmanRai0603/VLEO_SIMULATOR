/*
  The shell.

  It holds every view and no physics. There is no fast path here for a small
  case: a fast path is a second implementation that will eventually disagree,
  and the disagreement will be found in a review rather than in a test.

  One selected row and one loaded case in the whole tool. The tree, the paths
  column and the matrix are three drawings of one state, never three states that
  have to be kept in step — that is the class of bug that once put three correct
  numbers on one screen, correct at three different times.

  The figure is one grid. The nested boxes on the diagonal are the tree; the
  marks off the diagonal are the dependency. They are not placed side by side,
  because a side-car tree drifts out of alignment as branches open, and a tree
  alone cannot show the second relation at all.
*/
'use strict';

const S = {
  version: null,
  index: null,

  rows: [],            // every node row, in table order
  byId: new Map(),     // node id -> row
  consumers: [],       // i -> [i]   derived on load, never stored
  producers: [],       // i -> [i]
  G: new Map(),        // group id -> group
  gkids: new Map(),    // group id -> [group id]
  gnodes: new Map(),   // group id -> [i]

  layer: 1,
  subsys: '',
  caseSel: 'c1',
  concept: 'constellation',
  size: 'M',
  expanded: new Set(),
  selected: null,
  view: 'layer',       // layer | node | run

  engineCase: 'nominal',
  mode: 'branch',
  lastRun: null,
  runTarget: null,

  disp: [],            // the display rows currently drawn
  dispIndex: new Map(),
};

const $  = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => Array.from(r.querySelectorAll(s));
const esc = s => String(s == null ? '' : s)
  .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

const SIZES = { S: 18, M: 24, L: 32 };
const PAD   = 5;                       // the gap the row height adds to the cell
const cell  = () => SIZES[S.size] + PAD;

const LAYERS = {
  1: { root: 'mgt_orbitt', name: 'Management layer' },
  2: { root: 'sys_root',   name: 'The system' },
  3: { root: '',           name: 'Subsystem' },
  4: { root: '',           name: 'The run' },
};

const CAPTIONS = {
  1: ['Who wants what, what it costs, and what the programme has promised.',
      'One architecture, many cases. A customer is a case, never a copy of the tree.'],
  2: ['The system: what it is made of, and what reads what.',
      'Its row is what a node feeds. Its column is what feeds it.'],
  3: ['One subsystem layer, decomposed until every row is a question one person can answer.',
      'Exactly one row in each layer crosses upward. That row is the whole interface.'],
  4: ['The run. One case, one chain hash, one set of numbers.',
      'A refusal is a value. Nothing here is a magic number the caller may forget to check.'],
};

const HOWTO = [
  '<b>Pick a layer.</b> Four of them: management, the system, a subsystem, the run. The layer decides what the figure is about; nothing else changes.',
  '<b>Open a branch.</b> Click a box to expand it. A closed box is one row, and every edge inside it rolls up onto that row — so a closed branch never hides a dependency, it only summarises one.',
  '<b>Select a row.</b> The colours are relative to the selection: amber is what it feeds, violet is what feeds it, green is both. Solid is one hop; pale is further down the chain.',
  '<b>Read the matrix.</b> A row is what that node feeds. A column is what feeds it. A mark inside a box is coupling the branch owns; a mark outside one crosses a boundary and needs an interface.',
  '<b>Open the node.</b> Click the numbered square on the diagonal — or press <code>Enter</code>. Eight tabs: question, interface, algorithm, generated code, evidence, flags, credibility, design space.',
  '<b>Run it.</b> Alone, the branch, or everything. The button says what is missing rather than going grey without a reason.',
];

// ---------------------------------------------------------------------------
// boot

async function boot() {
  try {
    S.version = await (await fetch('/v1/version')).json();
  } catch (e) {
    $('#status').textContent = 'the engine did not answer — reload';
  }
  S.index = await (await fetch('/v1/index')).json();
  ingest();
  fillSubsys();
  fillHowto();
  wire();
  setLayer(1);
}

/* Reverse indexes are derived on every load and never stored, so they cannot
   disagree with the graph they came from. */
function ingest() {
  S.rows = S.index.rows;
  for (const r of S.rows) S.byId.set(r.id, r);
  S.consumers = S.rows.map(() => []);
  S.producers = S.rows.map(() => []);
  for (const r of S.rows) {
    for (const p of r.in) {
      S.producers[r.i].push(p);
      S.consumers[p].push(r.i);
    }
  }
  for (const g of S.index.groups) {
    S.G.set(g.id, g);
    S.gkids.set(g.id, []);
    S.gnodes.set(g.id, []);
  }
  for (const g of S.index.groups) {
    if (S.G.has(g.parent)) S.gkids.get(g.parent).push(g.id);
  }
  for (const r of S.rows) {
    if (S.gnodes.has(r.parent)) S.gnodes.get(r.parent).push(r.i);
  }
  S.engineCase = S.index.cases.some(c => c.id === 'nominal') ? 'nominal' : S.index.cases[0].id;
}

function fillSubsys() {
  const kids = (S.gkids.get('root') || []).filter(id => S.G.get(id).layer === 3);
  kids.sort((a, b) => S.G.get(a).label.localeCompare(S.G.get(b).label));
  S.subsys = kids[0] || '';
  $('#subsys').innerHTML = kids.map(id => {
    const g = S.G.get(id);
    return '<option value="' + esc(id) + '">' + esc(g.label) +
      ' · ' + (S.gnodes.get(id) || []).length + '</option>';
  }).join('');
  $('#subsys').value = S.subsys;
}

function fillHowto() {
  $('#howto-steps').innerHTML = HOWTO.map(t => '<li>' + t + '</li>').join('');
}

// ---------------------------------------------------------------------------
// the display rows
//
// One walk produces the tree, the paths column and the matrix. A closed box is
// one display row that stands for its whole subtree: its members are every node
// beneath it, so an edge into the subtree becomes a mark on the box. Nothing is
// hidden by closing a branch — it is summarised, and the summary is exact.

const caseOk = g => !g || !g.cases.length || g.cases.indexOf(S.caseSel) >= 0;

function subtreeNodes(gid) {
  const g = S.G.get(gid);
  if (!g || !caseOk(g)) return [];
  let out = (S.gnodes.get(gid) || []).slice();
  for (const k of (S.gkids.get(gid) || [])) out = out.concat(subtreeNodes(k));
  return out;
}

function layerRoot() {
  return S.layer === 3 ? S.subsys : LAYERS[S.layer].root;
}

function buildDisplay() {
  const rows = [];
  const rootId = layerRoot();
  if (!rootId || !S.G.has(rootId)) return rows;

  const rec = (gid, depth, bars, last) => {
    const g = S.G.get(gid);
    if (!caseOk(g)) return;
    const own  = (S.gnodes.get(gid) || []).slice().sort((a, b) => a - b);
    const kids = (S.gkids.get(gid) || []).filter(k => caseOk(S.G.get(k)));
    const has  = own.length > 0 || kids.length > 0;
    const open = has && S.expanded.has(gid);

    rows.push({
      kind: 'group', id: gid, label: g.label, owner: g.owner, tone: g.tone || 'slate',
      depth, bars: bars.slice(), last, hasKids: has, open, box: !!g.box,
      members: open ? [] : subtreeNodes(gid), crosses: '', parent: g.parent,
    });
    if (!open) return;

    const childBars = depth === 0 ? [] : bars.concat([!last]);
    const items = own.map(i => ({ t: 'n', i })).concat(kids.map(k => ({ t: 'g', k })));
    items.forEach((it, n) => {
      const isLast = n === items.length - 1;
      if (it.t === 'g') { rec(it.k, depth + 1, childBars, isLast); return; }
      const r = S.rows[it.i];
      rows.push({
        kind: 'node', id: r.id, label: r.label, owner: r.owner, tone: g.tone || 'slate',
        depth: depth + 1, bars: childBars.slice(), last: isLast,
        hasKids: false, open: false, box: false,
        members: [it.i], crosses: r.crosses, parent: gid, row: r,
      });
    });
  };

  rec(rootId, 0, [], true);
  return rows;
}

function reachFrom(seed, edges) {
  const seen = new Set();
  const stack = seed.slice();
  while (stack.length) {
    const n = stack.pop();
    for (const m of edges[n]) if (!seen.has(m)) { seen.add(m); stack.push(m); }
  }
  return seen;
}

// ---------------------------------------------------------------------------
// drawing

function draw() {
  document.documentElement.style.setProperty('--cell', SIZES[S.size] + 'px');

  $('#figure').hidden  = S.view !== 'layer';
  $('#nodeview').hidden = S.view !== 'node';
  $('#runview').hidden  = S.view !== 'run';
  $('#caption-a').hidden = S.view === 'node';
  $('#caption-b').hidden = S.view === 'node';
  $('#caserow').style.display = S.layer === 4 ? 'none' : '';
  $('#controls').style.display = S.view === 'layer' && S.layer !== 4 ? '' : 'none';
  $('#subsys-grp').style.display = S.layer === 3 ? '' : 'none';
  $('#concept-grp').style.display = S.layer === 1 ? '' : 'none';

  $$('.tab[data-layer]').forEach(b => b.classList.toggle('sel', +b.dataset.layer === S.layer));
  $$('.ctl.sz').forEach(b => b.classList.toggle('sel', b.dataset.size === S.size));
  $$('.ctl.case').forEach(b => b.classList.toggle('sel', b.dataset.case === S.caseSel));
  $$('.ctl.cpt').forEach(b => b.classList.toggle('sel', b.dataset.concept === S.concept));
  $('#concept-tag').textContent = S.concept === 'single' ? 'Single satellite' : 'Constellation';

  const cap = CAPTIONS[S.layer];
  $('#caption-a').textContent = cap[0];
  $('#caption-b').textContent = cap[1];
  drawStepper();

  if (S.layer === 4) { S.view = 'run'; $('#figure').hidden = true; $('#runview').hidden = false; }

  if (S.view === 'run') { drawRunView(); drawStatus([]); drawFoot(); return; }
  if (S.view === 'node') { drawStatus(S.disp); drawFoot(); return; }

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
  drawReach(disp, rel);
  drawNotes(disp, rel);
  drawStatus(disp);
  drawFoot();
}

function drawStepper() {
  const at = S.view === 'run' ? 4 : S.view === 'node' ? 3 : 2;
  const steps = ['pick a layer', 'read the tree', 'open a node', 'run it'];
  const total = S.rows.length;
  $('#stepper').innerHTML = steps.map((t, i) =>
    '<span class="step' + (i + 1 === at ? ' on' : '') + '"><span class="n">' + (i + 1) + '</span>' +
    esc(t) + '</span>' + (i < 3 ? '<span class="dash">—</span>' : '')).join('') +
    '<span class="aside">' + total + ' rows · ' + S.index.groups.length + ' groups · ' +
    S.index.relations.length + ' declared relations</span>';
}

/* Every colour on the page is relative to the selection, and the pairing is the
   same in the tree, the paths column and the matrix, so a reader learns it once. */
function relations(disp) {
  const out = { dOut: new Set(), dIn: new Set(), rOut: new Set(), rIn: new Set(), sel: null, marks: [] };
  const sel = disp.find(d => d.id === S.selected);
  if (!sel) return out;
  out.sel = sel;
  const mem = sel.members;
  for (const i of mem) {
    for (const c of S.consumers[i]) out.dOut.add(c);
    for (const p of S.producers[i]) out.dIn.add(p);
  }
  out.rOut = reachFrom(mem, S.consumers);
  out.rIn  = reachFrom(mem, S.producers);
  return out;
}

function pillClass(d, rel) {
  if (!rel.sel) return '';
  if (d.id === S.selected) return ' sel';
  const hit = (set) => d.members.some(i => set.has(i));
  const o = hit(rel.dOut), n = hit(rel.dIn);
  if (o && n) return ' both';
  if (o) return ' out';
  if (n) return ' in';
  const po = hit(rel.rOut), pn = hit(rel.rIn);
  if (po && pn) return ' both';
  if (po) return ' outp';
  if (pn) return ' inp';
  return '';
}

function drawTree(disp, rel) {
  const selIdx = S.dispIndex.get(S.selected);
  const lit = new Set();
  {
    let d = disp.find(x => x.id === S.selected);
    while (d) { lit.add(d.id); d = disp.find(x => x.id === d.parent); }
  }

  const html = disp.map((d, i) => {
    let g = '';
    for (let k = 0; k < d.depth; k++) {
      const isLast = k === d.depth - 1;
      const on = isLast ? lit.has(d.id) : (lit.has(ancestorAt(disp, i, k)) && i <= selIdx);
      const cls = isLast ? (d.last ? 'elbow' : 'tee') : (d.bars[k] ? 'line' : '');
      g += '<i class="' + cls + (cls && on ? ' lit' : '') + '"></i>';
    }
    const tri = d.kind === 'group' && d.hasKids
      ? '<span class="tri">' + (d.open ? '▾' : '▸') + '</span>' : '';
    const seeded = d.kind === 'node' && d.row.state === 'empty';
    const txt = seeded ? '<span class="txt seeded">' + esc(d.label) + '</span>'
                       : '<span class="txt">' + esc(d.label) + '</span>';
    const n = d.kind === 'group' && !d.open ? '<span class="tri">' + d.members.length + '</span>' : '';
    const title = d.kind === 'node'
      ? d.row.id + (d.row.question ? ' — ' + d.row.question : ' — not yet specified')
      : d.label + ' — ' + subtreeNodes(d.id).length + ' rows, owner ' + d.owner;
    return '<div class="trow" data-i="' + i + '">' +
      '<span class="gutter">' + g + '</span>' +
      '<span class="pill t-' + esc(d.tone) + pillClass(d, rel) +
        (d.crosses ? ' crossing' : '') + '" title="' + esc(title) + '">' +
        tri + txt + n +
      '</span></div>';
  }).join('');
  $('#tree').innerHTML = html;
}

function ancestorAt(disp, i, level) {
  let d = disp[i];
  while (d && d.depth > level) d = disp.find(x => x.id === d.parent);
  return d ? d.id : '';
}

// ---------------------------------------------------------------------------
// the paths column — numbered, and one arc per direct neighbour of the
// selection. It is the join between a name in the tree and a mark in the grid.

function drawPaths(disp, rel) {
  const C = cell();
  const dots = $('#paths-dots');
  dots.innerHTML = disp.map((d, i) =>
    '<div class="pdot' + (d.id === S.selected ? ' sel' : '') + '">' + (i + 1) + '</div>').join('');

  const svg = $('#paths-svg');
  svg.style.top = dots.offsetTop + 'px';
  svg.setAttribute('height', disp.length * C);
  svg.setAttribute('viewBox', '0 0 62 ' + (disp.length * C));
  if (!rel.sel) { svg.innerHTML = ''; return; }

  const k = S.dispIndex.get(S.selected);
  const y = n => (n + 0.5) * C;
  const arcs = [];
  disp.forEach((d, i) => {
    if (i === k) return;
    const o = d.members.some(m => rel.dOut.has(m));
    const n = d.members.some(m => rel.dIn.has(m));
    if (!o && !n) return;
    const col = o && n ? 'var(--both)' : o ? 'var(--out)' : 'var(--in)';
    arcs.push('<path d="M 22 ' + y(i).toFixed(1) + ' C 42 ' + y(i).toFixed(1) +
      ', 42 ' + y(k).toFixed(1) + ', 60 ' + y(k).toFixed(1) +
      '" fill="none" stroke="' + col + '" stroke-width="1" opacity=".75"/>');
  });
  svg.innerHTML = arcs.join('');
}

// ---------------------------------------------------------------------------
// the matrix
//
// A flat 1333x1333 grid is 1.8 million cells of which 0.03% are filled and is
// useless; a rollup to eight branches is one cell in sixty-four and is useless
// for the opposite reason. The nested form keeps every subtree a contiguous
// band, so a mark inside a box is coupling that subtree owns and a mark outside
// it crosses a boundary that needs an interface.

function drawMatrix(disp, rel) {
  const C = cell();
  const n = disp.length;
  const inner = $('#matrix-inner');
  const cells = $('#matrix-cells');
  const svg = $('#matrix-svg');
  inner.style.width = (n * C) + 'px';
  inner.style.height = (n * C) + 'px';
  cells.style.width = (n * C) + 'px';
  cells.style.height = (n * C) + 'px';

  const owner = new Map();          // node index -> display row that carries it
  disp.forEach((d, i) => { for (const m of d.members) owner.set(m, i); });

  const marks = new Map();          // "r:c" -> true
  for (let r = 0; r < n; r++) {
    for (const m of disp[r].members) {
      for (const c of S.consumers[m]) {
        const j = owner.get(c);
        if (j == null || j === r) continue;
        marks.set(r + ':' + j, true);
      }
    }
  }

  const k = S.dispIndex.get(S.selected);
  const out = [];

  if (k != null) {
    out.push('<div class="band rband" style="top:' + (k * C) + 'px;height:' + C + 'px;width:' + (n * C) + 'px"></div>');
    out.push('<div class="band cband" style="left:' + (k * C) + 'px;width:' + C + 'px;height:' + (n * C) + 'px"></div>');
  }

  // Nested boxes: every open group is a contiguous band, drawn as the tree
  // inside the matrix.
  let crossing = 0, below = 0;
  const span = new Map();
  for (let i = n - 1; i >= 0; i--) {
    let end = i;
    for (let j = i + 1; j < n; j++) { if (disp[j].depth > disp[i].depth) end = j; else break; }
    span.set(i, end);
    if (disp[i].kind === 'group' && disp[i].open && end > i) {
      const x = i * C, w = (end - i + 1) * C;
      out.push('<div class="boxrect" style="left:' + x + 'px;top:' + x + 'px;width:' + w + 'px;height:' + w + 'px"></div>');
      out.push('<div class="boxlabel" style="left:' + (x + C + 3) + 'px;top:' + (x + 3) + 'px">' +
        esc(disp[i].label) + '</div>');
    }
  }

  for (let i = 0; i < n; i++) {
    const d = disp[i];
    const seeded = d.kind === 'node' && d.row.state === 'empty';
    out.push('<div class="mcell diag' + (d.id === S.selected ? ' sel' : '') + (seeded ? ' seeded' : '') +
      '" data-open="' + esc(d.id) + '" data-i="' + i + '" title="' + esc(d.label) +
      ' — click to open" style="left:' + (i * C) + 'px;top:' + (i * C) + 'px">' +
      '<span class="num">' + (i + 1) + '</span></div>');
  }

  for (const key of marks.keys()) {
    const [r, c] = key.split(':').map(Number);
    const inside = disp[r].parent === disp[c].parent;
    if (!inside) crossing++;
    if (c < r) below++;
    let cls = 'mk ' + (inside ? 'inside' : 'crosses');
    if (r === k) cls = 'mk rowsel';
    if (c === k) cls = 'mk colsel';
    if (c < r) cls += ' below';
    out.push('<div class="mcell" style="left:' + (c * C) + 'px;top:' + (r * C) + 'px" title="' +
      esc(disp[r].label) + ' feeds ' + esc(disp[c].label) + '"><i class="' + cls + '"></i></div>');
  }

  cells.innerHTML = out.join('');
  S.matrixStats = { marks: marks.size, crossing, below, boxes: [...span.keys()].filter(i => disp[i].kind === 'group' && disp[i].open && span.get(i) > i).length };

  // The routed arrows: out along the selected row, then down the far column
  // into that node's own square. Two straight legs, because a curve across a
  // grid cannot be followed back to the cell it came from.
  svg.setAttribute('width', n * C);
  svg.setAttribute('height', n * C);
  if (k == null) { svg.innerHTML = ''; return; }
  const mid = i => i * C + C / 2;
  const legs = [];
  for (const key of marks.keys()) {
    const [r, c] = key.split(':').map(Number);
    if (r === k) legs.push(leg(mid(k), mid(k), mid(c), mid(k), mid(c), mid(c), 'var(--out)'));
    else if (c === k) legs.push(leg(mid(r), mid(r), mid(k), mid(r), mid(k), mid(k), 'var(--in)'));
  }
  svg.innerHTML =
    '<defs><marker id="ah-out" viewBox="0 0 8 8" refX="6" refY="4" markerWidth="5" markerHeight="5" orient="auto">' +
    '<path d="M0 0 L8 4 L0 8 z" fill="var(--out)"/></marker>' +
    '<marker id="ah-in" viewBox="0 0 8 8" refX="6" refY="4" markerWidth="5" markerHeight="5" orient="auto">' +
    '<path d="M0 0 L8 4 L0 8 z" fill="var(--in)"/></marker></defs>' + legs.join('');
}

function leg(x0, y0, x1, y1, x2, y2, col) {
  const head = col === 'var(--out)' ? 'ah-out' : 'ah-in';
  return '<polyline points="' + [x0, y0, x1, y1, x2, y2].map(v => v.toFixed(1)).join(' ') +
    '" fill="none" stroke="' + col + '" stroke-width="1.2" opacity=".85" marker-end="url(#' + head + ')"/>';
}

// ---------------------------------------------------------------------------

function drawReach(disp, rel) {
  const d = rel.sel;
  if (!d) { $('#reach').textContent = ''; return; }
  const stale = rel.rOut.size;
  const up = rel.rIn.size;
  $('#reach').innerHTML =
    'Change <b>' + esc(d.label) + '</b> and <b>' + stale + '</b> node' + (stale === 1 ? '' : 's') +
    ' become stale — unknown, not wrong. It stands on <b>' + up + '</b> node' + (up === 1 ? '' : 's') +
    ' upstream. Both counted from the graph on this draw, never remembered.';
}

function drawNotes(disp, rel) {
  const st = S.matrixStats || { marks: 0, crossing: 0, below: 0, boxes: 0 };
  const seeded = S.rows.filter(r => r.state === 'empty').length;
  const rels = S.index.relations.filter(r => S.dispIndex.has(r.from) || S.dispIndex.has(r.to));
  const put = [
    ['a row', 'One small question, one answer, one folder, one row. The variable id <b>is</b> the node id.'],
    ['a box', 'A group of rows with an owner. Closed, it is one row and every edge inside it rolls up onto that row — summarised, never hidden.'],
    ['a mark', 'The row feeds the column. <b>' + st.marks + '</b> marks are drawn here; <b>' + st.crossing + '</b> of them leave their own box and so need an interface.'],
    ['against tree order', '<b>' + st.below + '</b> marks sit below the diagonal — the row feeds something earlier in the tree. That is either a declared cycle or a decomposition that has the order wrong.'],
    ['crossing upward', 'Exactly one row in each subsystem layer carries that layer to the system. It is marked with a thin teal outline.'],
    ['a case', 'A case selects which boxes are in scope. It is never a copy of the tree: a cloned architecture is two architectures that will disagree.'],
    ['seeded', '<b>' + seeded + '</b> of ' + S.rows.length + ' rows are seeded — the folder, the sheet and the row exist, and nothing is specified in them. Running one returns <code>NotRun</code>, by name.'],
    ['a relation', '<b>' + rels.length + '</b> of ' + S.index.relations.length + ' declared group relations touch this layer. A relation is stated by the layer file, never inferred from the marks.'],
  ];
  $('#notes').innerHTML = put.map(([k, v]) => '<dt>' + k + '</dt><dd>' + v + '</dd>').join('');
}

function drawStatus(disp) {
  const st = S.matrixStats || { crossing: 0, below: 0, boxes: 0 };
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
    $('#status').textContent = ['Layer 4', 'The run', r ? r.id : '—',
      S.engineCase, S.mode].filter(Boolean).join(' · ');
    $('#caserow-note').textContent = '';
    return;
  }
  const sel = S.selected ? S.selected : '—';
  const caseLabel = S.caseSel.toUpperCase();
  const bits = ['Layer ' + S.layer, LAYERS[S.layer].name];
  if (S.layer === 3 && S.G.has(S.subsys)) bits[1] = S.G.get(S.subsys).label;
  bits.push(caseLabel, sel);
  bits.push(disp.length + ' rows', st.boxes + ' box' + (st.boxes === 1 ? '' : 'es'),
    st.crossing + ' crossing', st.below + ' against tree order');
  $('#status').textContent = bits.join(' · ');
  const g = S.G.get(layerRoot());
  $('#caserow-note').textContent = S.layer === 1
    ? 'the same architecture, read as ' + (S.concept === 'single' ? 'one satellite' : 'a constellation')
    : g ? 'owner ' + g.owner : '';
}

function drawFoot() {
  const v = S.version || {};
  const k = {};
  for (const r of S.rows) k[r.kind] = (k[r.kind] || 0) + 1;
  const data = (v.data || []);
  $('#foot').innerHTML =
    'kernel <b>' + esc(v.kernel || '—') + '</b> · graph <b>' + esc(v.graph || '—') +
    '</b> · engine <b>' + esc(v.endpoint || 'none') + '</b> · ' +
    Object.entries(k).map(([a, b]) => b + ' ' + a).join(' · ') +
    ' · ' + (data.length
      ? 'data <b>' + esc(data.map(x => x.split('#')[0]).join(' · ')) + '</b>'
      : 'data <b>none synced</b> — every node that declares a bundle will refuse, naming what to sync');
}

// ---------------------------------------------------------------------------
// selection and navigation

function select(id) {
  if (!S.dispIndex.has(id) && !S.byId.has(id)) return;
  S.selected = id;
  if (S.view !== 'layer') { S.view = 'layer'; }
  draw();
  const el = $('#matrix-cells .mcell.diag.sel');
  if (el) el.scrollIntoView({ block: 'nearest', inline: 'nearest' });
}

function toggle(id) {
  if (S.expanded.has(id)) S.expanded.delete(id); else S.expanded.add(id);
  S.selected = id;
  draw();
}

function setLayer(n) {
  S.layer = n;
  S.view = n === 4 ? 'run' : 'layer';
  S.expanded = new Set([layerRoot()]);
  S.selected = null;
  const g = $('.grid');
  if (g) { g.scrollTop = 0; g.scrollLeft = 0; }
  draw();
}

/* A node the current layer does not show is still reachable — following a
   cross-reference moves the layer to wherever that node lives. */
function goto(id) {
  const r = S.byId.get(id);
  if (!r) return;
  if (r.layer !== S.layer) {
    S.layer = r.layer;
    S.expanded = new Set([layerRoot()]);
  }
  if (r.layer === 3) {
    let g = S.G.get(r.parent);
    while (g && g.parent !== 'root') g = S.G.get(g.parent);
    if (g) { S.subsys = g.id; $('#subsys').value = g.id; S.expanded = new Set([g.id]); }
  }
  let g = S.G.get(r.parent);
  while (g) { S.expanded.add(g.id); g = S.G.get(g.parent); }
  S.selected = id;
  S.view = 'layer';
  draw();
}

async function openNode(id) {
  const r = S.byId.get(id);
  if (!r) return;
  S.selected = id;
  S.runTarget = id;
  S.view = 'node';
  draw();
  const body = $('#node-body');
  body.innerHTML = '<p class="muted">loading the sheet…</p>';
  try {
    body.innerHTML = await (await fetch('/v1/fragment/' + encodeURIComponent(id))).text();
  } catch (e) {
    body.innerHTML = '<p class="empty">The sheet for <code>' + esc(id) +
      '</code> is not on disk. Run <code>cargo xtask docs</code>.</p>';
  }
  $$('.tabs .tab', body).forEach(t => {
    t.classList.add('tab-btn');
    t.onclick = () => {
      $$('.tabs .tab', body).forEach(x => x.classList.remove('sel'));
      $$('[data-panel]', body).forEach(x => x.classList.remove('sel'));
      t.classList.add('sel');
      const p = $('[data-panel="' + t.dataset.tab + '"]', body);
      if (p) p.classList.add('sel');
    };
  });
  $$('.xref[data-goto]', body).forEach(a => a.onclick = () => goto(a.dataset.goto));
  $$('.xref[data-group]', body).forEach(a => a.onclick = () => {
    const g = S.G.get(a.dataset.group);
    if (!g) return;
    S.view = 'layer';
    S.selected = g.id;
    S.expanded.add(g.id);
    draw();
  });
  renderRun($('#run-panel'), r);
}

// ---------------------------------------------------------------------------
// the run
//
// A disabled control that does not say why is a defect. "Alone" is available
// only when every dependency already holds a value, and when it is not it names
// what is missing.

function drawRunView() {
  const id = S.runTarget || (S.byId.has('prop_thrust_to_drag') ? 'prop_thrust_to_drag' : S.rows[0].id);
  S.runTarget = id;
  renderRun($('#run-body'), S.byId.get(id), true);
}

function renderRun(host, r, standalone) {
  if (!host || !r) return;
  const known = new Set((S.lastRun && S.lastRun.values || []).map(v => v.id));
  const missing = r.in.map(i => S.rows[i].id).filter(x => !known.has(x));
  const closure = reachFrom([r.i], S.producers).size + 1;
  const seeded = r.state === 'empty';

  let h = '';
  if (standalone) {
    h += '<div class="node-head"><h2>' + esc(r.label) + '</h2>' +
      '<p class="ident"><code>' + esc(r.id) + '</code> · <span class="kind">' + esc(r.kind) +
      '</span> · owner <b>' + esc(r.owner) + '</b> · layer <b>' + r.layer + '</b></p></div>' +
      '<div class="sweepctl">target <select id="run-target">' +
      S.rows.filter(x => x.state !== 'empty').map(x =>
        '<option value="' + esc(x.id) + '"' + (x.id === r.id ? ' selected' : '') + '>' +
        esc(x.id) + '</option>').join('') + '</select>' +
      '<span class="muted">only rows with something specified in them can be a target</span></div>';
  }
  h += '<div class="runbar">' +
    '<button class="ctl mode' + (S.mode === 'alone' ? ' sel' : '') + '" data-mode="alone">alone</button>' +
    '<button class="ctl mode' + (S.mode === 'branch' ? ' sel' : '') + '" data-mode="branch">the branch</button>' +
    '<button class="ctl mode' + (S.mode === 'all' ? ' sel' : '') + '" data-mode="all">everything</button>' +
    '<span class="lbl">case</span><select class="ctl" id="engine-case">' +
    S.index.cases.map(c => '<option value="' + esc(c.id) + '"' +
      (c.id === S.engineCase ? ' selected' : '') + ' title="' + esc(c.note) + '">' +
      esc(c.label) + '</option>').join('') + '</select>' +
    '<button class="ctl" id="run-go"' + (seeded ? ' disabled' : '') + '>run</button>' +
    '<span class="why" id="run-why"></span></div>';

  if (seeded) {
    h += '<p class="empty">This row is seeded. The folder, the sheet and the row exist; nothing is ' +
      'specified in them yet, so the generated stub returns <code>NotRun</code> rather than a number. ' +
      'Six hundred grey rows on day one is not a failure — it is the decomposition, written down before ' +
      'anyone has been told to fill it in.</p>';
  }
  h += '<div id="run-out"></div>';
  host.innerHTML = h;

  const why = $('#run-why', host);
  const aloneBtn = $('.mode[data-mode="alone"]', host);
  aloneBtn.disabled = missing.length > 0;
  aloneBtn.title = missing.length
    ? 'Disabled: ' + missing.slice(0, 4).join(', ') +
      (missing.length > 4 ? ' and ' + (missing.length - 4) + ' more' : '') + ' ' +
      (missing.length === 1 ? 'has' : 'have') + ' never run. Run the branch instead.'
    : 'Only this node. Upstream values are whatever the store already holds.';
  if (aloneBtn.disabled && S.mode === 'alone') S.mode = 'branch';
  why.textContent = S.mode === 'branch' ? closure + ' nodes in the closure'
    : S.mode === 'all' ? S.rows.length + ' rows, everything buildable'
    : '1 node';

  $$('.mode', host).forEach(b => b.onclick = () => { S.mode = b.dataset.mode; renderRun(host, r, standalone); });
  $('#engine-case', host).onchange = e => { S.engineCase = e.target.value; S.lastRun = null; renderResult(host, r); };
  $('#run-go', host).onclick = () => run(r, host, standalone);
  const t = $('#run-target', host);
  if (t) t.onchange = e => { S.runTarget = e.target.value; drawRunView(); };
  renderResult(host, r);
}

async function run(r, host, standalone) {
  const btn = $('#run-go', host);
  btn.disabled = true;
  $('#run-why', host).textContent = 'running…';
  const body = new URLSearchParams();
  body.set('node', r.id);
  body.set('mode', S.mode);
  body.set('case', S.engineCase);
  let res;
  try {
    res = await (await fetch('/v1/run', {
      method: 'POST',
      headers: { 'content-type': 'application/x-www-form-urlencoded' },
      body: body.toString(),
    })).json();
  } catch (e) {
    res = { ok: false, fault: 'no answer', node: r.id, message: 'the engine did not answer: ' + e };
  }
  S.lastRun = res.ok ? res : null;
  renderRun(host, r, standalone);
  renderResult(host, r, res);
}

function renderResult(host, r, res) {
  const el = $('#run-out', host);
  if (!el) return;
  res = res || S.lastRun;
  if (!res) {
    el.innerHTML = '<p class="muted">Not run. The eight tabs are what the node <i>is</i>; ' +
      'this is what it <i>returns</i>.</p>' + sweepControls(r);
    wireSweep(host, r);
    return;
  }
  if (!res.ok) {
    el.innerHTML = '<div class="blocked"><b>' + esc(res.fault || 'refused') + '</b> · ' +
      esc(res.node || '') + '<div>' + esc(res.message || '') + '</div></div>' +
      '<p class="muted">A refusal is a value, not a magic number the caller may forget to check. ' +
      'It names the field, the bound it broke and the reason that bound exists.</p>' + sweepControls(r);
    wireSweep(host, r);
    return;
  }
  const v = res.values.find(x => x.id === r.id);
  let h = '';
  if (v) {
    h += '<div class="answer">' + esc(v.symbol || r.symbol) + ' = ' + esc(v.shown) +
      ' <span class="unit">' + esc(v.unit) + '</span></div>' + credBars(v) +
      '<p class="muted">Eight factors, and the lowest governs — averaging would let a strong factor ' +
      'hide a zero, which is the whole failure the score exists to prevent. This node is held down by ' +
      '<b>' + esc(v.governing) + '</b>. Never stored: a badge read out of a field is a claim about last March.</p>';
  } else {
    h += '<div class="answer none">not computed on this run</div>';
  }
  h += '<div class="chainline">' + res.manifest.ran + ' ran · ' + res.manifest.blocked +
    ' blocked · ' + res.manifest.iterations + ' cycle sweep' +
    (res.manifest.iterations === 1 ? '' : 's') + '</div>';
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
    (res.manifest.data.length ? ' · data ' + esc(res.manifest.data.join(' ')) : ' · no data bundle') + '</div>';
  h += '<p class="muted">The chain hash is the run\'s identity and its cache key. It covers every node ' +
    'the run reached and each one\'s implementation content — not the case alone, because rewriting a ' +
    'node\'s arithmetic without touching its interface must invalidate every result downstream of it.</p>';
  h += sweepControls(r);
  el.innerHTML = h;
  wireSweep(host, r);
}

function credBars(v) {
  const names = ['maths', 'assump', 'verif', 'valid', 'pedigree', 'uncert', 'underst', 'reprod'];
  const min = Math.min.apply(null, v.vec);
  return '<div class="cred">' + v.vec.map((s, i) =>
    '<div class="f' + (s === min ? ' gov' : '') + '" title="' + names[i] + ': ' + s + ' of 4">' +
    '<i style="height:' + (s / 4 * 100) + '%"></i><span>' + names[i] + '</span></div>').join('') + '</div>';
}

// ---------------------------------------------------------------------------
// the sweep — the one place a canvas is right, because it is a field and not
// prose. Drawn from the numbers the engine returned; the page computes nothing.

function sweepControls(r) {
  const ins = Array.from(reachFrom([r.i], S.producers))
    .map(i => S.rows[i])
    .filter(x => x.kind === 'declared' && x.hi > x.lo)
    .sort((a, b) => a.id.localeCompare(b.id));
  if (!ins.length) {
    return '<p class="muted">Nothing declared upstream of this node with a range, so there is no ' +
      'decision to sweep.</p>';
  }
  const def = ins.find(x => x.id === 'orbit_altitude') || ins[0];
  return '<h4>behaviour sweep</h4><div class="sweepctl">over <select id="sw-over">' +
    ins.map(x => '<option value="' + esc(x.id) + '"' + (x.id === def.id ? ' selected' : '') + '>' +
      esc(x.id) + '</option>').join('') + '</select>' +
    ' from <input id="sw-from" value="' + def.lo + '"> to <input id="sw-to" value="' + def.hi + '">' +
    ' <span class="muted">SI, and the declared range is ' + fmt(def.lo) + ' … ' + fmt(def.hi) + '</span>' +
    ' <button class="ctl" id="sw-go">sweep</button></div>' +
    '<canvas class="plot" id="sw-plot" width="900" height="300"></canvas>' +
    '<div id="sw-note" class="muted"></div>';
}

function wireSweep(host, r) {
  const go = $('#sw-go', host);
  if (!go) return;
  const from = $('#sw-from', host), to = $('#sw-to', host), over = $('#sw-over', host);
  const check = () => {
    const d = S.byId.get(over.value);
    for (const el of [from, to]) {
      const v = parseFloat(el.value);
      // Refuse, never clamp. A value silently corrected is a design that
      // drifted without anyone deciding to.
      const bad = !isFinite(v) || v < d.lo || v > d.hi;
      el.classList.toggle('bad', bad);
      el.title = bad ? d.id + ' is declared valid over ' + d.lo + ' … ' + d.hi +
        '. Out of range is refused, not corrected.' : '';
    }
    go.disabled = from.classList.contains('bad') || to.classList.contains('bad');
  };
  over.onchange = () => { const d = S.byId.get(over.value); from.value = d.lo; to.value = d.hi; check(); };
  from.oninput = to.oninput = check;
  check();
  go.onclick = async () => {
    const p = new URLSearchParams({
      node: r.id, over: over.value, from: from.value, to: to.value,
      points: '80', case: S.engineCase, mode: 'branch',
    });
    $('#sw-note', host).textContent = 'sweeping…';
    plot(host, await (await fetch('/v1/sweep?' + p.toString())).json());
  };
}

function plot(host, res) {
  const c = $('#sw-plot', host);
  if (!c) return;
  if (!res.ok) { $('#sw-note', host).textContent = res.message || 'the sweep was refused'; return; }
  const ctx = c.getContext('2d');
  const W = c.width, H = c.height, L = 78, B = 34, T = 14, R = 16;
  ctx.clearRect(0, 0, W, H);
  const xs = res.x.map(v => v / res.x_factor);
  const ys = res.y.map(v => v / res.y_factor);
  if (!xs.length) { $('#sw-note', host).textContent = 'every point was refused.'; return; }
  const x0 = Math.min.apply(null, xs), x1 = Math.max.apply(null, xs);
  let y0 = Math.min.apply(null, ys), y1 = Math.max.apply(null, ys);
  if (y0 === y1) { y0 -= 1; y1 += 1; }
  const px = v => L + (v - x0) / (x1 - x0 || 1) * (W - L - R);
  const py = v => H - B - (v - y0) / (y1 - y0 || 1) * (H - B - T);

  ctx.strokeStyle = '#ece8de'; ctx.lineWidth = 1;
  ctx.fillStyle = '#8a8880'; ctx.font = '10px ui-monospace, monospace';
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
  ctx.strokeStyle = '#b5731a'; ctx.lineWidth = 1.6;
  ctx.beginPath();
  xs.forEach((x, i) => i ? ctx.lineTo(px(x), py(ys[i])) : ctx.moveTo(px(x), py(ys[i])));
  ctx.stroke();
  ctx.fillStyle = '#1a1a1a'; ctx.font = '11px ui-monospace, monospace';
  ctx.fillText(res.y_id + '  [' + res.y_unit + ']', L, 11);
  ctx.fillText(res.x_id + '  [' + res.x_unit + ']', W - R - 220, H - 6);

  $('#sw-note', host).innerHTML = res.x.length + ' point' + (res.x.length === 1 ? '' : 's') + ' ran' +
    (res.refused.length
      ? ', <b>' + res.refused.length + ' refused</b> — ' + esc(res.refused[0].why) +
        '. Refusals are recorded, never dropped: a sweep in which some rows quietly used a substituted ' +
        'value is a sweep whose conclusion is unknown.'
      : ', none refused.');
}

// ---------------------------------------------------------------------------

function wire() {
  $$('.tab[data-layer]').forEach(b => b.onclick = () => setLayer(+b.dataset.layer));
  $('#prev').onclick = () => setLayer(Math.max(1, S.layer - 1));
  $('#next').onclick = () => setLayer(Math.min(4, S.layer + 1));

  $('#expand-all').onclick = () => {
    const add = gid => { S.expanded.add(gid); (S.gkids.get(gid) || []).forEach(add); };
    add(layerRoot());
    draw();
  };
  $('#collapse-all').onclick = () => { S.expanded = new Set([layerRoot()]); draw(); };
  $('#howto').onclick = () => { $('#howto-panel').hidden = !$('#howto-panel').hidden; };
  $('#howto-close').onclick = () => { $('#howto-panel').hidden = true; };

  $$('.ctl.sz').forEach(b => b.onclick = () => { S.size = b.dataset.size; draw(); });
  $$('.ctl.cpt').forEach(b => b.onclick = () => { S.concept = b.dataset.concept; draw(); });
  $$('.ctl.case').forEach(b => b.onclick = () => { S.caseSel = b.dataset.case; draw(); });
  $('#subsys').onchange = e => { S.subsys = e.target.value; setLayer(3); };

  $('#tree').addEventListener('click', e => {
    const row = e.target.closest('.trow');
    if (!row) return;
    const d = S.disp[+row.dataset.i];
    if (!d) return;
    if (d.kind === 'group' && (d.hasKids && (d.id === S.selected || e.target.closest('.tri')))) toggle(d.id);
    else select(d.id);
  });
  $('#tree').addEventListener('dblclick', e => {
    const row = e.target.closest('.trow');
    if (!row) return;
    const d = S.disp[+row.dataset.i];
    if (d && d.kind === 'node') openNode(d.id);
    else if (d) toggle(d.id);
  });
  $('#paths-dots').addEventListener('click', e => {
    const dot = e.target.closest('.pdot');
    if (!dot) return;
    const i = Array.prototype.indexOf.call(dot.parentNode.children, dot);
    if (S.disp[i]) select(S.disp[i].id);
  });
  $('#matrix-cells').addEventListener('click', e => {
    const c = e.target.closest('.mcell.diag');
    if (!c) return;
    const d = S.disp[+c.dataset.i];
    if (!d) return;
    if (d.kind === 'node') openNode(d.id); else toggle(d.id);
  });
  $('#back').onclick = () => { S.view = 'layer'; draw(); };

  document.addEventListener('keydown', e => {
    if (e.target.tagName === 'INPUT' || e.target.tagName === 'SELECT') return;
    if (S.view !== 'layer') {
      if (e.key === 'Escape') { S.view = 'layer'; draw(); }
      return;
    }
    const i = S.dispIndex.get(S.selected);
    if (e.key === 'ArrowDown' && i != null && i + 1 < S.disp.length) { e.preventDefault(); select(S.disp[i + 1].id); }
    if (e.key === 'ArrowUp' && i != null && i > 0) { e.preventDefault(); select(S.disp[i - 1].id); }
    if ((e.key === 'ArrowRight' || e.key === 'ArrowLeft') && i != null) {
      const d = S.disp[i];
      if (d.kind === 'group' && d.hasKids) { e.preventDefault(); toggle(d.id); }
    }
    if (e.key === 'Enter' && i != null) {
      e.preventDefault();
      const d = S.disp[i];
      if (d.kind === 'node') openNode(d.id); else toggle(d.id);
    }
  });
}

function fmt(v) {
  if (v == null || !isFinite(v)) return '—';
  const a = Math.abs(v);
  if (a === 0) return '0';
  if (a >= 1e6 || a < 1e-3) return v.toExponential(3);
  return String(Number(v.toPrecision(6)));
}

boot();
