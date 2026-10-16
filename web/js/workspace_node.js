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
    the steps that exist: sign the revision, ask for a contract change or a
    breakdown, save a filled lesson form. One next step is named, by the
    program.

  Every step is the program's: nothing here judges a node, signs or writes a
  file. A file that changed in the copy since it was opened is the second
  writer: the step is refused with nothing written, and the file is opened
  again on a click, never behind the person's back.

  The steps that write carry the launch's word, as every write does: the shell
  hands this module the one function that posts with it.
*/
'use strict';

import { $, $$, esc, answerFirst, plural } from './dom.js';
import { S as E, load, loadSaved } from './state.js';
import { loadOverrides } from './inputs.js';
import { drawNode } from './node.js';
import { mountParts } from './nodeparts.js';

const W = { id: '', file: null, notice: null, stale: null, loaded: null, host: null, post: null, nodes: [] };

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

async function get(path) {
  try { return await (await fetch(path)).json(); } catch (e) { return { ok: false, message: 'the program did not answer: ' + e }; }
}

/** Open the workspace at node `id`, the next time it is drawn. */
export function openAt(id) {
  if (id && id !== W.id) { W.id = id; W.file = null; W.notice = null; W.stale = null; }
}

/** Draw the workspace into `host`. `post(path, fields)` writes with the launch's word;
 *  `nodes` are the person's own, as My work lists them. */
export async function mountNode(host, post, nodes) {
  W.host = host; W.post = post; W.nodes = nodes || [];
  if (!W.id && W.nodes.length) W.id = W.nodes[0].id;
  if (!host.dataset.wired) {
    host.dataset.wired = '1';
    host.addEventListener('click', click);
    host.addEventListener('change', changed);
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
  await ready().catch(() => {});
  const file = keep && W.file ? W.file : await get('/v1/node/' + encodeURIComponent(id) + '/file');
  if (id !== W.id) return;
  W.file = file;
  if (!keep) W.stale = null;
  const row = E.byId && E.byId.get(id);
  host.innerHTML = picker() + head(id, file, row) +
    '<div class="nw-grid">' +
      '<aside class="nw-left"><h3 class="seg-h">its parts</h3><div class="nw-parts"></div></aside>' +
      '<section class="nw-centre"><h3 class="seg-h">its page, as today\'s design holds it</h3>' +
        (row ? '<div class="nw-page"></div><div class="nw-run"></div>'
          : '<p class="empty">Today\'s design holds no node <code>' + esc(id) + '</code>, so no page is drawn: ' +
            'a node joins it when its group seals a release that holds it.</p>') +
      '</section>' +
      '<aside class="nw-right">' + yourFile(id, file) + '</aside>' +
    '</div>';
  if (row) {
    mountParts($('.nw-parts', host), id);
    await drawNode($('.nw-page', host), $('.nw-run', host), id, { parts: false });
  } else {
    $('.nw-parts', host).innerHTML = '<p class="empty">No parts: today\'s design does not hold it.</p>';
  }
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
        '<h4>ask</h4><p><label>for <select id="nw-ask-kind" class="ctl"><option value="contract">a contract change</option>' +
          '<option value="breakdown">a breakdown</option></select></label></p>' +
        '<p><textarea id="nw-ask-body" rows="3" class="nw-wide" placeholder="what you ask for, and why"></textarea></p>' +
        '<p><button class="ctl" id="nw-ask">Ask, signed</button></p>' +
        '<h4>its lesson</h4><p><a class="ctl" href="/v1/lesson-form/' + encodeURIComponent(id) + '" download="' +
          esc(id) + '.lesson-form.html">the lesson form</a> — fill it anywhere, then save it here: ' +
          '<input type="file" id="nw-lesson" accept=".html,.toml"></p>'
      : '<p class="muted">Not yours to write: you read it here, and ask its node engineer or raise an issue.</p>') +
    '<h4>its requests</h4>' + (reqs.length
      ? '<ul class="nw-list">' + reqs.map(r => '<li><b>' + esc(r.uid) + '</b> · ' + esc(r.kind) + ' · ' + esc(r.status) +
          ' · ' + esc(r.raised_by) + ', ' + esc(r.at) + '<div>' + esc(r.body) + '</div>' +
          (r.counts ? '' : '<div class="nw-not">does not count: ' + esc(r.why) + '</div>') + '</li>').join('') + '</ul>'
      : '<p class="muted">none</p>');
}

/** The program's answer to a step, said, and drawn: a second writer keeps the
 *  file as it was opened on screen, and offers it again. */
function said(r, fallback) {
  W.notice = { ok: !!(r && r.ok), text: (r && r.message) || fallback };
  W.stale = r && r.conflict ? { writer: r.writer || 'someone', revision: r.revision } : null;
  return draw(!!W.stale);
}

async function click(e) {
  const t = e.target.closest('button, .xref');
  if (!t || !W.host.contains(t)) return;
  if (t.classList.contains('xref')) {
    // A row named on the page opens here; anything else is today's page's.
    if (t.dataset.goto && E.byId.has(t.dataset.goto)) {
      e.preventDefault();
      openAt(t.dataset.goto);
      return draw();
    }
    return;
  }
  if (t.dataset.open) { openAt(t.dataset.open); return draw(); }
  if (t.id === 'nw-reopen') { W.notice = null; return draw(); }
  const opened = W.file && W.file.opened;
  switch (t.id) {
    case 'nw-go': {
      const id = $('#nw-id', W.host).value.trim();
      if (id) { openAt(id); return draw(); }
      return;
    }
    case 'nw-sign': {
      const r = await W.post('/v1/node/' + encodeURIComponent(W.id) + '/sign', { opened });
      return said(r, 'the node was not signed');
    }
    case 'nw-ask': {
      const r = await W.post('/v1/node/' + encodeURIComponent(W.id) + '/ask', {
        opened, kind: $('#nw-ask-kind', W.host).value, body: $('#nw-ask-body', W.host).value,
      });
      return said(r, 'nothing was asked');
    }
  }
}

/** A filled lesson form, read from the file the person picked, and saved. */
async function changed(e) {
  if (!W.host || e.target.id !== 'nw-lesson' || !e.target.files.length) return;
  const text = await e.target.files[0].text();
  const r = await W.post('/v1/node/' + encodeURIComponent(W.id) + '/lesson', { opened: W.file && W.file.opened, lesson: text });
  return said(r, 'the lesson was not saved');
}
