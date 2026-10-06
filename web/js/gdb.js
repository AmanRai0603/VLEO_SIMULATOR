/*
  A group's database, read as its folder and written from one; and the three
  files a group keeps — structure, node, release — made from each other.

  THE FOLDER IS THE SHAPE, THE DATABASE IS THE STORE. Everything that draws or
  checks a group (gmodel, gcheck, gview) reads the folder pattern of
  groups/SPEC.toml. A database is turned into that folder in memory to be shown,
  and a folder — the worked example, a group's export, one an assistant
  prepared — is turned into a database to be kept. The two directions are
  exact inverses, which is what lets either be trusted.

  THE THREE FILES. The structure file holds the contracts (every node's row and
  inputs) and the group's own text. A node file holds one node's content and a
  copy of the contracts. A release is the structure with every node file's
  content assembled into it. Content of a node is only ever written in its node
  file; the release is rebuilt from the node files, never edited.
*/
'use strict';

import { parseCsv, records, toCsv } from './csv.js';
import { Folder } from './gfolder.js';
import { create, copy } from './gstore.js';
import { sha256 } from './gseal.js';

const GROUP_DOCS = { 'explanation.md': 'explanation', 'theory.md': 'theory', 'flow.txt': 'flow' };
const NODE_DOCS = { 'explanation.md': 'explanation', 'theory.md': 'theory', 'pseudocode.txt': 'pseudocode', 'results/how-run.md': 'how_run' };
const BUILT = ['group.csv', 'members.csv', 'nodes.csv', 'reviews.csv'];
const SKIP = p => p.startsWith('packages/') || p.startsWith('issues/');
const TYPES = { png: 'image/png', jpg: 'image/jpeg', jpeg: 'image/jpeg', gif: 'image/gif', webp: 'image/webp', svg: 'image/svg+xml',
  mp4: 'video/mp4', webm: 'video/webm', pdf: 'application/pdf', md: 'text/markdown', txt: 'text/plain' };
const typeOf = p => TYPES[p.split('.').pop().toLowerCase()] || 'application/octet-stream';
const blank = v => (v === null || v === undefined ? '' : String(v));

// ── folder → database ──────────────────────────────────────────────────────

/** A database of `kind` holding everything in a folder of the pattern. */
export async function dbFromFolder(folder, kind = 'release') {
  const db = await create(kind);
  const text = async p => folder.text(p);
  const rec = async p => { const t = await text(p); return t === null ? [] : records(parseCsv(t)); };
  const g = (await rec('group.csv'))[0] || {};
  const nodes = await rec('nodes.csv');
  const ids = new Set(nodes.map(n => n.id));
  db.tx(() => {
    for (const [k, v] of [['group_id', g.id], ['group_name', g.name], ['owner', g.owner], ['version', g.version], ['summary', g.summary]]) {
      db.setMeta(k, blank(v));
    }
    db.setMeta('contract_version', '1');
  });
  const members = await rec('members.csv');
  const inputs = {};
  for (const n of nodes) inputs[n.id] = await rec('nodes/' + n.id + '/inputs.csv');
  const reviews = await rec('reviews.csv');
  const files = [];
  for (const p of folder.list()) if (!SKIP(p)) files.push([p, folder.file(p)]);
  const payload = [];
  for (const [p, f] of files) payload.push([p, p.endsWith('.csv') || p.endsWith('.md') || p.endsWith('.txt') ? await f.text() : new Uint8Array(await f.arrayBuffer())]);

  db.tx(() => {
    for (const m of members) db.exec('INSERT INTO member (name, role) VALUES (?, ?)', [m.name, m.role || 'author']);
    // A member's `nodes` column is who is node engineer of what: it lands on each node.
    const authorOf = id => members.filter(m => String(m.nodes || '').split(/\s+/).includes(id)).map(m => m.name).join(', ');
    nodes.forEach((n, i) => {
      db.exec('INSERT INTO node (uid, id, question, kind, output, unit, lower, upper, value, author, ord) VALUES (?,?,?,?,?,?,?,?,?,?,?)',
        [n.id, n.id, blank(n.question), n.kind || 'computed', blank(n.output), blank(n.unit), blank(n.lower), blank(n.upper), blank(n.value), authorOf(n.id), i]);
      (inputs[n.id] || []).forEach((r, k) => {
        db.exec('INSERT INTO input (node_uid, ord, name, symbol, source, unit, dflt, min, max, says) VALUES (?,?,?,?,?,?,?,?,?,?)',
          [n.id, k, blank(r.name), blank(r.symbol), blank(r.from), blank(r.unit), blank(r.default), blank(r.min), blank(r.max), blank(r.says)]);
      });
    });
    for (const r of reviews) {
      db.exec('INSERT INTO review (name, scope, version, fingerprint, date, verdict, note) VALUES (?,?,?,?,?,?,?)',
        [blank(r.name), blank(r.scope) || 'group', blank(r.version), blank(r.fingerprint), blank(r.date), r.verdict || 'ok', blank(r.note)]);
    }
    for (const [p, body] of payload) placeFile(db, p, body, ids);
  });
  return db;
}

/**
 * One file of the pattern, put where it belongs in the database: a text in
 * `doc`, a table in `tbl`, anything else in `media`. The group's own lists
 * (group.csv, members.csv, nodes.csv, inputs.csv, reviews.csv) are columns,
 * not files, and are left to the caller. Answers false for a file it does not
 * keep (a node the database does not have, or packages/ and issues/).
 */
export function placeFile(db, p, body, ids) {
  if (BUILT.includes(p) || SKIP(p)) return false;
  const m = /^nodes\/([^/]+)\/(.+)$/.exec(p);
  const scope = m ? uidOf(db, m[1], ids) : 'group', rel = m ? m[2] : p;
  if (!scope || rel === 'inputs.csv') return false;
  const docs = m ? NODE_DOCS : GROUP_DOCS;
  const text = () => (typeof body === 'string' ? body : new TextDecoder().decode(body));
  if (docs[rel] !== undefined) db.exec('INSERT OR REPLACE INTO doc (scope, kind, body) VALUES (?,?,?)', [scope, docs[rel], text()]);
  else if (rel.endsWith('.csv')) db.exec('INSERT OR REPLACE INTO tbl (scope, path, csv) VALUES (?,?,?)', [scope, rel, text()]);
  else {
    const bytes = typeof body === 'string' ? new TextEncoder().encode(body) : body;
    db.exec('INSERT OR REPLACE INTO media (scope, path, type, sha256, bytes) VALUES (?,?,?,?,?)', [scope, rel, typeOf(rel), sha256(bytes), bytes]);
  }
  return true;
}

/** A file of the pattern taken out of the database. */
export function removeFile(db, p) {
  const m = /^nodes\/([^/]+)\/(.+)$/.exec(p);
  const scope = m ? uidOf(db, m[1]) : 'group', rel = m ? m[2] : p;
  if (!scope) return;
  const docs = m ? NODE_DOCS : GROUP_DOCS;
  if (docs[rel] !== undefined) db.exec('DELETE FROM doc WHERE scope = ? AND kind = ?', [scope, docs[rel]]);
  db.exec('DELETE FROM tbl WHERE scope = ? AND path = ?', [scope, rel]);
  db.exec('DELETE FROM media WHERE scope = ? AND path = ?', [scope, rel]);
}

// A node's uid from the id a path names. While a folder is being read in, the
// uid IS the id (`ids` is the set of them).
function uidOf(db, id, ids) {
  if (ids) return ids.has(id) ? id : null;
  const r = db.one('SELECT uid FROM node WHERE id = ?', [id]);
  return r ? r.uid : null;
}

// ── database → folder ──────────────────────────────────────────────────────

/** The folder a database holds, in memory, for the views and the checks. */
export function folderFromDb(db, name) {
  const files = new Map();
  const put = (p, body, type) => files.set(p, new File([body], p.split('/').pop(), { type: type || 'text/plain' }));
  const live = db.all('SELECT * FROM node WHERE archived = 0 ORDER BY ord, id');
  const idOf = new Map(db.all('SELECT uid, id FROM node').map(r => [r.uid, r.id]));
  const dirOf = scope => (scope === 'group' ? '' : 'nodes/' + (idOf.get(scope) || scope) + '/');
  const liveUids = new Set(live.map(n => n.uid));
  const inScope = scope => scope === 'group' || liveUids.has(scope);

  put('group.csv', toCsv(['id', 'name', 'owner', 'version', 'summary'],
    [[db.meta('group_id'), db.meta('group_name'), db.meta('owner'), db.meta('version'), db.meta('summary')]]), 'text/csv');
  const members = db.all('SELECT * FROM member ORDER BY name');
  put('members.csv', toCsv(['name', 'role', 'nodes'], members.map(m => [m.name, m.role,
    m.role === 'owner' && !live.some(n => (n.author || '').split(/,\s*/).includes(m.name)) ? '*'
      : live.filter(n => (n.author || '').split(/,\s*/).includes(m.name)).map(n => n.id).join(' ')])), 'text/csv');
  put('nodes.csv', toCsv(['id', 'question', 'kind', 'output', 'unit', 'lower', 'upper', 'value'],
    live.map(n => [n.id, n.question, n.kind, n.output, n.unit, n.lower, n.upper, n.value])), 'text/csv');
  for (const n of live) {
    const ins = db.all('SELECT * FROM input WHERE node_uid = ? ORDER BY ord', [n.uid]);
    if (ins.length || n.kind === 'computed') {
      // `symbol` is optional in the pattern: written only when some input has one.
      const sym = ins.some(r => r.symbol);
      put('nodes/' + n.id + '/inputs.csv', toCsv(['name'].concat(sym ? ['symbol'] : [], ['from', 'unit', 'default', 'min', 'max', 'says']),
        ins.map(r => [r.name].concat(sym ? [r.symbol] : [], [idOf.get(r.source) || r.source, r.unit, r.dflt, r.min, r.max, r.says]))), 'text/csv');
    }
  }
  const reviews = db.all('SELECT * FROM review');
  if (reviews.length) {
    put('reviews.csv', toCsv(['name', 'scope', 'version', 'fingerprint', 'date', 'verdict', 'note'],
      reviews.map(r => [r.name, r.scope === 'group' ? 'group' : (idOf.get(r.scope) || r.scope), r.version, r.fingerprint, r.date, r.verdict, r.note])), 'text/csv');
  }
  const docPath = (scope, kind) => {
    const table = scope === 'group' ? GROUP_DOCS : NODE_DOCS;
    const rel = Object.keys(table).find(k => table[k] === kind);
    return rel ? dirOf(scope) + rel : null;
  };
  for (const d of db.all('SELECT * FROM doc')) {
    if (!inScope(d.scope)) continue;
    const p = docPath(d.scope, d.kind);
    if (p) put(p, d.body, 'text/plain');
  }
  for (const t of db.all('SELECT * FROM tbl')) if (inScope(t.scope)) put(dirOf(t.scope) + t.path, t.csv, 'text/csv');
  for (const m of db.all('SELECT * FROM media')) if (inScope(m.scope)) put(dirOf(m.scope) + m.path, m.bytes, m.type);
  return new DbFolder(db, name || db.meta('group_id') || 'group', files);
}

/**
 * The folder a database holds, written back into it. What the views write —
 * a sign-off into reviews.csv, an issue into issues/ — lands in the database's
 * own tables, and `onChange` says the file needs saving. A package is not kept
 * in a database: sealing one makes the release file instead.
 */
export class DbFolder extends Folder {
  constructor(db, name, files) {
    super(name, files);
    this.db = db;
    this.onChange = () => {};
  }
  get writable() { return true; }
  async write(path, body, type = 'text/plain') {
    if (path.startsWith('packages/')) return super.write(path, body, type);
    const textual = /\.(csv|md|txt)$/i.test(path);
    const raw = typeof body === 'string' ? new TextEncoder().encode(body)
      : body instanceof Blob ? new Uint8Array(await body.arrayBuffer()) : body;
    const text = textual ? new TextDecoder().decode(raw) : '';
    const db = this.db;
    db.tx(() => {
      if (path === 'reviews.csv') {
        const uid = id => (id === 'group' || !id ? 'group' : uidOf(db, id) || id);
        db.exec('DELETE FROM review');
        for (const r of records(parseCsv(text))) {
          db.exec('INSERT INTO review (name, scope, version, fingerprint, date, verdict, note) VALUES (?,?,?,?,?,?,?)',
            [blank(r.name), uid(r.scope), blank(r.version), blank(r.fingerprint), blank(r.date), r.verdict || 'ok', blank(r.note)]);
        }
      } else if (path.startsWith('issues/')) {
        db.exec("INSERT INTO comment (scope, section, author, at, body) VALUES ('group', 'issue', ?, ?, ?)",
          [(/raised by: (.*)/.exec(text) || ['', ''])[1].trim(), new Date().toISOString(), text]);
      } else {
        placeFile(db, path, textual ? text : raw);
      }
    });
    this.files.set(path, new File([textual ? text : raw], path.split('/').pop(), { type: textual ? type : typeOf(path) }));
    this.urls.delete(path);
    this.onChange(path);
    return 'kept';
  }
  /** Take a file out, from the database and the folder alike. */
  remove(path) {
    this.db.tx(() => removeFile(this.db, path));
    this.files.delete(path);
    this.urls.delete(path);
    this.onChange(path);
  }
}

// ── the three files ────────────────────────────────────────────────────────

const SCOPED = ['doc', 'tbl', 'media'];

/** The structure file: contracts, members and the group's own content — no node content. */
export async function structureOf(db) {
  const s = await copy(db);
  s.tx(() => {
    for (const t of SCOPED) s.exec('DELETE FROM ' + t + " WHERE scope <> 'group'");
    s.exec("DELETE FROM review WHERE scope <> 'group'");
    s.exec("DELETE FROM comment WHERE scope <> 'group'");
    s.setMeta('file_kind', 'structure');
    for (const k of ['node_uid', 'sealed', 'fingerprint']) s.exec('DELETE FROM meta WHERE key = ?', [k]);
  });
  s.exec('VACUUM');
  return s;
}

/**
 * One node's file, issued from the structure (or a release): every contract,
 * so the node engineer sees their neighbours, the group's tables, and only this
 * node's content.
 */
export async function issueNode(db, uid) {
  if (!db.one('SELECT uid FROM node WHERE uid = ?', [uid])) throw new Error('there is no node ' + uid);
  const n = await copy(db);
  n.tx(() => {
    // The group's tables stay — sources, symbols, constants, requirements — so
    // the node engineer can cite and check against them; its texts and pictures go.
    n.exec('DELETE FROM doc WHERE scope <> ?', [uid]);
    n.exec("DELETE FROM tbl WHERE scope <> ? AND scope <> 'group'", [uid]);
    n.exec('DELETE FROM media WHERE scope <> ?', [uid]);
    n.exec("DELETE FROM meta WHERE key LIKE 'issued:%'");
    n.exec('DELETE FROM review WHERE scope <> ?', [uid]);
    n.exec('DELETE FROM comment WHERE scope <> ?', [uid]);
    n.exec('DELETE FROM request WHERE node_uid <> ?', [uid]);
    n.setMeta('file_kind', 'node');
    n.setMeta('node_uid', uid);
    for (const k of ['sealed', 'fingerprint']) n.exec('DELETE FROM meta WHERE key = ?', [k]);
  });
  // What was taken out leaves free pages behind; a file is sent without them.
  n.exec('VACUUM');
  return n;
}

/**
 * The release: the structure with each node file's content put in. Answers
 * `{ db, notes }`; a note says what did not fit — a node file whose contract is
 * older than the structure's, one for a node the structure does not have, a
 * node with no file at all.
 */
export async function assemble(structure, nodeFiles) {
  const r = await copy(structure);
  const notes = [];
  const seen = new Set();
  r.tx(() => {
    for (const nf of nodeFiles) {
      const uid = nf.meta('node_uid');
      const row = r.one('SELECT * FROM node WHERE uid = ?', [uid]);
      if (!row) { notes.push({ level: 'error', uid, msg: 'is a node file for ' + uid + ', which the structure does not have' }); continue; }
      if (seen.has(uid)) { notes.push({ level: 'error', uid, msg: 'has two files; only the first is used' }); continue; }
      seen.add(uid);
      const own = nf.one('SELECT * FROM node WHERE uid = ?', [uid]);
      if (own && Number(own.contract_version) < Number(row.contract_version)) {
        notes.push({ level: 'warning', uid, msg: row.id + "'s file was written to contract v" + own.contract_version + '; the structure is at v' + row.contract_version });
      }
      for (const t of SCOPED) r.exec('DELETE FROM ' + t + ' WHERE scope = ?', [uid]);
      r.exec('DELETE FROM review WHERE scope = ?', [uid]);
      r.exec('DELETE FROM comment WHERE scope = ?', [uid]);
      r.exec('DELETE FROM request WHERE node_uid = ?', [uid]);
      for (const d of nf.all('SELECT * FROM doc WHERE scope = ?', [uid])) r.exec('INSERT INTO doc VALUES (?,?,?)', [d.scope, d.kind, d.body]);
      for (const t of nf.all('SELECT * FROM tbl WHERE scope = ?', [uid])) r.exec('INSERT INTO tbl VALUES (?,?,?)', [t.scope, t.path, t.csv]);
      for (const m of nf.all('SELECT * FROM media WHERE scope = ?', [uid])) r.exec('INSERT INTO media VALUES (?,?,?,?,?)', [m.scope, m.path, m.type, m.sha256, m.bytes]);
      for (const v of nf.all('SELECT * FROM review WHERE scope = ?', [uid])) {
        r.exec('INSERT INTO review VALUES (?,?,?,?,?,?,?)', [v.name, v.scope, v.version, v.fingerprint, v.date, v.verdict, v.note]);
      }
      for (const c of nf.all('SELECT * FROM comment WHERE scope = ?', [uid])) r.exec('INSERT INTO comment VALUES (?,?,?,?,?,?)', [c.scope, c.section, c.author, c.at, c.body, c.resolved]);
      for (const q of nf.all('SELECT * FROM request WHERE node_uid = ?', [uid])) r.exec('INSERT INTO request VALUES (?,?,?,?,?,?)', [q.node_uid, q.author, q.at, q.body, q.status, q.answer]);
      if (own) r.exec('UPDATE node SET revision = ? WHERE uid = ?', [own.revision, uid]);
      // An input's name, source and unit are the contract; how it is written,
      // its default and its range are the node engineer's, and come from their file.
      for (const i of nf.all('SELECT * FROM input WHERE node_uid = ?', [uid])) {
        r.exec('UPDATE input SET symbol = ?, dflt = ?, min = ?, max = ?, says = ? WHERE node_uid = ? AND name = ?',
          [i.symbol, i.dflt, i.min, i.max, i.says, uid, i.name]);
      }
    }
    for (const n of r.all('SELECT uid, id FROM node WHERE archived = 0')) {
      if (!seen.has(n.uid)) notes.push({ level: 'note', uid: n.uid, msg: n.id + ' had no node file among these: it keeps what this file already held' });
    }
    r.setMeta('file_kind', 'release');
    r.exec('DELETE FROM meta WHERE key = ?', ['node_uid']);
  });
  r.exec('VACUUM');
  return { db: r, notes };
}

/** Record one change, for history, undo and "what changed since". */
export function logChange(db, who, scope, what, before, after) {
  db.exec('INSERT INTO change (at, who, scope, what, before, after) VALUES (?,?,?,?,?,?)',
    [new Date().toISOString(), who || '', scope, what, before === undefined ? null : before, after === undefined ? null : after]);
}
