/*
  The Node workspace of the application (docs/OPERATING_1_0.md, section 4;
  docs/briefs/F3.md, F3-g1): one node, its file in your copy, and the day's
  work on it.

  * LEFT, ITS SIX PARTS. Each done, missing or failing as the gate says, and
    *run my code again* (nodeparts.js, B's drawing, unchanged).
  * CENTRE, ITS PAGE AS TODAY'S DESIGN HOLDS IT. Today's tool page's node view,
    drawn by the same function (node.js drawNode), so the two cannot drift
    while both exist. It is the design the engine has open, and says so: the
    revision in your copy joins it only once its group seals a release (F5).
  * RIGHT, ITS FILE IN YOUR COPY. Its revision and writer, every signature
    and whether the node is signed as it is now, every request, and what its
    checks find, as the program reads them (`GET /v1/node/<id>/file`). Then
    the steps that exist: write it (its question, the explanation's three
    texts, its method and its cases, F3-g2a), sign the revision, ask for a
    contract change or a breakdown, save a filled lesson form. One next step
    is named, by the program.
  * WRITING IT, CHECKED AS IT IS TYPED. What is typed is kept here
    (`W.editing`): the revision it began on, and only the parts the person
    touched. It is sent, a moment after typing stops, as a dry run
    (`POST /v1/node/<id>/edit`, `dry_run`): nothing is written, and the
    program says what a save would be refused for, what signing would find,
    and the method on each case. Only that answer is drawn while typing, never
    the form under the caret; an answer to older typing is dropped. Saving
    writes the next revision of the one it began on, so a file changed since
    is a second writer, never written over. Opened again, what was typed is
    brought onto the file as it now is: the parts not touched take its new
    text, and a part both changed is shown beside the file's.

  Every step is the program's: nothing here judges a node, signs or writes a
  file. A file that changed in the copy since it was opened is the second
  writer: the step is refused with nothing written, and the file is opened
  again on a click, never behind the person's back.

  The steps that write carry the launch's word, as every write does: the shell
  hands this module the one function that posts with it.
*/
'use strict';

import { $, $$, esc, answerFirst, plural } from './dom.js';
import { S as E, load, loadSaved, theCase, onCaseChange } from './state.js';
import { loadOverrides, onOverrideChange } from './inputs.js';
import { caseSaid, drawWhatIf } from './casebar.js';
import { showContract, showMock } from './banners.js';
import { drawNode } from './node.js';
import { mountParts } from './nodeparts.js';

const W = {
  id: '', file: null, notice: null, stale: null, loaded: null, unread: '', host: null, post: null, nodes: [], asking: null,
  editing: null, // what is typed, and the revision it began on, until the program takes it
  rebasing: false, // the file opened again on a click, to bring the typing onto it
  behind: null, // the file as it is now, when the typing began on an earlier one
  seq: 0, timer: 0, // the typing checked last, and the check waiting to be asked
  view: 'today', // the centre: today's page, or the copy's revision over it
  listening: false, // the case and the what-if heard, once
  centreSeq: 0, // the centre drawn last: an older drawing's answer is dropped
};

/** How long after typing stops a dry run is asked. */
const SETTLE_MS = 600;

/** The fields a person writes, as the program names them, with what each is
 *  in the file now (`GET …/file`'s `content`). */
const FIELDS = [
  ['question', c => c.question],
  ['explain_simply', c => c.explain.simply],
  ['explain_breaks', c => c.explain.breaks],
  ['explain_wrong', c => c.explain.wrong],
  ['method_text', c => c.method_text],
  ['case_csv', c => c.case_csv],
];
const DECLARED = ['author', 'ai', 'source', 'checked_by'];
const WHY = [
  ['believed', 'what was believed'], ['tested', 'how it was tested'], ['learned', 'what was learned'],
  ['changed', 'what changed'], ['rests_on', 'what it rests on'], ['breaks_if', 'what would break it'],
  ['risks', 'risk moves (optional)'], ['cost', 'what it cost (optional)'],
];

/** The index and the case, read once, as Explore reads them. */
function ready() {
  if (!W.loaded) {
    W.loaded = (async () => {
      if (!E.index) await load();
      await loadSaved();
      loadOverrides();
    })();
    W.loaded.catch(() => { W.loaded = null; });
  }
  return W.loaded;
}

/** Today's design read, or why it could not be: never taken for a design
 *  that does not hold the node. */
async function readDesign() {
  try { await ready(); W.unread = ''; } catch (e) { W.unread = String(e); }
  // A page in doubt says so before anything else, as today's tool page does.
  showContract();
  showMock();
}

async function get(path) {
  try { return await (await fetch(path)).json(); } catch (e) { return { ok: false, message: 'the program did not answer: ' + e }; }
}

/** Open the workspace at node `id`, the next time it is drawn. */
export function openAt(id) {
  if (id && id !== W.id) {
    clearTimeout(W.timer);
    W.id = id; W.file = null; W.notice = null; W.stale = null; W.asking = null;
    W.editing = null; W.behind = null; W.rebasing = false; W.seq++; W.view = 'today';
  }
}

/** Draw the workspace into `host`. `post(path, fields)` writes with the launch's word;
 *  `nodes` are the person's own, as My work lists them. */
export async function mountNode(host, post, nodes) {
  W.host = host; W.post = post; W.nodes = nodes || [];
  // What was said before the workspace was left is not said again.
  W.notice = null; W.stale = null;
  if (!W.id && W.nodes.length) W.id = W.nodes[0].id;
  if (!host.dataset.wired) {
    host.dataset.wired = '1';
    host.addEventListener('click', click);
    host.addEventListener('change', changed);
    host.addEventListener('input', typed);
  }
  if (!W.listening) {
    W.listening = true;
    // A what-if set from the node's own input control, or a case saved, and
    // every number on the page was for the inputs before: the case line, the
    // what-if bar and the node's page are drawn again, as today's tool page
    // opens its node again.
    onOverrideChange(() => { if (W.host && W.host.isConnected) whatIf(); });
    onCaseChange(() => { if (W.host && W.host.isConnected && W.id) { whatIf(); centre(W.id, W.file); } });
  }
  await draw();
}

/** Draw the node: its file read again from the copy, unless `keep`, when a
 *  second writer was found and the file is opened again only on a click. */
async function draw(keep) {
  const host = W.host;
  host.innerHTML = picker() + (W.id ? '<p class="sh-loading">opening ' + esc(W.id) + '…</p>' : '');
  if (!W.id) {
    host.insertAdjacentHTML('beforeend', answerFirst(W.nodes.length
      ? 'Pick one of your nodes, or name any node to read it.'
      : 'No node is yours to write here: name any node to read it.', [], 'how-to'));
    return;
  }
  const id = W.id;
  await readDesign();
  const file = keep && W.file ? W.file : await get('/v1/node/' + encodeURIComponent(id) + '/file');
  if (id !== W.id) return;
  W.file = file;
  if (!keep) W.stale = null;
  if (file && file.ok !== false && file.content && !keep) {
    const e = W.editing;
    W.behind = null;
    if (!e || (e.opened !== file.opened && !e.touched.size)) W.editing = fromFile(file);
    else if (e.opened !== file.opened && W.rebasing) rebaseOn(e, file);
    else if (e.opened !== file.opened) W.behind = { writer: file.writer, revision: file.revision };
  }
  W.rebasing = false;
  const row = E.byId && E.byId.get(id);
  host.innerHTML = picker() + head(id, file, row) + (row ? theInputs() : '') +
    '<div class="nw-grid">' +
      '<aside class="nw-left"><h3 class="seg-h">its parts</h3><div class="nw-parts"></div></aside>' +
      '<section class="nw-centre"><h3 class="seg-h">its page' + (views(file) ? '' : ', as today\'s design holds it') + '</h3>' +
        (row ? views(file) + '<div class="nw-yours-said"></div><div class="nw-page"></div><div class="nw-run"></div>'
          : W.unread
            ? '<div class="blocked"><b>today\'s design could not be read</b><div>' + esc(W.unread) +
              ' — so whether it holds <code>' + esc(id) + '</code> is not known.</div></div>'
            : '<p class="empty">Today\'s design holds no node <code>' + esc(id) + '</code>, so no page is drawn: ' +
              'a node joins it when its group seals a release that holds it.</p>') +
      '</section>' +
      '<aside class="nw-right">' + yourFile(id, file) + '</aside>' +
    '</div>';
  if (row) {
    whatIf();
    mountParts($('.nw-parts', host), id);
    await centre(id, file);
  } else {
    $('.nw-parts', host).innerHTML = '<p class="empty">' + (W.unread
      ? 'No parts: today\'s design could not be read.' : 'No parts: today\'s design does not hold it.') + '</p>';
  }
}

/** The centre's two pages: today's, and the copy's revision over it, when the
 *  copy holds the node's file. */
function views(file) {
  if (!file || file.ok === false) return '';
  const b = (v, l) => '<button class="ctl nw-view' + (W.view === v ? ' sel' : '') + '" data-view="' + v + '">' + l + '</button>';
  return '<div class="tabrow sub nw-views">' + b('today', 'as today\'s design holds it') +
    b('yours', 'as revision ' + esc(file.revision) + ' in your copy would make it') + '</div>';
}

/** Draw the centre: today's page, or the copy's revision over today's design,
 *  as the program draws it (`GET /v1/node/<id>/live`). Nothing of today's
 *  design is written or run for it. */
async function centre(id, file) {
  const host = W.host;
  const page = $('.nw-page', host), run = $('.nw-run', host), at = $('.nw-yours-said', host);
  if (!page) return;
  // Only the drawing asked last draws: one asked before it, answering late,
  // is dropped, today's page among them.
  const seq = ++W.centreSeq;
  const current = () => seq === W.centreSeq && id === W.id;
  if (W.view !== 'yours' || !file || file.ok === false) {
    at.innerHTML = '';
    return drawNode(page, run, id, { parts: false, current });
  }
  at.innerHTML = '<p class="muted">drawing revision ' + esc(file.revision) + ' over today\'s design…</p>';
  page.innerHTML = ''; run.innerHTML = '';
  const l = await get('/v1/node/' + encodeURIComponent(id) + '/live');
  if (!current()) return;
  // The page named on the button is the revision that was opened: a file
  // changed since is said, never drawn under the old revision's name.
  if (l && l.ok !== false && l.opened !== file.opened) {
    at.innerHTML = '<div class="blocked"><b>the file changed since you opened it</b><div>It is now revision ' +
      esc(l.revision) + ', written by ' + esc(l.writer || 'someone') + '; this page was opened on revision ' + esc(file.revision) +
      '. <button class="ctl" id="nw-live-reopen">Open it again</button></div></div>';
    return;
  }
  at.innerHTML = yoursSaid(l);
  if (!l || l.ok === false) return;
  // Drawn over today's design: the program's sheet alone. Not drawn, or
  // changing nothing: today's page, whole, as today's.
  if (l.drawn && !l.changes_nothing && typeof l.html === 'string') {
    await drawNode(page, run, id, { parts: false, fragment: l.html, current });
  } else {
    await drawNode(page, run, id, { parts: false, current });
  }
}

/** What the program says of the copy's revision over today's design. */
function yoursSaid(l) {
  if (!l || l.ok === false) {
    return '<div class="blocked"><b>your revision is not drawn</b><div>' +
      esc((l && l.message) || 'the program did not answer') + '</div></div>';
  }
  const first = t => { const line = String(t || '').split('\n')[0]; return line.length > 90 ? line.slice(0, 90) + '…' : line; };
  const list = (title, items, f) => items && items.length
    ? '<h4>' + title + '</h4><ul>' + items.map(f).join('') + '</ul>' : '';
  const drawn = l.drawn && !l.changes_nothing;
  return '<div class="nw-yours' + (drawn ? ' is-drawn' : '') + '">' +
    '<p><b>' + esc(l.said) + '</b></p>' +
    list('what its checks find', l.checks, c => '<li><code>' + esc(c.place) + '</code>: ' + esc(c.what) + '</li>') +
    list('what it changes', l.changes, c => '<li><code>' + esc(c.what) + '</code>' +
      (c.to ? ' — ' + esc(first(c.to)) : '') + '</li>') +
    list('what the plan withholds, and why', l.withheld, w => '<li><code>' + esc(w.what) + '</code>: ' + esc(w.why) + '</li>') +
    list('what a release does not bring into today\'s design', l.not_taken, n =>
      '<li>' + esc(n.what) + ': yours ' + (n.yours ? 'is on the right' : 'is empty') + '; the page keeps «' + esc(first(n.page)) + '»</li>') +
    (l.record ? '<p class="muted">' + esc(l.record) + '</p>' : '') +
    (drawn
      ? '<p class="muted">Only the sheet is drawn: what it is made of, what it is joined to, its figures, its lesson ' +
        'and its run are today\'s design\'s, on the other page.</p>'
      : '') + '</div>';
}

/** Which inputs every number on the page is for, and the what-if bar when a
 *  what-if is set (`casebar.js`), as today's tool page carries them. */
function theInputs() {
  const c = theCase();
  return '<div class="tabrow sub nw-case"><span class="lbl">case</span> <b id="case-name">' +
    esc(c ? c.label : 'no case') + '</b> <span class="hint-inline" id="case-state"></span> ' +
    '<a class="ctl" href="#explore/inputs" title="every input of the case, in Explore">inputs…</a></div>' +
    '<div id="ovrbar" hidden></div>';
}

/** The case line and the what-if bar, drawn again: put back, and a what-if
 *  made the case, draw the node's page again. */
function whatIf() {
  const at = $('#case-state', W.host);
  if (at) at.textContent = caseSaid();
  drawWhatIf($('#ovrbar', W.host), () => { whatIf(); centre(W.id, W.file); });
}

function picker() {
  return '<div class="nw-pick"><span class="nw-pick-k">your nodes</span> ' +
    (W.nodes.length ? W.nodes.map(n =>
      '<button class="ctl nw-mine' + (n.id === W.id ? ' sel' : '') + '" data-open="' + esc(n.id) + '" title="' +
      esc(n.group + ' · revision ' + n.revision + (n.deputy_for ? ' · as deputy for ' + n.deputy_for : '')) + '">' +
      esc(n.id) + (n.signed ? ' ✓' : '') + '</button>').join(' ')
      : '<span class="muted">none assigned to you</span>') +
    ' <label class="nw-any">any node <input id="nw-id" size="24" placeholder="its id" value=""></label>' +
    ' <button class="ctl" id="nw-go">Open</button></div>';
}

/** The answer first: which node, which revision, and the one next step. */
function head(id, file, row) {
  const n = W.notice;
  const said = n ? '<p class="' + (n.ok ? 'sh-said' : 'sh-refused') + ' nw-notice">' + esc(n.text) + '</p>' : '';
  if (!file || file.ok === false) {
    return said + answerFirst('<b>' + esc(id) + '</b>' + (row ? ' — ' + esc(row.label) : '') + '. ' +
      esc((file && file.message) || 'Its file in your copy cannot be read.'), [], 'reference');
  }
  const stale = W.stale
    ? '<div class="blocked nw-stale"><b>what is shown is revision ' + esc(file.revision) + ', as you opened it</b><div>' +
      esc(W.stale.writer) + ' saved revision ' + esc(W.stale.revision) + ' in your copy since. ' +
      '<button class="ctl" id="nw-reopen">Open it again</button></div></div>'
    : '';
  return said + stale + answerFirst('<b>' + esc(id) + '</b>' + (row ? ' — ' + esc(row.label) : '') +
    ': revision ' + esc(file.revision) + ' in your copy, written by ' + esc(file.writer || 'nobody yet') + ', ' +
    (file.signed && file.signed.ok ? 'signed as it is now.' : 'not signed as it is now.') +
    ' <span class="nw-next">Next: ' + esc(file.next || '') + '</span>', [], 'reference');
}

/** The node's file in the copy, and what can be done with it. */
function yourFile(id, f) {
  if (!f || f.ok === false) {
    return '<h3 class="seg-h">its file in your copy</h3><div class="blocked"><b>not opened</b><div>' +
      esc((f && f.message) || '') + '</div></div>';
  }
  const findings = f.findings || [];
  const errors = findings.filter(x => x.level === 'error');
  const sigs = f.signatures || [];
  const reqs = f.requests || [];
  return '<h3 class="seg-h">its file in your copy</h3>' +
    '<dl class="nw-kv"><dt>file</dt><dd><code>' + esc(f.path) + '</code></dd>' +
    '<dt>revision</dt><dd>' + esc(f.revision) + ', written by ' + esc(f.writer || '—') + '</dd>' +
    '<dt>its writer</dt><dd>' + esc(f.assigned || 'nobody is assigned it') +
      (f.yours ? ' — <b>yours to write' + (f.deputy_for ? ', as deputy for ' + esc(f.deputy_for) : '') + '</b>' : '') + '</dd>' +
    '<dt>signed</dt><dd class="' + (f.signed && f.signed.ok ? 'nw-ok' : 'nw-not') + '">' +
      esc(f.signed ? f.signed.says : '') + '</dd></dl>' +
    '<h4>its signatures</h4>' + (sigs.length
      ? '<ul class="nw-list">' + sigs.map(s => '<li>' + esc(s.signer) + ' · ' + esc(s.scope) + ' · revision ' +
          esc(s.revision) + ' · ' + esc(s.signed_at) + ' · ' + esc(s.verdict) + '</li>').join('') + '</ul>'
      : '<p class="muted">none</p>') +
    '<h4>its checks</h4>' + (findings.length
      ? '<ul class="nw-list nw-findings">' + findings.map(x => '<li class="nw-' + esc(x.level) + '"><b>' + esc(x.level) +
          '</b> ' + esc(x.place) + ': ' + esc(x.what) + '</li>').join('') + '</ul>'
      : '<p class="nw-ok">nothing found</p>') +
    (f.yours
      ? '<h4>sign the day\'s work</h4>' +
        '<p><button class="ctl" id="nw-sign"' + (errors.length ? ' title="its checks find ' +
          esc(plural(errors.length, 'error')) + ': the program will refuse, and say each"' : '') + '>Sign revision ' +
          esc(f.revision) + '</button></p>' +
        '<h4>ask</h4><p><label>for <select id="nw-ask-kind" class="ctl">' +
          [['contract', 'a contract change'], ['breakdown', 'a breakdown']].map(([v, l]) =>
            '<option value="' + v + '"' + (W.asking && W.asking.kind === v ? ' selected' : '') + '>' + l + '</option>').join('') +
          '</select></label></p>' +
        '<p><textarea id="nw-ask-body" rows="3" class="nw-wide" placeholder="what you ask for, and why">' +
          esc((W.asking && W.asking.body) || '') + '</textarea></p>' +
        '<p><button class="ctl" id="nw-ask">Ask, signed</button></p>' +
        '<h4>its lesson</h4><p><a class="ctl" href="/v1/lesson-form/' + encodeURIComponent(id) + '" download="' +
          esc(id) + '.lesson-form.html">the lesson form</a> — fill it anywhere, then save it here: ' +
          '<input type="file" id="nw-lesson" accept=".html,.toml"></p>'
      : '<p class="muted">Not yours to write: you read it here, and ask its node engineer or raise an issue.</p>') +
    writeIt(f) +
    '<h4>its requests</h4>' + (reqs.length
      ? '<ul class="nw-list">' + reqs.map(r => '<li><b>' + esc(r.uid) + '</b> · ' + esc(r.kind) + ' · ' + esc(r.status) +
          ' · ' + esc(r.raised_by) + ', ' + esc(r.at) + '<div>' + esc(r.body) + '</div>' +
          (r.counts ? '' : '<div class="nw-not">does not count: ' + esc(r.why) + '</div>') + '</li>').join('') + '</ul>'
      : '<p class="muted">none</p>');
}

/** What the file says, field by field. */
function baseOf(f) {
  const c = f.content, b = {};
  for (const [k, now] of FIELDS) b[k] = now(c) || '';
  for (const k of DECLARED) b['declared.' + k] = (c.declared && c.declared[k]) || '';
  for (const [k] of WHY) b['why.' + k] = '';
  return b;
}

/** The typing begun on the file `f`: what it says, nothing touched yet. */
function fromFile(f) {
  const base = baseOf(f);
  const vals = { ...base };
  // The person writes the declaration as they make it: an assistant's help is
  // never filled in for them.
  vals['declared.ai'] = '';
  return { opened: f.opened, revision: f.revision, base, vals, touched: new Set(), theirs: {} };
}

/** The typing brought onto the file `f` as it now is: what was not touched
 *  takes its new text; what was touched is kept, and where the file changed it
 *  too, the file's is kept beside it to compare. */
function rebaseOn(e, f) {
  const now = baseOf(f);
  for (const k of Object.keys(now)) {
    if (!e.touched.has(k)) e.vals[k] = k === 'declared.ai' ? '' : now[k];
    else if (now[k] !== e.base[k]) e.theirs[k] = now[k];
  }
  e.base = now; e.opened = f.opened; e.revision = f.revision;
}

/** What is to be sent: each part the person touched, when it differs from the
 *  file it began on; the declaration with a method or cases; the record when
 *  any of it is written. */
function changes() {
  const e = W.editing;
  if (!e) return {};
  const out = {};
  for (const [k] of FIELDS) if (e.touched.has(k) && e.vals[k] !== e.base[k]) out[k] = e.vals[k];
  if ('method_text' in out || 'case_csv' in out) for (const k of DECLARED) out['declared.' + k] = e.vals['declared.' + k];
  if (WHY.some(([k]) => (e.vals['why.' + k] || '').trim())) for (const [k] of WHY) out['why.' + k] = e.vals['why.' + k];
  return out;
}

/** The write-it block: the four parts, the declaration and the record. */
function writeIt(f) {
  if (!f.editable) {
    return '<h4>write it</h4><p class="muted nw-cannot">Not written here: ' + esc(f.edit_why_not || '') + '.</p>';
  }
  const e = W.editing || { vals: {}, theirs: {} };
  const v = k => esc(e.vals[k] || '');
  const theirs = k => k in e.theirs
    ? '<div class="nw-theirs">changed in the file meanwhile too; yours is kept above, and a save writes yours. The file\'s:' +
      '<pre>' + esc(e.theirs[k]) + '</pre></div>'
    : '';
  const area = (k, rows, cls) => '<textarea data-edit="' + k + '" rows="' + rows + '" class="nw-wide' + (cls ? ' ' + cls : '') + '">' + v(k) + '</textarea>' + theirs(k);
  const behind = W.behind
    ? '<div class="blocked nw-behind"><b>what you typed began on revision ' + esc(e.revision) + '</b><div>The file is now revision ' +
      esc(W.behind.revision) + ', written by ' + esc(W.behind.writer || 'someone') + '. A save is refused until you bring your typing onto it. ' +
      '<button class="ctl" id="nw-rebase">Bring it onto revision ' + esc(W.behind.revision) + '</button></div></div>'
    : '';
  return '<h4>write it</h4>' + behind +
    '<p class="muted">Checked as you type, and written only when you save it. The centre shows today\'s page, or your ' +
      'saved revision\'s over it: what you type is on it once it is saved.</p>' +
    '<p><label>its question<br><input data-edit="question" class="nw-wide" value="' + v('question') + '"></label>' + theirs('question') + '</p>' +
    '<p><label>said simply<br>' + area('explain_simply', 3) + '</label></p>' +
    '<p><label>where it breaks<br>' + area('explain_breaks', 3) + '</label></p>' +
    '<p><label>the common misreading<br>' + area('explain_wrong', 3) + '</label></p>' +
    '<p><label>its method<br>' + area('method_text', 10, 'nw-code') + '</label></p>' +
    '<p><label>its cases (results/isolation.csv)<br>' + area('case_csv', 6, 'nw-code') + '</label></p>' +
    '<fieldset class="nw-declared"><legend>who made the method and its numbers</legend>' +
      '<label>author <input data-edit="declared.author" value="' + v('declared.author') + '"></label> ' +
      '<label>an assistant helped <select data-edit="declared.ai" class="ctl">' +
        [['', 'say which'], ['none', 'not at all'], ['wording', 'with the wording'], ['transcribed', 'it was transcribed'], ['relation', 'it supplied the relation']]
          .map(([o, l]) => '<option value="' + o + '"' + ((e['declared.ai'] || '') === o ? ' selected' : '') + '>' + l + '</option>').join('') +
      '</select></label> ' +
      '<label>source <input data-edit="declared.source" value="' + v('declared.source') + '"></label> ' +
      '<label>checked by <input data-edit="declared.checked_by" value="' + v('declared.checked_by') + '"></label></fieldset>' +
    '<fieldset class="nw-why"><legend>which belief broke: needed when its question or its method changes</legend>' +
      WHY.map(([k, l]) => '<label>' + esc(l) + '<br>' + area('why.' + k, 2) + '</label>').join('') + '</fieldset>' +
    '<div class="nw-live" aria-live="polite"></div>' +
    '<p><button class="ctl" id="nw-save">Save as the next revision</button></p>';
}

/** The program's answer to a dry run, drawn into the live part alone. */
function drawLive(r) {
  const at = $('.nw-live', W.host);
  if (!at) return;
  if (!r || r.conflict) {
    at.innerHTML = '<p class="sh-refused">' + esc((r && r.message) || 'not checked') + '</p>';
    return;
  }
  if (r.ok === false && !r.dry_run) {
    at.innerHTML = '<p class="sh-refused">' + esc(r.message || 'not checked') + '</p>';
    return;
  }
  if (r.checked === false) {
    // What was typed could not be checked: nothing is shown in its place.
    at.innerHTML = '<p class="sh-refused nw-would">' + esc('A save would be refused: ' + r.refused) + '</p>' +
      '<p class="muted">What you typed is not checked until this is mended.</p>';
    return;
  }
  const m = r.method || { cases: [], diags: [] };
  const errors = (r.findings || []).filter(x => x.level === 'error');
  at.innerHTML =
    '<p class="' + (r.refused ? 'sh-refused' : 'sh-said') + ' nw-would">' +
      esc(r.refused ? 'A save would be refused: ' + r.refused : 'A save would write revision ' + r.revision + '.') + '</p>' +
    (m.diags.length ? '<ul class="nw-list nw-diags">' + m.diags.map(d => '<li class="nw-' + esc(d.severity) + '">line ' +
      esc(d.line) + ': ' + esc(d.msg) + '</li>').join('') + '</ul>' : '') +
    (m.cases.length
      ? '<table class="sh-table nw-cases"><thead><tr><th>case</th><th></th></tr></thead><tbody>' +
        m.cases.map(c => '<tr class="' + (c.agrees ? '' : 'nw-differs') + '"><td>' + esc(c.label) + '</td><td>' +
          esc(c.agrees ? 'agrees' : c.says) + '</td></tr>').join('') + '</tbody></table>'
      : '<p class="muted">' + esc(m.runs ? 'its method runs on no case' : 'its method does not run on its cases yet') + '</p>') +
    '<p class="' + (errors.length ? 'nw-not' : 'nw-ok') + ' nw-sign-would">' + esc(errors.length
      ? 'Signing it would be refused: ' + plural(errors.length, 'finding') + ', ' + errors.map(x => x.what).join('; ')
      : 'Its checks would let it be signed.') + '</p>';
}

/** Typing: kept, and checked a moment after it stops. */
function typed(e) {
  const k = e.target && e.target.dataset && e.target.dataset.edit;
  const ed = W.editing;
  if (!k || !ed) return;
  ed.vals[k] = e.target.value;
  ed.touched.add(k);
  // A method or cases written anew are somebody's anew: who wrote, copied
  // and checked the old ones is not carried over to them.
  if ((k === 'method_text' || k === 'case_csv') && ed.vals[k] !== ed.base[k]) {
    for (const d of ['author', 'source', 'checked_by']) {
      const f = 'declared.' + d;
      if (ed.touched.has(f) || !ed.vals[f]) continue;
      ed.vals[f] = '';
      const input = $('[data-edit="' + f + '"]', W.host);
      if (input) input.value = '';
    }
  }
  // Older typing's answer, still on its way, is no longer this typing's.
  W.seq++;
  clearTimeout(W.timer);
  W.timer = setTimeout(check, SETTLE_MS);
}

/** Ask for a dry run of what is typed; an answer to older typing is dropped. */
async function check() {
  const id = W.id, seq = W.seq, ed = W.editing;
  if (!W.file || !ed) return;
  const sent = changes();
  if (!Object.keys(sent).length) { drawLive({ ok: true, dry_run: true, checked: false, refused: 'nothing has changed yet' }); return; }
  const r = await W.post('/v1/node/' + encodeURIComponent(id) + '/edit', { opened: ed.opened, dry_run: '1', ...sent });
  if (seq !== W.seq || id !== W.id) return;
  drawLive(r);
}

/** The program's answer to a step, said, and drawn: a second writer keeps the
 *  file as it was opened on screen, and offers it again. */
function said(id, r, fallback) {
  // An answer to a step on another node, while this one was opened, is not
  // this node's to show.
  if (id !== W.id) return;
  W.notice = { ok: !!(r && r.ok), text: (r && r.message) || fallback };
  W.stale = r && r.conflict ? { writer: r.writer || 'someone', revision: r.revision } : null;
  return draw(!!W.stale);
}

async function click(e) {
  const t = e.target.closest('button, .xref');
  if (!t || !W.host.contains(t)) return;
  if (t.classList.contains('xref')) {
    // A row named on the page opens here; a group, in the rows on today's
    // tool page; the inputs and a result kept, in Explore.
    if (t.dataset.goto && E.byId.has(t.dataset.goto)) {
      e.preventDefault();
      openAt(t.dataset.goto);
      return draw();
    }
    if (t.dataset.group) {
      e.preventDefault();
      location.href = '/#group/' + encodeURIComponent(t.dataset.group);
    } else if (t.dataset.inputs !== undefined) {
      e.preventDefault();
      location.hash = '#explore/inputs';
    } else if (t.dataset.results !== undefined) {
      e.preventDefault();
      location.hash = '#explore/results' + (t.dataset.results ? '/' + encodeURIComponent(t.dataset.results) : '');
    }
    return;
  }
  if (t.dataset.open) { openAt(t.dataset.open); return draw(); }
  if (t.dataset.view) {
    W.view = t.dataset.view;
    $$('.nw-view', W.host).forEach(b => b.classList.toggle('sel', b.dataset.view === W.view));
    return centre(W.id, W.file);
  }
  if (t.id === 'nw-reopen' || t.id === 'nw-rebase' || t.id === 'nw-live-reopen') { W.notice = null; W.rebasing = true; return draw(); }
  const opened = W.file && W.file.opened;
  switch (t.id) {
    case 'nw-go': {
      const id = $('#nw-id', W.host).value.trim();
      if (id) { openAt(id); return draw(); }
      return;
    }
    case 'nw-save': {
      const id = W.id;
      clearTimeout(W.timer); W.seq++;
      // Saved over the revision the typing began on: a file changed since is
      // a second writer, never written over.
      const r = await W.post('/v1/node/' + encodeURIComponent(id) + '/edit',
        { opened: W.editing ? W.editing.opened : opened, ...changes() });
      // Taken: what the file now says is shown. Refused, or a second writer:
      // what was typed is kept.
      if (r && r.ok && r.written && id === W.id) W.editing = null;
      return said(id, r, 'the node was not saved');
    }
    case 'nw-sign': {
      const id = W.id;
      const r = await W.post('/v1/node/' + encodeURIComponent(id) + '/sign', { opened });
      return said(id, r, 'the node was not signed');
    }
    case 'nw-ask': {
      const id = W.id;
      // Kept until the program takes it: a refusal or a second writer never
      // loses what was typed.
      W.asking = { kind: $('#nw-ask-kind', W.host).value, body: $('#nw-ask-body', W.host).value };
      const r = await W.post('/v1/node/' + encodeURIComponent(id) + '/ask', { opened, ...W.asking });
      if (r && r.ok && id === W.id) W.asking = null;
      return said(id, r, 'nothing was asked');
    }
  }
}

/** A filled lesson form, read from the file the person picked, and saved. */
async function changed(e) {
  if (!W.host || e.target.id !== 'nw-lesson' || !e.target.files.length) return;
  const id = W.id;
  const text = await e.target.files[0].text();
  const r = await W.post('/v1/node/' + encodeURIComponent(id) + '/lesson', { opened: W.file && W.file.opened, lesson: text });
  return said(id, r, 'the lesson was not saved');
}
