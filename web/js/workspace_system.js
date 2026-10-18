/*
  The System workspace of the application, its first part (docs/OPERATING_1_0.md,
  section 4; docs/briefs/F4.md, F4-b): a branch's breakdown, as your copy
  holds it, grown by its owner.

  * LEFT, THE BRANCH AS A TREE. Every block of the group's branch, the
    headings and blocks framed in its group file and each issued node, by its
    parent, marked with what it is read as and with how much is still open
    in it. A removed block is shown struck through: archived, never deleted.
  * RIGHT, THE BLOCK CHOSEN. Its question, its ports, what is still open,
    and, for a node broken down, the child that answers it and whether the
    children reproduce its own cases, case by case beside its estimate. Its
    history: who changed it, when, and why.
  * THE EDITS, for the branch's owner or their deputy only, as the program
    says (`chain::may_frame`): add a block, fill one a part at a time, break
    a node down, fold it back, move, rename, split, remove. Each says why.

  Every answer is the program's. Nothing here decides who may frame a
  branch, writes a file or judges a breakdown. A group file changed in the
  copy since it was opened is the second writer: the edit is refused with
  nothing written, and the branch is opened again on a click.

  It shows your copy, said so: a branch joins today's design when its group
  seals a release that holds it (F5).
*/
'use strict';

import { $, esc, answerFirst, plural } from './dom.js';

const W = { group: '', tree: null, sel: '', notice: null, stale: false, host: null, post: null, groups: [], why: '', view: null };

async function get(path) {
  try { return await (await fetch(path)).json(); } catch (e) { return { ok: false, message: 'the program did not answer: ' + e }; }
}

/** Open the workspace at group `id`, the next time it is drawn. */
export function openAt(id) {
  if (id && id !== W.group) { W.group = id; W.tree = null; W.sel = ''; W.notice = null; W.stale = false; W.view = null; }
}

/** Draw the workspace into `host`. `post(path, fields)` writes with the launch's word;
 *  `groups` are the branches the person owns, as the program says. */
export async function mountSystem(host, post, groups) {
  W.host = host; W.post = post; W.groups = groups || [];
  W.notice = null; W.stale = false;
  if (!W.group && W.groups.length) W.group = W.groups[0];
  if (!host.dataset.wired) {
    host.dataset.wired = '1';
    host.addEventListener('click', click);
  }
  await draw();
}

/** Draw the branch: read again from the copy, unless `keep`, after a second writer. */
async function draw(keep) {
  const host = W.host;
  if (!W.group) {
    host.innerHTML = picker() + answerFirst(W.groups.length
      ? 'Pick one of your branches, or name any group to read its branch.'
      : 'No branch is yours to frame: name any group to read its branch.', [], 'how-to');
    return;
  }
  const group = W.group;
  if (!keep || !W.tree) {
    host.innerHTML = picker() + '<p class="sh-loading">opening ' + esc(group) + '…</p>';
    const t = await get('/v1/branch?group=' + encodeURIComponent(group));
    if (group !== W.group) return;
    W.tree = t;
    W.stale = false;
  }
  const t = W.tree;
  if (!t || t.ok === false) {
    host.innerHTML = picker() + notice() + answerFirst('<b>' + esc(group) + '</b>: ' +
      esc((t && t.message) || 'its branch cannot be read.'), [], 'reference');
    return;
  }
  if (!W.sel || !t.blocks.some(b => b.uid === W.sel)) W.sel = (t.blocks.find(b => !b.archived) || {}).uid || '';
  host.innerHTML = picker() + notice() + stale() +
    answerFirst('<b>' + esc(group) + '</b>, ' + esc(t.shows) + ' ' +
      (t.yours ? 'You frame it.' : 'Framed by ' + esc(t.owner || 'nobody') + '.') +
      ' <span class="sy-next">Next: ' + esc(t.next) + '</span>', [
        esc(plural(t.blocks.filter(b => !b.archived).length, 'block')) + ', ' + esc(plural(t.open, 'part')) + ' still open.',
        esc(t.signed.says) + '.',
      ], 'reference') +
    view() +
    '<div class="sy-grid"><nav class="sy-tree" aria-label="the branch">' + tree(t) + '</nav>' +
    '<section class="sy-block">' + block(t) + '</section></div>';
}

/** Today's view of the branch and its impact (F5-d), once asked for: what the
 *  program built and ran, said as it said it. */
function view() {
  const v = W.view;
  if (!v) return '<p><button class="ctl" id="sy-view">Today\'s view</button> today\'s design with every node of this branch signed as it is in your copy, and every value outside it that moves.</p>';
  if (v.ok === false) return '<p class="sh-refused sy-view">' + esc(v.message || 'today\'s view cannot be built') + '</p>';
  const num = x => x === null || x === undefined ? 'refused' : String(+x.toPrecision(6));
  return '<section class="sy-view"><h4>today\'s view</h4><p>' + esc(v.says) + '</p>' +
    '<p>' + (v.changes.length ? 'It changes ' + esc(v.changes.join(', ')) + '.' : 'Nothing signed in your copy changes today\'s design.') + '</p>' +
    (v.left_out.length ? '<p>Left out: ' + v.left_out.map(l => esc(l.node + ' — ' + l.why)).join('<br>') + '</p>' : '') +
    (v.impact.length
      ? '<table class="sh-table sy-impact"><thead><tr><th>outside the branch</th><th>group</th><th>today</th><th>in the view</th></tr></thead><tbody>' +
        v.impact.map(m => '<tr><td>' + esc(m.id) + '</td><td>' + esc(m.group) + '</td><td>' + esc(num(m.before)) + '</td><td>' + esc(num(m.after)) + '</td></tr>').join('') +
        '</tbody></table>'
      : '<p class="muted">No value outside the branch moves.</p>') +
    '<p><button class="ctl" id="sy-view">Build it again</button></p>' +
    (W.tree && W.tree.yours
      ? '<p class="sy-seal"><b>seal</b> it as release <input id="sy-seal-version" size="6" placeholder="1.0"> ' +
        '<button class="ctl" id="sy-seal">Seal</button> every node signed at its contract, the group\'s checks passing, or every reason it is not.</p>'
      : '') +
    '</section>';
}

function picker() {
  return '<div class="nw-pick"><span class="nw-pick-k">your branches</span> ' +
    (W.groups.length ? W.groups.map(g =>
      '<button class="ctl nw-mine' + (g === W.group ? ' sel' : '') + '" data-group-open="' + esc(g) + '">' + esc(g) + '</button>').join(' ')
      : '<span class="muted">none</span>') +
    ' <label class="nw-any">any group <input id="sy-id" size="18" placeholder="its id"></label>' +
    ' <button class="ctl" id="sy-go">Open</button></div>';
}

function notice() {
  const n = W.notice;
  return n ? '<p class="' + (n.ok ? 'sh-said' : 'sh-refused') + ' sy-notice">' + esc(n.text) + '</p>' : '';
}

function stale() {
  return W.stale
    ? '<div class="blocked sy-stale"><b>what is shown is the branch as you opened it</b><div>Its group file changed in your copy since. ' +
      '<button class="ctl" id="sy-reopen">Open it again</button></div></div>'
    : '';
}

/** The branch as a tree: each block under its parent; a block whose parent the
 *  branch does not hold is a root. */
function tree(t) {
  const uids = new Set(t.blocks.map(b => b.uid));
  const kids = new Map();
  for (const b of t.blocks) {
    const p = uids.has(b.parent) ? b.parent : '';
    if (!kids.has(p)) kids.set(p, []);
    kids.get(p).push(b);
  }
  for (const list of kids.values()) list.sort((a, b) => a.ord - b.ord || a.id.localeCompare(b.id));
  // A block no root reaches (a parent cycle the file holds) is still shown,
  // at the top: never dropped from the screen.
  const reached = new Set();
  const walk = p => (kids.get(p) || []).forEach(b => { if (!reached.has(b.uid)) { reached.add(b.uid); walk(b.uid); } });
  walk('');
  for (const b of t.blocks) {
    if (!reached.has(b.uid)) { kids.get('').push(b); reached.add(b.uid); walk(b.uid); }
  }
  if (!kids.has('')) kids.set('', []);
  const item = b => '<li><button class="sy-node' + (b.uid === W.sel ? ' sel' : '') + (b.archived ? ' sy-archived' : '') +
    '" data-block="' + esc(b.uid) + '" title="' + esc(b.question || 'no question yet') + '">' + esc(b.id) + '</button>' +
    ' <span class="tag sy-' + esc(b.read_as) + '">' + esc(b.read_as) + '</span>' +
    (b.open.length && !b.archived ? ' <span class="sy-open">' + esc(plural(b.open.length, 'open part')) + '</span>' : '') +
    (b.reproduces ? ' <span class="sy-' + (b.reproduces.state === 'agrees' ? 'ok' : 'not') + '">' +
      ({ agrees: 'children reproduce it', differs: 'children do not reproduce it' }[b.reproduces.state] || 'not judged') + '</span>' : '') +
    (kids.has(b.uid) ? '<ul>' + kids.get(b.uid).map(item).join('') + '</ul>' : '') + '</li>';
  return '<ul class="sy-roots">' + (kids.get('') || []).map(item).join('') + '</ul>';
}

/** The block chosen, and what its owner may do to it. */
function block(t) {
  const b = t.blocks.find(x => x.uid === W.sel);
  if (!b) return '<p class="muted">Pick a block of the branch.</p>';
  const children = t.blocks.filter(x => x.parent === b.uid && !x.archived);
  const outs = b.ports.filter(p => p.direction === 'out');
  const history = t.changes.filter(c => c.scope === b.uid);
  const r = b.reproduces;
  return '<h3 class="seg-h">' + esc(b.id) + ' <span class="tag sy-' + esc(b.read_as) + '">' + esc(b.read_as) + '</span></h3>' +
    '<dl class="nw-kv"><dt>question</dt><dd>' + esc(b.question || '—') + '</dd>' +
    '<dt>written in</dt><dd>' + esc(b.in) + (b.archived ? ', removed' : '') + (b.mount ? ', a group hangs here' : '') + '</dd>' +
    '<dt>its ports</dt><dd>' + (b.ports.length ? b.ports.map(p => esc(p.direction + ' ' + p.name + ' · ' + (p.type || 'no type') +
      (p.unit ? ' [' + p.unit + ']' : '') + ' · ' + (p.state || 'no state'))).join('<br>') : '—') + '</dd>' +
    '<dt>still open</dt><dd class="sy-open-parts">' + (b.open.length ? esc(b.open.join('; ')) : 'nothing') + '</dd></dl>' +
    (b.answers.length
      ? '<h4>broken down</h4><p>' + b.answers.map(a => esc(a.output + ' is answered by ' + a.child + '.' + a.port)).join('<br>') + '</p>' +
        '<p class="sy-' + (r && r.state === 'agrees' ? 'ok' : 'not') + ' sy-says">' + esc(r ? r.says : '') + '</p>' +
        (r && r.cases.length
          ? '<table class="sh-table sy-cases"><thead><tr><th>case</th><th>expects</th><th>children</th><th>its estimate</th><th></th></tr></thead><tbody>' +
            r.cases.map(c => '<tr class="' + (c.agrees ? '' : 'sy-differs') + '"><td>' + esc(c.label) + '</td><td>' + esc(c.expected) +
              '</td><td>' + esc(c.children) + '</td><td>' + esc(c.estimate) + '</td><td>' + esc(c.agrees ? 'agrees' : c.says) +
              '</td></tr>').join('') + '</tbody></table>'
          : '')
      : '') +
    '<h4>its history</h4>' + (history.length
      ? '<ul class="nw-list sy-history">' + history.map(c => '<li>' + esc(c.at) + ' · ' + esc(c.who) + ' · ' + esc(c.what) + '</li>').join('') + '</ul>'
      : '<p class="muted">nothing changed here yet</p>') +
    (t.yours ? edits(b, children, outs) : '<p class="muted sy-not-yours">' + esc(t.why_not || 'Not yours to frame.') + '</p>');
}

/** The edit forms, drawn for the branch's owner only; each says why. */
function edits(b, children, outs) {
  const inGroup = b.in === 'the group file';
  const open = b.read_as === 'open';
  const opt = (v, l) => '<option value="' + esc(v) + '">' + esc(l || v) + '</option>';
  let h = '<h4>frame it</h4><p><label>why <textarea id="sy-why" rows="2" class="nw-wide" placeholder="which belief this follows from">' +
    esc(W.why) + '</textarea></label></p>';
  if (!b.archived) {
    h += '<p><b>add a block under it</b> <input id="sy-add-id" size="18" placeholder="its id"> ' +
      '<input id="sy-add-q" size="30" placeholder="the question it answers"> <button class="ctl" id="sy-add">Add</button></p>';
  }
  if (inGroup && open && !b.archived) {
    h += '<p><b>its question</b> <input id="sy-q" size="40" value="' + esc(b.question) + '"> <button class="ctl" id="sy-fill-q">Fill</button></p>' +
      '<p><b>a port</b> <select id="sy-p-dir" class="ctl">' + opt('out') + opt('in') + '</select> ' +
      '<input id="sy-p-name" size="16" placeholder="name"> <select id="sy-p-type" class="ctl">' +
      ['number', 'integer', 'choice', 'boolean', 'list', 'table', 'series', 'uncertain', 'text', 'file'].map(v => opt(v)).join('') + '</select> ' +
      '<input id="sy-p-unit" size="6" placeholder="unit"> <input id="sy-p-lo" size="5" placeholder="from"> <input id="sy-p-hi" size="5" placeholder="to"> ' +
      '<select id="sy-p-state" class="ctl">' + opt('open') + opt('decided') + opt('allocated') + opt('achieved') + '</select> ' +
      '<button class="ctl" id="sy-fill-p">Fill</button></p>';
  }
  if (!inGroup && !b.answers.length && children.length && outs.length) {
    // One answer for each output: the program refuses fewer.
    h += '<p><b>break it down</b>, each output answered by a child\'s answer</p>' + outs.map(o =>
      '<p class="sy-bd-row" data-bd-out="' + esc(o.name) + '">its ' + esc(o.name) + ' by ' +
      '<select class="ctl sy-bd-child">' + children.map(c => opt(c.uid, c.id)).join('') + '</select>\'s ' +
      '<input class="sy-bd-port" size="16" placeholder="its output" value="' + esc(o.name) + '"></p>').join('') +
      '<p><button class="ctl" id="sy-break-down">Break down</button></p>';
  }
  if (b.answers.length) h += '<p><button class="ctl" id="sy-fold-back">Fold back</button> its estimate answers again; its children stay.</p>';
  if (inGroup && !b.archived) {
    h += '<p><b>move</b> under <input id="sy-mv-parent" size="18" placeholder="a block\'s uid"> <button class="ctl" id="sy-move">Move</button> ' +
      '<b>rename</b> <input id="sy-rn-id" size="18" placeholder="its new id"> <button class="ctl" id="sy-rename">Rename</button></p>' +
      (open ? '<p><b>split</b> into <input id="sy-sp-id" size="18" placeholder="the new block\'s id"> taking ports ' +
        '<input id="sy-sp-ports" size="20" placeholder="names, by commas"> <button class="ctl" id="sy-split">Split</button></p>' : '') +
      '<p><button class="ctl" id="sy-remove">Remove</button> archived, never deleted.</p>';
  }
  return h;
}

/** Send one edit; the program's answer is said, and the branch read again,
 *  but after a second writer it is kept as it was opened. */
async function send(edit, fields) {
  const group = W.group;
  W.why = ($('#sy-why', W.host) || {}).value || '';
  const r = await W.post('/v1/branch/' + edit, { group, opened: W.tree.opened, why: W.why, ...fields });
  if (group !== W.group) return;
  W.notice = { ok: !!(r && r.ok), text: (r && r.message) || 'nothing was changed' };
  if (r && r.ok) W.why = '';
  W.stale = !!(r && r.conflict);
  if (W.stale) return draw(true);
  return draw();
}

async function click(e) {
  const t = e.target.closest('button');
  if (!t || !W.host.contains(t)) return;
  const v = id => ($('#' + id, W.host) || {}).value || '';
  if (t.dataset.groupOpen) { openAt(t.dataset.groupOpen); return draw(); }
  if (t.dataset.block) {
    W.why = v('sy-why');
    W.sel = t.dataset.block;
    return draw(true);
  }
  const b = W.sel;
  switch (t.id) {
    case 'sy-go': { const id = v('sy-id').trim(); if (id) { openAt(id); return draw(); } return; }
    case 'sy-reopen': W.notice = null; return draw();
    case 'sy-seal': {
      const r = await W.post('/v1/seal', { group: W.group, version: v('sy-seal-version').trim() });
      W.notice = { ok: !!(r && r.ok), text: (r && r.message) || 'nothing was sealed' };
      return draw(true);
    }
    case 'sy-view': {
      const group = W.group;
      const r = await get('/v1/branch/view?group=' + encodeURIComponent(group));
      if (group !== W.group) return;
      W.view = r;
      return draw(true);
    }
    case 'sy-add': return send('add', { parent: b, id: v('sy-add-id').trim(), question: v('sy-add-q') });
    case 'sy-fill-q': return send('fill', { block: b, question: v('sy-q') });
    case 'sy-fill-p': return send('fill', {
      block: b, direction: v('sy-p-dir'), name: v('sy-p-name').trim(), type: v('sy-p-type'), unit: v('sy-p-unit').trim(),
      lower: v('sy-p-lo').trim(), upper: v('sy-p-hi').trim(), state: v('sy-p-state'),
    });
    case 'sy-break-down': return send('break-down', {
      node: b,
      answers: [...W.host.querySelectorAll('.sy-bd-row')].map(r =>
        r.dataset.bdOut + '=' + r.querySelector('.sy-bd-child').value + '.' + r.querySelector('.sy-bd-port').value.trim()).join('\n'),
    });
    case 'sy-fold-back': return send('fold-back', { node: b });
    case 'sy-move': return send('move', { block: b, parent: v('sy-mv-parent').trim() });
    case 'sy-rename': return send('rename', { block: b, id: v('sy-rn-id').trim() });
    case 'sy-split': return send('split', { block: b, id: v('sy-sp-id').trim(), ports: v('sy-sp-ports') });
    case 'sy-remove': return send('remove', { block: b });
  }
}
