/*
  The application's shell (docs/briefs/F1.md, PR 3): the five workspaces, and
  My work, which it opens on.

  My work holds, for now, what F1 can know: the copy of the database, the key
  and who it says you are, what needs you (a node of yours unsigned, a group of
  yours with nothing sealed, a file below you the chain refuses, a request on a
  node of a group of yours), and People, for whoever holds a valve. Explore
  (F2) holds today's design, its runs, its inputs and the results kept, loaded
  the first time it is opened (workspace_explore.js). The other three
  workspaces say which stage brings
  them.

  The launch's word arrives in the address's fragment, is kept in this tab's
  session storage and taken out of the address at once, so it is never in the
  history, a bookmark or a screen shared. Every request that opens or uses a
  key, or writes the copy, carries it.

  The anchor is never accepted for the person: the fingerprint START HERE
  names is shown, and the person types the one they were given. The program
  compares the two.
*/
'use strict';

import { $, esc, answerFirst, plural } from './dom.js';

const POST = { method: 'POST', headers: { 'content-type': 'application/x-www-form-urlencoded' } };
const WORD = 'vleo-session';

/** The workspaces, in the order the shell shows them, and the stage that fills each. */
const WORKSPACES = [
  ['my-work', 'My work', 'everything that needs you now', ''],
  ['node', 'Node', 'one node, edited in its live page, signed when it is ready', 'F3'],
  ['system', 'System', 'your branch: its valves, today\'s view of it, and its release', 'F4 to F6'],
  ['programme', 'Programme', 'mission health, gates, risks, the organisation and decisions', 'F7'],
  ['explore', 'Explore', 'today\'s design or any released one: run, sweep, compare, trace to cause', ''],
];

const S = {
  ws: 'my-work',
  me: null, copy: null, keys: null, work: null, people: null, pfile: '',
  anchor: null,   // { fingerprint, drive } while a first use waits to be accepted
  pending: null,  // { file, passphrase, at } held only until the anchor is accepted or dropped, two minutes at most
  handover: null, // { person, public_key, proof } of a key that opened but is registered for nobody
  made: null,     // the key just made
  notice: null,   // { ok, text }
  drawn: 0,       // how many times the page was drawn, for a check to wait on
};

/** How long a passphrase is held while a first use waits to be accepted. */
const PENDING_MS = 2 * 60 * 1000;

/** The passphrase dropped, and the first use with it. */
function dropPending() {
  S.pending = null;
  S.anchor = null;
}

// ---------------------------------------------------------------------------
// the launch's word, and asking the program

function takeWord() {
  const m = location.hash.match(/(?:^#|&)session=([0-9a-f]+)/);
  if (!m) return;
  try { sessionStorage.setItem(WORD, m[1]); } catch (e) { /* a tab that keeps nothing asks again */ }
  history.replaceState(null, '', location.pathname + location.search);
}

function word() {
  try { return sessionStorage.getItem(WORD) || ''; } catch (e) { return ''; }
}

async function get(path) {
  try { return await (await fetch(path)).json(); } catch (e) { return { ok: false, message: 'the program did not answer: ' + e }; }
}

async function post(path, fields) {
  const body = new URLSearchParams({ session: word(), ...fields });
  try { return await (await fetch(path, { ...POST, body: body.toString() })).json(); } catch (e) {
    return { ok: false, message: 'the program did not answer: ' + e };
  }
}

/** The files whose people this person may change: the programme's for its manager, and each branch they own. */
function valves(me) {
  if (!me || !me.open) return [];
  const out = [];
  if ((me.roles || []).some(r => r.role === 'programme manager' && r.in === 'the programme\'s file')) out.push('programme');
  for (const g of me.owns || []) out.push(g);
  return out;
}

async function refresh() {
  const [me, copy, keys, work] = await Promise.all([
    get('/v1/me'), get('/v1/copy'), get('/v1/keys'), get('/v1/my-work'),
  ]);
  Object.assign(S, { me, copy, keys, work });
  const files = valves(me);
  if (!files.includes(S.pfile)) S.pfile = files[0] || '';
  S.people = S.pfile ? await get('/v1/people?file=' + encodeURIComponent(S.pfile)) : null;
  draw();
}

// ---------------------------------------------------------------------------
// drawing

function draw() {
  const me = S.me;
  $('#sh-who').innerHTML = me && me.open
    ? esc(me.name) + ' · ' + esc((me.roles || []).map(r => r.role).join(', '))
    : 'no key open';
  $('#sh-nav').innerHTML = WORKSPACES.map(([id, name, holds]) =>
    '<button class="tab' + (S.ws === id ? ' sel' : '') + '" data-ws="' + id + '" title="' + esc(holds) + '">' +
    esc(name) + '</button>').join('');
  const n = S.notice;
  $('#sh-notice').innerHTML = n ? '<p class="' + (n.ok ? 'sh-said' : 'sh-refused') + '">' + esc(n.text) + '</p>' : '';
  if (S.ws !== 'explore') {
    $('#sh-main').innerHTML = S.ws === 'my-work' ? myWork() : empty(S.ws);
  } else if (!$('#sh-explore')) {
    // Drawn once on the way in, and left alone by every later draw: Explore
    // keeps its own state, and asking the engine again on each notice would
    // lose it.
    $('#sh-main').innerHTML = '<div id="sh-explore"></div>';
    openExplore($('#sh-explore'));
  }
  document.body.dataset.drawn = String(++S.drawn);
}

function empty(id) {
  const [, name, holds, stage] = WORKSPACES.find(w => w[0] === id);
  return answerFirst('The ' + esc(name) + ' workspace arrives in ' + esc(stage) + '.',
    ['It will hold ' + esc(holds) + '.'], 'reference');
}

/** Where `#explore/<view>[/<result>]` asked Explore to open, until it does. */
let exploreAt = null;

/** The Explore workspace, loaded the first time it is opened. */
async function openExplore(host) {
  try {
    const { mountExplore, openAt } = await import('./workspace_explore.js');
    if (exploreAt) { openAt(exploreAt.view, exploreAt.file); exploreAt = null; }
    await mountExplore(host, raiseIssue);
  } catch (e) {
    host.innerHTML = '<p class="sh-refused">Explore did not load: ' + esc(String(e)) + '</p>';
  }
}

/** Ask the program to write an issue: the one request Explore makes that
 *  carries the launch's word. */
async function raiseIssue(params) {
  params.set('session', word());
  try {
    return await (await fetch('/v1/issues/raise', { ...POST, body: params.toString() })).json();
  } catch (e) {
    return { ok: false, message: 'the program did not answer: ' + e };
  }
}

function myWork() {
  const copied = S.copy && S.copy.copied;
  const open = S.me && S.me.open;
  const items = (S.work && S.work.items) || [];
  // An answer is given only from what was read: a list that could not be read
  // is never told as an empty one.
  const unread = open && (!S.work || S.work.ok === false || S.work.message);
  const answer = S.copy && S.copy.unreadable
    ? 'Your copy of the database cannot be read, so what needs you cannot be read either: ' + esc(S.copy.message)
    : !copied
    ? 'Copy the database from the drive to start: you work on that copy, and your own signed work goes back.'
    : !open
      ? 'Open your key: who you are, and what needs you, is read from the database\'s files.'
      : unread
        ? esc((S.work && S.work.message) || 'What needs you cannot be read: the program did not answer.') +
          (items.length ? ' What could be read is below.' : '')
        : items.length ? plural(items.length, 'thing needs', 'things need') + ' you now.' : 'Nothing needs you now.';
  return answerFirst(answer, [], 'reference') +
    (open ? workList(items) : '') +
    keyCard() + copyCard() + (open ? peopleCard() : '');
}

function workList(items) {
  if (!items.length) return '';
  return '<section class="sh-card"><h2>What needs you ' + kind('reference') + '</h2><ul id="sh-items">' +
    items.map(it => '<li class="sh-item" data-kind="' + esc(it.kind) + '"><span class="tag">' + esc(it.kind) +
      '</span> ' + esc(it.says) + '</li>').join('') + '</ul></section>';
}

function copyCard() {
  const c = S.copy || {};
  const state = c.copied
    ? '<p>Your copy is at <code>' + esc(c.copy) + '</code>, taken from <code>' + esc(c.drive) + '</code> at ' +
      esc(c.copied_at) + '.</p><p>' + plural((c.changed_here || []).length, 'file') + ' changed here since; ' +
      (c.drive_reached ? plural((c.changed_on_drive || []).length, 'file') + ' changed on the drive.' : 'the drive is not reached.') + '</p>'
    : '<p>' + esc(c.message || 'No copy of the database yet.') + '</p>';
  return '<section class="sh-card" id="sh-copy-card"><h2>The database ' + kind('how-to') + '</h2>' + state +
    '<p><label>The drive folder <input id="sh-drive" class="sh-wide" placeholder="the database folder on the shared drive" value="' +
    esc(c.drive || '') + '"></label> <button class="ctl" id="sh-copy">Copy from the drive</button></p></section>';
}

function keyCard() {
  const me = S.me || {};
  if (me.open) {
    return '<section class="sh-card" id="sh-key-card"><h2>Your key ' + kind('reference') + '</h2><p>Open: <b>' + esc(me.name) + '</b>, ' +
      esc((me.roles || []).map(r => r.role + ' in ' + r.in).join('; ')) + '.</p>' +
      '<p class="sh-fp">Fingerprint <code>' + esc(me.fingerprint) + '</code></p>' +
      (me.warning ? '<p class="sh-refused">' + esc(me.warning) + '</p>' : '') +
      '<p><button class="ctl" id="sh-close">Close your key</button></p></section>';
  }
  const keys = (S.keys && S.keys.keys) || [];
  const anchor = S.anchor
    ? '<div class="sh-anchor" id="sh-anchor-ask"><p>This computer has not opened the drive <code>' + esc(S.anchor.drive) +
      '</code> before. Its START HERE names a programme manager\'s key whose fingerprint starts</p><p class="sh-fp"><code>' +
      esc(S.anchor.fingerprint.slice(0, 8)) + '…</code></p><p>Type the whole fingerprint the programme manager gave you: ' +
      'it is not shown here, so what is compared is what you were given. If the two agree, ' +
      'this computer remembers it for that drive.</p><p><input id="sh-anchor" class="sh-wide" autocomplete="off" spellcheck="false"> ' +
      '<button class="ctl" id="sh-accept">Accept</button> <button class="ctl" id="sh-drop">Not now</button></p></div>'
    : '';
  const handover = S.handover
    ? '<div class="sh-handover" id="sh-handover"><p>Give these to whoever registers you, the programme manager or your subsystem engineer, ' +
      'with your name exactly as it is here: <b id="sh-person">' + esc(S.handover.person) + '</b>.</p>' +
      '<p>Public key <code id="sh-pub">' + esc(S.handover.public_key) + '</code></p><p>Proof <code id="sh-proof">' +
      esc(S.handover.proof) + '</code></p></div>'
    : '';
  const made = S.made
    ? '<div class="sh-handover" id="sh-made"><p>' + esc(S.made.message) + '</p><p>Public key <code>' + esc(S.made.public_key) +
      '</code></p><p>Proof <code>' + esc(S.made.proof) + '</code></p></div>'
    : '';
  return '<section class="sh-card" id="sh-key-card"><h2>Your key ' + kind('how-to') + '</h2>' +
    (keys.length
      ? '<p><label>Key <select id="sh-keyfile" class="ctl">' + keys.map(k =>
          '<option value="' + esc(k.file) + '">' + esc(k.person) + ' · ' + esc(k.file) + '</option>').join('') +
        '</select></label> <label>Passphrase <input id="sh-pass" type="password" autocomplete="current-password"></label> ' +
        '<button class="ctl" id="sh-open">Open</button></p>'
      : '<p>No key on this computer yet: make yours below.</p>') +
    anchor + handover +
    '<details id="sh-make-box"' + (keys.length ? '' : ' open') + '><summary>Make a key</summary>' +
    '<p>Your key is made and kept on this computer, locked by a passphrase of ten characters or more, never on the drive.</p>' +
    '<p><label>Your name, as the files will register it <input id="sh-mkname"></label></p>' +
    '<p><label>Passphrase <input id="sh-mkpass" type="password" autocomplete="new-password"></label> ' +
    '<label>Again <input id="sh-mkpass2" type="password" autocomplete="new-password"></label> ' +
    '<button class="ctl" id="sh-make">Make the key</button></p>' + made + '</details></section>';
}

function peopleCard() {
  const files = valves(S.me);
  if (!files.length) return '';
  const p = S.people || {};
  const rows = (p.people || []).map(person =>
    (person.keys.length ? person.keys : [{ fingerprint: '', registered_at: '', revoked_from: '', public_key: '' }]).map(k =>
      '<tr><td>' + esc(person.name) + '</td><td>' + esc(person.role) + (person.deputy_for ? ', deputy for ' + esc(person.deputy_for) : '') +
      '</td><td><code>' + esc((k.fingerprint || '').slice(0, 16)) + '</code></td><td>' + esc(k.registered_at) + '</td><td>' +
      (k.revoked_from ? 'revoked from ' + esc(k.revoked_from)
        : (k.public_key && k.proof_holds === false ? 'no proof: it signs nothing; replace it with a new key. ' : '') +
          (k.public_key && person.name !== S.me.name
            ? '<button class="ctl" data-revoke="' + esc(person.name) + '" data-key="' + esc(k.public_key) + '">Revoke from today</button>' : '')) +
      '</td></tr>').join('')).join('');
  const roles = S.pfile === 'programme'
    ? ['system engineer', 'subsystem engineer', 'programme manager'] : ['node engineer'];
  return '<section class="sh-card" id="sh-people"><h2>People ' + kind('how-to') + '</h2>' +
    '<p>Whom a file registers is changed here, in your copy, signed with your key. It stays in your copy for now: a group\'s file does not go back to the drive yet.</p>' +
    '<p><label>File <select id="sh-pfile" class="ctl">' + files.map(f =>
      '<option' + (f === S.pfile ? ' selected' : '') + '>' + esc(f) + '</option>').join('') + '</select></label></p>' +
    (p.ok === false ? '<p class="sh-refused">' + esc(p.message) + '</p>' : '') +
    '<table class="sh-table"><thead><tr><th>name</th><th>role</th><th>key</th><th>from</th><th></th></tr></thead><tbody>' +
    rows + '</tbody></table>' +
    '<h3>Register someone</h3><p>They give you their public key and its proof, made with their key.</p>' +
    '<p><label>Name <input id="sh-rname"></label> <label>Role <select id="sh-rrole" class="ctl">' +
    roles.map(r => '<option>' + esc(r) + '</option>').join('') + '</select></label> ' +
    '<label>Deputy for <input id="sh-rdeputy" placeholder="nobody"></label></p>' +
    '<p><label>Public key <input id="sh-rkey" class="sh-wide"></label></p>' +
    '<p><label>Proof <input id="sh-rproof" class="sh-wide"></label> <button class="ctl" id="sh-register">Register</button></p>' +
    '</section>';
}

/** The kind of reading a block is (docs/EXPLAINING.md, E8). */
function kind(k) {
  return '<span class="dx dx-' + esc(k) + '">' + esc(k) + '</span>';
}

// ---------------------------------------------------------------------------
// what a click does

function say(r, fallback) {
  S.notice = { ok: !!(r && r.ok), text: (r && r.message) || fallback };
}

async function openKey(file, passphrase, acceptAnchor) {
  const fields = { file, passphrase };
  if (acceptAnchor) fields.accept_anchor = acceptAnchor;
  const r = await post('/v1/key/open', fields);
  S.handover = null;
  if (r.ok) {
    dropPending();
    say({ ok: true, message: 'Your key is open: ' + r.name + '.' });
  } else if (r.anchor) {
    S.anchor = { fingerprint: r.anchor, drive: r.drive || '' };
    S.pending = { file, passphrase, at: Date.now() };
    S.notice = acceptAnchor
      ? { ok: false, text: 'What you typed is not the fingerprint START HERE names: nothing is accepted. Ask the programme manager.' }
      : null;
  } else {
    dropPending();
    if (r.public_key) S.handover = { person: r.person || '', public_key: r.public_key, proof: r.proof };
    say(r, 'the key did not open');
  }
  await refresh();
}

async function click(e) {
  const t = e.target.closest('button');
  if (!t) return;
  if (t.dataset.ws) { S.ws = t.dataset.ws; S.notice = null; dropPending(); draw(); return; }
  switch (t.id) {
    case 'sh-copy': {
      const r = await post('/v1/copy', { drive: $('#sh-drive').value.trim() });
      say(r, 'the database was not copied');
      return refresh();
    }
    case 'sh-open':
      return openKey($('#sh-keyfile').value, $('#sh-pass').value);
    case 'sh-accept': {
      // What the person typed, never what the program showed.
      const typed = $('#sh-anchor').value.replace(/\s+/g, '').toLowerCase();
      if (!S.pending || Date.now() - S.pending.at > PENDING_MS) {
        dropPending();
        S.notice = { ok: false, text: 'The passphrase is no longer held: open your key again.' };
        return draw();
      }
      return openKey(S.pending.file, S.pending.passphrase, typed || 'nothing typed');
    }
    case 'sh-drop':
      dropPending(); S.notice = null;
      return draw();
    case 'sh-close':
      await post('/v1/key/close', {});
      S.notice = { ok: true, text: 'Your key is closed.' };
      return refresh();
    case 'sh-make': {
      if ($('#sh-mkpass').value !== $('#sh-mkpass2').value) {
        S.notice = { ok: false, text: 'The two passphrases differ: nothing is made.' };
        return draw();
      }
      const r = await post('/v1/key/make', { name: $('#sh-mkname').value, passphrase: $('#sh-mkpass').value });
      S.made = r.ok ? r : null;
      say(r, 'the key was not made');
      return refresh();
    }
    case 'sh-register': {
      const r = await post('/v1/people/register', {
        file: S.pfile, name: $('#sh-rname').value.trim(), role: $('#sh-rrole').value,
        deputy_for: $('#sh-rdeputy').value.trim(), public_key: $('#sh-rkey').value.trim(), proof: $('#sh-rproof').value.trim(),
      });
      say(r, 'nobody was registered');
      return refresh();
    }
  }
  if (t.dataset.revoke) {
    const today = new Date().toISOString().slice(0, 10);
    if (!window.confirm('Revoke ' + t.dataset.revoke + '\'s key from ' + today +
        '? Signatures made with it from today stop counting.')) return;
    const r = await post('/v1/people/revoke', { file: S.pfile, name: t.dataset.revoke, public_key: t.dataset.key, from: today });
    say(r, 'nothing was revoked');
    return refresh();
  }
}

async function change(e) {
  if (e.target.id === 'sh-pfile') { S.pfile = e.target.value; await refresh(); }
}

// ---------------------------------------------------------------------------
// boot

takeWord();
// Today's tool page sends its inputs and its results kept here.
{
  const m = location.hash.match(/^#explore\/(\w+)(?:\/(.+))?$/);
  if (m) {
    S.ws = 'explore';
    // A name that does not decode opens Explore at its view, at no result:
    // never a page that does not start.
    let file = '';
    try { file = m[2] ? decodeURIComponent(m[2]) : ''; } catch (e) { file = ''; }
    exploreAt = { view: m[1], file };
    history.replaceState(null, '', location.pathname + location.search);
  }
}
document.addEventListener('click', click);
document.addEventListener('change', change);
// The passphrase is held two minutes at most: long enough to find the
// fingerprint one was given, in another tab or on paper, and no longer.
setInterval(() => {
  if (S.pending && Date.now() - S.pending.at > PENDING_MS) {
    dropPending();
    S.notice = { ok: false, text: 'Two minutes passed, so the passphrase is no longer held: open your key again to accept the anchor.' };
    draw();
  }
}, 10000);
refresh();
