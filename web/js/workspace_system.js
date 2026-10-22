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
    Budgets across releases: each closure's state and margin in every
    released design that kept them, read from its margins.csv as the engine
    answered when it was released, never computed here (F6-c, item 7); a
    design released before margins were kept is said. Today's margins are a
    link to Explore's health map: the engine's.

  Every answer is the program's. Nothing here decides who may frame a
  branch, writes a file or judges a breakdown. A group file changed in the
  copy since it was opened is the second writer: the edit is refused with
  nothing written, and the branch is opened again on a click.

  It shows your copy, said so: a branch joins today's design when its group
  seals a release that holds it (F5).

  * RELEASE THE DESIGN (docs/briefs/F6.md, F6-b; W8), on the systems'
    branch: the integration checks of today's design, a line for each of
    W8's five for each group, held or failing with why; the red marks beside
    them, each closure that does not close and each issue open, to be marked
    understood or raised as an issue in Explore; and *Release design*, with
    its version and what changed. The program refuses it by name when a line
    fails or a mark is neither; the page never decides it. Released, it is in
    your copy, and reaches everyone once it is put back.
*/
'use strict';

import { $, esc, answerFirst, plural, fmt } from './dom.js';

const W = { group: '', tree: null, sel: '', notice: null, stale: false, host: null, post: null, groups: [], why: '', seq: 0, view: null };
/** The release panel: the checks as last read, what was typed, and the program's answer. */
const R = { seq: 0, checks: null, version: '', changed: '', understood: new Set(), notice: null, sending: false, previewing: false, previewed: null, deciding: false };
/** The budgets across releases, read again with each drawing of the frame. */
const B = { seq: 0 };

async function get(path) {
  try { return await (await fetch(path)).json(); } catch (e) { return { ok: false, message: 'the program did not answer: ' + e }; }
}

/** Open the workspace at group `id`, or `<group>/<block>` at that block, the
 *  next time it is drawn. */
export function openAt(at) {
  const [id, block] = String(at || '').split('/');
  if (id && id !== W.group) { W.group = id; W.tree = null; W.sel = ''; W.notice = null; W.stale = false; W.view = null; R.notice = null; R.previewed = null; }
  if (id && block) W.sel = block;
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
    // What is typed and ticked in the release panel is kept across its redraws.
    host.addEventListener('input', e => {
      if (e.target.id === 'sy-r-version') R.version = e.target.value;
      if (e.target.id === 'sy-r-changed') R.changed = e.target.value;
    });
    host.addEventListener('change', e => {
      const m = e.target.dataset && e.target.dataset.mark;
      if (m) { if (e.target.checked) R.understood.add(m); else R.understood.delete(m); }
    });
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
    await drawBudgets();
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
    view() +
    (t.yours ? answerForm() : '') +
    (t.yours ? baseForm() : '') +
    '<div class="sy-grid"><nav class="sy-tree" aria-label="the branch">' + tree(t) + '</nav>' +
    '<section class="sy-block">' + block(t) + '</section></div>' +
    (group === 'systems' ? '<section class="sy-release" id="sy-release"><p class="sh-loading">reading the integration checks…</p></section>' : '');
  // The budgets and the release panel are read side by side: neither waits
  // on the other.
  await Promise.all([drawBudgets(), group === 'systems' ? drawRelease() : null]);
}

/** Budgets across releases, as the program reads them from each released
 *  design's margins.csv: a row for each closure, a column for each design
 *  that kept them, each cell its state and margin as the engine answered
 *  when that design was released. Nothing is computed here. */
async function drawBudgets() {
  // A frame refused has no budgets drawn: it says why already.
  if (!W.host || !W.host.querySelector('.sy-budgets')) return;
  const seq = ++B.seq;
  const b = await get('/v1/design/budgets');
  const host = W.host && W.host.querySelector('.sy-budgets');
  if (seq !== B.seq || !host) return;
  if (!b || b.ok === false) {
    host.innerHTML = '<p class="sh-refused sy-budgets-refused">' + esc((b && b.message) || 'the budgets cannot be read') + '</p>';
    return;
  }
  const kept = b.designs.filter(d => d.kept);
  const cell = (c, v) => {
    const a = c.answers.find(x => x.version === v);
    return a
      ? '<td class="sy-b-' + esc(a.state) + '" title="' + esc(a.why) + '">' + esc(a.state) +
        (a.margin === null ? '' : ' ' + esc(fmt(a.margin))) + '</td>'
      : '<td class="muted">not a closure then</td>';
  };
  host.innerHTML = '<p class="sy-budgets-said">' + esc(b.message) + '</p>' +
    (kept.length && b.closures.length
      ? '<table class="nw-table sy-budgets-table"><thead><tr><th>group</th><th>closure</th>' +
        kept.map(d => '<th>' + esc(d.version) + '</th>').join('') + '</tr></thead><tbody>' +
        b.closures.map(c => '<tr><td>' + esc(c.group) + '</td><td>' + esc(c.node) + '</td>' +
          kept.map(d => cell(c, d.version)).join('') + '</tr>').join('') +
        '</tbody></table>'
      : '') +
    b.designs.filter(d => !d.kept).map(d => '<p class="muted sy-budgets-before">' + esc(d.says) + '</p>').join('');
}

/** The release panel, read again from the program: the integration checks of
 *  today's design, its red marks, the previews and their answers, and
 *  Release design. */
async function drawRelease() {
  const seq = ++R.seq;
  const c = await get('/v1/design/integration');
  if (seq !== R.seq || W.group !== 'systems') return;
  R.checks = c;
  const at = $('#sy-release', W.host);
  if (!at) return;
  if (!c || c.ok === false) {
    at.innerHTML = '<h3 class="seg-h">release the design</h3><p class="sh-refused sy-r-refused">' +
      esc((c && c.message) || 'the integration checks cannot be read') + '</p>';
    return;
  }
  if (!R.version) R.version = c.next_version || '';
  const lines = c.lines.map(l =>
    '<tr class="' + (l.holds ? 'sy-r-holds' : 'sy-r-fails') + '"><td>' + esc(l.group) + '</td><td>' + esc(l.check) +
    '</td><td><b>' + (l.holds ? 'holds' : 'fails') + '</b></td><td>' + esc(l.says) + '</td></tr>').join('');
  const marks = c.marks.length ? '<ul class="sy-r-marks">' + c.marks.map(m =>
    '<li>' + esc(m.said) + ' ' + (m.raised !== null
      ? '<span class="muted">raised as issue ' + esc(String(m.raised)) + '</span>'
      : '<label><input type="checkbox" class="sy-r-understood" data-mark="' + esc(m.id) + '"' +
        (R.understood.has(m.id) ? ' checked' : '') + '> understood</label>' +
        (m.id.startsWith('closure:') ? ' <a href="#explore/design" class="sy-r-raise">or raise it as an issue, in Explore</a>' : '')) +
    '</li>').join('') + '</ul>' : '<p class="muted sy-r-marks">no red mark: every closure closes, and no issue is open</p>';
  const n = R.notice;
  at.innerHTML = '<h3 class="seg-h">release the design</h3>' +
    answerFirst(esc(c.message), [
      c.released ? 'The design released now is ' + esc(c.released) + '.' : 'No design has been released yet.',
      'Today\'s design, built from the drive ' + esc(c.drive) + '.',
    ], 'reference') +
    '<table class="sy-r-lines"><thead><tr><th>group</th><th>check</th><th></th><th>why</th></tr></thead><tbody>' +
    lines + '</tbody></table>' +
    '<h4 class="seg-h">red marks</h4>' + marks + previews(c) +
    '<div class="sy-r-form"><label>version <input id="sy-r-version" size="12" value="' + esc(R.version) + '"></label> ' +
    '<label>what changed <textarea id="sy-r-changed" rows="2" cols="60">' + esc(R.changed) + '</textarea></label> ' +
    '<button class="ctl" id="sy-r-release"' + (R.sending ? ' disabled' : '') + '>Release design</button></div>' +
    (n ? '<p class="' + (n.ok ? 'sh-said' : 'sh-refused') + ' sy-r-notice">' + esc(n.text) + '</p>' : '');
}

/** The previews and their answers (F6-c), as the integration checks carry
 *  them: each group whose release in today's design is not the one in the
 *  design released now, with Preview where it has none; then every preview,
 *  what moved, and what each group it names has said, unanswered first. What
 *  moved is the engine's, built when Preview is pressed, never here. */
/** What was decided of an objection, or the decisions offered (W7): ask for
 *  its next version, or release anyway; or change an allocation, which is
 *  Reallocate in the systems' branch, its why saying what the server names,
 *  and the systems' file put back to the drive, where it is read. */
function decide(a, p) {
  if (a.decision && a.decision !== 'undecided') {
    return ' <span class="sy-p-decision">decided: ' + esc(a.decision) + '</span>';
  }
  const on = ' data-group="' + esc(p.group) + '" data-version="' + esc(p.version) + '" data-answering="' + esc(a.group) + '"';
  return ' <span class="sy-p-decide"><b>undecided</b>: ' +
    '<button class="ctl" data-decide="next"' + on + (R.deciding ? ' disabled' : '') + '>Ask for its next version</button> ' +
    '<button class="ctl" data-decide="anyway"' + on + (R.deciding ? ' disabled' : '') + '>Release anyway</button>' +
    (a.reallocation_names ? ' or change an allocation: Reallocate, its why saying <code class="sy-p-names">' +
      esc(a.reallocation_names) + '</code>, then Put back the systems\' file' : '') +
    ((a.reallocations_not_counted || []).length ? '<ul class="sy-p-not-counted">' +
      a.reallocations_not_counted.map(w => '<li class="sh-refused">' + esc(w) + '</li>').join('') + '</ul>' : '') + '</span>';
}

function previews(c) {
  const num = (x, why) => x === null || x === undefined ? (why ? 'refused: ' + why : 'not in this design') : String(x);
  const pending = (c.pending || []).map(p =>
    '<li class="sy-p-pending">' + esc(p.group + ' ' + p.version) + ' is in today\'s design, and design ' + esc(c.previews_on) +
    ' holds ' + esc(p.held ? p.group + ' ' + p.held : 'none of ' + p.group) + ': ' +
    (p.preview ? 'previewed in ' + esc(p.preview)
      : '<button class="ctl sy-p-preview" data-preview="' + esc(p.group) + '"' + (R.previewing ? ' disabled' : '') +
        '>Preview ' + esc(p.group + ' ' + p.version) + '</button>') + '</li>').join('');
  const said = (a, p) => '<li class="sy-p-said sy-p-' + esc(a.said) + '"><b>' + esc(a.group) + '</b>: ' + esc(a.said) +
    (a.because ? ', because ' + esc(a.because) : '') + (a.why ? ': ' + esc(a.why) : '') +
    (a.in_copy ? ' <span class="muted">answered in your copy, not put back yet, so not counted</span>' : '') +
    (a.said === 'object' ? decide(a, p) : '') + '</li>';
  const one = p => p.opens === false
    ? '<li class="sh-refused sy-p-refused">' + esc(p.why) + '</li>'
    : '<li class="sy-p-one"><b>' + esc(p.group + ' ' + p.version) + '</b> on design ' + esc(p.based_on) +
      (p.stale ? ' <b>stale</b>: the design released now is ' + esc(c.previews_on || 'none') : '') +
      ', written by ' + esc(p.writer) + ', in ' + esc(p.in) + '. ' +
      (p.named.length ? 'Named to answer: ' + esc(p.named.join(', ')) + '.' : 'It names no subsystem\'s group: what moved is shown, not answered.') +
      '<ul class="sy-p-answers">' + p.answers.map(a => said(a, p)).join('') + '</ul>' +
      '<details><summary>' + esc(String(p.moved.length)) + ' value' + (p.moved.length === 1 ? '' : 's') + ' moved, the engine\'s</summary>' +
      '<table class="sh-table sy-p-moved"><thead><tr><th>value</th><th>group</th><th>design ' + esc(p.based_on) +
      '</th><th>with ' + esc(p.group + ' ' + p.version) + '</th></tr></thead><tbody>' +
      p.moved.map(m => '<tr><td>' + esc(m.value) + '</td><td>' + esc(m.group) + '</td><td>' + esc(num(m.before, m.before_why)) +
        '</td><td>' + esc(num(m.after, m.after_why)) + '</td></tr>').join('') + '</tbody></table></details></li>';
  const n = R.previewed;
  return '<h4 class="seg-h">previews</h4>' +
    (c.previews_said ? '<p class="muted sy-p-none">' + esc(c.previews_said) + '</p>' : '') +
    (pending ? '<ul class="sy-p-pendings">' + pending + '</ul>'
      : (c.previews_said ? '' : '<p class="muted sy-p-none">every release in today\'s design is the one design ' + esc(c.previews_on) + ' holds</p>')) +
    (n ? '<p class="' + (n.ok ? 'sh-said' : 'sh-refused') + ' sy-p-notice">' + esc(n.text) + '</p>' : '') +
    ((c.previews || []).length ? '<ul class="sy-p-list">' + c.previews.map(one).join('') + '</ul>' : '');
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
    '<dt>budgets across releases</dt><dd class="sy-budgets"><p class="sh-loading">reading the released designs…</p></dd>' +
    '<dt>margins today</dt><dd><a href="#explore/design" class="sy-margins-today">the health map, in Explore</a>: ' +
      'every margin as the engine computes it, never computed here</dd></dl></section>';
}

/** Today's view of the branch and its impact (F5-d), once asked for: what the
 *  program built and ran, said as it said it. */
function view() {
  const v = W.view;
  if (!v) return '<p><button class="ctl" id="sy-view">Today\'s view</button> today\'s design with every node of this branch signed as it is in your copy, and every value outside it that moves.</p>';
  if (v.ok === false) return '<p class="sh-refused sy-view">' + esc(v.message || 'today\'s view cannot be built') + '</p>';
  const num = (x, why) => x === null || x === undefined ? 'refused' + (why ? ': ' + why : '') : String(+x.toPrecision(6));
  const ran = (k, r) => '<p>' + k + ': ' + r.ran + ' ran, ' + r.blocked.length + ' blocked' +
    (r.blocked.length ? ' — ' + r.blocked.map(b => esc(b.node + ': ' + b.why)).join('<br>') : '') + '</p>';
  return '<section class="sy-view"><h4>today\'s view</h4><p>' + esc(v.says) + '</p>' +
    '<p>Run on ' + esc(v.case) + '.</p>' +
    '<p>' + (v.changes.length ? 'It changes ' + esc(v.changes.join(', ')) + '.' : 'Nothing signed in your copy changes today\'s design.') + '</p>' +
    (v.left_out.length ? '<p>Left out: ' + v.left_out.map(l => esc(l.node + ' — ' + l.why)).join('<br>') + '</p>' : '') +
    (v.seal_refuses.length ? '<p class="sh-refused">Sealed now, it would be refused: ' + v.seal_refuses.map(esc).join('<br>') + '</p>' : '') +
    ran('Today\'s design', v.today) + ran('The view', v.view) +
    (v.impact.length
      ? '<table class="sh-table sy-impact"><thead><tr><th>outside the branch</th><th>group</th><th>today</th><th>in the view</th></tr></thead><tbody>' +
        v.impact.map(m => '<tr><td>' + esc(m.id) + '</td><td>' + esc(m.group) + '</td><td>' + esc(num(m.before, m.before_why)) + '</td><td>' + esc(num(m.after, m.after_why)) + '</td></tr>').join('') +
        '</tbody></table>'
      : '<p class="muted">No value outside the branch moves.</p>') +
    '<p><button class="ctl" id="sy-view">Build it again</button></p></section>';
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

/** A release that moves this branch's values, answered by its owner (W7):
 *  fine, or object, because…, signed, written in the copy. */
function answerForm() {
  return '<details class="sy-answer"><summary>answer a release that moves ' + esc(W.group) + '\'s values</summary>' +
    '<p><input id="sy-an-group" size="14" placeholder="its group"> <input id="sy-an-version" size="6" placeholder="version"> ' +
    '<select id="sy-an-said" class="ctl"><option value="fine">fine</option><option value="object">object, because…</option></select></p>' +
    '<p><textarea id="sy-an-because" rows="2" class="nw-wide" placeholder="why, for an objection"></textarea></p>' +
    '<p><button class="ctl" id="sy-answer">Answer</button></p></details>';
}

/** The base moved to the next released design (W12): what changed under the
 *  branch is said in its history, for the node engineers who read it. */
function baseForm() {
  return '<p class="sy-base"><b>update base</b> to the released design <input id="sy-base-design" size="12" placeholder="2026.11.1"> ' +
    '<button class="ctl" id="sy-update-base">Update base</button></p>' +
    // An issue addressed to the branch, given to a node engineer (W11).
    '<p class="sy-issue"><b>assign issue</b> <input id="sy-is-number" size="5" placeholder="number"> to ' +
    '<input id="sy-is-person" size="18" placeholder="a node engineer\'s name"> <button class="ctl" id="sy-assign-issue">Assign</button></p>';
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
      '<input id="sy-a-says" size="24" placeholder="what the closure says"> <button class="ctl" id="sy-allocate">Allocate</button> ' +
      '<button class="ctl" id="sy-reallocate" title="another value or range for the allocation named; its sense only when one is chosen">Reallocate</button></p>';
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
  h += '<details class="sy-sheet"><summary>set up many blocks from a sheet</summary>' +
    '<p class="muted">A row a block: id,parent,question,output,type,unit,lower,upper,inputs,assign — inputs as name:unit=from, by ;. Every row or none.</p>' +
    '<p><textarea id="sy-sheet" rows="5" class="nw-wide" placeholder="id,parent,question,output,type,unit,lower,upper,inputs,assign"></textarea></p>' +
    '<p><button class="ctl" id="sy-sheet-go">Set up</button></p></details>';
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
  if (!inGroup && !b.archived && !b.mount) {
    // A contract changes in the group file and issues the node's file again;
    // it reads behind until its node engineer signs it again (W4).
    h += '<p><b>change its contract</b> a port <select id="sy-c-dir" class="ctl">' + opt('out') + opt('in') + '</select> ' +
      '<input id="sy-c-name" size="16" placeholder="its name, or none"> <input id="sy-c-type" size="8" placeholder="type" value="number"> ' +
      '<input id="sy-c-unit" size="6" placeholder="unit"> <input id="sy-c-lo" size="5" placeholder="from"> <input id="sy-c-hi" size="5" placeholder="to"> ' +
      '<button class="ctl" id="sy-contract">Change contract</button></p>';
  }
  if (b.answers.length) h += '<p><button class="ctl" id="sy-fold-back">Fold back</button> its estimate answers again; its children stay.</p>';
  // A node is assigned to one node engineer its group registers; a heading is nobody's.
  if (!b.archived && !b.mount && !(inGroup && b.read_as === 'children')) {
    h += '<p><b>assign</b> to <input id="sy-as-person" size="18" placeholder="a node engineer\'s name"> <button class="ctl" id="sy-assign">Assign</button> <button class="ctl" id="sy-unassign">Unassign</button></p>';
  }
  if (inGroup && !b.archived && !b.mount && b.read_as !== 'children') {
    h += '<p><b>issue</b> it as a <select id="sy-is-behaviour" class="ctl">' + opt('method') + opt('stated') + '</select> node, ' +
      'a file of its own for the node engineer it is assigned to <button class="ctl" id="sy-issue">Issue</button></p>';
  }
  if (inGroup && !b.archived) {
    h += b.ports.filter(p => p.direction === 'in').map(p =>
      '<p class="sy-wire-row" data-wire-port="' + esc(p.name) + '"><b>wire</b> its ' + esc(p.name) + ' from ' +
      '<input class="sy-wire-from" size="28" placeholder="block.port, group.node.port or case"> ' +
      '<button class="ctl sy-wire" type="button">Wire</button></p>').join('');
  }
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
  if (t.dataset.decide) {
    if (R.deciding) return;
    R.deciding = true;
    t.disabled = true;
    let r;
    try {
      r = await W.post('/v1/design/decide', {
        group: t.dataset.group, version: t.dataset.version, answering: t.dataset.answering, decision: t.dataset.decide,
      });
    } finally { R.deciding = false; }
    R.previewed = { ok: !!(r && r.ok), text: (r && r.message) || 'nothing was decided' };
    return drawRelease();
  }
  if (t.dataset.preview) {
    if (R.previewing) return;
    R.previewing = true;
    t.disabled = true;
    t.textContent = 'building both designs…';
    let r;
    try {
      r = await W.post('/v1/design/preview', { group: t.dataset.preview });
    } finally { R.previewing = false; }
    R.previewed = { ok: !!(r && r.ok), text: (r && r.message) || 'nothing was written' };
    return drawRelease();
  }
  if (t.dataset.block) {
    W.why = v('sy-why');
    W.sel = t.dataset.block;
    return draw(true);
  }
  const b = W.sel;
  if (t.classList.contains('sy-wire')) {
    const row = t.closest('.sy-wire-row');
    return send('wire', { block: b, port: row.dataset.wirePort, from: row.querySelector('.sy-wire-from').value.trim() });
  }
  switch (t.id) {
    case 'sy-r-release': {
      if (R.sending) return;
      R.sending = true;
      t.disabled = true;
      R.version = v('sy-r-version').trim();
      R.changed = v('sy-r-changed');
      R.understood = new Set([...W.host.querySelectorAll('.sy-r-understood:checked')].map(c => c.dataset.mark));
      let r;
      try {
        r = await W.post('/v1/design/release', { version: R.version, changed: R.changed, understood: [...R.understood].join(',') });
      } finally { R.sending = false; }
      R.notice = { ok: !!(r && r.ok), text: r && r.ok ? r.message + ' ' + r.announce : (r && r.message) || 'nothing was released' };
      if (r && r.ok) { R.version = ''; R.changed = ''; R.understood = new Set(); }
      return drawRelease();
    }
    case 'sy-go': { const id = v('sy-id').trim(); if (id) { openAt(id); return draw(); } return; }
    case 'sy-reopen': W.notice = null; return draw();
    case 'sy-view': {
      const group = W.group;
      const r = await get('/v1/branch/view?group=' + encodeURIComponent(group));
      if (group !== W.group) return;
      W.view = r;
      return draw(true);
    }
    case 'sy-answer': {
      const r = await W.post('/v1/answer', {
        group: v('sy-an-group').trim(), version: v('sy-an-version').trim(), answering: W.group,
        said: v('sy-an-said'), because: v('sy-an-because'),
      });
      W.notice = { ok: !!(r && r.ok), text: (r && r.message) || 'nothing was answered' };
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
    case 'sy-mount': return sendFrame('mount', { block: b, mounts: v('sy-m-group').trim(), owner: v('sy-m-owner').trim() });
    case 'sy-reallocate': return sendFrame('reallocate', {
      block: b, name: v('sy-a-name').trim(), value: v('sy-a-value').trim(), lower: v('sy-a-lo').trim(),
      upper: v('sy-a-hi').trim(), sense: v('sy-a-sense'),
    });
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
    case 'sy-contract': return send('contract', {
      node: b, direction: v('sy-c-dir'), name: v('sy-c-name').trim(), type: v('sy-c-type').trim(),
      unit: v('sy-c-unit').trim(), lower: v('sy-c-lo').trim(), upper: v('sy-c-hi').trim(), state: 'decided',
    });
    case 'sy-assign-issue': return send('assign-issue', { number: v('sy-is-number').trim(), person: v('sy-is-person').trim() });
    case 'sy-update-base': return send('update-base', { design: v('sy-base-design').trim() });
    case 'sy-sheet-go': return send('sheet', { sheet: v('sy-sheet') });
    case 'sy-issue': return send('issue', { block: b, behaviour: v('sy-is-behaviour') });
    case 'sy-assign': return send('assign', { block: b, person: v('sy-as-person').trim() });
    case 'sy-unassign': return send('unassign', { block: b });
  }
}
