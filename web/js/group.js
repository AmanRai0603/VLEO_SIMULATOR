/*
  THE GROUP APPLICATION — a group's folder, seen as everyone will see it.

  One HTML file, opened in a browser, with no network and nothing to install
  (web/group.html, built by `cargo run -p xtask -- group-app`). A member
  opens their group's folder — on their own disk or synced from the group's
  Google shared drive — and the page reads every file in it and shows the
  group and each node: the explanation as stations, the equations typeset,
  the flow and its walk-through, the charts, the pseudocode beside the same
  lines as equations, their own results plotted. It checks the folder
  against the pattern (groups/SPEC.toml), records each member's sign-off,
  and seals the folder into the one file the developer receives.

  A group keeps its work as database files (groups/schema.sql): the
  STRUCTURE (<group>.vgroup) the lead edits here — the nodes, what each one
  answers and what feeds it — one NODE file per node (<node>.vnode) that its
  author fills in the node application, and the RELEASE (<group>-<v>.vleo)
  this page assembles from the node files, has signed, and seals. A folder of
  the pattern opens too, and is kept as a database in one step.

  It runs nothing and compiles nothing. Computing is the developer's engine's
  work, done after the seal and tested against the results in this folder.
*/
'use strict';

import { $, esc } from './dom.js';
import { initDepth } from './depth.js';
import { fromFileList, fromDrop, fromPicker, fromHandle, canPick, download } from './gfolder.js';
import { loadGroup } from './gmodel.js';
import { checkGroup } from './gcheck.js';
import { groupPage, nodePage, mountFigures, mountResults, wireUp, wireAlgorithm, wireCode, findingsList, NODE_TABS } from './gview.js';
import { reviewState, maySign, sign, sealBlockers, seal, raiseIssue } from './gseal.js';
import { texToMathml, shortToTex } from './texmath.js';
import { dbFromFolder, folderFromDb, structureOf, issueNode, assemble, logChange } from './gdb.js';
import { DbFile, fromFile, pickForSaving, canSaveInPlace, openMany, fileName, saveBytesAs } from './gfile.js';
import { create, copy } from './gstore.js';
import { structurePage, wireStructure, setMember } from './gstruct.js';
import { fingerprint, zip } from './gseal.js';
import { deliveryPage, wireDelivery } from './gaccept.js';

const SPEC = window.VLEO_GROUP_SPEC || { file: [], text: [], embed: [] };
const ctx = { folder: null, model: null, findings: [], spec: SPEC, file: null, me: '' };
const ME = 'vleo.group.me';
let me = '';
try { me = localStorage.getItem(ME) || ''; } catch { /* no storage */ }
ctx.me = me;

// ── opening a folder ───────────────────────────────────────────────────────

async function open(folder, keepFile) {
  if (!folder) return;
  if (!keepFile) ctx.file = null;
  $('#gmain').innerHTML = '<p class="muted">Reading ' + esc(folder.name) + '…</p>';
  ctx.folder = folder;
  ctx.model = await loadGroup(folder);
  ctx.findings = await checkGroup(ctx.model, SPEC);
  document.title = (ctx.model.meta.name || folder.name) + ' · VLEO group';
  nav();
  topbar();
  if (!location.hash || location.hash === '#/open') location.hash = ctx.file && !ctx.model.nodes.size ? '#/structure' : '#/';
  else route();
}

/** A database file, opened: shown through the folder it holds, and written back into. */
async function openDb(file) {
  if (!file) return;
  ctx.file = file;
  await open(dbView(file), true);
}

function dbView(file) {
  const folder = folderFromDb(file.db, file.name.replace(/\.[^.]+$/, ''));
  folder.onChange = () => { file.dirty = true; topbar(); };
  return folder;
}

/** After an edit to the open database: everything read again from it, the page drawn again where it was. */
async function fromDb(after) {
  const y = window.scrollY;
  ctx.file.dirty = true;
  ctx.folder = dbView(ctx.file);
  ctx.model = await loadGroup(ctx.folder);
  ctx.findings = await checkGroup(ctx.model, SPEC);
  nav();
  topbar();
  if (after && after !== location.hash) { location.hash = after; return; }
  await route();
  window.scrollTo(0, y);
}

async function trying(fn) {
  try { await fn(); } catch (e) {
    $('#gmain').insertAdjacentHTML('afterbegin', '<p class="gs-err" role="alert">' + esc(e.message || String(e)) + '</p>');
    window.scrollTo(0, 0);
  }
}

function topbar() {
  const f = ctx.file;
  const note = $('#gfile'), save = $('#gsave');
  if (!f) {
    note.textContent = ctx.folder ? 'a folder · offline' : 'offline';
    save.hidden = true;
    return;
  }
  const kind = { structure: 'structure', node: 'node file', release: f.db.meta('sealed') ? 'sealed release' : 'release' }[f.kind] || f.kind;
  note.innerHTML = esc(f.name) + ' · ' + esc(kind) + (f.dirty ? ' · <b class="gdirty">not saved</b>' : '');
  save.hidden = false;
  save.textContent = f.handle ? 'Save' : 'Save (download)';
}

async function saveFile() {
  if (!ctx.file) return;
  const how = await ctx.file.save();
  topbar();
  flash(how === 'saved' ? 'Saved ' + ctx.file.name + '.' : ctx.file.name + ' was downloaded — put it on the group\'s drive in place of the old one.');
}

function flash(msg) {
  const el = $('#gflash');
  el.textContent = msg;
  el.hidden = false;
  clearTimeout(flash.t);
  flash.t = setTimeout(() => { el.hidden = true; }, 6000);
}

function welcome() {
  const pick = canPick(), inPlace = canSaveInPlace();
  $('#gmain').innerHTML =
    '<section class="answer-first view-af"><p class="af-k">Answer first</p><p class="af-a">Open your group\'s database file and this page shows ' +
    'everything in it the way the VLEO application will show it. The group lead edits the structure here, issues each author their node file, ' +
    'assembles the files into a release, has it signed and seals it for the developer.</p>' +
    '<ul class="af-points"><li>Nothing leaves this computer. The page has no network and runs nothing.</li>' +
    '<li>A file on your Google shared drive, synced to this computer, is an ordinary file here.</li>' +
    '<li>An author fills their node file in the node application (node.html), not here.</li></ul></section>' +
    '<div class="gopen gcards">' +
    '<div class="gdrop gcard" id="gdrop"><h2 class="gh3">Open the group\'s database</h2><p class="muted small"><code>.vgroup</code> (the structure) or <code>.vleo</code> (a release). Drop it here, or</p>' +
    '<label class="ctl gbtn">Choose the file…<input type="file" id="gpickdb" accept=".vgroup,.vleo,.vnode" hidden></label>' +
    (inPlace ? ' <button class="ctl gbtn" type="button" id="gpickdbrw">Open it for saving…</button><p class="muted small">"Open it for saving" writes your changes back into the same file. ' +
      'Otherwise Save downloads it, and you put it back on the drive.</p>' : '<p class="muted small">This browser saves by downloading: put the saved file back on the drive.</p>') + '</div>' +
    '<div class="gcard"><h2 class="gh3">Start a new group</h2><p class="muted small">You set out its nodes and how they connect; the authors fill them.</p>' +
    '<p><label>Group id <input id="gn-id" class="gs-in" placeholder="solar" aria-label="new group id"></label></p>' +
    '<p><label>Name <input id="gn-name" class="gs-in gs-wide" placeholder="Solar and geomagnetic activity" aria-label="new group name"></label></p>' +
    '<p><label>You, the owner <input id="gn-me" class="gs-in" placeholder="your name" aria-label="your name"></label></p>' +
    '<button class="ctl gbtn" type="button" id="gn-go">Start it</button><p id="gn-out" class="gout" aria-live="polite"></p></div>' +
    '<div class="gcard"><h2 class="gh3">Open a group folder</h2><p class="muted small">A folder of the pattern — the worked example, or one exported from the design. It can then be kept as a database.</p>' +
    '<label class="ctl gbtn">Choose the folder…<input type="file" id="gpickdir" webkitdirectory directory multiple hidden></label>' +
    (pick ? ' <button class="ctl gbtn" type="button" id="gpickrw">Open it for saving…</button>' : '') + '</div>' +
    '</div>' +
    '<p class="muted">No file yet? <a href="#/pattern">See what a group holds</a>, or try the <a href="#/helper">equation helper</a>.</p>';
  $('#gpickdir').addEventListener('change', e => open(fromFileList(e.target.files)));
  $('#gpickdb').addEventListener('change', e => trying(async () => openDb(await fromFile(e.target.files[0]))));
  if (inPlace) $('#gpickdbrw').addEventListener('click', () => trying(async () => openDb(await pickForSaving())));
  if (pick) $('#gpickrw').addEventListener('click', async () => { try { open(await fromPicker()); } catch { /* cancelled */ } });
  $('#gn-go').addEventListener('click', () => trying(async () => {
    const id = $('#gn-id').value.trim(), who = $('#gn-me').value.trim();
    if (!/^[a-z][a-z0-9_]*$/.test(id)) throw new Error('a group id is lower case letters, digits and _, starting with a letter');
    if (!who) throw new Error('say who you are: the owner signs the group');
    const db = await create('structure');
    db.tx(() => {
      for (const [k, v] of [['group_id', id], ['group_name', $('#gn-name').value.trim() || id], ['owner', who], ['version', '1.0'], ['summary', '']]) db.setMeta(k, v);
      db.exec("INSERT INTO doc (scope, kind, body) VALUES ('group', 'flow', ?)", ['# The group\'s flow: which node feeds which, in the order they are worked out.\n']);
    });
    setMember(db, who, who, 'owner');
    logChange(db, who, 'group', 'started the group', null, id);
    ctx.me = me = who;
    try { localStorage.setItem(ME, me); } catch { /* */ }
    const f = new DbFile(db, fileName(id, 'structure'));
    f.dirty = true;
    location.hash = '#/structure';
    await openDb(f);
  }));
}

function dropAnywhere() {
  document.addEventListener('dragover', e => { e.preventDefault(); document.body.classList.add('gdragging'); });
  document.addEventListener('dragleave', e => { if (!e.relatedTarget) document.body.classList.remove('gdragging'); });
  document.addEventListener('drop', async e => {
    e.preventDefault();
    document.body.classList.remove('gdragging');
    const one = e.dataTransfer.files.length === 1 ? e.dataTransfer.files[0] : null;
    if (one && /\.(vgroup|vleo|vnode)$/i.test(one.name)) { await trying(async () => openDb(await fromFile(one))); return; }
    const f = await fromDrop(e.dataTransfer);
    if (f) open(f);
  });
}

async function reload() {
  if (ctx.file) await fromDb();
  else if (ctx.folder && ctx.folder.handle) await open(await fromHandle(ctx.folder.handle));
  else location.hash = '#/open';
}

// ── the frame around every page ────────────────────────────────────────────

function nav() {
  const m = ctx.model;
  const count = level => ctx.findings.filter(f => f.level === level).length;
  const nodeState = n => {
    const mine = ctx.findings.filter(f => f.where.startsWith(n.dir) && f.level === 'error').length;
    return mine ? 'gdot-err' : 'gdot-ok';
  };
  $('#gtitle').textContent = (m.meta.name || ctx.folder.name) + (m.meta.version ? ' · v' + m.meta.version : '');
  $('#gnav').innerHTML =
    '<a class="gnv" href="#/" data-r="">The group</a>' +
    (ctx.file ? '<a class="gnv" href="#/structure" data-r="structure">Structure</a>' : '') +
    '<a class="gnv" href="#/files" data-r="files">' + (ctx.file ? 'Node files &amp; release' : 'Keep as a database') + '</a>' +
    '<a class="gnv" href="#/checks" data-r="checks">Checks <span class="gcount' + (count('error') ? ' bad' : '') + '">' + count('error') + ' · ' + count('warning') + '</span></a>' +
    '<a class="gnv" href="#/sign" data-r="sign">Sign &amp; seal</a>' +
    (ctx.file ? '<a class="gnv" href="#/delivery" data-r="delivery">Delivery &amp; acceptance</a>' : '') +
    '<p class="gnv-h">Nodes, in flow order</p>' +
    m.order.map(id => '<a class="gnv gnv-node" href="#/node/' + esc(id) + '" data-r="node/' + esc(id) + '"><span class="gdot ' + nodeState(m.nodes.get(id)) + '"></span>' + esc(id) + '</a>').join('') +
    '<p class="gnv-h">Tools</p><a class="gnv" href="#/helper" data-r="helper">Equation helper</a>' +
    '<a class="gnv" href="#/pattern" data-r="pattern">The folder pattern</a>' +
    '<a class="gnv" href="#/issue" data-r="issue">Raise an issue</a>' +
    (ctx.file ? '' : '<button class="ctl small gnv-reload" type="button" id="greload">Read the folder again</button>');
  if ($('#greload')) $('#greload').addEventListener('click', reload);
}

function markNav(r) {
  document.querySelectorAll('.gnv').forEach(a => {
    const d = a.dataset.r;
    a.classList.toggle('sel', d === r || (d && (d.startsWith('node/') || d === 'structure') && r.startsWith(d + '/')) || (d && r === d));
  });
}

// ── pages ──────────────────────────────────────────────────────────────────

async function route() {
  const r = location.hash.replace(/^#\/?/, '');
  const main = $('#gmain');
  if (r === 'pattern') { markNav(r); main.innerHTML = patternPage(); return; }
  if (r === 'helper') { markNav(r); main.innerHTML = helperPage(); wireHelper(); return; }
  if (!ctx.model || r === 'open') { welcome(); return; }
  markNav(r);
  if (r === '' ) { main.innerHTML = groupPage(ctx); await after(main); return; }
  if (r === 'checks') { main.innerHTML = checksPage(); return; }
  if (r === 'files') { main.innerHTML = filesPage(); wireFiles(); return; }
  const st = /^structure(?:\/([^/]+))?$/.exec(r);
  if (st && ctx.file) {
    const sel = st[1] ? decodeURIComponent(st[1]) : '';
    main.innerHTML = '<div class="gs-root">' + structurePage(ctx, sel) + '</div>';
    wireStructure(ctx, main.firstChild, sel, fromDb);
    if (sel && !window.scrollY) { const n = $('#gs-node'); if (n) n.scrollIntoView({ block: 'start' }); }
    return;
  }
  if (r === 'sign') { main.innerHTML = '<p class="muted">Working out what is signed…</p>'; main.innerHTML = await signPage(); wireSign(); return; }
  if (r === 'issue') { main.innerHTML = issuePage(); wireIssue(); return; }
  if (r === 'delivery' && ctx.file) { main.innerHTML = deliveryPage(ctx); wireDelivery(ctx); return; }
  const m = /^node\/([^/]+)(?:\/([a-z]+))?$/.exec(r);
  if (m) {
    const tab = NODE_TABS.some(([k]) => k === m[2]) ? m[2] : 'explain';
    main.innerHTML = nodePage(ctx, decodeURIComponent(m[1]), tab);
    await after(main);
    return;
  }
  main.innerHTML = '<p class="gmiss">Nothing at ' + esc(r) + '.</p>';
}

async function after(host) {
  wireUp(ctx, host);
  wireAlgorithm(host);
  wireCode(ctx, host);
  await mountFigures(ctx, host);
  await mountResults(ctx, host);
  window.scrollTo(0, 0);
}

function checksPage() {
  const f = ctx.findings;
  const by = l => f.filter(x => x.level === l);
  return '<h1 class="gh1">Checks</h1><p>The folder against its pattern (groups/SPEC.toml). An <b>error</b> stops it being sealed; a <b>warning</b> is ' +
    'worth fixing and does not; a <b>note</b> is for information. Each one names its file and line — click it to go there.</p>' +
    '<h2 class="gh">Errors (' + by('error').length + ')</h2>' + findingsList(by('error'), 'No errors.') +
    '<h2 class="gh">Warnings (' + by('warning').length + ')</h2>' + findingsList(by('warning'), 'No warnings.') +
    '<h2 class="gh">Notes (' + by('note').length + ')</h2>' + findingsList(by('note'), 'No notes.');
}

// ── signing and sealing ────────────────────────────────────────────────────


async function signPage() {
  const m = ctx.model;
  const { reviews, fingerprintOf } = await reviewState(ctx.folder, m);
  const scopes = ['group'].concat(m.order);
  const rows = [];
  for (const s of scopes) {
    const fp = await fingerprintOf(s);
    const last = reviews.filter(r => (r.scope || 'group') === s).slice(-1)[0];
    const state = !last ? '<span class="muted">not signed</span>' : last.current
      ? (last.verdict === 'ok' ? '<b class="gsig-ok">signed ok</b> by ' + esc(last.name) : '<b class="gsig-ch">changes asked</b> by ' + esc(last.name) + (last.note ? ': ' + esc(last.note) : ''))
      : '<span class="gsig-stale">stale</span> — ' + esc(last.name) + ' signed an earlier content';
    const can = me && maySign(m, me, s);
    rows.push('<tr><td>' + (s === 'group' ? '<b>the whole group</b>' : '<a href="#/node/' + esc(s) + '">' + esc(s) + '</a>') + '</td><td><code title="' + fp + '">' + fp.slice(0, 12) + '</code></td><td>' + state + '</td><td>' +
      (can ? '<button class="ctl small gsig" data-scope="' + esc(s) + '" data-v="ok" type="button">sign ok</button> ' +
        '<button class="ctl small gsig" data-scope="' + esc(s) + '" data-v="changes" type="button">ask for changes</button>' : '<span class="muted">' + (me ? 'not yours to sign' : '') + '</span>') + '</td></tr>');
  }
  const blockers = await sealBlockers(ctx.folder, m, ctx.findings);
  return '<h1 class="gh1">Sign &amp; seal</h1>' +
    '<p>Each node is signed by its author and the whole group by its owner (members.csv). A signature is for the content as it is now — its ' +
    '<i>fingerprint</i>. Change a file and the signature goes stale.</p>' +
    (ctx.file ? '<p class="gsave-ok">Sign-offs are kept in ' + esc(ctx.file.name) + ' — <b>Save</b> it afterwards.</p>' : ctx.folder.writable ? '<p class="gsave-ok">This folder is open for saving: sign-offs and the package are written straight into it.</p>'
      : '<p class="gsave-dl">This folder was opened read-only, so sign-offs and the package are <b>downloaded</b>. Put each downloaded file into the folder where its name says ' +
        '(<code>packages__' + esc(m.meta.id || 'group') + '-' + esc(m.meta.version || '') + '.zip</code> goes in <code>packages/</code>).</p>') +
    '<p><label>I am <select id="gme"><option value="">— choose your name —</option>' + m.group.members.map(x =>
      '<option' + (x.name === me ? ' selected' : '') + '>' + esc(x.name) + '</option>').join('') + '</select></label> ' +
    '<input id="gnote" class="gnote" placeholder="what you want changed (when asking for changes)"></p>' +
    '<div class="ri-wrap"><table class="fx gtable"><thead><tr><th>What</th><th>Fingerprint now</th><th>Signed</th><th></th></tr></thead><tbody>' + rows.join('') + '</tbody></table></div>' +
    '<h2 class="gh">Seal for the developer</h2>' +
    (blockers.length ? '<p>Not yet. Still standing in the way:</p><ul class="gfind">' + blockers.map(b => '<li class="gf-error">' + esc(b) + '</li>').join('') + '</ul>'
      : (ctx.file ? '<p>Everything is checked and signed. Sealing writes the release, <code>' + esc(fileName(m.meta.id, 'release', m.meta.version)) + '</code>: every node\'s content, every sign-off and the fingerprint they were given for. ' +
        'It is never changed afterwards, and it is what the developer builds from.</p>'
        : '<p>Everything is checked and signed. Sealing writes one file, <code>packages/' + esc(m.meta.id || 'group') + '-' + esc(m.meta.version || '') + '.zip</code>, holding every file, a manifest of their fingerprints and who signed. That file is what the developer builds from.</p>') +
        '<button class="ctl gbtn" id="gseal" type="button">Seal version ' + esc(m.meta.version || '') + '</button>') +
    '<p id="gseal-out" class="gout" aria-live="polite"></p>';
}

function wireSign() {
  const sel = $('#gme');
  if (sel) sel.addEventListener('change', () => { ctx.me = me = sel.value; try { localStorage.setItem(ME, me); } catch { /* */ } route(); });
  document.querySelectorAll('.gsig').forEach(b => b.addEventListener('click', async () => {
    const how = await sign(ctx.folder, ctx.model, { name: me, scope: b.dataset.scope, verdict: b.dataset.v, note: $('#gnote').value });
    await refresh();
    $('#gseal-out').textContent = how === 'kept' ? 'Signed — Save the file to keep it.' : how === 'saved' ? 'Signed, and saved into reviews.csv.' : 'Signed. reviews.csv was downloaded — put it into the folder, replacing the old one.';
  }));
  const s = $('#gseal');
  if (s && ctx.file) s.addEventListener('click', () => trying(async () => {
    const r = await sealDb();
    $('#gseal-out').innerHTML = (r.how === 'saved' ? 'Sealed: ' + esc(r.name) + ' is written.' : r.how === 'cancelled' ? 'Not sealed: no file was chosen.' : 'Sealed: ' + esc(r.name) + ' was downloaded — put it on the group\'s drive.') +
      ' Fingerprint <code>' + r.fingerprint.slice(0, 16) + '…</code>. Tell the developer it is ready.';
  }));
  else if (s) s.addEventListener('click', async () => {
    const r = await seal(ctx.folder, ctx.model, me || 'unknown');
    $('#gseal-out').innerHTML = (r.how === 'saved' ? 'Sealed: ' + esc(r.name) + ' is in the folder.' : 'Sealed: the package was downloaded — put it in packages/.') +
      ' Fingerprint <code>' + r.fingerprint.slice(0, 16) + '…</code>. Tell the developer it is ready.';
  });
}

async function refresh() {
  if (ctx.file) { await fromDb(); return; }
  ctx.model = await loadGroup(ctx.folder);
  ctx.findings = await checkGroup(ctx.model, SPEC);
  nav();
  await route();
}

// ── node files and the release ─────────────────────────────────────────────

/** Whether a node has any content in the open file. */
function hasContent(db, uid) {
  return Number(db.value("SELECT (SELECT COUNT(*) FROM doc WHERE scope = ?) + (SELECT COUNT(*) FROM tbl WHERE scope = ?) + (SELECT COUNT(*) FROM media WHERE scope = ?)", [uid, uid, uid])) > 0;
}

function filesPage() {
  if (!ctx.file) {
    return '<h1 class="gh1">Keep this folder as a database</h1><p>The folder becomes one file, <code>' + esc(fileName(ctx.model.meta.id || ctx.folder.name, 'release', ctx.model.meta.version)) + '</code>: ' +
      'every node, its content, the group\'s text and the sign-offs, in an ordinary SQLite database. From then on the group works from that file — the lead edits its structure here, ' +
      'and issues each author a node file to fill.</p><p><button class="ctl gbtn" type="button" id="gk-go">Make the database</button></p><p id="gk-out" class="gout" aria-live="polite"></p>';
  }
  const db = ctx.file.db;
  const live = db.all('SELECT * FROM node WHERE archived = 0 ORDER BY ord, id');
  const sealed = db.meta('sealed');
  const rows = live.map(n => {
    const issued = Number(db.meta('issued:' + n.uid) || 0);
    const state = !issued ? '<span class="muted">not issued</span>' : issued < Number(n.contract_version) ? '<b class="gsig-ch">behind — contract v' + n.contract_version + '</b>' : 'issued at v' + issued;
    return '<tr><td><a href="#/node/' + esc(n.id) + '">' + esc(n.id) + '</a></td><td class="small">' + esc(n.author || '—') + '</td><td class="small">v' + esc(n.contract_version) + '</td><td class="small">' + state +
      '</td><td class="small">' + (hasContent(db, n.uid) ? 'filled (rev ' + esc(n.revision) + ')' : '<span class="muted">empty</span>') + '</td>' +
      '<td><button class="ctl small" type="button" data-issue="' + esc(n.uid) + '">issue</button></td></tr>';
  }).join('');
  const dir = typeof window.showDirectoryPicker === 'function';
  return '<h1 class="gh1">Node files &amp; the release</h1>' +
    '<section class="answer-first view-af"><p class="af-k">Answer first</p><p class="af-a">Each author fills one node file. You issue it from here, they fill it in the node application and put it back on the drive, ' +
    'and you assemble every node file into the release.</p><ul class="af-points">' +
    '<li>A node file carries every node\'s contract, so its author sees their neighbours, and only their own node\'s content.</li>' +
    '<li>Issued from a release, it carries that release\'s content: the author starts from what was sealed.</li>' +
    '<li>Node files are never deleted. Each release records the revision of every node it took.</li></ul></section>' +
    (sealed ? '<p class="gsave-ok">This is the sealed release ' + esc(db.meta('version')) + '. Issue node files from it to start the next version.</p>' : '') +
    '<h2 class="gh">Issue node files</h2><div class="ri-wrap"><table class="fx gtable"><thead><tr><th>Node</th><th>Author</th><th>Contract</th><th>Its file</th><th>Content here</th><th></th></tr></thead><tbody>' + rows + '</tbody></table></div>' +
    '<p><button class="ctl gbtn" type="button" id="gf-all">' + (dir ? 'Issue every node file into a folder…' : 'Issue every node file (one zip)') + '</button></p>' +
    (sealed ? '' : '<h2 class="gh">Assemble the release</h2><p>Choose the node files the authors put back — several at once, or their whole folder. Each node\'s content is taken from its file; ' +
      'a node with no file keeps what this file already holds. What does not fit is listed.</p>' +
      '<p><label class="ctl gbtn">Choose node files…<input type="file" id="gf-nodes" accept=".vnode" multiple hidden></label> ' +
      '<label class="ctl gbtn">Choose their folder…<input type="file" id="gf-dir" webkitdirectory directory multiple hidden></label></p>') +
    (ctx.file.kind === 'release' ? '<h2 class="gh">The structure on its own</h2><p>The contracts, the people and the group\'s own text, without any node\'s content: the file to keep editing the structure in.</p>' +
      '<p><button class="ctl gbtn" type="button" id="gf-struct">Save the structure (' + esc(fileName(db.meta('group_id'), 'structure')) + ')</button></p>' : '') +
    '<p id="gf-out" class="gout" aria-live="polite"></p><div id="gf-notes"></div>';
}

async function issueOne(uid) {
  const db = ctx.file.db;
  const n = db.one('SELECT id, contract_version FROM node WHERE uid = ?', [uid]);
  const nf = await issueNode(db, uid);
  nf.setMeta('issued_from', ctx.file.name);
  nf.setMeta('issued_at', new Date().toISOString());
  db.setMeta('issued:' + uid, n.contract_version);
  logChange(db, me, uid, 'issued the node file at contract v' + n.contract_version, null, null);
  return { name: fileName(n.id, 'node'), bytes: nf.bytes(), close: () => nf.close() };
}

function wireFiles() {
  const out = t => { $('#gf-out') ? ($('#gf-out').innerHTML = t) : ($('#gk-out').innerHTML = t); };
  if (!ctx.file) {
    $('#gk-go').addEventListener('click', () => trying(async () => {
      const db = await dbFromFolder(ctx.folder, 'release');
      logChange(db, me, 'group', 'made the database from the folder ' + ctx.folder.name, null, null);
      const f = new DbFile(db, fileName(ctx.model.meta.id || ctx.folder.name, 'release', ctx.model.meta.version));
      const how = await f.saveAs(f.name);
      if (how === 'cancelled') { out('Not made: no file was chosen.'); return; }
      await openDb(f);
      location.hash = '#/files';
      flash(how === 'saved' ? 'Made ' + f.name + ', and opened it.' : 'Made ' + f.name + ' (downloaded) and opened it. Put it on the group\'s drive.');
    }));
    return;
  }
  document.querySelectorAll('[data-issue]').forEach(b => b.addEventListener('click', () => trying(async () => {
    const r = await issueOne(b.dataset.issue);
    await saveBytesAs(r.name, r.bytes);
    r.close();
    await fromDb();
    flash('Issued ' + r.name + '. Send it to its author, or put it in their folder on the drive. Save this file too: it records what was issued.');
  })));
  $('#gf-all').addEventListener('click', () => trying(async () => {
    const live = ctx.file.db.all('SELECT uid FROM node WHERE archived = 0 ORDER BY ord, id');
    if (typeof window.showDirectoryPicker === 'function') {
      let dir;
      try { dir = await window.showDirectoryPicker({ mode: 'readwrite' }); } catch { return; }
      for (const n of live) {
        const r = await issueOne(n.uid);
        const fh = await dir.getFileHandle(r.name, { create: true });
        const w = await fh.createWritable();
        await w.write(r.bytes);
        await w.close();
        r.close();
      }
    } else {
      const entries = [];
      for (const n of live) { const r = await issueOne(n.uid); entries.push({ path: r.name, data: r.bytes }); r.close(); }
      download(ctx.file.db.meta('group_id') + '-node-files.zip', zip(entries));
    }
    await fromDb();
    flash('Issued ' + live.length + ' node files. Save this file too: it records what was issued.');
  }));
  const collect = async files => {
    const { files: got, refused } = await openMany(files);
    const nodes = got.filter(f => f.kind === 'node');
    if (!nodes.length) { out('None of those is a node file (.vnode).' + (refused.length ? ' Refused: ' + esc(refused.join('; ')) : '')); return; }
    const { db, notes } = await assemble(ctx.file.db, nodes.map(f => f.db));
    logChange(db, me, 'group', 'assembled ' + nodes.length + ' node files', null, nodes.map(f => f.name).join(' '));
    for (const f of nodes) f.db.close();
    const g = ctx.file.db.meta('group_id'), v = ctx.file.db.meta('version');
    const rel = new DbFile(db, fileName(g, 'release', v));
    rel.dirty = true;
    await openDb(rel);
    location.hash = '#/files';
    setTimeout(() => {
      const box = $('#gf-notes');
      if (!box) return;
      box.innerHTML = '<h2 class="gh">Assembled ' + nodes.length + ' node file(s)</h2>' + (notes.length || refused.length
        ? '<ul class="gfind">' + notes.map(n => '<li class="gf-' + n.level + '">' + esc(n.msg) + '</li>').join('') + refused.map(r => '<li class="gf-error">' + esc(r) + '</li>').join('') + '</ul>'
        : '<p>Every node had its file, and every file was written to its node\'s current contract.</p>') +
        '<p>This is the release, not yet sealed: <b>Save</b> it, look through it, have it signed, then seal it.</p>';
    }, 0);
  };
  const pickN = $('#gf-nodes'), pickD = $('#gf-dir');
  if (pickN) pickN.addEventListener('change', e => trying(() => collect([...e.target.files])));
  if (pickD) pickD.addEventListener('change', e => trying(() => collect([...e.target.files])));
  const st = $('#gf-struct');
  if (st) st.addEventListener('click', () => trying(async () => {
    const s = await structureOf(ctx.file.db);
    const f = new DbFile(s, fileName(ctx.file.db.meta('group_id'), 'structure'));
    const how = await f.saveAs(f.name);
    s.close();
    out(how === 'saved' ? 'Saved ' + esc(f.name) + '.' : how === 'cancelled' ? '' : esc(f.name) + ' was downloaded.');
  }));
}

/** Seal the open database: a copy marked sealed, with the fingerprint every sign-off was given for. */
async function sealDb() {
  const fp = await fingerprint(ctx.folder, 'group');
  const m = ctx.model.meta;
  const r = await copy(ctx.file.db);
  r.tx(() => {
    r.setMeta('file_kind', 'release');
    r.setMeta('sealed', new Date().toISOString());
    r.setMeta('sealed_by', me || 'unknown');
    r.setMeta('fingerprint', fp.fingerprint);
    r.setMeta('spec', String(SPEC.version || ''));
    logChange(r, me, 'group', 'sealed version ' + (m.version || ''), null, fp.fingerprint);
  });
  const f = new DbFile(r, fileName(m.id, 'release', m.version));
  const how = await f.saveAs(f.name);
  if (how !== 'cancelled') { ctx.file = f; await fromDb(); ctx.file.dirty = false; topbar(); }
  return { how, name: f.name, fingerprint: fp.fingerprint };
}

// ── raising an issue ───────────────────────────────────────────────────────

function issuePage() {
  return '<h1 class="gh1">Raise an issue</h1><p>Something wrong, missing or unclear — in this folder, or in the test application the developer sent. ' +
    'It is written into the folder\'s <code>issues/</code>, where the developer reads it.</p>' +
    '<p><label>Your name <input id="gi-by" value="' + esc(me) + '"></label></p>' +
    '<p><label>Where <input id="gi-where" class="gnote" placeholder="e.g. node sw_f107_design, Results tab; or the test app, Run page"></label></p>' +
    '<p><textarea id="gi-text" rows="6" class="gnote" placeholder="What you saw, and what you expected"></textarea></p>' +
    '<button class="ctl gbtn" id="gi-go" type="button">Write the issue</button><p id="gi-out" class="gout" aria-live="polite"></p>';
}

function wireIssue() {
  $('#gi-go').addEventListener('click', async () => {
    const text = $('#gi-text').value.trim();
    if (!text) { $('#gi-out').textContent = 'Say what the issue is first.'; return; }
    const r = await raiseIssue(ctx.folder, { by: $('#gi-by').value || 'unknown', where: $('#gi-where').value, text, version: ctx.model.meta.version });
    $('#gi-out').textContent = r.how === 'saved' ? 'Written to ' + r.path + '.' : 'Downloaded — put it in the folder as ' + r.path + '.';
  });
}

// ── the equation helper ────────────────────────────────────────────────────

function helperPage() {
  return '<h1 class="gh1">Equation helper</h1>' +
    '<p>Type an equation the way you would in an email. The helper shows it typeset and gives the LaTeX to paste into the <code>latex</code> column of equations.csv.</p>' +
    '<p><input id="gh-in" class="gnote gh-in" value="v = sqrt(mu / r)" aria-label="the equation, typed plainly"></p>' +
    '<div id="gh-out" class="gh-out"></div>' +
    '<p><label>LaTeX <input id="gh-tex" class="gnote" aria-label="LaTeX"></label> <button class="ctl small" id="gh-copy" type="button">copy</button></p>' +
    '<p class="muted" id="gh-prob"></p>' +
    '<h2 class="gh">How to type</h2><div class="ri-wrap"><table class="fx gtable"><thead><tr><th>You type</th><th>You get</th></tr></thead><tbody>' +
    [['a / b', 'a fraction'], ['x^2, x^(n+1)', 'a power'], ['x_0, F_10.7 as F_107', 'an index'], ['sqrt(x), abs(x), exp(x), sin(x)', 'a function'],
      ['mu, sigma, Omega, MU_EARTH', 'Greek; a constant\'s name keeps its index'], ['<=, >=, !=, ->', 'relations and arrows'], ['6371 [km]', 'a number with its unit'],
      ['3/2 J2 (R/r)^2', 'a product, written with spaces']].map(([a, b]) => '<tr><td><code>' + esc(a) + '</code></td><td>' + esc(b) + '</td></tr>').join('') +
    '</tbody></table></div><p class="muted">Already have LaTeX — from a paper, from Word\'s "copy as LaTeX", or written by an assistant from a photo? Paste it into the LaTeX box to see it typeset.</p>';
}

function wireHelper() {
  const inp = $('#gh-in'), tex = $('#gh-tex'), out = $('#gh-out'), prob = $('#gh-prob');
  const show = t => {
    const r = texToMathml(t, true);
    out.innerHTML = r.html;
    return r.problems;
  };
  const fromShort = () => {
    const r = shortToTex(inp.value);
    tex.value = r.tex;
    prob.textContent = r.problems.concat(show(r.tex)).join('; ');
  };
  inp.addEventListener('input', fromShort);
  tex.addEventListener('input', () => { prob.textContent = show(tex.value).join('; '); });
  $('#gh-copy').addEventListener('click', () => { tex.select(); try { navigator.clipboard.writeText(tex.value); } catch { document.execCommand('copy'); } });
  fromShort();
}

// ── the pattern itself, from the spec ──────────────────────────────────────

function patternPage() {
  const files = SPEC.file || [];
  const part = level => files.filter(f => f.level === level || (level === 'group' && f.level === 'both')).map(f =>
    '<li><code>' + esc(level === 'node' ? 'nodes/<id>/' + f.path : f.path) + '</code> ' + (f.required ? '<b>required</b>' + (f.kinds ? ' for ' + esc(f.kinds.join(', ')) + ' nodes' : '') : '<span class="muted">optional</span>') +
    (f.written_by_app ? ' <span class="muted">— written by this page</span>' : '') +
    ' — ' + esc(f.says.trim()) + (f.columns ? '<br><span class="muted small">columns: ' + f.columns.map(c => esc(c.name) + (c.optional ? '?' : '')).join(', ') + '</span>' : '') + '</li>').join('');
  return '<h1 class="gh1">The folder pattern</h1><p>Version ' + esc(SPEC.version || '') + ' of the pattern every group folder follows (groups/SPEC.toml). The checks on this page are read from it.</p>' +
    '<h2 class="gh">The two kinds of text</h2>' + (SPEC.text || []).map(t => '<h3 class="gh3"><code>' + esc(t.file) + '</code> — ' + esc(t.role) + '</h3><p>' + esc(t.says.trim()) + '</p><p class="muted">Sections, in order: ' + t.headings.map(esc).join(' · ') + '</p>').join('') +
    '<h2 class="gh">What text can hold</h2><ul>' + (SPEC.embed || []).map(e => '<li><code>' + esc(e.form) + '</code> — ' + esc(e.says) + '</li>').join('') + '</ul>' +
    '<h2 class="gh">The group\'s files</h2><ul class="gpat">' + part('group') + '</ul>' +
    '<h2 class="gh">Each node\'s files</h2><ul class="gpat">' + part('node') + '</ul>' +
    '<h2 class="gh">Conventions</h2>' + Object.entries(SPEC.conventions || {}).map(([k, v]) => '<h3 class="gh3">' + esc(k) + '</h3><p>' + esc(v.trim()) + '</p>').join('');
}

// ── start ──────────────────────────────────────────────────────────────────

initDepth();
dropAnywhere();
$('#gsave').addEventListener('click', () => trying(saveFile));
window.addEventListener('beforeunload', e => { if (ctx.file && ctx.file.dirty) { e.preventDefault(); e.returnValue = ''; } });
window.addEventListener('hashchange', route);
route();
