/*
  THE GROUP'S STRUCTURE — what the group lead decides, edited in place.

  A group is its nodes and the arrows between them. Before anybody writes a
  word of explanation, the lead says how many nodes there are, what each one
  answers, in what unit, and which node feeds which: the CONTRACT. Each
  author then fills their own node's file against it, and the release is
  assembled from those files.

  Everything here edits the open database (the structure file, or a release
  being prepared) and records each change in its `change` table. A change to
  a node's contract after its file was issued raises the node's contract
  version, so the file its author holds is known to be behind, and the page
  names every node that reads the one changed — those are the authors to tell.

  Nothing here edits a node's content: that is its author's, in their file.
*/
'use strict';

import { esc } from './dom.js';
import { logChange } from './gdb.js';
import { wiringSvg, wireUp } from './gview.js';
import { topoOrder } from './gmodel.js';
import { mountTable } from './gtable.js';
import { parseCsv, toCsv } from './csv.js';

export const KINDS = ['computed', 'declared', 'required', 'achieved'];
const CONTRACT = ['id', 'question', 'kind', 'output', 'unit', 'lower', 'upper', 'value'];
const INPUT_CONTRACT = ['name', 'source', 'unit'];
const ID = /^[a-z][a-z0-9_]*$/;
const ROLES = ['owner', 'author', 'reviewer'];

// ── the operations, on the database ────────────────────────────────────────

/** A node's contract changed: once its file has been issued, that file is behind. */
function touch(db, uid) {
  const n = db.one('SELECT contract_version FROM node WHERE uid = ?', [uid]);
  const issued = Number(db.meta('issued:' + uid) || 0);
  if (n && issued && issued >= Number(n.contract_version)) {
    db.exec('UPDATE node SET contract_version = contract_version + 1 WHERE uid = ?', [uid]);
  }
}

/** The nodes that read `uid` as an input: the authors a change to it reaches. */
export function readersOf(db, uid) {
  return db.all('SELECT DISTINCT n.id FROM input i JOIN node n ON n.uid = i.node_uid WHERE i.source = ? AND n.archived = 0 ORDER BY n.ord', [uid]).map(r => r.id);
}

export function setGroup(db, who, key, value) {
  const before = db.meta(key);
  if (before === value) return;
  if (key === 'group_id' && !ID.test(value)) throw new Error('a group id is lower case letters, digits and _, starting with a letter');
  db.tx(() => { db.setMeta(key, value); logChange(db, who, 'group', key, before, value); });
}

export function addNode(db, who, id, kind) {
  if (!ID.test(id)) throw new Error('a node id is lower case letters, digits and _, starting with a letter: "' + id + '" is not');
  if (db.one('SELECT uid FROM node WHERE id = ?', [id])) throw new Error('there is already a node ' + id);
  let uid = id;
  for (let k = 2; db.one('SELECT uid FROM node WHERE uid = ?', [uid]); k++) uid = id + '_' + k;
  const ord = Number(db.value('SELECT COALESCE(MAX(ord), -1) + 1 FROM node'));
  db.tx(() => {
    db.exec('INSERT INTO node (uid, id, kind, ord) VALUES (?,?,?,?)', [uid, id, KINDS.includes(kind) ? kind : 'computed', ord]);
    logChange(db, who, uid, 'added the node', null, id);
  });
  return uid;
}

export function setNode(db, who, uid, field, value) {
  const n = db.one('SELECT * FROM node WHERE uid = ?', [uid]);
  if (!n) throw new Error('there is no node ' + uid);
  if (!CONTRACT.includes(field) && field !== 'author') throw new Error('not a field of a node: ' + field);
  const before = String(n[field] ?? '');
  if (before === value) return;
  if (field === 'id') {
    if (!ID.test(value)) throw new Error('a node id is lower case letters, digits and _, starting with a letter: "' + value + '" is not');
    if (db.one('SELECT uid FROM node WHERE id = ? AND uid <> ?', [value, uid])) throw new Error('there is already a node ' + value);
  }
  if (field === 'kind' && !KINDS.includes(value)) throw new Error('a kind is one of ' + KINDS.join(', '));
  if ((field === 'lower' || field === 'upper' || field === 'value') && value !== '' && !Number.isFinite(Number(value))) {
    throw new Error(field + ' is a number, or blank');
  }
  db.tx(() => {
    db.exec('UPDATE node SET ' + field + ' = ? WHERE uid = ?', [value, uid]);
    if (field !== 'author') touch(db, uid);
    logChange(db, who, uid, field, before, value);
  });
}

export function archiveNode(db, who, uid, archived) {
  db.tx(() => {
    db.exec('UPDATE node SET archived = ? WHERE uid = ?', [archived ? 1 : 0, uid]);
    logChange(db, who, uid, archived ? 'archived the node' : 'brought the node back', null, null);
  });
}

/** Move a node earlier or later in the group's listing. */
export function moveNode(db, who, uid, by) {
  const list = db.all('SELECT uid FROM node ORDER BY ord, id').map(r => r.uid);
  const i = list.indexOf(uid), j = i + by;
  if (i < 0 || j < 0 || j >= list.length) return;
  [list[i], list[j]] = [list[j], list[i]];
  db.tx(() => { list.forEach((u, k) => db.exec('UPDATE node SET ord = ? WHERE uid = ?', [k, u])); logChange(db, who, uid, 'moved', null, null); });
}

export function addInput(db, who, uid, name) {
  const nm = name || 'input_' + (Number(db.value('SELECT COUNT(*) FROM input WHERE node_uid = ?', [uid])) + 1);
  if (!/^[A-Za-z][A-Za-z0-9_]*$/.test(nm)) throw new Error('an input name is letters, digits and _: "' + nm + '" is not');
  if (db.one('SELECT name FROM input WHERE node_uid = ? AND name = ?', [uid, nm])) throw new Error('this node already has an input ' + nm);
  const ord = Number(db.value('SELECT COALESCE(MAX(ord), -1) + 1 FROM input WHERE node_uid = ?', [uid]));
  db.tx(() => {
    db.exec("INSERT INTO input (node_uid, ord, name, source) VALUES (?,?,?, 'case')", [uid, ord, nm]);
    touch(db, uid);
    logChange(db, who, uid, 'added the input ' + nm, null, 'case');
  });
}

export function setInput(db, who, uid, name, field, value) {
  const r = db.one('SELECT * FROM input WHERE node_uid = ? AND name = ?', [uid, name]);
  if (!r) throw new Error(uid + ' has no input ' + name);
  const cols = { name: 'name', source: 'source', unit: 'unit', symbol: 'symbol', default: 'dflt', min: 'min', max: 'max', says: 'says' };
  if (!cols[field]) throw new Error('not a field of an input: ' + field);
  let v = value;
  if (field === 'name') {
    if (!/^[A-Za-z][A-Za-z0-9_]*$/.test(v)) throw new Error('an input name is letters, digits and _: "' + v + '" is not');
    if (v !== name && db.one('SELECT name FROM input WHERE node_uid = ? AND name = ?', [uid, v])) throw new Error('this node already has an input ' + v);
  }
  if (field === 'source') {
    // A node of this group by its id is stored by its uid, so a rename never breaks the arrow.
    const n = db.one('SELECT uid FROM node WHERE id = ?', [v]);
    if (n) v = n.uid;
    else if (v !== 'case' && !/^[a-z0-9_]+\.[A-Za-z0-9_]+$/.test(v)) throw new Error('a source is a node of this group, `case`, or `group.node` for another group\'s node');
    if (v === uid) throw new Error('a node cannot feed itself');
  }
  if ((field === 'default' || field === 'min' || field === 'max') && v !== '' && !Number.isFinite(Number(v))) throw new Error(field + ' is a number, or blank');
  const before = String(r[cols[field]] ?? '');
  if (before === v) return;
  db.tx(() => {
    db.exec('UPDATE input SET ' + cols[field] + ' = ? WHERE node_uid = ? AND name = ?', [v, uid, name]);
    if (INPUT_CONTRACT.includes(field)) touch(db, uid);
    logChange(db, who, uid, 'input ' + name + ' ' + field, before, v);
  });
}

export function removeInput(db, who, uid, name) {
  db.tx(() => {
    db.exec('DELETE FROM input WHERE node_uid = ? AND name = ?', [uid, name]);
    touch(db, uid);
    logChange(db, who, uid, 'removed the input ' + name, name, null);
  });
}

export function setMember(db, who, name, role) {
  if (!name.trim()) throw new Error('a member has a name');
  if (!ROLES.includes(role)) throw new Error('a role is one of ' + ROLES.join(', '));
  const before = db.one('SELECT role FROM member WHERE name = ?', [name]);
  db.tx(() => {
    db.exec('INSERT OR REPLACE INTO member (name, role) VALUES (?, ?)', [name.trim(), role]);
    logChange(db, who, 'group', 'member ' + name, before ? before.role : null, role);
  });
}

export function removeMember(db, who, name) {
  const authors = db.all('SELECT id FROM node WHERE archived = 0').filter(n => authorsOf(db, n.id).includes(name));
  if (authors.length) throw new Error(name + ' still authors ' + authors.map(a => a.id).join(', ') + ': give those nodes to somebody else first');
  db.tx(() => { db.exec('DELETE FROM member WHERE name = ?', [name]); logChange(db, who, 'group', 'removed the member ' + name, name, null); });
}

function authorsOf(db, id) {
  const r = db.one('SELECT author FROM node WHERE id = ?', [id]);
  return r ? String(r.author || '').split(/,\s*/).filter(Boolean) : [];
}

/** The group's flow, written from the inputs: one line per node that has inputs, in the order things are worked out. */
export function flowFromInputs(db) {
  const nodes = db.all('SELECT uid, id FROM node WHERE archived = 0 ORDER BY ord, id');
  const idOf = new Map(nodes.map(n => [n.uid, n.id]));
  const ins = db.all('SELECT node_uid, source FROM input ORDER BY node_uid, ord').filter(i => idOf.has(i.node_uid));
  const edges = ins.filter(i => idOf.has(i.source)).map(i => ({ from: idOf.get(i.source), to: idOf.get(i.node_uid) }));
  const order = topoOrder(nodes.map(n => n.id), edges);
  const lines = ['# The group\'s flow: which node feeds which, in the order they are worked out.'];
  for (const id of order) {
    const uid = nodes.find(n => n.id === id).uid;
    const from = [...new Set(ins.filter(i => i.node_uid === uid).map(i => idOf.get(i.source) || i.source))];
    if (from.length) lines.push(id + ' <- ' + from.join(', '));
  }
  return lines.join('\n') + '\n';
}

export function setFlow(db, who, text) {
  const before = (db.one("SELECT body FROM doc WHERE scope = 'group' AND kind = 'flow'") || {}).body || '';
  if (before === text) return;
  db.tx(() => {
    db.exec("INSERT OR REPLACE INTO doc (scope, kind, body) VALUES ('group', 'flow', ?)", [text]);
    logChange(db, who, 'group', 'flow', before, text);
  });
}

/** A flow's steps, without its comments, spacing or order (any order the arrows allow is right): what is compared. */
const steps = t => String(t).split('\n').map(l => l.replace(/\s+/g, ' ').trim()).filter(l => l && !l.startsWith('#')).sort().join('\n');

// ── the page ───────────────────────────────────────────────────────────────

const field = (op, value, opts = {}) => (opts.type === 'select'
  ? '<select class="gs-in" data-op="' + op + '"' + (opts.key ? ' data-key="' + esc(opts.key) + '"' : '') + ' aria-label="' + esc(opts.label || op) + '">' +
    opts.options.map(o => { const [v, t] = Array.isArray(o) ? o : [o, o]; return '<option value="' + esc(v) + '"' + (String(v) === String(value) ? ' selected' : '') + '>' + esc(t) + '</option>'; }).join('') + '</select>'
  : '<input class="gs-in' + (opts.wide ? ' gs-wide' : '') + '" data-op="' + op + '"' + (opts.key ? ' data-key="' + esc(opts.key) + '"' : '') +
    (opts.list ? ' list="' + opts.list + '"' : '') + ' value="' + esc(value) + '" aria-label="' + esc(opts.label || op) + '"' + (opts.ph ? ' placeholder="' + esc(opts.ph) + '"' : '') + '>');

/** The structure page: the map, the group, its people, its nodes, the selected node's contract, the flow, the record. */
export function structurePage(ctx, sel) {
  const db = ctx.file.db;
  const live = db.all('SELECT * FROM node WHERE archived = 0 ORDER BY ord, id');
  const gone = db.all('SELECT * FROM node WHERE archived = 1 ORDER BY ord, id');
  const members = db.all('SELECT * FROM member ORDER BY role, name');
  const idOf = new Map(db.all('SELECT uid, id FROM node').map(r => [r.uid, r.id]));
  const node = sel ? db.one('SELECT * FROM node WHERE id = ?', [sel]) : null;
  const behind = live.filter(n => Number(db.meta('issued:' + n.uid) || 0) && Number(db.meta('issued:' + n.uid)) < Number(n.contract_version));
  const sealed = db.meta('sealed');
  const memberOpts = [['', '— nobody yet —']].concat(members.map(m => [m.name, m.name + ' (' + m.role + ')']));

  let html = '<h1 class="gh1">Structure</h1>' +
    '<section class="answer-first view-af"><p class="af-k">Answer first</p><p class="af-a">The group\'s nodes and the arrows between them — ' +
    'what each node answers, in what unit, fed by what. You decide this; each author then fills their own node\'s file against it.</p>' +
    '<ul class="af-points"><li>' + live.length + ' nodes, ' + db.value('SELECT COUNT(*) FROM input i JOIN node n ON n.uid = i.node_uid WHERE n.archived = 0') + ' inputs, ' + members.length + ' people.</li>' +
    (behind.length ? '<li><b>' + behind.length + ' node file(s) are behind their contract</b>: ' + behind.map(n => esc(n.id)).join(', ') + ' — <a href="#/files">re-issue them</a>.</li>'
      : '<li>Every issued node file matches its contract.</li>') +
    '<li>Every change is recorded below, with who made it and what it said before.</li></ul></section>';
  if (sealed) {
    return html + '<p class="gsave-dl">This file is a <b>sealed release</b> (' + esc(db.meta('version')) + ', sealed ' + esc(sealed.slice(0, 10)) + '). ' +
      'A release is never changed. To change the structure, open the group\'s structure file — or <button class="ctl small" type="button" data-act="unseal">start the next version from this one</button>.</p>';
  }

  html += '<h2 class="gh">The map</h2><p class="muted">Click a node to edit its contract below. Point at one to see what feeds it and what it feeds.</p>' +
    '<div class="gwire gs-map">' + (live.length ? wiringSvg(ctx.model) : '<p class="muted">No nodes yet — add the first one below.</p>') + '</div>';

  html += '<div class="gs-cols"><div class="gs-col">';
  html += '<h2 class="gh">The group</h2><div class="gs-grid">' +
    '<label>Id ' + field('group', db.meta('group_id'), { key: 'group_id', label: 'group id' }) + '</label>' +
    '<label>Name ' + field('group', db.meta('group_name'), { key: 'group_name', label: 'group name', wide: true }) + '</label>' +
    '<label>Owner ' + field('group', db.meta('owner'), { key: 'owner', type: 'select', options: memberOpts, label: 'owner' }) + '</label>' +
    '<label>Version ' + field('group', db.meta('version'), { key: 'version', label: 'version', ph: '1.0' }) + '</label>' +
    '<label class="gs-span">Summary — what the group answers, in one sentence ' + field('group', db.meta('summary'), { key: 'summary', label: 'summary', wide: true }) + '</label></div>';

  html += '<h2 class="gh">People</h2><div class="ri-wrap"><table class="fx gtable gs-table"><thead><tr><th>Name</th><th>Role</th><th>Authors</th><th></th></tr></thead><tbody>' +
    members.map(m => '<tr><td>' + esc(m.name) + '</td><td>' + field('role', m.role, { key: m.name, type: 'select', options: ROLES, label: 'role of ' + m.name }) + '</td><td class="small">' +
      esc(live.filter(n => String(n.author || '').split(/,\s*/).includes(m.name)).map(n => n.id).join(' ') || '—') + '</td><td><button class="ctl small gs-x" type="button" data-act="rm-member" data-key="' + esc(m.name) + '" aria-label="remove ' + esc(m.name) + '" title="remove">×</button></td></tr>').join('') +
    '</tbody></table></div><p class="gs-add"><input class="gs-in" id="gs-mname" placeholder="a name" aria-label="new member\'s name"> ' + field('', 'author', { type: 'select', options: ROLES, label: 'new member\'s role' }).replace('data-op=""', 'id="gs-mrole"') +
    ' <button class="ctl small" type="button" data-act="add-member">add a person</button></p>';

  html += '<h2 class="gh">Nodes</h2><p class="muted">In the order the group lists them. A node\'s <b>uid</b> never changes; its id can be renamed and every arrow follows.</p>' +
    '<div class="ri-wrap"><table class="fx gtable gs-table"><thead><tr><th>Node</th><th>Kind</th><th>Answers</th><th>Author</th><th>Contract</th><th></th></tr></thead><tbody>' +
    live.map(n => '<tr' + (node && node.uid === n.uid ? ' class="gs-sel"' : '') + '><td><a href="#/structure/' + esc(n.id) + '"><b>' + esc(n.id) + '</b></a></td><td><span class="gkind k-' + esc(n.kind) + '">' + esc(n.kind) + '</span></td><td class="small">' +
      esc(n.output || '—') + (n.unit ? ' [' + esc(n.unit) + ']' : '') + '</td><td class="small">' + esc(n.author || '—') + '</td><td class="small">v' + esc(n.contract_version) + '</td><td class="gs-acts">' +
      '<button class="ctl small" type="button" data-act="up" data-key="' + esc(n.uid) + '" aria-label="move ' + esc(n.id) + ' up">↑</button>' +
      '<button class="ctl small" type="button" data-act="down" data-key="' + esc(n.uid) + '" aria-label="move ' + esc(n.id) + ' down">↓</button></td></tr>').join('') +
    '</tbody></table></div><p class="gs-add"><input class="gs-in" id="gs-nid" placeholder="new_node_id" aria-label="new node\'s id"> ' +
    '<select class="gs-in" id="gs-nkind" aria-label="new node\'s kind">' + KINDS.map(k => '<option>' + k + '</option>').join('') + '</select> ' +
    '<button class="ctl small" type="button" data-act="add-node">add a node</button></p>' +
    (gone.length ? '<p class="muted small">Archived — kept, out of the group: ' + gone.map(n => '<a href="#/structure/' + esc(n.id) + '">' + esc(n.id) + '</a>').join(', ') + '</p>' : '');
  html += '</div><div class="gs-col">';

  // The selected node's contract.
  if (!node) {
    html += '<h2 class="gh">A node\'s contract</h2><p class="muted">Choose a node — on the map or in the list — to edit what it answers and what feeds it.</p>';
  } else {
    const ins = db.all('SELECT * FROM input WHERE node_uid = ? ORDER BY ord', [node.uid]);
    const readers = readersOf(db, node.uid);
    const issued = Number(db.meta('issued:' + node.uid) || 0);
    const srcOpts = live.filter(n => n.uid !== node.uid).map(n => n.id);
    html += '<h2 class="gh" id="gs-node">' + esc(node.id) + (node.archived ? ' <span class="muted">(archived)</span>' : '') + '</h2>' +
      '<p class="muted small">uid <code>' + esc(node.uid) + '</code> · contract v' + esc(node.contract_version) +
      (issued ? ' · its file was issued at v' + issued + (issued < Number(node.contract_version) ? ' — <b>behind: re-issue it</b>' : '') : ' · no file issued yet') + '</p>' +
      '<div class="gs-impact">' + (readers.length ? 'Read by <b>' + readers.map(esc).join(', ') + '</b>. A change to what this node answers reaches their authors too.' : 'No node of this group reads it.') + '</div>' +
      '<div class="gs-grid">' +
      '<label>Id ' + field('node', node.id, { key: 'id', label: 'node id' }) + '</label>' +
      '<label>Kind ' + field('node', node.kind, { key: 'kind', type: 'select', options: KINDS, label: 'kind' }) + '</label>' +
      '<label class="gs-span">The question it answers ' + field('node', node.question, { key: 'question', wide: true, label: 'question', ph: 'What is …?' }) + '</label>' +
      '<label>Output symbol ' + field('node', node.output, { key: 'output', label: 'output symbol', ph: 'v' }) + '</label>' +
      '<label>Unit ' + field('node', node.unit, { key: 'unit', label: 'unit', ph: 'm/s' }) + '</label>' +
      '<label>Lower bound ' + field('node', node.lower, { key: 'lower', label: 'lower bound' }) + '</label>' +
      '<label>Upper bound ' + field('node', node.upper, { key: 'upper', label: 'upper bound' }) + '</label>' +
      (node.kind === 'declared' || node.kind === 'required' ? '<label>Value ' + field('node', node.value, { key: 'value', label: 'value' }) + '</label>' : '') +
      '<label>Author ' + field('node', node.author, { key: 'author', type: 'select', options: memberOpts, label: 'author' }) + '</label></div>';
    html += '<h3 class="gh3">Inputs — what feeds it</h3><p class="muted small">Name, where it comes from and its unit are the contract. ' +
      'How it is written, its default and range are filled by the author in their file, and shown here.</p>' +
      '<datalist id="gs-srcs"><option value="case">a value the user sets</option>' + srcOpts.map(s => '<option value="' + esc(s) + '"></option>').join('') + '</datalist>' +
      '<div class="ri-wrap"><table class="fx gtable gs-table"><thead><tr><th>Name</th><th>From</th><th>Unit</th><th>Default</th><th>Range</th><th></th></tr></thead><tbody>' +
      ins.map(i => '<tr><td>' + field('input', i.name, { key: i.name + '|name', label: 'input name' }) + '</td><td>' +
        field('input', idOf.get(i.source) || i.source, { key: i.name + '|source', list: 'gs-srcs', label: 'where ' + i.name + ' comes from' }) + '</td><td>' +
        field('input', i.unit, { key: i.name + '|unit', label: 'unit of ' + i.name }) + '</td><td class="small">' + esc(i.dflt || '—') + '</td><td class="small">' +
        esc(i.min || i.max ? (i.min || '…') + ' to ' + (i.max || '…') : '—') + '</td><td><button class="ctl small gs-x" type="button" data-act="rm-input" data-key="' + esc(i.name) + '" aria-label="remove the input ' + esc(i.name) + '" title="remove">×</button></td></tr>').join('') +
      '</tbody></table></div><p><button class="ctl small" type="button" data-act="add-input">add an input</button> ' +
      (node.archived ? '<button class="ctl small" type="button" data-act="restore">bring it back</button>'
        : '<button class="ctl small" type="button" data-act="archive">archive this node</button>') + '</p>';
  }

  // The flow and the record.
  const flow = (db.one("SELECT body FROM doc WHERE scope = 'group' AND kind = 'flow'") || {}).body || '';
  const fresh = flowFromInputs(db);
  html += '<h2 class="gh">The flow</h2><p class="muted small">The group\'s pseudocode: which node feeds which, in order. It is checked against the inputs.</p>' +
    '<textarea class="gnote gs-flow" data-op="flow" rows="' + Math.min(14, flow.split('\n').length + 1) + '" aria-label="the flow">' + esc(flow) + '</textarea>' +
    (steps(fresh) !== steps(flow) ? '<p><button class="ctl small" type="button" data-act="flow">write the flow from the inputs</button> <span class="muted small">— it differs from what the inputs say</span></p>' : '<p class="muted small">It agrees with the inputs.</p>');
  html += '<h2 class="gh">The record of each version</h2><p class="muted small">What the group believed, what tested it, what it now knows and what changed — ' +
    'and what this version rests on and what would break it. The developer takes a node\'s method only with this record complete.</p><div id="gs-versions"></div>';
  const log = db.all('SELECT * FROM change ORDER BY rowid DESC LIMIT 30');
  html += '<h2 class="gh">What changed</h2>' + (log.length ? '<ol class="gs-log">' + log.map(c => '<li><span class="muted small">' + esc(c.at.slice(0, 16).replace('T', ' ')) + ' · ' + esc(c.who || '—') + '</span> ' +
    '<b>' + esc(c.scope === 'group' ? 'group' : idOf.get(c.scope) || c.scope) + '</b> ' + esc(c.what) +
    (c.before !== null && c.after !== null ? ': <s>' + esc(String(c.before).slice(0, 60)) + '</s> → ' + esc(String(c.after).slice(0, 60)) : '') + '</li>').join('') + '</ol>'
    : '<p class="muted">Nothing yet.</p>');
  html += '</div></div><p id="gs-out" class="gout" aria-live="polite"></p>';
  return html;
}

/** Wire the structure page: every edit goes to the database, then `done()` redraws. */
export function wireStructure(ctx, host, sel, done) {
  const db = ctx.file.db;
  const who = () => ctx.me || db.meta('owner') || '';
  const node = sel ? db.one('SELECT * FROM node WHERE id = ?', [sel]) : null;
  const run = async (fn, after) => {
    try {
      fn();
      await done(after);
    } catch (e) {
      const out = host.querySelector('#gs-out');
      if (out) out.textContent = e.message;
      alertNear(host, e.message);
    }
  };
  wireUp(ctx, host, id => { location.hash = '#/structure/' + id; });
  const VCOLS = ['version', 'date', 'by', 'believed', 'tested', 'learned', 'changed', 'risks', 'cost', 'rests_on', 'breaks_if'];
  const vrow = db.one("SELECT csv FROM tbl WHERE scope = 'group' AND path = 'versions.csv'");
  const vt = vrow ? parseCsv(vrow.csv) : { head: VCOLS, rows: [] };
  const vhost = host.querySelector('#gs-versions');
  if (vhost) {
    mountTable(vhost, {
      head: VCOLS, fixedHead: true,
      rows: vt.rows.map(r => VCOLS.map(c => { const k = vt.head.indexOf(c); return k < 0 ? '' : r[k]; })),
      onChange: (head, rows) => run(() => {
        const text = toCsv(head, rows.filter(r => r.some(c => String(c).trim())));
        db.tx(() => {
          db.exec("INSERT OR REPLACE INTO tbl (scope, path, csv) VALUES ('group', 'versions.csv', ?)", [text]);
          logChange(db, who(), 'group', 'versions.csv', vrow ? vrow.csv : null, text);
        });
      }),
    });
  }
  host.addEventListener('change', e => {
    const t = e.target, op = t.dataset.op, key = t.dataset.key, v = t.value.trim();
    if (!op) return;
    if (op === 'group') run(() => setGroup(db, who(), key, v));
    else if (op === 'role') run(() => setMember(db, who(), key, v));
    else if (op === 'node' && node) run(() => setNode(db, who(), node.uid, key, v), key === 'id' ? '#/structure/' + v : null);
    else if (op === 'input' && node) { const [name, f] = key.split('|'); run(() => setInput(db, who(), node.uid, name, f, v)); }
    else if (op === 'flow') run(() => setFlow(db, who(), t.value));
  });
  host.addEventListener('click', e => {
    const b = e.target.closest('[data-act]');
    if (!b) return;
    const act = b.dataset.act, key = b.dataset.key;
    if (act === 'add-member') run(() => setMember(db, who(), host.querySelector('#gs-mname').value, host.querySelector('#gs-mrole').value));
    else if (act === 'rm-member') run(() => removeMember(db, who(), key));
    else if (act === 'add-node') {
      const id = host.querySelector('#gs-nid').value.trim();
      run(() => addNode(db, who(), id, host.querySelector('#gs-nkind').value), '#/structure/' + id);
    } else if (act === 'up' || act === 'down') run(() => moveNode(db, who(), key, act === 'up' ? -1 : 1));
    else if (act === 'add-input' && node) run(() => addInput(db, who(), node.uid));
    else if (act === 'rm-input' && node) run(() => removeInput(db, who(), node.uid, key));
    else if (act === 'archive' && node) run(() => archiveNode(db, who(), node.uid, true), '#/structure');
    else if (act === 'restore' && node) run(() => archiveNode(db, who(), node.uid, false));
    else if (act === 'flow') run(() => setFlow(db, who(), flowFromInputs(db)));
    else if (act === 'unseal') run(() => {
      db.tx(() => {
        for (const k of ['sealed', 'sealed_by', 'fingerprint']) db.exec('DELETE FROM meta WHERE key = ?', [k]);
        db.setMeta('file_kind', 'release');
        logChange(db, who(), 'group', 'started the next version from ' + db.meta('version'), null, null);
      });
    });
  });
}

function alertNear(host, msg) {
  let el = host.querySelector('.gs-err');
  if (!el) { el = document.createElement('p'); el.className = 'gs-err'; el.setAttribute('role', 'alert'); host.prepend(el); }
  el.textContent = msg;
  el.scrollIntoView({ block: 'nearest' });
}
