/*
  The architecture, as a page of the tool rather than a document beside it.

  The order is the order the thing has to be understood in, and it is the order
  it has to be *built* in: one node, then how nodes join, then how the joins
  stack into layers, then the hierarchy those layers hang on. Everything on
  this page is counted from the same tables the engine walks, so a claim here
  cannot outlive the thing it describes.

  This page exists because the first deliverable is a repeatable architecture.
  A pattern that lives in one person's head is not repeatable; a pattern the
  tool can show, count and check is.
*/
'use strict';

import { $, esc, plural } from './dom.js';
import { S, subtreeAll, subsystemLayers } from './state.js';

export const SECTIONS = [
  ['node',   'the node, and its assembly'],
  ['conn',   'connectivity, as declared'],
  ['layers', 'layer by layer'],
  ['tree',   'the tree hierarchy, and its connections'],
  ['rings',  'the rings, and what may call what'],
];

export function drawArchitecture() {
  $('#arch-nav').innerHTML = SECTIONS.map(([k, label], i) =>
    '<button class="ctl arch' + (k === S.arch ? ' sel' : '') + '" data-arch="' + k + '">' +
    (i + 1) + ' · ' + esc(label) + '</button>').join('');
  const draw = { node: secNode, conn: secConn, layers: secLayers, tree: secTree, rings: secRings };
  $('#arch-body').innerHTML = (draw[S.arch] || secNode)();
}

// ---------------------------------------------------------------------------
// 1 — the node

function secNode() {
  const total = S.rows.length;
  const seeded = S.rows.filter(r => r.state === 'empty').length;
  const withCode = total - seeded;
  const withFixture = S.rows.filter(r => r.fixtures > 0).length;

  return lead(
    'One node is one small question, one answer, one folder and one row on the tree. ' +
    'The variable id <b>is</b> the node id, so there is no second name to keep in step.') +

  '<div class="cards">' +
    card(total, 'folders', 'one per row, all built from the same eight files') +
    card(withCode, 'specified', 'the sheet is filled and the six generators have run') +
    card(seeded, 'seeded', 'folder and sheet exist, nothing is specified in them') +
    card(withFixture, 'carry evidence', 'a known-good value from outside this code') +
  '</div>' +

  h4('the eight files, every time') +
  '<table class="asm-t"><thead><tr><th>file</th><th>written</th><th>what it is</th></tr></thead><tbody>' +
  [['node.toml', 'by hand', 'the sheet — the only file in the folder written by hand'],
   ['fixtures.toml', 'by hand', 'known-good values, and where each came from'],
   ['model.rs', 'generated', 'the whole file, with one numbered HOLE per algorithm step'],
   ['contract.rs', 'generated', 'the untyped adapter the bus calls'],
   ['mod.rs', 'generated', 'the module wiring'],
   ['evidence.rs', 'generated', 'the fixtures, as tests'],
   ['page.html', 'generated', 'the eight tabs you read on the node'],
   ['meta.json', 'generated', 'state and hashes, written by the gate']].map(([f, w, d]) =>
    '<tr><td><code>' + f + '</code></td><td><span class="by ' +
    (w === 'by hand' ? 'hand' : 'gen') + '">' + w + '</span></td><td>' + esc(d) + '</td></tr>').join('') +
  '</tbody></table>' +

  note('Two files are written, six are printed. A hand edit outside a numbered <code>HOLE</code> ' +
    'block is discarded by the next regeneration and fails the regeneration diff in the gate — which ' +
    'is what makes the generated region genuinely owned by the generator rather than merely labelled ' +
    'that way.') +

  h4('the repeatable act — adding the next one') +
  '<pre class="code">' +
  'cargo run -p xtask -- new prop_intake_throat --like prop_capture_efficiency\n' +
  '$EDITOR crates/vleo-mod-prop/nodes/intake_throat/node.toml   # the physics\n' +
  'cargo run -p xtask -- docs prop_intake_throat                # six artefacts, none typed\n' +
  '# fill the numbered HOLE blocks in model.rs — a few typed lines each\n' +
  'cargo run -p xtask -- gate prop_intake_throat\n' +
  'cargo test -p vleo-mod-prop</pre>' +
  note('Two human reviews per node, and everything between them is a command. If a node takes ' +
    'materially longer than that the template has a defect — worth finding, because it will be paid ' +
    total + ' times.') +

  h4('what is specified, and what is deliberately not') +
  '<p class="prose">The ' + withCode + ' specified rows are the <b>engine-sizing chain</b> — atmosphere, ' +
  'aerodynamics, intake, thruster, power — carried end to end to the thrust-against-drag closure, plus ' +
  'one simple reference design case to run it against. That is enough to exercise every mechanism in ' +
  'the tool: units, guards, cycles, evidence, credibility, the chain hash and the sweep.</p>' +
  '<p class="prose">The other ' + seeded + ' rows are seeded on purpose. The structure is the ' +
  'deliverable first; the content arrives per node, through the loop above, and each one lands in a ' +
  'folder that already exists with an owner already on it. A decomposition that only exists where ' +
  'someone has already done the work is a decomposition nobody can plan against.</p>';
}

// ---------------------------------------------------------------------------
// 2 — connectivity

function secConn() {
  const derivation = S.rows.reduce((n, r) => n + r.in.length, 0);
  const contribution = S.rows.reduce((n, r) => n + r.kpi.length, 0);
  const relation = S.index.relations.length;
  const specified = S.rows.filter(r => r.state !== 'empty');
  const unread = specified.filter(r => S.consumers[r.i].length === 0 && r.kind !== 'kpi').length;
  const crossings = S.rows.filter(r => r.crosses);

  return lead(
    'Three graphs, never merged. Merging them has a specific consequence: execute the union and a ' +
    'KPI gets computed as though it were derived; check coverage against the union and a navigation ' +
    'link counts as evidence.') +

  '<table class="grid-t"><thead><tr><th>graph</th><th>edge</th><th>declared by</th>' +
  '<th class="num">edges</th><th>used for</th></tr></thead><tbody>' +
  '<tr><td><b>derivation</b></td><td>variable → node</td>' +
  '<td>the consuming node — knowing its inputs changes <i>its</i> implementation</td>' +
  '<td class="num">' + derivation + '</td><td>execution</td></tr>' +
  '<tr><td><b>contribution</b></td><td>variable → KPI</td>' +
  '<td>the contributing variable</td><td class="num">' + contribution + '</td><td>coverage</td></tr>' +
  '<tr><td><b>relation</b></td><td>group → group</td>' +
  '<td>the layer file</td><td class="num">' + relation + '</td><td>navigation and impact</td></tr>' +
  '</tbody></table>' +

  note('Every edge is declared exactly once, by the end the edge <i>changes</i>. Everything else — ' +
    'who consumes this, what feeds a KPI, what a change reaches — is derived on assembly and never ' +
    'stored, so it cannot go stale.') +

  h4('the one route between layers') +
  '<p class="prose">Exactly one row in each subsystem layer crosses upward, and it is the only way ' +
  'in: a subsystem is reached through its interface node, never by reaching into it. The crossing is ' +
  'declared on the sheet rather than left to convention, so a boundary that has stopped being one is ' +
  'something the gate can see.</p>' +
  '<table class="grid-t"><thead><tr><th>crossing row</th><th>from</th><th>to</th></tr></thead><tbody>' +
  crossings.map(r =>
    '<tr><td><a class="xref" data-goto="' + esc(r.id) + '">' + esc(r.id) + '</a></td>' +
    '<td>' + esc(groupLabel(r.parent)) + ' <span class="muted">layer ' + r.layer + '</span></td>' +
    '<td>' + esc(groupLabel(r.crosses)) + ' <span class="muted">layer ' +
    (S.G.get(r.crosses) ? S.G.get(r.crosses).layer : '?') + '</span></td></tr>').join('') +
  '</tbody></table>' +

  h4('what nothing reads') +
  '<p class="prose"><b>' + unread + '</b> of the ' + specified.length + ' specified rows are read by ' +
  'nothing. Every one of them is a leaf of the design or an oversight, and the tool will not decide ' +
  'which — but it will keep counting them, because a list nobody is shown is a list nobody acts on. ' +
  'Seeded rows have no edges yet by construction and are not counted here: counting them would bury ' +
  'the number that means something under one that does not.</p>';
}

// ---------------------------------------------------------------------------
// 3 — layers

function secLayers() {
  const names = { 1: 'management', 2: 'the system', 3: 'subsystem' };
  const per = [1, 2, 3].map(l => {
    const rows = S.rows.filter(r => r.layer === l);
    return {
      l, name: names[l], rows: rows.length,
      spec: rows.filter(r => r.state !== 'empty').length,
      seeded: rows.filter(r => r.state === 'empty').length,
      groups: S.index.groups.filter(g => g.layer === l).length,
    };
  });

  return lead(
    'Four layers, one architecture. The layer decides what the figure is about and nothing else ' +
    'changes: the same tree, the same matrix, the same node page, the same run.') +

  '<table class="grid-t"><thead><tr><th>layer</th><th>what it answers</th><th class="num">rows</th>' +
  '<th class="num">specified</th><th class="num">seeded</th><th class="num">boxes</th></tr></thead><tbody>' +
  per.map(p =>
    '<tr><td><b>' + p.l + ' ' + p.name + '</b></td><td>' + esc(layerQuestion(p.l)) + '</td>' +
    '<td class="num">' + p.rows + '</td><td class="num">' + p.spec + '</td>' +
    '<td class="num">' + p.seeded + '</td><td class="num">' + p.groups + '</td></tr>').join('') +
  '<tr><td><b>4 the run</b></td><td>one case, one chain hash, one set of numbers</td>' +
  '<td class="num">—</td><td class="num">—</td><td class="num">—</td><td class="num">—</td></tr>' +
  '<tr class="tot"><td>total</td><td></td><td class="num">' + S.rows.length + '</td>' +
  '<td class="num">' + per.reduce((n, p) => n + p.spec, 0) + '</td>' +
  '<td class="num">' + per.reduce((n, p) => n + p.seeded, 0) + '</td>' +
  '<td class="num">' + S.index.groups.filter(g => g.layer > 0).length + '</td></tr>' +
  '</tbody></table>' +

  h4('layer 3, layer by layer') +
  '<table class="grid-t"><thead><tr><th>subsystem layer</th><th>owner</th><th class="num">rows</th>' +
  '<th>reports to</th></tr></thead><tbody>' +
  subsystemLayers().map(id => {
    const g = S.G.get(id);
    const rel = S.index.relations.find(x => x.from === id);
    return '<tr><td><b>' + esc(g.label) + '</b> <span class="muted">' + esc(id) + '</span></td>' +
      '<td>' + esc(g.owner) + '</td><td class="num">' + subtreeAll(id).length + '</td>' +
      '<td>' + esc(rel ? groupLabel(rel.to) : '—') + '</td></tr>';
  }).join('') + '</tbody></table>' +

  note('A case selects which boxes are in scope. It is never a copy of the tree: two customers are ' +
    'two cases against one architecture, because a cloned architecture is two architectures that will ' +
    'disagree, and the disagreement is found late.');
}

function layerQuestion(l) {
  return l === 1 ? 'who wants what, what it costs, what the programme promised'
    : l === 2 ? 'what the satellite is made of, and what reads what'
    : 'decomposed until each row is a question one person can answer';
}

// ---------------------------------------------------------------------------
// 4 — the tree hierarchy

function secTree() {
  const out = [];
  const walk = (gid, depth) => {
    const g = S.G.get(gid);
    if (!g || g.layer === 0) return;
    const direct = (S.gnodes.get(gid) || []).length;
    const all = subtreeAll(gid).length;
    const rels = S.index.relations.filter(r => r.from === gid);
    out.push('<tr><td style="padding-left:' + (depth * 14 + 4) + 'px">' +
      '<span class="tone t-' + esc(g.tone || 'slate') + '"></span>' +
      '<a class="xref" data-group="' + esc(gid) + '">' + esc(g.label) + '</a>' +
      '<span class="muted"> ' + esc(gid) + '</span></td>' +
      '<td class="num">' + g.layer + '</td><td>' + esc(g.owner) + '</td>' +
      '<td class="num">' + direct + '</td><td class="num">' + all + '</td>' +
      '<td>' + (g.cases.length ? g.cases.join(' ') : '<span class="muted">all</span>') + '</td>' +
      '<td>' + (rels.length
        ? rels.map(r => '→ ' + esc(groupLabel(r.to))).join('<br>')
        : '<span class="muted">—</span>') + '</td></tr>');
    (S.gkids.get(gid) || []).forEach(k => walk(k, depth + 1));
  };
  ['mgt_orbitt', 'sys_root'].forEach(r => walk(r, 0));
  subsystemLayers().forEach(r => walk(r, 0));

  return lead(
    'The hierarchy is the first relation and the matrix draws it on the diagonal. Every box has an ' +
    'owner, and ownership is a path rule rather than a convention — the crate a node lives in is a ' +
    'manifest line, so a sibling a crate did not declare will not compile.') +

  '<table class="grid-t tree-t"><thead><tr><th>box</th><th class="num">layer</th><th>owner</th>' +
  '<th class="num">direct</th><th class="num">subtree</th><th>cases</th><th>relates to</th>' +
  '</tr></thead><tbody>' + out.join('') + '</tbody></table>' +

  h4('the declared relations, and why each exists') +
  '<table class="grid-t"><thead><tr><th>from</th><th>to</th><th>because</th></tr></thead><tbody>' +
  S.index.relations.map(r =>
    '<tr><td>' + esc(groupLabel(r.from)) + '</td><td>' + esc(groupLabel(r.to)) + '</td>' +
    '<td>' + esc(r.why) + '</td></tr>').join('') + '</tbody></table>' +

  note('A relation is stated by the layer file and never inferred from the marks. The reason travels ' +
    'with it, because a relation whose reason is not written down gets deleted by the next person who ' +
    'finds it awkward.');
}

// ---------------------------------------------------------------------------
// 5 — the rings

function secRings() {
  const crates = (S.index.crates || []).filter(c => c.name.startsWith('vleo-mod-'));
  return lead(
    'Four rings. A ring may call inward and never outward, so a change in a face cannot reach the ' +
    'physics and a change in the physics reaches every face at once, through one interface.') +

  '<table class="grid-t"><thead><tr><th>ring</th><th>crate</th><th>what is in it</th></tr></thead><tbody>' +
  '<tr><td><b>0</b></td><td><code>vleo-units</code></td>' +
  '<td>units and frames as types, constants, portable maths</td></tr>' +
  '<tr><td><b>1</b></td><td><code>vleo-core</code></td>' +
  '<td>every formula, the fault taxonomy, credibility, the resolver</td></tr>' +
  '<tr><td><b>2</b></td><td><code>vleo-bus</code></td><td>the wire contract every face speaks</td></tr>' +
  '<tr><td><b>3</b></td><td><code>vleo-mod-*</code></td>' +
  '<td>' + crates.length + ' subsystem crates, one per team, isolated by the compiler</td></tr>' +
  '<tr><td><b>face</b></td><td><code>vleo-daemon</code> · <code>vleo-cli</code> · <code>vleo-ffi</code>' +
  ' · <code>vleo-wasm</code> · <code>vleo-py</code></td>' +
  '<td>browser, command line, C ABI, WebAssembly, Python — each takes one dependency on the facade</td>' +
  '</tr></tbody></table>' +

  h4('ring 3, as it stands') +
  '<table class="grid-t"><thead><tr><th>crate</th><th class="num">node folders</th></tr></thead><tbody>' +
  crates.map(c => '<tr><td><code>' + esc(c.name) + '</code></td><td class="num">' + c.nodes +
    '</td></tr>').join('') + '</tbody></table>' +

  note('Separate crates rather than one crate with modules: inside a crate the isolation rule is ' +
    'unenforced, and a rule the compiler does not check is a rule that is already broken somewhere ' +
    'nobody has looked. This was found by building it the other way first.');
}

// ---------------------------------------------------------------------------

const groupLabel = id => (S.G.get(id) ? S.G.get(id).label : id);
const lead = t => '<p class="arch-lead">' + t + '</p>';
const h4 = t => '<h4>' + esc(t) + '</h4>';
const note = t => '<p class="note">' + t + '</p>';
const card = (n, k, why) =>
  '<div class="card"><b>' + n + '</b><span class="k">' + esc(k) + '</span>' +
  '<span class="w">' + esc(why) + '</span></div>';
