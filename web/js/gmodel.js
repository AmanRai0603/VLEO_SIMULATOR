/*
  The group folder, read once into the shape every view and the checker use.

  Nothing here judges: a file that is missing is null, a table that did not
  parse keeps its problems, and the checker (gcheck.js) says what that means.
  The wiring is DERIVED — each node's inputs.csv says where every input comes
  from, and the arrows of the group are those, never typed a second time.
*/
'use strict';

import { parseCsv, records, splitUnit } from './csv.js';

const CSV = ['group.csv', 'members.csv', 'nodes.csv', 'publishes.csv', 'requirements.csv', 'loops.csv',
  'constants.csv', 'sources.csv', 'versions.csv', 'equations.csv', 'symbols.csv', 'figures.csv', 'reviews.csv',
  'results/group.csv'];
const TEXT = ['explanation.md', 'theory.md', 'flow.txt'];
const NODE_CSV = ['inputs.csv', 'evidence.csv', 'equations.csv', 'symbols.csv', 'figures.csv', 'sources.csv', 'results/isolation.csv'];
const NODE_TEXT = ['explanation.md', 'theory.md', 'pseudocode.txt', 'results/how-run.md'];

/** Read a whole folder. Answers `{ folder, group, nodes: Map, order, edges, external, problems }`. */
export async function loadGroup(folder) {
  const csv = async p => {
    const t = await folder.text(p);
    return t === null ? null : parseCsv(t);
  };
  const g = { files: {}, text: {} };
  for (const p of CSV) g.files[p] = await csv(p);
  for (const p of TEXT) g.text[p] = await folder.text(p);
  const meta = g.files['group.csv'] ? records(g.files['group.csv'])[0] || {} : {};
  const nodeRows = g.files['nodes.csv'] ? records(g.files['nodes.csv']) : [];

  const nodes = new Map();
  for (const row of nodeRows) {
    if (!row.id) continue;
    const dir = 'nodes/' + row.id + '/';
    const n = { id: row.id, row, dir, files: {}, text: {}, present: folder.list(dir).length > 0 };
    for (const p of NODE_CSV) n.files[p] = await csv(dir + p);
    for (const p of NODE_TEXT) n.text[p] = await folder.text(dir + p);
    n.inputs = n.files['inputs.csv'] ? records(n.files['inputs.csv']) : [];
    n.code = folder.list(dir + 'code/');
    n.images = folder.list(dir + 'images/');
    nodes.set(row.id, n);
  }
  // Folders under nodes/ that nodes.csv does not list.
  const stray = [...new Set(folder.list('nodes/').map(p => p.split('/')[1]))].filter(id => id && !nodes.has(id));

  // The wiring, from every input's `from`.
  const edges = [], external = [];
  for (const n of nodes.values()) {
    for (const inp of n.inputs) {
      const from = String(inp.from || '').trim();
      if (!from || from === 'case') continue;
      if (nodes.has(from)) edges.push({ from, to: n.id, input: inp.name });
      else external.push({ from, to: n.id, input: inp.name });
    }
  }
  const order = topoOrder([...nodes.keys()], edges);
  return {
    folder, g, meta, nodes, order, edges, external, stray,
    group: {
      members: tableRecords(g.files['members.csv']),
      publishes: tableRecords(g.files['publishes.csv']),
      requirements: tableRecords(g.files['requirements.csv']),
      loops: tableRecords(g.files['loops.csv']),
      constants: tableRecords(g.files['constants.csv']),
      sources: tableRecords(g.files['sources.csv']),
      versions: tableRecords(g.files['versions.csv']),
      equations: tableRecords(g.files['equations.csv']),
      symbols: tableRecords(g.files['symbols.csv']),
      figures: tableRecords(g.files['figures.csv']),
      reviews: tableRecords(g.files['reviews.csv']),
    },
  };
}

export function tableRecords(t) { return t ? records(t) : []; }

/** Nodes in an order where every node comes after the nodes that feed it; a loop keeps file order. */
export function topoOrder(ids, edges) {
  const into = new Map(ids.map(id => [id, 0]));
  for (const e of edges) into.set(e.to, (into.get(e.to) || 0) + 1);
  const ready = ids.filter(id => !into.get(id));
  const out = [];
  const seen = new Set();
  while (ready.length) {
    const id = ready.shift();
    if (seen.has(id)) continue;
    seen.add(id); out.push(id);
    for (const e of edges) if (e.from === id) {
      into.set(e.to, into.get(e.to) - 1);
      if (!into.get(e.to)) ready.push(e.to);
    }
  }
  for (const id of ids) if (!seen.has(id)) out.push(id);
  return out;
}

/** Layer of each node: 0 for the first, then one more than the deepest node feeding it. */
export function layers(order, edges) {
  const L = new Map(order.map(id => [id, 0]));
  for (let pass = 0; pass < order.length; pass++) {
    let moved = false;
    for (const e of edges) {
      if (!L.has(e.from) || !L.has(e.to)) continue;
      const want = L.get(e.from) + 1;
      if (want > L.get(e.to) && want < order.length) { L.set(e.to, want); moved = true; }
    }
    if (!moved) break;
  }
  return L;
}

/** An equation by id: the node's own first, then the group's. */
export function findEquation(model, node, id) {
  const own = node && node.files['equations.csv'] ? records(node.files['equations.csv']) : [];
  return own.find(e => e.id === id) || model.group.equations.find(e => e.id === id) || null;
}

/** A figure by id: the node's own first, then the group's. `base` is the folder it is relative to. */
export function findFigure(model, node, id) {
  const own = node && node.files['figures.csv'] ? records(node.files['figures.csv']) : [];
  const f = own.find(e => e.id === id);
  if (f) return { fig: f, base: node.dir };
  const gf = model.group.figures.find(e => e.id === id);
  return gf ? { fig: gf, base: '' } : null;
}

/** Every symbol known to a node: its own, then the group's. */
export function symbolsFor(model, node) {
  const own = node && node.files['symbols.csv'] ? records(node.files['symbols.csv']) : [];
  return own.concat(model.group.symbols);
}

/** The input and answer columns of a results table, with their units. */
export function resultColumns(t) {
  if (!t) return { inputs: [], answers: [], other: {} };
  const inputs = [], answers = [], other = {};
  t.head.forEach((h, k) => {
    const { name, unit } = splitUnit(h);
    if (name === 'answer' || name.startsWith('answer.') || name.startsWith('answer ')) answers.push({ k, name, unit });
    else if (['tolerance', 'refuses', 'origin', 'note', 'label'].includes(name)) other[name] = k;
    else inputs.push({ k, name, unit });
  });
  return { inputs, answers, other };
}
