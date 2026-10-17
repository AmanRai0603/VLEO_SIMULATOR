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

/**
 * The contract this page was built for (contract/README.md). The engine says
 * which it speaks in /v1/version; when they differ the page says so rather
 * than misreading an answer. Held equal to contract/VERSION by the contract
 * tests.
 */
export const CONTRACT = '1';

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
  view: 'rows',        // rows | arch | manual | forms | explore
  arch: 'node',        // which architecture section

  // The inputs a reader has moved: node id -> value in SI. A what-if, held in
  // this browser and sent with every run as `set=`; it never reaches a sheet
  // and never reaches the repository. See inputs.js for why that is the whole
  // design rather than a limitation of it.
  overrides: new Map(),
  // The same run with nothing overridden, kept so "what did this move" can be
  // answered by comparing two runs rather than by trusting one.
  baseline: null,

  // the run
  //
  // THE CASE EVERY NUMBER IS FOR. There is one — `engineCase` is its id, the
  // multipayload design — and what varies is its inputs: the saved case, which
  // the daemon keeps outside the repository and lays under every run it
  // answers. `saved` is what `/v1/inputs` last said about it, and `caseStamp`
  // moves each time it is saved or reset, so a number cached for the old
  // inputs is never shown as one for the new.
  engineCase: '',
  saved: null,
  caseStamp: 0,
  mode: 'branch',
  lastRun: null,
  runTarget: null,
};

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
  // The one case. The engine runs it when none is named, so this is only ever
  // a name to show.
  const first = (S.index.cases || [])[0];
  S.engineCase = first ? first.id : '';
}

/**
 * The layer-3 groups no other layer-3 group holds, in label order. Each is one
 * subsystem layer. A subsystem hangs from the root in the sheets, and from the
 * system block it mounts on in the design's files; either way nothing of its
 * own layer is above it.
 */
export function subsystemLayers() {
  return [...S.G.keys()]
    .filter(id => {
      const g = S.G.get(id);
      const up = S.G.get(g.parent);
      return g.layer === 3 && !(up && up.layer === 3);
    })
    .sort((a, b) => S.G.get(a).label.localeCompare(S.G.get(b).label));
}

/** The one case, as the index describes it. */
export const theCase = () => (S.index.cases || [])[0] || null;

/**
 * What the saved case is, from the daemon: every input, its group, default and
 * range, and the value it runs at when one is saved. Refreshed on load and
 * after every save or reset.
 */
export async function loadSaved() {
  try {
    const d = await (await fetch('/v1/inputs')).json();
    S.saved = d.ok ? d : null;
  } catch (e) {
    S.saved = null;
  }
  return S.saved;
}

/** A short key for "the inputs every run is on now", for caching a run. */
export const caseKey = () => S.engineCase + '#' + S.caseStamp;

// Who else needs telling when the saved case changes. Inputs saves
// and resets it; everything showing a number for the old inputs has to be
// redrawn for the new ones.
const caseListeners = [];
export function onCaseChange(fn) { caseListeners.push(fn); }

/** The saved case changed: re-read it and tell everyone showing a number. */
export async function caseChanged() {
  S.caseStamp += 1;
  S.lastRun = null;
  S.baseline = null;
  await loadSaved();
  for (const f of caseListeners) f();
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

