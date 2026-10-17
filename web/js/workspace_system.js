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
  * THE FRAME (docs/briefs/F6.md, F6-a): the valve above the branch and what
    it is allocated, which way each binds; in the systems' branch, every
    group mounted below. For the system engineer or their deputy, *mount a
    group here* on a leaf of any branch, and *allocate* on a mount framed in
    the systems' file, the sense chosen, never defaulted. A group mounted
    and not started says whose step it is, and *Start <g>* is its owner's.
    Budgets and margins are a link to Explore's health map: the engine's.

  Every answer is the program's. Nothing here decides who may frame a
  branch, writes a file or judges a breakdown. A group file changed in the
  copy since it was opened is the second writer: the edit is refused with
  nothing written, and the branch is opened again on a click.

  It shows your copy, said so: a branch joins today's design when its group
  seals a release that holds it (F5).
*/
'use strict';

import { $, esc, answerFirst, plural } from './dom.js';

const W = { group: '', tree: null, sel: '', notice: null, stale: false, host: null, post: null, groups: [], why: '', seq: 0 };

async function get(path) {
  try { return await (await fetch(path)).json(); } catch (e) { return { ok: false, message: 'the program did not answer: ' + e }; }
}

/** Open the workspace at group `id`, the next time it is drawn. */
export function openAt(id) {
  if (id && id !== W.group) { W.group = id; W.tree = null; W.sel = ''; W.notice = null; W.stale = false; }
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
    // Only the latest reading is drawn: an earlier one of the same branch,
    // answered late, would draw over what was typed since.
    const seq = ++W.seq;
    const t = await get('/v1/branch?group=' + encodeURIComponent(group));
    if (group !== W.group || seq !== W.seq) return;
    W.tree = t;
    W.stale = false;
  }
  const t = W.tree;
  if (!t || t.ok === false) {
    host.innerHTML = picker() + notice() + answerFirst('<b>' + esc(group) + '</b>: ' +
      esc((t && t.message) || 'its branch cannot be read.'), [], 'reference');
    return;
  }
  if (t.started === false) {
    host.innerHTML = picker() + notice() +
      answerFirst('<b>' + esc(group) + '</b> is mounted for ' + esc(t.owner || 'nobody') + ', and its file is not started yet.' +
        ' <span class="sy-next">Next: ' + esc(t.next) + '</span>', [esc(t.shows)], 'reference') +
      frame(t) +
      (t.yours
        ? '<p><button class="ctl" id="sy-start">Start ' + esc(group) + '</button> one heading, its question the mount\'s, signed by you. ' +
          'It shows in today\'s design once ' + esc(group) + ' seals a release (F5).</p>'
        : '<p class="muted sy-not-yours">' + esc(t.why_not || 'Not yours to start.') + '</p>');
    return;
  }
  if (!W.sel || !t.blocks.some(b => b.uid === W.sel)) W.sel = (t.blocks.find(b => !b.archived) || {}).uid || '';
  host.innerHTML = picker() + notice() + stale() +
    answerFirst('<b>' + esc(group) + '</b>, ' + esc(t.shows) + ' ' +
      (t.yours ? 'You frame it.' : 'Framed by ' + esc(t.owner || 'nobody') + '.') +
      ' <span class="sy-next">Next: ' + esc(t.next) + '</span>', [
        esc(plural(t.blocks.filter(b => !b.archived).length, 'block')) + ', ' + esc(plural(t.open, 'part')) + ' still open.',
        esc(t.signed.says) + '.',
      ], 'reference') + frame(t) +
    '<div class="sy-grid"><nav class="sy-tree" aria-label="the branch">' + tree(t) + '</nav>' +
    '<section class="sy-block">' + block(t) + '</section></div>';
}

/** The frame around the branch, as the systems' file says it: the valve
 *  above, what the branch is allocated and which way each binds, every group
 *  the systems' file mounts below, and the budgets, which are the engine's. */
function frame(t) {
  const v = t.valve_above;
  const amount = a => a.value ? a.value : a.lower + ' to ' + a.upper;
  const below = t.below || [];
  if (t.frame_refused) {
    return '<section class="sy-frame"><h3 class="seg-h">the frame</h3><p class="sh-refused sy-frame-refused">' +
      esc(t.frame_refused) + '</p></section>';
  }
  return '<section class="sy-frame"><h3 class="seg-h">the frame</h3><dl class="nw-kv">' +
    '<dt>the valve above</dt><dd class="sy-valve-above">' + (v
      ? esc(v.owner) + ', who owns ' + esc(v.branch) + ', the branch that holds the leaf ' + esc(t.group) + ' hangs on'
      : (t.group === 'systems' ? 'the programme\'s file, its programme manager' : 'not read from the systems\' file')) + '</dd>' +
    '<dt>allocated</dt><dd class="sy-allocations">' + ((t.allocations || []).length
      ? t.allocations.map(a => esc(a.name + ' = ' + amount(a) + (a.unit ? ' [' + a.unit + ']' : '') + ': ' + a.bounds + ' ' +
          (a.sense || 'with no sense') + ' it') + (a.says ? ' <span class="muted">' + esc(a.says) + '</span>' : '')).join('<br>')
      : 'nothing yet') + '</dd>' +
    (t.group === 'systems'
      ? '<dt>below</dt><dd class="sy-below">' + (below.length ? below.map(m =>
          '<button class="ctl" data-group-open="' + esc(m.group) + '">' + esc(m.group) + '</button> on ' + esc(m.block) +
          ', owned by ' + esc(m.owner || 'nobody') + (m.started ? '' : ', <b>not started</b>') +
          (m.valve_above && m.valve_above.branch !== 'systems' ? ', below ' + esc(m.valve_above.owner) + '\'s ' + esc(m.valve_above.branch) : '')).join('<br>')
        : 'nothing mounted') + '</dd>'
      : '') +
    '<dt>budgets and margins</dt><dd><a href="#explore/design" class="sy-budgets">the health map, in Explore</a>: ' +
      'every margin as the engine computes it, never computed here</dd></dl></section>';
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
    ? '<div class="blocked sy-stale"><b>what is shown is the branch as you opened it</b><div>' +
      (W.stale === 'systems' ? 'The systems\' file changed in your copy since. ' : 'Its group file changed in your copy since. ') +
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
    '<dt>written in</dt><dd>' + esc(b.in) + (b.archived ? ', removed' : '') +
      (b.carries ? ', ' + esc(b.carries) + ' hangs here' : b.mount ? ', a group hangs here' : '') + '</dd>' +
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
    // A leaf a group hangs from is not framed while it does: no form is
    // offered that the program refuses.
    (b.carries && t.group !== 'systems'
      ? '<p class="muted sy-carried">' + esc(b.carries) + ' is mounted on it in the systems\' file: it is not filled, broken down, moved or removed while it hangs there.</p>'
      : t.yours ? edits(b, children, outs) : '<p class="muted sy-not-yours">' + esc(t.why_not || 'Not yours to frame.') + '</p>') +
    (t.may_mount ? frameEdits(t, b, children, outs) : '');
}

/** Mount a group on a leaf, and allocate to a mount framed in the systems'
 *  file: the system engineer's, or their deputy's, written in the systems'
 *  file whichever branch is open. The sense is chosen, never defaulted. */
function frameEdits(t, b, children, outs) {
  if (b.archived) return '';
  let h = '';
  // A leaf of a group file, or an issued node of a subsystem's branch: what
  // the program mounts on. Not the branch's heading, nor a systems' node.
  const leaf = !children.length && !b.carries && !b.mount && !b.answers.length && b.uid !== t.group &&
    !(t.group === 'systems' && b.in !== 'the group file');
  if (leaf) {
    h += '<p><b>mount a group here</b> <input id="sy-m-group" size="16" placeholder="the group\'s id"> for ' +
      '<input id="sy-m-owner" size="16" placeholder="its owner"> <button class="ctl" id="sy-mount">Mount</button></p>';
  }
  if (t.group === 'systems' && b.mount && b.in === 'the group file') {
    const opt = (v, l) => '<option value="' + esc(v) + '">' + esc(l || v) + '</option>';
    h += '<p><b>allocate</b> <input id="sy-a-name" size="16" placeholder="its name"> <select id="sy-a-type" class="ctl">' +
      opt('number') + opt('integer') + '</select> <input id="sy-a-value" size="6" placeholder="a value"> or ' +
      '<input id="sy-a-lo" size="5" placeholder="from"> <input id="sy-a-hi" size="5" placeholder="to"> <input id="sy-a-unit" size="6" placeholder="unit"> ' +
      'bounding <select id="sy-a-bounds" class="ctl">' + outs.map(o => opt(o.name)).join('') + '</select> ' +
      '<select id="sy-a-sense" class="ctl">' + opt('', 'which way it binds') + opt('<=', '<= stays under it') + opt('>=', '>= reaches it') + '</select> ' +
      '<input id="sy-a-says" size="24" placeholder="what the closure says"> <button class="ctl" id="sy-allocate">Allocate</button></p>';
  }
  if (!h) return '';
  return '<h4>the frame, in the systems\' file</h4><p><label>why <textarea id="sy-f-why" rows="2" class="nw-wide" placeholder="which belief this follows from">' +
    '</textarea></label></p>' + h;
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
      // Not `allocated`: an allocation is given to a mounted group, with its sense.
      '<select id="sy-p-state" class="ctl">' + opt('open') + opt('decided') + opt('achieved') + '</select> ' +
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

/** Send one edit of the frame: made in the systems' file, held to its digest,
 *  whichever branch is open. */
async function sendFrame(edit, fields) {
  const group = W.group;
  const why = ($('#sy-f-why', W.host) || {}).value || '';
  const r = await W.post('/v1/branch/' + edit, { group: 'systems', opened: W.tree.systems_opened, why, ...fields });
  if (group !== W.group) return;
  W.notice = { ok: !!(r && r.ok), text: (r && r.message) || 'nothing was changed' };
  // The systems' file changed, not this branch's: said as that.
  W.stale = r && r.conflict ? 'systems' : false;
  return draw(!!W.stale);
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
    case 'sy-mount': return sendFrame('mount', { block: b, mounts: v('sy-m-group').trim(), owner: v('sy-m-owner').trim() });
    case 'sy-allocate': return sendFrame('allocate', {
      block: b, name: v('sy-a-name').trim(), type: v('sy-a-type'), value: v('sy-a-value').trim(), lower: v('sy-a-lo').trim(),
      upper: v('sy-a-hi').trim(), unit: v('sy-a-unit').trim(), bounds: v('sy-a-bounds'), sense: v('sy-a-sense'), closure_says: v('sy-a-says'),
    });
    case 'sy-start': {
      const group = W.group;
      const r = await W.post('/v1/group/start', { group });
      if (group !== W.group) return;
      W.notice = { ok: !!(r && r.ok), text: (r && r.message) || 'nothing was started' };
      return draw();
    }
  }
}
