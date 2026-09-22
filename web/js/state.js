/*
  The one state, and the graphs derived from it.

  There is one selected row, one loaded case and one open view in the whole
  tool. Three drawings of one state, never three states that have to be kept in
  step — that is the class of bug that once put three correct numbers on one
  screen, correct at three different times.

  Every reverse index below is derived on load and never stored, so it cannot
  disagree with the graph it came from. Nothing in this module draws anything.
*/
'use strict';

export const S = {
  version: null,
  index: null,

  rows: [],            // every node row, in table order
  byId: new Map(),     // node id -> row
  consumers: [],       // node index -> [node index]   who reads what this makes
  producers: [],       // node index -> [node index]   what this reads
  G: new Map(),        // group id -> group
  gkids: new Map(),    // group id -> [group id]
  gnodes: new Map(),   // group id -> [node index]

  // what is on screen
  view: 'layer',       // arch | layer | node | run
  arch: 'node',        // which architecture section
  layer: 1,
  subsys: '',
  caseSel: 'c1',
  concept: 'constellation',
  size: 'M',
  expanded: new Set(),
  selected: null,

  // The inputs a reader has moved: node id -> value in SI. A what-if, held in
  // this browser and sent with every run as `set=`; it never reaches a sheet
  // and never reaches the repository. See inputs.js for why that is the whole
  // design rather than a limitation of it.
  overrides: new Map(),
  // The same run with nothing overridden, kept so "what did this move" can be
  // answered by comparing two runs rather than by trusting one.
  baseline: null,

  // the run
  engineCase: 'nominal',
  mode: 'branch',
  lastRun: null,
  runTarget: null,

  // the last walk, so a click can name what it hit
  disp: [],
  dispIndex: new Map(),
  matrixStats: { marks: 0, crossing: 0, below: 0, boxes: 0 },
};

export const SIZES = { S: 18, M: 24, L: 32 };
/** The five pixels the row height adds to the cell, so a mark has a gutter. */
export const PAD = 5;
export const cell = () => SIZES[S.size] + PAD;

/* The four layers. A layer's root is the group the engine reports at that layer
   directly under the tree's root — found, never hard-coded, because a group id
   written down here is a second place the tree is defined and it goes stale the
   first time the tree is reseeded. */
export const LAYERS = {
  1: { root: '', name: 'Management layer' },
  2: { root: '', name: 'The system' },
  3: { root: '', name: 'Subsystem' },
  4: { root: '', name: 'The run' },
};

function findRoots() {
  for (const n of [1, 2]) {
    const g = (S.gkids.get('root') || []).find(id => S.G.get(id).layer === n);
    LAYERS[n].root = g || '';
  }
}

export const CAPTIONS = {
  1: ['Who wants what, what it costs, and what the programme has promised.',
      'One architecture, many cases. A customer is a case, never a copy of the tree.'],
  2: ['The system: what it is made of, and what reads what.',
      'Its row is what a node feeds. Its column is what feeds it.'],
  3: ['One subsystem layer, decomposed until every row is a question one person can answer.',
      'Exactly one row in each layer crosses upward. That row is the whole interface.'],
  4: ['The run. One case, one chain hash, one set of numbers.',
      'A refusal is a value. Nothing here is a magic number the caller may forget to check.'],
};

export const HOWTO = [
  '<b>Pick a layer.</b> Four of them: management, the system, a subsystem, the run. The layer decides what the figure is about; nothing else changes.',
  '<b>Open a branch.</b> Click a box to expand it. A closed box is one row, and every edge inside it rolls up onto that row — so a closed branch never hides a dependency, it only summarises one.',
  '<b>Select a row.</b> The colours are relative to the selection: amber is what it feeds, violet is what feeds it, green is both. Solid is one hop; pale is further down the chain.',
  '<b>Read the matrix.</b> A row is what that node feeds. A column is what feeds it. A mark inside a box is coupling the branch owns; a mark outside one crosses a boundary and needs an interface.',
  '<b>Open the node.</b> Click the numbered square on the diagonal — or press <code>Enter</code>. Eight tabs: question, interface, algorithm, generated code, evidence, flags, credibility, design space.',
  '<b>Run it.</b> Alone, the branch, or everything. The button says what is missing rather than going grey without a reason.',
  '<b>Read the architecture</b> if you are going to add to it. The first tab is the pattern every one of the 1333 folders repeats.',
];

// ---------------------------------------------------------------------------

export async function load() {
  try {
    S.version = await (await fetch('/v1/version')).json();
  } catch (e) {
    S.version = null;
  }
  S.index = await (await fetch('/v1/index')).json();
  ingest();
}

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
  findRoots();
  S.engineCase = S.index.cases.some(c => c.id === 'nominal') ? 'nominal' : S.index.cases[0].id;
  S.subsys = subsystemLayers()[0] || '';
}

/** The layer-3 groups, in label order. Each is one subsystem layer. */
export function subsystemLayers() {
  return (S.gkids.get('root') || [])
    .filter(id => S.G.get(id).layer === 3)
    .sort((a, b) => S.G.get(a).label.localeCompare(S.G.get(b).label));
}

/** A group is in scope unless it names cases and the chosen one is not among them. */
export const caseOk = g => !g || !g.cases.length || g.cases.indexOf(S.caseSel) >= 0;

export function subtreeNodes(gid) {
  const g = S.G.get(gid);
  if (!g || !caseOk(g)) return [];
  let out = (S.gnodes.get(gid) || []).slice();
  for (const k of (S.gkids.get(gid) || [])) out = out.concat(subtreeNodes(k));
  return out;
}

/**
 * The subtree, ignoring the case filter.
 *
 * A case decides what is in scope for a *reading*; it does not change what the
 * architecture is. Anything describing the architecture counts with this, or it
 * reports a box as empty on the day someone looks at it under the other case.
 */
export function subtreeAll(gid) {
  let out = (S.gnodes.get(gid) || []).slice();
  for (const k of (S.gkids.get(gid) || [])) out = out.concat(subtreeAll(k));
  return out;
}

export function layerRoot() {
  return S.layer === 3 ? S.subsys : LAYERS[S.layer].root;
}

/** Everything reachable from `seed` along one of the two derivation directions. */
export function reachFrom(seed, edges) {
  const seen = new Set();
  const stack = seed.slice();
  while (stack.length) {
    const n = stack.pop();
    for (const m of edges[n]) if (!seen.has(m)) { seen.add(m); stack.push(m); }
  }
  return seen;
}

export const isSeeded = r => r && r.state === 'empty';
// A row whose question is still real but whose answer nothing should read any
// more. The face drew it exactly like a live row, so a reader had no way to
// tell a retired relation from a current one without opening it — and a
// deprecated row that looks live is worse than one that is gone, because it
// invites being used.
export const isDeprecated = r => r && r.state === 'deprecated';

/* WHETHER THE ROW ANSWERS, and it is not the same question as `state`.
   `state` is how far through its life the row is; this is whether the design
   has made the number.

   The two halves of the tree answer it differently and they should. An INPUT
   is defined by carrying a value — a default is a decision somebody made, and
   the face exists to let it be moved. A FUNCTION is defined by its derivation:
   not by its expression, which is one line anybody can type, and not by its
   citation, which says a paper exists rather than that the relation came out of
   it. Until that derivation is written the engine refuses the row, so drawing
   it like any other one offers a reader a control that cannot work.

   `fn` and `derived` arrive on every row from the index. Computed in the
   engine, carried here — never re-derived from the sheet text, which would be
   a second implementation of the rule and would go wrong on the first row that
   disagreed. */
export const isUndefined = r => !!(r && r.fn && !r.derived);
/* Whether anybody has read the relation against its source. Weaker, and it
   does NOT silence the row — it is what the mathematics factor is scored on. */
export const isUnconfirmed = r => !!(r && r.fn && r.derived && !r.by);

/* Does this row ANSWER — the two halves together, and retired excluded.

   A deprecated row still answers, deliberately, so a consumer that has not
   migrated keeps working. It is not work in progress, so counting it beside
   the live rows overstates what the tree currently produces: solar read 62
   here when 55 of its rows are live and 7 are retired. */
export const answers = r =>
  !!(r && r.state !== 'empty' && r.state !== 'deprecated' && !isUndefined(r));

/* WHERE THE ANSWER GOES, which is not whether there is one.

   The tree's stated purpose is twelve KPI closures — promises to a customer —
   and every other row exists to move one of them. A row that answers and
   reaches none is not wrong and is not a defect in the row: it is work the
   design is not currently reading. A subsystem can answer on every row it has
   and be wired to nothing, and nothing on the screen used to say so.

   A CONCLUSION is excluded by kind. An achieved row is a margin and a KPI row
   is a promise; both are the end of a chain, so having no consumer is what
   they are for. Marking those would put the best rows in the tree under a
   warning, which is how a warning stops being read. */
export const isConclusion = r => !!(r && (r.kind === 'achieved' || r.kind === 'kpi'));
export const isUnread = r => !!(answers(r) && !isConclusion(r) && !r.kpi_reach);

// Whether the tree lists retired rows. Off by default: a reader opening a
// subsystem should see the work that is live in it.
S.showRetired = false;
