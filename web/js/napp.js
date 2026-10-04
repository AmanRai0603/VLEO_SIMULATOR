/*
  THE NODE APPLICATION — one author fills one node's file.

  One HTML file, offline, nothing to install (web/node.html, built by
  `cargo run -p xtask -- group-app`). The group lead issues each author their
  node's database file (<node>.vnode); the author opens it here and is walked
  through what the node needs, step by step: what it answers (the contract —
  theirs to read, the lead's to change), the explanation, the theory, the
  pseudocode, the inputs' defaults, the results from their own code, the
  evidence, the pictures, the code. Every step shows what it will look like,
  as it is typed. Then a preview of the whole node as the group will see it,
  the checks, and the author's sign-off. Save writes the file back.

  Nothing here runs anything. The pseudocode is read and typeset, never
  executed; the results are the author's, from their own code, a hand
  calculation or a paper — the developer's code is later tested against them.
*/
'use strict';

import { $, esc } from './dom.js';
import { initDepth } from './depth.js';
import { loadGroup, resultColumns } from './gmodel.js';
import { checkGroup } from './gcheck.js';
import { nodePage, mountFigures, mountResults, wireUp, wireAlgorithm, wireCode, findingsList, NODE_TABS, md, wiringSvg } from './gview.js';
import { sections } from './md.js';
import { parseCsv, records, toCsv, splitUnit } from './csv.js';
import { texToMathml, shortToTex, pseudocodeEquations } from './texmath.js';
import { folderFromDb, logChange } from './gdb.js';
import { fromFile, pickForSaving, canSaveInPlace } from './gfile.js';
import { sign, reviewState } from './gseal.js';
import { mountTable } from './gtable.js';
import { readMethod, checkerProblem } from './gmethod.js';

const SPEC = window.VLEO_GROUP_SPEC || { file: [], text: [], embed: [], limits: {} };
const ctx = { file: null, folder: null, model: null, findings: [], all: [], spec: SPEC, uid: '', id: '', me: '' };
const ME = 'vleo.node.me';
try { ctx.me = localStorage.getItem(ME) || ''; } catch { /* no storage */ }

const textRule = file => (SPEC.text || []).find(t => t.file === file) || { headings: [], says: '' };
const fileRule = (path, level = 'node') => (SPEC.file || []).find(f => f.path === path && (f.level === level || f.level === 'both')) || {};

// What each heading asks for, in a line: the prompt beside its box.
const ASK = {
  'In one line': 'The answer, in one sentence a newcomer understands.',
  'Said simply': 'The idea in plain words, as you would tell a colleague from another team. No equations.',
  'Picture it': 'A picture that makes it obvious: place one with {{fig id}} (made in Pictures), and say what to look at.',
  'Guess first': 'A question the reader answers before reading on: {{guess the question || the answer}}.',
  'Where it breaks': 'When the simple version stops being true, and what happens then.',
  'Common misreading': 'The mistake people most often make with this, and the right reading.',
  'Equations': 'The governing equations: place each with {{eq E1}} (written in the table below) or write $$…$$.',
  'Derivation': 'How the equations follow, step by step — a numbered list works well.',
  'Assumptions': 'Each assumption as a condition the inputs must meet.',
  'Validity': 'Where the result can be trusted, and where it cannot.',
};

// ── opening the file ───────────────────────────────────────────────────────

async function openFile(file) {
  if (!file) return;
  const kind = file.kind;
  if (kind !== 'node') {
    throw new Error(file.name + ' is a ' + (kind || 'database') + ' file, not a node file. ' +
      (kind ? 'Open it in the group application (group.html); a node file (.vnode) is issued from there.' : ''));
  }
  ctx.file = file;
  ctx.uid = file.db.meta('node_uid');
  const row = file.db.one('SELECT id FROM node WHERE uid = ?', [ctx.uid]);
  if (!row) throw new Error('this node file names a node (' + ctx.uid + ') its own contracts do not have');
  ctx.id = row.id;
  await rebuild();
  document.title = ctx.id + ' · VLEO node';
  if (!location.hash || location.hash === '#/open') location.hash = '#/step/contract'; else route();
}

/** Everything read again from the database: after opening, and after each change that redraws. */
async function rebuild() {
  ctx.folder = folderFromDb(ctx.file.db, ctx.file.name.replace(/\.[^.]+$/, ''));
  ctx.folder.onChange = () => { ctx.file.dirty = true; ctx.stale = true; topbar(); };
  ctx.model = await loadGroup(ctx.folder);
  ctx.all = await checkGroup(ctx.model, SPEC);
  const dir = 'nodes/' + ctx.id + '/';
  ctx.findings = ctx.all.filter(f => f.where.startsWith(dir));
  ctx.stale = false;
  nav();
  topbar();
}

const node = () => ctx.model.nodes.get(ctx.id);
const contract = () => ctx.file.db.one('SELECT * FROM node WHERE uid = ?', [ctx.uid]);
const dir = () => 'nodes/' + ctx.id + '/';

function topbar() {
  const f = ctx.file;
  $('#gtitle').textContent = f ? ctx.id : 'nothing open';
  $('#gfile').innerHTML = f ? esc(f.name) + (f.dirty ? ' · <b class="gdirty">not saved</b>' : ' · saved') : 'offline';
  $('#gsave').hidden = !f;
  if (f) $('#gsave').textContent = f.handle ? 'Save' : 'Save (download)';
}

async function save() {
  if (!ctx.file) return;
  const db = ctx.file.db;
  // Each save is a revision: the lead sees which revision a release took.
  db.tx(() => {
    db.exec('UPDATE node SET revision = revision + 1 WHERE uid = ?', [ctx.uid]);
    logChange(db, ctx.me, ctx.uid, 'saved revision ' + (Number(contract().revision)), null, null);
  });
  const how = await ctx.file.save();
  topbar();
  flash(how === 'saved' ? 'Saved ' + ctx.file.name + ' (revision ' + contract().revision + ').'
    : ctx.file.name + ' was downloaded (revision ' + contract().revision + '). Put it back in your folder on the group\'s drive, replacing the old one.');
}

function flash(msg) {
  const el = $('#gflash');
  el.textContent = msg;
  el.hidden = false;
  clearTimeout(flash.t);
  flash.t = setTimeout(() => { el.hidden = true; }, 7000);
}

async function trying(fn) {
  try { await fn(); } catch (e) {
    $('#gmain').insertAdjacentHTML('afterbegin', '<p class="gs-err" role="alert">' + esc(e.message || String(e)) + '</p>');
    window.scrollTo(0, 0);
  }
}

function welcome() {
  const inPlace = canSaveInPlace();
  $('#gnav').innerHTML = '<p class="muted small">Open your node file to begin.</p>';
  $('#gmain').innerHTML =
    '<section class="answer-first view-af"><p class="af-k">Answer first</p><p class="af-a">Open the node file your group lead gave you ' +
    '(<code>&lt;node&gt;.vnode</code>) and this page walks you through everything your node needs, showing each part as the group will see it.</p>' +
    '<ul class="af-points"><li>Nothing leaves this computer. The page has no network and runs nothing.</li>' +
    '<li>What your node answers and what feeds it — its contract — is set by your lead. You can read it here and ask for a change.</li>' +
    '<li>Save often. Then put the file back in your folder on the group\'s drive.</li></ul></section>' +
    '<div class="gopen gcards"><div class="gdrop gcard"><h2 class="gh3">Open your node file</h2><p class="muted small">Drop it here, or</p>' +
    '<label class="ctl gbtn">Choose the file…<input type="file" id="npick" accept=".vnode" hidden></label>' +
    (inPlace ? ' <button class="ctl gbtn" type="button" id="npickrw">Open it for saving…</button>' +
      '<p class="muted small">"Open it for saving" writes straight back into the same file when you Save. Otherwise Save downloads it.</p>'
      : '<p class="muted small">This browser saves by downloading: put the saved file back in your folder.</p>') + '</div>' +
    '<div class="gcard"><h2 class="gh3">What you will fill</h2><ol class="small"><li>Read your node: what it answers, what feeds it</li><li>The explanation, for anyone</li>' +
    '<li>The theory and maths</li><li>The pseudocode — the algorithm, line by line</li><li>Your inputs\' defaults and ranges</li><li>Results from your own code</li>' +
    '<li>Evidence from outside any code</li><li>Pictures: charts, flows, steps, images, video</li><li>Your code, kept for the record</li><li>Preview, check and sign</li></ol></div></div>';
  $('#npick').addEventListener('change', e => trying(async () => openFile(await fromFile(e.target.files[0]))));
  if (inPlace) $('#npickrw').addEventListener('click', () => trying(async () => openFile(await pickForSaving())));
}

// ── the steps ──────────────────────────────────────────────────────────────

function steps() {
  const k = contract().kind;
  const computed = k === 'computed';
  return [
    ['contract', 'Your node', true],
    ['explain', 'Explanation', true],
    ['theory', 'Theory & maths', true],
    ['pseudocode', 'Pseudocode', computed],
    ['inputs', 'Inputs', computed],
    ['results', 'Results', computed],
    ['evidence', 'Evidence & sources', true],
    ['pictures', 'Pictures', true],
    ['code', 'Code & files', true],
    ['preview', 'Preview', true],
    ['sign', 'Check & sign', true],
  ].filter(s => s[2]);
}

/** A step's state: 'ok' when its files are there and clean, 'err' when something in them is wrong, '' when untouched. */
function stepState(step) {
  const n = node(), d = dir();
  const files = { explain: ['explanation.md'], theory: ['theory.md', 'equations.csv', 'symbols.csv'], pseudocode: ['pseudocode.txt'], inputs: ['inputs.csv'],
    results: ['results/isolation.csv', 'results/how-run.md'], evidence: ['evidence.csv', 'sources.csv'], pictures: ['figures.csv'], code: [] }[step];
  if (!files) return '';
  const errs = ctx.findings.filter(f => f.level === 'error' && files.some(p => f.where.startsWith(d + p)));
  if (errs.length) return 'err';
  if (step === 'code') return n.code.length ? 'ok' : '';
  return files.some(p => ctx.folder.has(d + p)) ? 'ok' : '';
}

function nav() {
  const errors = ctx.findings.filter(f => f.level === 'error').length;
  $('#gnav').innerHTML = '<p class="gnv-h">' + esc(ctx.id) + '</p>' + steps().map(([k, label], i) => {
    const st = stepState(k);
    return '<a class="gnv" href="#/step/' + k + '" data-r="step/' + k + '"><span class="gdot ' + (st === 'err' ? 'gdot-err' : st === 'ok' ? 'gdot-ok' : 'gdot-none') + '"></span>' +
      (i + 1) + '. ' + esc(label) + (k === 'sign' ? ' <span class="gcount' + (errors ? ' bad' : '') + '">' + errors + '</span>' : '') + '</a>';
  }).join('') + '<p class="gnv-h">Tools</p><a class="gnv" href="#/step/helper" data-r="step/helper">Equation helper</a>';
}

function markNav(r) {
  document.querySelectorAll('.gnv').forEach(a => a.classList.toggle('sel', a.dataset.r === r || (r.startsWith('node/') && a.dataset.r === 'step/preview')));
}

async function route() {
  const r = location.hash.replace(/^#\/?/, '');
  const main = $('#gmain');
  if (!ctx.file || r === 'open') { welcome(); return; }
  if (ctx.stale) await rebuild();
  markNav(r);
  const pv = /^node\/([^/]+)(?:\/([a-z]+))?$/.exec(r);
  if (pv) { await preview(main, pv[2]); return; }
  const m = /^step\/([a-z]+)$/.exec(r);
  const step = m ? m[1] : 'contract';
  const page = PAGES[step];
  if (!page) { main.innerHTML = '<p class="gmiss">Nothing at ' + esc(r) + '.</p>'; return; }
  main.innerHTML = '';
  const host = document.createElement('div');
  host.className = 'nstep';
  main.appendChild(host);
  await page(host);
  const list = steps(), i = list.findIndex(s => s[0] === step);
  if (i >= 0) {
    host.insertAdjacentHTML('beforeend', '<p class="nnext">' + (i > 0 ? '<a class="ctl" href="#/step/' + list[i - 1][0] + '">← ' + esc(list[i - 1][1]) + '</a> ' : '') +
      (i < list.length - 1 ? '<a class="ctl" href="#/step/' + list[i + 1][0] + '">' + esc(list[i + 1][1]) + ' →</a>' : '') + '</p>');
  }
  window.scrollTo(0, 0);
}

const lead = (title, answer, points) => '<h1 class="gh1">' + esc(title) + '</h1><section class="answer-first view-af"><p class="af-k">Answer first</p><p class="af-a">' + answer + '</p>' +
  (points && points.length ? '<ul class="af-points">' + points.map(p => '<li>' + p + '</li>').join('') + '</ul>' : '') + '</section>';

const findingsFor = paths => {
  const f = ctx.findings.filter(x => paths.some(p => x.where.startsWith(dir() + p)));
  return f.length ? '<div class="nfind"><p class="gh3">What the checks say</p>' + findingsList(f, '') + '</div>' : '';
};

/** Write one file of this node, and recheck it on the next redraw. */
async function put(rel, text) { await ctx.folder.write(dir() + rel, text, rel.endsWith('.csv') ? 'text/csv' : 'text/plain'); }

// ── 1 · the contract ───────────────────────────────────────────────────────

async function pageContract(host) {
  const db = ctx.file.db, c = contract();
  const ins = db.all('SELECT * FROM input WHERE node_uid = ? ORDER BY ord', [ctx.uid]);
  const idOf = new Map(db.all('SELECT uid, id FROM node').map(r => [r.uid, r.id]));
  const readers = db.all('SELECT DISTINCT n.id FROM input i JOIN node n ON n.uid = i.node_uid WHERE i.source = ? AND n.archived = 0', [ctx.uid]).map(r => r.id);
  const reqs = db.all('SELECT * FROM request WHERE node_uid = ? ORDER BY at DESC', [ctx.uid]);
  const issuedFrom = db.meta('issued_from'), issuedAt = db.meta('issued_at');
  host.innerHTML = lead(ctx.id, '<span class="af-q">' + esc(c.question || 'Your lead has not yet written the question this node answers.') + '</span><br><b>' +
    esc(c.output || ctx.id) + '</b>' + (c.unit ? ' [' + esc(c.unit) + ']' : '') + ((c.lower || c.upper) ? ' · from ' + esc(c.lower || '—') + ' to ' + esc(c.upper || '—') : ''), [
    '<span class="gkind k-' + esc(c.kind) + '">' + esc(c.kind) + '</span> · author ' + esc(c.author || '—') + ' · contract v' + esc(c.contract_version) + ' · revision ' + esc(c.revision),
    'Group ' + esc(db.meta('group_name') || db.meta('group_id')) + ', version ' + esc(db.meta('version')) + (issuedFrom ? ' · issued from ' + esc(issuedFrom) + ' on ' + esc(String(issuedAt).slice(0, 10)) : ''),
  ]) +
    '<p>This is your node\'s <b>contract</b>: what it answers, in what unit, and what feeds it. Your lead set it, so the nodes around yours can rely on it. ' +
    'You fill everything else. If the contract is wrong — a unit, a missing input — ask for a change below; do not work around it.</p>' +
    '<h2 class="gh">Where it sits</h2><div class="gwire">' + neighbourhood() + '</div>' +
    '<h2 class="gh">What feeds it</h2>' + (ins.length ? '<div class="ri-wrap"><table class="fx gtable"><thead><tr><th>Input</th><th>From</th><th>Unit</th><th>Default</th><th>Range</th></tr></thead><tbody>' +
      ins.map(i => '<tr><td><b>' + esc(i.name) + '</b></td><td>' + esc(i.source === 'case' ? 'a value the user sets' : idOf.get(i.source) || i.source) + '</td><td>' + esc(i.unit || '—') + '</td><td>' +
        esc(i.dflt || '—') + '</td><td>' + esc(i.min || i.max ? (i.min || '…') + ' to ' + (i.max || '…') : '—') + '</td></tr>').join('') + '</tbody></table></div>'
      : '<p class="muted">Nothing: a ' + esc(c.kind) + ' node takes no inputs.</p>') +
    '<p class="muted small">Read by: ' + (readers.length ? readers.map(esc).join(', ') : 'no node of this group') + '.</p>' +
    '<h2 class="gh">Ask your lead for a change</h2><p class="muted small">Kept in your node file; your lead sees it when the files are assembled.</p>' +
    '<p><textarea id="nreq" class="gnote" rows="3" placeholder="e.g. the input h should be in metres, not kilometres; or: this node also needs the solar flux"></textarea></p>' +
    '<p><button class="ctl gbtn" type="button" id="nreq-go">Ask</button></p>' +
    (reqs.length ? '<ul class="gs-log">' + reqs.map(q => '<li><span class="muted small">' + esc(q.at.slice(0, 10)) + ' · ' + esc(q.author) + ' · ' + esc(q.status) + '</span> ' + esc(q.body) +
      (q.answer ? '<br><i>' + esc(q.answer) + '</i>' : '') + '</li>').join('') + '</ul>' : '') +
    '<h2 class="gh">Who you are</h2><p><label>I am <input id="nme" class="gs-in" style="max-width:16rem" value="' + esc(ctx.me || c.author) + '" aria-label="your name"></label> ' +
    '<span class="muted small">— the name your sign-off and requests carry.</span></p>';
  wireUp(ctx, host, () => {});
  $('#nme').addEventListener('change', e => { ctx.me = e.target.value.trim(); try { localStorage.setItem(ME, ctx.me); } catch { /* */ } });
  if (!ctx.me) ctx.me = c.author;
  $('#nreq-go').addEventListener('click', () => trying(async () => {
    const body = $('#nreq').value.trim();
    if (!body) throw new Error('say what should change first');
    db.tx(() => {
      db.exec('INSERT INTO request (node_uid, author, at, body) VALUES (?,?,?,?)', [ctx.uid, ctx.me || c.author, new Date().toISOString(), body]);
      logChange(db, ctx.me, ctx.uid, 'asked for a contract change', null, body);
    });
    ctx.file.dirty = true;
    topbar();
    await route();
    flash('Asked. Save the file, and tell your lead.');
  }));
}

function neighbourhood() {
  const m = ctx.model, id = ctx.id;
  const near = m.edges.filter(e => e.to === id || e.from === id);
  const ext = m.external.filter(e => e.to === id);
  const ids = [...new Set([id, ...near.map(e => e.from), ...near.map(e => e.to), ...ext.map(e => e.from)])];
  if (ids.length === 1) return '<p class="muted">No node of this group feeds it or reads it.</p>';
  // Drawn as the group map is, through the same code, kept to this node's neighbours.
  const sub = { ...m, order: ids.filter(x => m.nodes.has(x)), edges: near, external: ext, group: { ...m.group, publishes: m.group.publishes.filter(p => p.node === id) } };
  return wiringSvg(sub);
}

// ── 2, 3 · the two texts ───────────────────────────────────────────────────

/** A text with required headings, one box per heading, previewed beside it as it is typed. */
async function textPage(host, file, title, answer, extra) {
  const rule = textRule(file);
  const cur = node().text[file] || '';
  const parts = sections(cur);
  const known = new Map(parts.sections.map(s => [s.title, s.body.replace(/^\n+|\n+$/g, '')]));
  const others = parts.sections.filter(s => !rule.headings.includes(s.title));
  host.innerHTML = lead(title, answer, []) +
    (rule.forbid_says ? '<p class="muted small">Not here: ' + esc(rule.forbid_says) + '.</p>' : '') +
    rule.headings.map((h, i) => '<section class="ntext"><div class="ntext-in"><h2 class="gh3">' + (i + 1) + '. ' + esc(h) + '</h2><p class="muted small">' + esc(ASK[h] || '') + '</p>' +
      '<textarea class="gnote ntext-box" data-h="' + esc(h) + '" rows="' + Math.max(4, Math.min(16, (known.get(h) || '').split('\n').length + 2)) + '" aria-label="' + esc(h) + '">' + esc(known.get(h) || '') + '</textarea></div>' +
      '<div class="ntext-pv prose" data-pv="' + esc(h) + '"></div></section>').join('') +
    (others.length ? '<p class="muted small">Also in the file, kept as written: ' + others.map(s => esc(s.title)).join(', ') + '.</p>' : '') +
    '<div id="nextra"></div>' + findingsFor([file]);
  const compose = () => {
    let out = parts.intro.replace(/\n+$/, '');
    out = out ? out + '\n\n' : '';
    for (const h of rule.headings) out += '## ' + h + '\n\n' + (host.querySelector('[data-h="' + CSS.escape(h) + '"]').value.replace(/\s+$/, '')) + '\n\n';
    for (const s of others) out += '## ' + s.title + '\n\n' + s.body.replace(/^\n+|\n+$/g, '') + '\n\n';
    return out.replace(/\n+$/, '\n');
  };
  const show = async h => {
    const pv = host.querySelector('[data-pv="' + CSS.escape(h) + '"]');
    const v = host.querySelector('[data-h="' + CSS.escape(h) + '"]').value;
    pv.innerHTML = v.trim() ? md(ctx, node(), v) : '<p class="muted small">How it will read appears here.</p>';
    for (const r of rule.forbid || []) if (v.includes(r)) pv.insertAdjacentHTML('afterbegin', '<p class="gs-err">' + esc(rule.forbid_says || 'not allowed here: ' + r) + '</p>');
    await mountFigures(ctx, pv);
  };
  let t = null;
  host.addEventListener('input', e => {
    const h = e.target.dataset && e.target.dataset.h;
    if (!h) return;
    clearTimeout(t);
    t = setTimeout(async () => { await put(file, compose()); await show(h); }, 400);
  });
  for (const h of rule.headings) await show(h);
  if (extra) await extra(host.querySelector('#nextra'));
}

const pageExplain = host => textPage(host, 'explanation.md', 'Explanation',
  'How to <b>understand</b> your node, for anyone — answer first, then simply, then a picture, a question to guess, where it breaks and the usual mistake.');

const pageTheory = host => textPage(host, 'theory.md', 'Theory & maths',
  'Only the mathematics and its reasoning: the equations, the derivation, the assumptions and where it holds.', async el => {
    el.innerHTML = '<h2 class="gh">Equations</h2><p class="muted small">Each equation once, in LaTeX, with an id. Type it plainly in the helper and copy the LaTeX across, or paste LaTeX from a paper. ' +
      'Place one in the text with <code>{{eq E1}}</code>.</p>' + helperBox() + '<div id="neq"></div><div id="neq-pv"></div>' +
      '<h2 class="gh">Symbols</h2><p class="muted small">What each symbol means — shown when a reader points at it.</p><div id="nsym"></div>';
    wireHelper(el);
    const eqPreview = async () => {
      const rows = records(parseCsv(ctx.folder.files.has(dir() + 'equations.csv') ? await ctx.folder.text(dir() + 'equations.csv') : ''));
      el.querySelector('#neq-pv').innerHTML = rows.filter(r => r.latex).map(r => '<div class="geq"><div class="geq-row"><span class="geq-id">' + esc(r.id) + '</span>' +
        texToMathml(r.latex, true).html + '</div>' + (r.says ? '<p class="small">' + esc(r.says) + '</p>' : '') + '</div>').join('');
    };
    await csvTable(el.querySelector('#neq'), 'equations.csv', ['id', 'latex', 'says', 'source'], eqPreview);
    await eqPreview();
    await csvTable(el.querySelector('#nsym'), 'symbols.csv', ['symbol', 'name', 'unit', 'means']);
  });

/** A CSV of this node as an editable table; a file with no rows is taken away. */
async function csvTable(el, rel, head, after, opts = {}) {
  const text = ctx.folder.has(dir() + rel) ? await ctx.folder.text(dir() + rel) : '';
  const t = text ? parseCsv(text) : { head, rows: [] };
  const rule = fileRule(rel);
  const says = new Map((rule.columns || []).map(c => [c.name, c.says]));
  mountTable(el, {
    head: t.head.length ? t.head : head, rows: t.rows, fixedHead: opts.fixedHead !== false && !opts.freeHead,
    hint: h => says.get(splitUnit(h).name) || says.get(h) || '',
    onChange: async (h, rows) => {
      const live = rows.filter(r => r.some(c => String(c).trim() !== ''));
      if (!live.length && !opts.keepEmpty) ctx.folder.remove(dir() + rel); else await put(rel, toCsv(h, live));
      if (after) await after();
    },
  });
}

// ── 4 · the pseudocode ─────────────────────────────────────────────────────

const KEYS = /\b(let|set|const|if|then|else|end|for|to|return|refuse|and|or|not)\b/;

async function pagePseudocode(host) {
  const c = contract();
  const ins = ctx.file.db.all('SELECT name, unit FROM input WHERE node_uid = ? ORDER BY ord', [ctx.uid]);
  const cur = node().text['pseudocode.txt'] || '';
  host.innerHTML = lead('Pseudocode', 'The algorithm, line by line, exactly as the developer will build it. Required for every computed node: the developer\'s code is generated from these lines, not from the theory.', [
    'It is read here by the same checker the developer uses — every line, every unit — and typeset, never run. The developer\'s code is then tested against your Results.',
    'Inputs it may use: ' + (ins.length ? ins.map(i => '<code>' + esc(i.name) + '</code>' + (i.unit ? ' [' + esc(i.unit) + ']' : '')).join(', ') : 'none') + '. It returns <b>' + esc(c.output || 'the answer') + '</b>' + (c.unit ? ' in ' + esc(c.unit) : '') + '.',
  ]) +
    '<div class="npc"><div><textarea class="gnote npc-box" id="npc" rows="' + Math.max(10, cur.split('\n').length + 3) + '" spellcheck="false" aria-label="the pseudocode">' + esc(cur) + '</textarea>' +
    '<details class="small"><summary>How to write it</summary><pre class="md-code">' + esc(
      '# a comment: say where each line comes from\n' +
      'let r_e = 6378137 [m]          # a named value, with its unit\n' +
      'if r <= R_EARTH then\n' +
      '  refuse "the orbit is inside the Earth"\n' +
      'end\n' +
      'let v = sqrt(MU_EARTH / r)\n' +
      'return v') + '</pre><p>Words: let, set, const, if … then … else … end, for … to … end, refuse "why", return. Functions: sqrt, abs, exp, log, sin, cos, tan, min, max, pow. ' +
    'Constants by their names (MU_EARTH, R_EARTH). Units in [square brackets].</p></details></div>' +
    '<div><p class="gh3">The same lines, as equations</p><div id="npc-eq"></div><div id="npc-lint"></div></div></div>' + findingsFor(['pseudocode.txt']);
  const box = $('#npc');
  let seq = 0;
  const show = async () => {
    const text = box.value, my = ++seq;
    const eqs = pseudocodeEquations(text, c.output || 'answer');
    $('#npc-eq').innerHTML = eqs.length ? eqs.map(e => '<div class="galgo-row"><span class="pc-n">' + e.line + '</span>' + texToMathml(e.tex, true).html + '</div>').join('')
      : '<p class="muted small">Each let and return line appears here as an equation.</p>';
    const lint = lintPseudocode(text, ins.map(i => i.name), eqs);
    // The language's own reading — the one the developer's tools make.
    // A node that publishes several values names each member in its results' answer.<member> columns.
    const members = resultColumns((node().files || {})['results/isolation.csv']).answers
      .filter(a => a.name.startsWith('answer.')).map(a => ({ name: a.name.slice(7), unit: a.unit }));
    const r = await readMethod(text, ins, c.unit, members);
    if (my !== seq) return;
    const found = (r ? [r.error ? ['error', r.error] : null].concat(r.diags.map(d => [d.severity === 'note' ? 'note' : d.severity, (d.line ? 'line ' + d.line + ': ' : '') + d.msg])).filter(Boolean) : [])
      .concat(lint.filter(l => !r || l[0] !== 'error'));
    $('#npc-lint').innerHTML = (r ? '' : '<p class="gsave-dl small">The method checker could not start here (' + esc(checkerProblem()) + '); the reading below is a simpler one.</p>') +
      (found.length ? '<ul class="gfind">' + found.map(l => '<li class="gf-' + l[0] + '">' + esc(l[1]) + '</li>').join('') + '</ul>'
        : '<p class="gsave-ok small">The method checker reads it: every line parses, every unit agrees, and it answers in ' + esc(c.unit || 'a pure number') + '.</p>');
  };
  let t = null;
  box.addEventListener('input', () => { clearTimeout(t); t = setTimeout(async () => { show(); await put('pseudocode.txt', box.value.replace(/\s+$/, '') + '\n'); }, 350); });
  show();
}

/** What can be told about pseudocode without running it. Each finding is `[level, message]`. */
export function lintPseudocode(text, inputs, eqs) {
  const out = [];
  const lines = String(text || '').replace(/\r/g, '').split('\n');
  const code = lines.map(l => l.replace(/#.*$/, ''));
  const words = new Set(code.join(' ').match(/[A-Za-z_][A-Za-z0-9_]*/g) || []);
  if (!code.some(l => l.trim())) return [['error', 'There is no pseudocode yet.']];
  for (const n of inputs) if (!words.has(n)) out.push(['warning', 'the input ' + n + ' is never used']);
  if (!code.some(l => /^\s*return\b/.test(l))) out.push(['error', 'it never returns the answer: end with a return line']);
  let depth = 0;
  code.forEach((l, i) => {
    const s = l.trim();
    if (/^(if|for)\b/.test(s)) depth++;
    if (/^end\b/.test(s)) { depth--; if (depth < 0) { out.push(['error', 'line ' + (i + 1) + ': an end with no if or for to close']); depth = 0; } }
    if (/^if\b/.test(s) && !/\bthen\s*$/.test(s)) out.push(['error', 'line ' + (i + 1) + ': an if ends with then']);
    if (/^refuse\b/.test(s) && !/^refuse\s+".+"\s*$/.test(s)) out.push(['warning', 'line ' + (i + 1) + ': say why, in quotes: refuse "the reason"']);
    if (s && !KEYS.test(s.split(/\s+/)[0]) && !/^(else|end)\b/.test(s)) out.push(['warning', 'line ' + (i + 1) + ': a line starts with let, set, const, if, for, refuse or return']);
    if ((s.match(/\(/g) || []).length !== (s.match(/\)/g) || []).length) out.push(['error', 'line ' + (i + 1) + ': the brackets do not pair']);
  });
  if (depth > 0) out.push(['error', depth + ' if or for not closed with end']);
  for (const e of eqs) for (const p of e.problems || []) out.push(['warning', 'line ' + e.line + ': ' + p]);
  if (!code.some(l => /^\s*refuse\b/.test(l))) out.push(['note', 'no refuse line: say what inputs the node must refuse, if any']);
  return out;
}

// ── 5 · the inputs ─────────────────────────────────────────────────────────

async function pageInputs(host) {
  const db = ctx.file.db;
  const ins = db.all('SELECT * FROM input WHERE node_uid = ? ORDER BY ord', [ctx.uid]);
  const idOf = new Map(db.all('SELECT uid, id FROM node').map(r => [r.uid, r.id]));
  host.innerHTML = lead('Inputs', 'For each input: how it is written, its default — the value a run takes when nobody sets it — and the range your node holds for.', [
    'Its name, where it comes from and its unit are the contract: your lead sets those.',
    'The range is where your node holds. A value outside it is refused, never quietly used.',
  ]) + (ins.length ? '<div class="ri-wrap"><table class="fx gtable"><thead><tr><th>Input</th><th>From</th><th>Unit</th><th>Written as</th><th>Default</th><th>Min</th><th>Max</th><th>What it is</th></tr></thead><tbody>' +
    ins.map(i => '<tr><td><b>' + esc(i.name) + '</b></td><td class="small">' + esc(i.source === 'case' ? 'user' : idOf.get(i.source) || i.source) + '</td><td class="small">' + esc(i.unit || '—') + '</td>' +
      ['symbol', 'dflt', 'min', 'max', 'says'].map(f => '<td><input class="gs-in" data-in="' + esc(i.name) + '" data-f="' + f + '" value="' + esc(i[f] || '') + '" aria-label="' + esc(i.name + ' ' + f) + '"></td>').join('') + '</tr>').join('') +
    '</tbody></table></div><p id="nin-out" class="gout" aria-live="polite"></p>' : '<p class="muted">This node takes no inputs.</p>') + findingsFor(['inputs.csv']);
  host.addEventListener('change', e => {
    const t = e.target;
    if (!t.dataset.in) return;
    const v = t.value.trim(), f = t.dataset.f;
    if (['dflt', 'min', 'max'].includes(f) && v !== '' && !Number.isFinite(Number(v))) { $('#nin-out').textContent = f.replace('dflt', 'default') + ' is a number, or blank'; return; }
    const before = db.one('SELECT ' + f + ' AS v FROM input WHERE node_uid = ? AND name = ?', [ctx.uid, t.dataset.in]).v;
    db.tx(() => {
      db.exec('UPDATE input SET ' + f + ' = ? WHERE node_uid = ? AND name = ?', [v, ctx.uid, t.dataset.in]);
      logChange(db, ctx.me, ctx.uid, 'input ' + t.dataset.in + ' ' + f, before, v);
    });
    ctx.file.dirty = true;
    ctx.stale = true;
    topbar();
    $('#nin-out').textContent = 'Kept. Save when you are done.';
  });
}

// ── 6 · the results ────────────────────────────────────────────────────────

async function pageResults(host) {
  const ins = ctx.file.db.all('SELECT name, unit FROM input WHERE node_uid = ? ORDER BY ord', [ctx.uid]);
  const c = contract();
  const head = ins.map(i => i.name + (i.unit ? ' [' + i.unit + ']' : '')).concat(['answer' + (c.unit ? ' [' + c.unit + ']' : ''), 'tolerance', 'refuses', 'origin']);
  host.innerHTML = lead('Results', 'Your node on its own: the inputs in, the answer out, from <b>your own</b> code, a hand calculation, a spreadsheet or a paper. ' +
    'The developer\'s code must give these same numbers — this is its test.', [
    'At least: the defaults, three ordinary cases, both ends of every input\'s range, and one input it must refuse.',
    'tolerance is relative (1e-9 means agreement to nine digits); refuses is yes when the node must refuse, with the answer left blank; origin is code, hand, spreadsheet or paper.']) +
    '<h2 class="gh">The table</h2><p class="muted small">Columns: ' + head.map(esc).join(', ') + '. Paste straight from your spreadsheet.</p><div id="nres"></div>' +
    '<h2 class="gh">How you made them</h2><p class="muted small">Language and version, the tool, the machine, the date, the command — so the numbers can be made again.</p>' +
    '<textarea class="gnote" id="nhow" rows="6" style="width:100%" aria-label="how the results were made">' + esc(node().text['results/how-run.md'] || '') + '</textarea>' +
    '<h2 class="gh">As the group will see them</h2><div id="nres-pv"></div>' + findingsFor(['results/']);
  const redraw = async () => {
    ctx.model = await loadGroup(ctx.folder);
    const pv = $('#nres-pv');
    pv.innerHTML = '<div class="gtab-body">' + nodePage(ctx, ctx.id, 'results').split('<div class="gtab-body">')[1];
    await mountResults(ctx, pv);
  };
  await csvTable($('#nres'), 'results/isolation.csv', head, redraw);
  await redraw();
  let t = null;
  $('#nhow').addEventListener('input', e => { clearTimeout(t); t = setTimeout(async () => {
    if (e.target.value.trim()) await put('results/how-run.md', e.target.value.replace(/\s+$/, '') + '\n'); else ctx.folder.remove(dir() + 'results/how-run.md');
  }, 400); });
}

// ── 7 · evidence and sources ───────────────────────────────────────────────

async function pageEvidence(host) {
  const groupSources = ctx.model.group.sources;
  host.innerHTML = lead('Evidence & sources', 'Values from <b>outside</b> any code — a paper, a handbook, another tool — that check the physics. Results check the code; evidence checks the physics.', [
    'Each value cites a source by its id: one of the group\'s, or one you add here.',
  ]) + '<h2 class="gh">Evidence</h2><div id="nev"></div>' +
    '<h2 class="gh">Your sources</h2><p class="muted small">Add a paper or a handbook your node rests on. A PDF you may keep goes in Code &amp; files, under sources/.</p><div id="nsrc"></div>' +
    (groupSources.length ? '<details><summary class="small">The group\'s sources (' + groupSources.length + ')</summary><ul class="small">' +
      groupSources.map(s => '<li><code>' + esc(s.id) + '</code> — ' + esc(s.cite || '') + '</li>').join('') + '</ul></details>' : '') +
    findingsFor(['evidence.csv', 'sources.csv']);
  await csvTable($('#nev'), 'evidence.csv', ['inputs', 'value', 'unit', 'source', 'says']);
  await csvTable($('#nsrc'), 'sources.csv', ['id', 'cite', 'file', 'url', 'licence']);
}

// ── 8 · pictures ───────────────────────────────────────────────────────────

const KIND_HELP = {
  line: 'a curve: a CSV with an x column and one or more y columns', scatter: 'points: a CSV with x and y', bar: 'bars: a CSV with x (labels) and y',
  heatmap: 'a map of values: a CSV with x, y and z (the value)', animation: 'a curve that moves: a CSV with x, y and z (the frame)',
  scene3d: 'a 3D view: a CSV with body, x, y, z', flow: 'a diagram: a CSV with from, to (and label)', steps: 'a walk-through of a flow: a CSV with step, caption (and on = the flow)',
  image: 'a picture that is not data: PNG, JPG, SVG', video: 'a short video: MP4 or WebM',
};

async function pagePictures(host) {
  const files = ctx.folder.list(dir()).map(p => p.slice(dir().length)).filter(p => p.startsWith('figures/') || p.startsWith('images/'));
  host.innerHTML = lead('Pictures', 'Every picture is drawn from data — a chart, a flow diagram, a step-by-step walk-through, a heatmap, a 3D view — or is an image or a video. Place one in your text with <code>{{fig id}}</code>.', [
    'A chart is a CSV of numbers and a line in the table below: the application draws it, in the group\'s colours, the same everywhere.',
  ]) +
    '<h2 class="gh">1. Add the data or the image</h2><p class="muted small">A CSV becomes figures/&lt;name&gt;.csv; an image or a video becomes images/&lt;name&gt;.</p>' +
    '<p><label class="ctl gbtn">Add files…<input type="file" id="nfig-add" multiple accept=".csv,.png,.jpg,.jpeg,.svg,.gif,.webp,.mp4,.webm" hidden></label></p>' +
    (files.length ? '<ul class="small">' + files.map(p => '<li><code>' + esc(p) + '</code> <button class="ctl small gs-x" type="button" data-rm="' + esc(p) + '" aria-label="remove ' + esc(p) + '">×</button></li>').join('') + '</ul>' : '') +
    '<h2 class="gh">2. Say how to draw each one</h2><p class="muted small">One row per picture. kind is one of: ' +
    Object.entries(KIND_HELP).map(([k, v]) => '<b>' + k + '</b> (' + esc(v) + ')').join('; ') + '.</p><div id="nfigs"></div>' +
    '<h2 class="gh">3. How they look</h2><div id="nfig-pv"></div>' + findingsFor(['figures.csv', 'figures/', 'images/']);
  const redraw = async () => {
    ctx.model = await loadGroup(ctx.folder);
    const pv = $('#nfig-pv');
    const rows = node().files['figures.csv'] ? records(node().files['figures.csv']) : [];
    pv.innerHTML = rows.length ? rows.map(f => '<h3 class="gh3">' + esc(f.id) + ' <span class="muted">' + esc(f.kind) + '</span></h3><div class="gfig" data-fig="' + esc(f.id) + '" data-node="' + esc(ctx.id) + '"></div>').join('')
      : '<p class="muted">No pictures yet.</p>';
    await mountFigures(ctx, pv);
  };
  await csvTable($('#nfigs'), 'figures.csv', ['id', 'kind', 'file', 'title', 'x', 'y', 'z', 'on'], redraw);
  await redraw();
  $('#nfig-add').addEventListener('change', e => trying(async () => { await addFiles([...e.target.files], f => (/\.csv$/i.test(f.name) ? 'figures/' : 'images/') + safe(f.name)); await route(); }));
  host.querySelectorAll('[data-rm]').forEach(b => b.addEventListener('click', async () => { ctx.folder.remove(dir() + b.dataset.rm); await route(); }));
}

const safe = name => name.replace(/[^A-Za-z0-9_.-]+/g, '_');

/** Put files into this node, refusing what the pattern refuses. */
async function addFiles(files, where) {
  const lim = SPEC.limits || {};
  const refused = [];
  for (const f of files) {
    const ext = f.name.split('.').pop().toLowerCase();
    if ((lim.refused_types || []).includes(ext)) { refused.push(f.name + ': ' + (lim.refused_says || 'not accepted')); continue; }
    if (lim.file_bytes && f.size > lim.file_bytes) { refused.push(f.name + ': ' + (lim.file_bytes_says || 'too large')); continue; }
    await ctx.folder.write(dir() + where(f), new Uint8Array(await f.arrayBuffer()), f.type || 'application/octet-stream');
  }
  if (refused.length) throw new Error('Not added — ' + refused.join('; '));
}

// ── 9 · code and other files ───────────────────────────────────────────────

async function pageCode(host) {
  const files = ctx.folder.list(dir()).map(p => p.slice(dir().length)).filter(p => p.startsWith('code/') || p.startsWith('sources/'));
  host.innerHTML = lead('Code & files', 'Your own code, if you have it — the code that made your Results — kept with the node for the record, and any source PDF you may keep. Nothing runs it.', [
    'The developer reads it to understand your method; the pseudocode is what they build from.',
  ]) + '<p><label class="ctl gbtn">Add code…<input type="file" id="ncode-add" multiple hidden></label> ' +
    '<label class="ctl gbtn">Add a source PDF…<input type="file" id="nsrc-add" accept=".pdf" multiple hidden></label></p>' +
    (files.length ? '<ul class="small">' + files.map(p => '<li><code>' + esc(p) + '</code> <button class="ctl small gs-x" type="button" data-rm="' + esc(p) + '" aria-label="remove ' + esc(p) + '">×</button></li>').join('') + '</ul>'
      : '<p class="muted">No files yet.</p>') + '<div id="ncode-pv"></div>';
  $('#ncode-add').addEventListener('change', e => trying(async () => { await addFiles([...e.target.files], f => 'code/' + safe(f.name)); await route(); }));
  $('#nsrc-add').addEventListener('change', e => trying(async () => { await addFiles([...e.target.files], f => 'sources/' + safe(f.name)); await route(); }));
  host.querySelectorAll('[data-rm]').forEach(b => b.addEventListener('click', async () => { ctx.folder.remove(dir() + b.dataset.rm); await route(); }));
  if (files.some(p => p.startsWith('code/'))) {
    const pv = $('#ncode-pv');
    pv.innerHTML = '<div class="gtab-body">' + nodePage(ctx, ctx.id, 'code').split('<div class="gtab-body">')[1];
    wireCode(ctx, pv);
  }
}

// ── 10 · the preview ───────────────────────────────────────────────────────

async function preview(main, tab) {
  const t = NODE_TABS.some(([k]) => k === tab) ? tab : 'explain';
  main.innerHTML = '<p class="nbanner">Preview — your node as the group and the application will show it. <a href="#/step/explain">Back to filling it</a></p>' + nodePage(ctx, ctx.id, t);
  wireUp(ctx, main, () => {});
  wireAlgorithm(main);
  wireCode(ctx, main);
  await mountFigures(ctx, main);
  await mountResults(ctx, main);
  window.scrollTo(0, 0);
}

async function pagePreview() { location.hash = '#/node/' + ctx.id + '/explain'; }

// ── 11 · check and sign ────────────────────────────────────────────────────

async function pageSign(host) {
  await rebuild();
  const f = ctx.findings;
  const by = l => f.filter(x => x.level === l);
  const { reviews, fingerprintOf } = await reviewState(ctx.folder, ctx.model);
  const fp = await fingerprintOf(ctx.id);
  const mine = reviews.filter(r => r.scope === ctx.id);
  const last = mine.slice(-1)[0];
  const c = contract();
  const errors = by('error').length;
  const decl = (node().files['declaration.csv'] ? records(node().files['declaration.csv'])[0] : null) || {};
  host.innerHTML = lead('Check & sign', errors ? '<b>' + errors + ' thing(s) to fix</b> before your node is ready. Each one says where.' : 'Your node passes its checks. Sign it, save it, and put the file back on the drive.', [
    'Signing says: this is my node, as it is now. Change anything afterwards and the signature goes stale — sign again.',
    'Your lead assembles every node\'s file into the group\'s release, and the owner signs the whole.',
  ]) +
    '<h2 class="gh">Errors (' + errors + ')</h2>' + findingsList(by('error'), 'No errors.') +
    '<h2 class="gh">Warnings (' + by('warning').length + ')</h2>' + findingsList(by('warning'), 'No warnings.') +
    '<h2 class="gh">Notes (' + by('note').length + ')</h2>' + findingsList(by('note'), 'No notes.') +
    '<h2 class="gh">Sign</h2><p>Fingerprint now <code>' + esc(fp.slice(0, 16)) + '…</code> · ' + (!last ? 'not signed yet' : last.current
      ? (last.verdict === 'ok' ? '<b class="gsig-ok">signed</b> by ' + esc(last.name) + ' on ' + esc(last.date) : 'changes asked by ' + esc(last.name))
      : '<span class="gsig-stale">signed earlier, by ' + esc(last.name) + ', for different content — sign again</span>') + '</p>' +
    '<fieldset class="nai"><legend>Did an assistant — an AI — help with this node?</legend>' +
    [['none', 'No'], ['wording', 'With the words only'], ['relation', 'With the pseudocode, the equations, the results or the evidence'],
      ['transcribed', 'It copied into pseudocode a relation a person had already written, and a person checked the copy against it']].map(([v, t]) =>
      '<label><input type="radio" name="nai" value="' + v + '"> ' + esc(t) + '</label>').join('') +
    // Asked afresh at every signing: the node may have changed since, and so
    // may the answer. The last one is shown, never chosen for the author.
    (decl.ai ? '<p class="muted small">Last time this node was signed, the answer was <b>' + esc(decl.ai) + '</b>.</p>' : '') +
    '<div id="nai-tr" hidden>' +
    '<p><label>Copied from <input id="nai-src" class="gs-in" value="' + esc(decl.source || '') + '" placeholder="a file and line, a paper and equation, a node" aria-label="what the relation was copied from"></label></p>' +
    '<p><label>Checked against it by <input id="nai-chk" class="gs-in" style="max-width:16rem" value="' + esc(decl.checked_by || '') + '" aria-label="the person who checked the copy"></label></p></div>' +
    '<p class="muted small">Said plainly, because an assistant may never supply mathematics: the developer takes a method or results an assistant supplied ' +
    'only once a person has derived them. A copy of a person\'s own relation is taken when it names what it was copied from and the person who read it against that. ' +
    'Kept in your node file as declaration.csv.</p></fieldset>' +
    '<p><label>I am <input id="nsig-me" class="gs-in" style="max-width:16rem" value="' + esc(ctx.me || c.author) + '" aria-label="your name"></label> ' +
    '<button class="ctl gbtn" type="button" id="nsig-go"' + (errors ? ' disabled title="fix the errors first"' : '') + '>Sign my node</button></p>' +
    '<p id="nsig-out" class="gout" aria-live="polite"></p>';
  host.querySelectorAll('input[name=nai]').forEach(r => r.addEventListener('change', () => {
    $('#nai-tr').hidden = r.value !== 'transcribed' || !r.checked;
  }));
  $('#nsig-go').addEventListener('click', () => trying(async () => {
    const name = $('#nsig-me').value.trim();
    if (!name) throw new Error('say who you are');
    const authors = String(c.author || '').split(/,\s*/).filter(Boolean);
    if (authors.length && !authors.includes(name)) throw new Error(name + ' is not this node\'s author (' + authors.join(', ') + '). Only its author signs it.');
    const ai = (host.querySelector('input[name=nai]:checked') || {}).value;
    if (!ai) throw new Error('say whether an assistant helped with this node');
    const src = ai === 'transcribed' ? $('#nai-src').value.trim() : '';
    const chk = ai === 'transcribed' ? $('#nai-chk').value.trim() : '';
    if (ai === 'transcribed' && !src) throw new Error('say what the relation was copied from');
    if (ai === 'transcribed' && !chk) throw new Error('say who checked the copy against it');
    ctx.me = name;
    try { localStorage.setItem(ME, name); } catch { /* */ }
    // The declaration is part of what is signed, so it is written first.
    await put('declaration.csv', toCsv(['author', 'ai', 'date', 'source', 'checked_by'], [[name, ai, new Date().toISOString().slice(0, 10), src, chk]]));
    await rebuild();
    await sign(ctx.folder, ctx.model, { name, scope: ctx.id, verdict: 'ok', note: '' });
    await rebuild();
    await route();
    flash('Signed. Now Save, and put the file back on the drive.');
  }));
}

// ── the equation helper ────────────────────────────────────────────────────

function helperBox() {
  return '<div class="nhelp"><p><input class="gnote gh-in" id="gh-in" value="v = sqrt(mu / r)" aria-label="an equation, typed plainly"></p><div id="gh-out" class="gh-out"></div>' +
    '<p><label>LaTeX <input id="gh-tex" class="gnote" aria-label="LaTeX"></label> <button class="ctl small" id="gh-copy" type="button">copy</button></p><p class="muted small" id="gh-prob"></p></div>';
}

function wireHelper(root) {
  const q = s => root.querySelector(s);
  const inp = q('#gh-in'), tex = q('#gh-tex'), out = q('#gh-out'), prob = q('#gh-prob');
  const show = t => { const r = texToMathml(t, true); out.innerHTML = r.html; return r.problems; };
  const fromShort = () => { const r = shortToTex(inp.value); tex.value = r.tex; prob.textContent = r.problems.concat(show(r.tex)).join('; '); };
  inp.addEventListener('input', fromShort);
  tex.addEventListener('input', () => { prob.textContent = show(tex.value).join('; '); });
  q('#gh-copy').addEventListener('click', () => { tex.select(); try { navigator.clipboard.writeText(tex.value); } catch { document.execCommand('copy'); } });
  fromShort();
}

async function pageHelper(host) {
  host.innerHTML = lead('Equation helper', 'Type an equation the way you would in an email; copy the LaTeX into your equations.', []) + helperBox();
  wireHelper(host);
}

const PAGES = {
  contract: pageContract, explain: pageExplain, theory: pageTheory, pseudocode: pagePseudocode, inputs: pageInputs, results: pageResults,
  evidence: pageEvidence, pictures: pagePictures, code: pageCode, preview: pagePreview, sign: pageSign, helper: pageHelper,
};

// ── start ──────────────────────────────────────────────────────────────────

initDepth();
document.addEventListener('dragover', e => e.preventDefault());
document.addEventListener('drop', e => {
  e.preventDefault();
  const f = e.dataTransfer.files[0];
  if (f) trying(async () => openFile(await fromFile(f)));
});
$('#gsave').addEventListener('click', () => trying(save));
window.addEventListener('beforeunload', e => { if (ctx.file && ctx.file.dirty) { e.preventDefault(); e.returnValue = ''; } });
window.addEventListener('hashchange', route);
route();
