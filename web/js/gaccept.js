/*
  THE GROUP'S ANSWER TO A TEST APPLICATION.

  The developer builds the tool with the group's sealed release in it and
  sends it as a folder: the programs, DELIVERY.toml (which release, which seal,
  which commit, which nodes, how many checks held) and DELIVERY.md (what to
  try). The group tries it. Then the subsystem engineer opens DELIVERY.toml here, beside
  their sealed release, and answers: ACCEPTED, or CHANGES with what they saw.

  The answer is a small file, <group>-<version>.accept.toml, which the subsystem engineer
  sends back. It names the delivery by its SHA-256 and the commit it was built
  from, so it accepts one build and nothing else: `cargo run -p xtask --
  group-accept <file>` refuses it against a branch that has changed since.
*/
'use strict';

import { $, esc } from './dom.js';
import { sha256 } from './gseal.js';
import { download } from './gfolder.js';

/** The flat TOML a delivery record is written in: key = "string" | number | bool | ["a", "b"]. */
export function readDelivery(text) {
  const out = {};
  for (const raw of String(text).split('\n')) {
    const line = raw.trim();
    if (!line || line.startsWith('#')) continue;
    const m = /^([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.+)$/.exec(line);
    if (!m) continue;
    const v = m[2].trim();
    try {
      out[m[1]] = v.startsWith('[') || v.startsWith('"') ? JSON.parse(v) : v === 'true' ? true : v === 'false' ? false : Number(v);
    } catch { out[m[1]] = v; }
  }
  return out;
}

/** The acceptance file: the delivery it answers, by hash and commit, and the answer. */
export function acceptance(d, deliveryText, a) {
  const q = s => JSON.stringify(String(s || ''));
  return '# The group\'s answer to a test application — written by the group application.\n' +
    '# Send it to the developer: `cargo run -p xtask -- group-accept <this file>` records it.\n' +
    'group = ' + q(d.group) + '\nversion = ' + q(d.version) + '\nfingerprint = ' + q(d.fingerprint) + '\n' +
    'commit = ' + q(d.commit) + '\ndelivery = ' + q(sha256(deliveryText)) + '\n' +
    'verdict = ' + q(a.verdict) + '\nby = ' + q(a.by) + '\nat = ' + q(a.at) + '\n' +
    'tried = ' + q(a.tried) + '\nnote = ' + q(a.note) + '\n';
}

/** The page: open DELIVERY.toml, see what it holds, answer. */
export function deliveryPage(ctx) {
  const m = ctx.model.meta;
  const people = (ctx.model.group.members || []).map(p => p.name).filter(Boolean);
  return '<h1 class="gh1">Delivery &amp; acceptance</h1>' +
    '<p>The developer sends a <b>test application</b>: the tool, built with your sealed release in it. Try it as <code>DELIVERY.md</code> in its folder says — ' +
    'every node, at values you know the answers to. Then open its <code>DELIVERY.toml</code> here and give the group\'s answer.</p>' +
    '<p><label>The delivery record <input type="file" id="gd-file" accept=".toml"></label></p>' +
    '<div id="gd-show"></div>' +
    '<div id="gd-answer" hidden>' +
    '<h2 class="gh">The group\'s answer</h2>' +
    '<p><label>Who answers <select id="gd-by">' + ['', ...people].map(p => '<option' + (p === ctx.me ? ' selected' : '') + '>' + esc(p) + '</option>').join('') + '</select></label> ' +
    '<span class="muted small">the person who sealed ' + esc(m.id || 'the group') + ', or its owner</span></p>' +
    '<p><label><input type="radio" name="gd-v" value="accepted"> <b>Accepted</b> — every node gives what we expect, and reads as we wrote it</label><br>' +
    '<label><input type="radio" name="gd-v" value="changes"> <b>Changes</b> — something is wrong; say what below</label></p>' +
    '<p><label>What we tried<br><textarea id="gd-tried" rows="3" class="gnote" placeholder="e.g. every node at our own results; orbit_period at 150, 300 and 450 km"></textarea></label></p>' +
    '<p><label>Note — required for changes<br><textarea id="gd-note" rows="3" class="gnote" placeholder="What you saw, where, and what you expected"></textarea></label></p>' +
    '<button class="ctl gbtn" id="gd-go" type="button">Write the answer</button>' +
    '<p id="gd-out" class="gout" aria-live="polite"></p></div>';
}

export function wireDelivery(ctx) {
  let d = null, text = '';
  const release = ctx.file && ctx.file.db ? { fingerprint: ctx.file.db.meta('fingerprint') || '', sealed: ctx.file.db.meta('sealed') || '' } : { fingerprint: '', sealed: '' };
  $('#gd-file').addEventListener('change', async e => {
    const f = e.target.files[0];
    if (!f) return;
    text = await f.text();
    d = readDelivery(text);
    const m = ctx.model.meta;
    const problems = [];
    if (d.group !== m.id) problems.push('it is a delivery of <b>' + esc(d.group) + '</b>, and this is <b>' + esc(m.id) + '</b>');
    if (String(d.version) !== String(m.version)) problems.push('it holds version ' + esc(d.version) + ', and this is version ' + esc(m.version));
    if (!release.sealed) problems.push('the file open here is not a sealed release: open the release you sealed, so the delivery can be checked against it');
    else if (d.fingerprint !== release.fingerprint) problems.push('its fingerprint is not your sealed release\'s: it was built from other files than the ones you signed');
    if (d.uncommitted) problems.push('it was built from uncommitted changes: a throwaway build, not one to accept');
    if (Number(d.checks_failed || 0) > 0) problems.push(esc(d.checks_failed) + ' of its checks failed');
    const built = Array.isArray(d.built) ? d.built : [];
    $('#gd-show').innerHTML = '<div class="ri-wrap"><table class="fx gtable"><tbody>' +
      [['Release', esc(d.group) + ' ' + esc(d.version)], ['Sealed', esc(d.sealed)], ['Fingerprint', '<code>' + esc(String(d.fingerprint || '').slice(0, 16)) + '…</code>'],
        ['Built from commit', '<code>' + esc(String(d.commit || '').slice(0, 10)) + '</code>'], ['Tool', esc(d.tool)],
        ['Built from your pseudocode', built.length ? built.map(b => '<code>' + esc(b) + '</code>').join(', ') : '<span class="muted">none</span>'],
        ['Checks', esc(d.checks_held) + ' held, ' + esc(d.checks_failed || 0) + ' failed']]
        .map(([k, v]) => '<tr><th>' + k + '</th><td>' + v + '</td></tr>').join('') + '</tbody></table></div>' +
      (problems.length ? '<div class="gcard gwarn"><p><b>This delivery cannot be accepted:</b></p><ul>' + problems.map(p => '<li>' + p + '</li>').join('') +
        '</ul><p>You can still answer <b>changes</b>, saying so.</p></div>' : '<p class="gsave-ok">It is a build of exactly the release you sealed.</p>');
    $('#gd-answer').hidden = false;
    const acc = document.querySelector('input[name="gd-v"][value="accepted"]');
    acc.disabled = problems.length > 0;
    if (acc.disabled) acc.checked = false;
  });
  $('#gd-go').addEventListener('click', () => {
    const out = $('#gd-out');
    const v = (document.querySelector('input[name="gd-v"]:checked') || {}).value;
    const by = $('#gd-by').value.trim(), note = $('#gd-note').value.trim(), tried = $('#gd-tried').value.trim();
    if (!d) { out.textContent = 'Open the delivery record first.'; return; }
    if (!by) { out.textContent = 'Say who answers.'; return; }
    if (!v) { out.textContent = 'Choose accepted or changes.'; return; }
    if (v === 'changes' && !note) { out.textContent = 'Say what must change: the note goes to the developer.'; return; }
    if (v === 'accepted' && !tried) { out.textContent = 'Say what you tried: an acceptance says what it rests on.'; return; }
    const name = d.group + '-' + d.version + '.accept.toml';
    const body = acceptance(d, text, { verdict: v, by, at: new Date().toISOString(), tried, note });
    download(name, new Blob([body], { type: 'application/toml' }));
    out.innerHTML = esc(name) + ' was downloaded — send it to the developer. ' +
      (v === 'accepted' ? 'Once recorded, this exact build can be merged and released.' : 'They take your note back into the work; your next release comes back to you as a new test application.');
  });
}
