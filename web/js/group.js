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

  It runs nothing and compiles nothing. Computing is the developer's engine's
  work, done after the seal and tested against the results in this folder.
*/
'use strict';

import { $, esc } from './dom.js';
import { initDepth } from './depth.js';
import { fromFileList, fromDrop, fromPicker, fromHandle, canPick } from './gfolder.js';
import { loadGroup } from './gmodel.js';
import { checkGroup } from './gcheck.js';
import { groupPage, nodePage, mountFigures, mountResults, wireUp, wireAlgorithm, wireCode, findingsList, NODE_TABS } from './gview.js';
import { reviewState, maySign, sign, sealBlockers, seal, raiseIssue } from './gseal.js';
import { texToMathml, shortToTex } from './texmath.js';

const SPEC = window.VLEO_GROUP_SPEC || { file: [], text: [], embed: [] };
const ctx = { folder: null, model: null, findings: [], spec: SPEC };
const ME = 'vleo.group.me';

// ── opening a folder ───────────────────────────────────────────────────────

async function open(folder) {
  if (!folder) return;
  $('#gmain').innerHTML = '<p class="muted">Reading ' + esc(folder.name) + '…</p>';
  ctx.folder = folder;
  ctx.model = await loadGroup(folder);
  ctx.findings = await checkGroup(ctx.model, SPEC);
  document.title = (ctx.model.meta.name || folder.name) + ' · VLEO group';
  nav();
  if (!location.hash || location.hash === '#/open') location.hash = '#/';
  else route();
}

function welcome() {
  const pick = canPick();
  $('#gmain').innerHTML =
    '<section class="answer-first view-af"><p class="af-k">Answer first</p><p class="af-a">Open your group\'s folder and this page shows ' +
    'everything in it the way the VLEO application will show it — and checks it, records your sign-off and seals it for the developer.</p>' +
    '<ul class="af-points"><li>Nothing leaves this computer. The page has no network and runs nothing.</li>' +
    '<li>A folder synced from your Google shared drive is an ordinary folder here.</li></ul></section>' +
    '<div class="gopen"><div class="gdrop" id="gdrop"><p><b>Drop your group folder here</b></p><p class="muted">or</p>' +
    '<label class="ctl gbtn">Choose the folder…<input type="file" id="gpickdir" webkitdirectory directory multiple hidden></label>' +
    (pick ? ' <button class="ctl gbtn" type="button" id="gpickrw">Open it for saving…</button>' +
      '<p class="muted small">"Open it for saving" lets this page write your sign-off and the sealed package straight into the folder. ' +
      'Otherwise they are downloaded, and you put them into the folder yourself.</p>' : '') +
    '</div></div>' +
    '<p class="muted">No folder yet? <a href="#/pattern">See the pattern a group folder follows</a>, or try the ' +
    '<a href="#/helper">equation helper</a>.</p>';
  $('#gpickdir').addEventListener('change', e => open(fromFileList(e.target.files)));
  if (pick) $('#gpickrw').addEventListener('click', async () => { try { open(await fromPicker()); } catch { /* cancelled */ } });
}

function dropAnywhere() {
  document.addEventListener('dragover', e => { e.preventDefault(); document.body.classList.add('gdragging'); });
  document.addEventListener('dragleave', e => { if (!e.relatedTarget) document.body.classList.remove('gdragging'); });
  document.addEventListener('drop', async e => {
    e.preventDefault();
    document.body.classList.remove('gdragging');
    const f = await fromDrop(e.dataTransfer);
    if (f) open(f);
  });
}

async function reload() {
  if (ctx.folder && ctx.folder.handle) await open(await fromHandle(ctx.folder.handle));
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
    '<a class="gnv" href="#/checks" data-r="checks">Checks <span class="gcount' + (count('error') ? ' bad' : '') + '">' + count('error') + ' · ' + count('warning') + '</span></a>' +
    '<a class="gnv" href="#/sign" data-r="sign">Sign &amp; seal</a>' +
    '<p class="gnv-h">Nodes, in flow order</p>' +
    m.order.map(id => '<a class="gnv gnv-node" href="#/node/' + esc(id) + '" data-r="node/' + esc(id) + '"><span class="gdot ' + nodeState(m.nodes.get(id)) + '"></span>' + esc(id) + '</a>').join('') +
    '<p class="gnv-h">Tools</p><a class="gnv" href="#/helper" data-r="helper">Equation helper</a>' +
    '<a class="gnv" href="#/pattern" data-r="pattern">The folder pattern</a>' +
    '<a class="gnv" href="#/issue" data-r="issue">Raise an issue</a>' +
    '<button class="ctl small gnv-reload" type="button" id="greload">Read the folder again</button>';
  $('#greload').addEventListener('click', reload);
}

function markNav(r) {
  document.querySelectorAll('.gnv').forEach(a => {
    const d = a.dataset.r;
    a.classList.toggle('sel', d === r || (d && d.startsWith('node/') && r.startsWith(d + '/')) || (d && r === d));
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
  if (r === 'sign') { main.innerHTML = '<p class="muted">Working out what is signed…</p>'; main.innerHTML = await signPage(); wireSign(); return; }
  if (r === 'issue') { main.innerHTML = issuePage(); wireIssue(); return; }
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

let me = '';
try { me = localStorage.getItem(ME) || ''; } catch { /* no storage */ }

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
    (ctx.folder.writable ? '<p class="gsave-ok">This folder is open for saving: sign-offs and the package are written straight into it.</p>'
      : '<p class="gsave-dl">This folder was opened read-only, so sign-offs and the package are <b>downloaded</b>. Put each downloaded file into the folder where its name says ' +
        '(<code>packages__' + esc(m.meta.id || 'group') + '-' + esc(m.meta.version || '') + '.zip</code> goes in <code>packages/</code>).</p>') +
    '<p><label>I am <select id="gme"><option value="">— choose your name —</option>' + m.group.members.map(x =>
      '<option' + (x.name === me ? ' selected' : '') + '>' + esc(x.name) + '</option>').join('') + '</select></label> ' +
    '<input id="gnote" class="gnote" placeholder="what you want changed (when asking for changes)"></p>' +
    '<div class="ri-wrap"><table class="fx gtable"><thead><tr><th>What</th><th>Fingerprint now</th><th>Signed</th><th></th></tr></thead><tbody>' + rows.join('') + '</tbody></table></div>' +
    '<h2 class="gh">Seal for the developer</h2>' +
    (blockers.length ? '<p>Not yet. Still standing in the way:</p><ul class="gfind">' + blockers.map(b => '<li class="gf-error">' + esc(b) + '</li>').join('') + '</ul>'
      : '<p>Everything is checked and signed. Sealing writes one file, <code>packages/' + esc(m.meta.id || 'group') + '-' + esc(m.meta.version || '') + '.zip</code>, holding every file, a manifest of their fingerprints and who signed. That file is what the developer builds from.</p>' +
        '<button class="ctl gbtn" id="gseal" type="button">Seal version ' + esc(m.meta.version || '') + '</button>') +
    '<p id="gseal-out" class="gout" aria-live="polite"></p>';
}

function wireSign() {
  const sel = $('#gme');
  if (sel) sel.addEventListener('change', () => { me = sel.value; try { localStorage.setItem(ME, me); } catch { /* */ } route(); });
  document.querySelectorAll('.gsig').forEach(b => b.addEventListener('click', async () => {
    const how = await sign(ctx.folder, ctx.model, { name: me, scope: b.dataset.scope, verdict: b.dataset.v, note: $('#gnote').value });
    await refresh();
    $('#gseal-out').textContent = how === 'saved' ? 'Signed, and saved into reviews.csv.' : 'Signed. reviews.csv was downloaded — put it into the folder, replacing the old one.';
  }));
  const s = $('#gseal');
  if (s) s.addEventListener('click', async () => {
    const r = await seal(ctx.folder, ctx.model, me || 'unknown');
    $('#gseal-out').innerHTML = (r.how === 'saved' ? 'Sealed: ' + esc(r.name) + ' is in the folder.' : 'Sealed: the package was downloaded — put it in packages/.') +
      ' Fingerprint <code>' + r.fingerprint.slice(0, 16) + '…</code>. Tell the developer it is ready.';
  });
}

async function refresh() {
  ctx.model = await loadGroup(ctx.folder);
  ctx.findings = await checkGroup(ctx.model, SPEC);
  nav();
  await route();
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
window.addEventListener('hashchange', route);
route();
