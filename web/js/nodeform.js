/*
  The node's form: download it, and check one that has come back.

  A node is changed by people who are not all developers, so each node can be
  downloaded as one HTML file that explains itself and asks every question its
  sheet answers — the same questions, from the same list, as the form beside
  this one. It is filled anywhere, offline, by hand or with an assistant, and
  the filled file goes to the developers.

  THIS PAGE ONLY CHECKS. It shows what a filled form would change, what it
  cannot — the node changed since the form was made, or an assistant supplied
  the relation — and the command a developer applies it with. The node itself
  is changed at a terminal, reviewed, and released like any other change, so a
  form cannot move the design behind anybody's back.
*/
'use strict';

import { $, esc } from './dom.js';

const POST = { method: 'POST', headers: { 'content-type': 'application/x-www-form-urlencoded' } };

export function mountNodeForm(host, id) {
  const file = id + '.node-form.html';
  host.innerHTML =
    '<p>One file, for anyone who should say what this node is: it explains itself, asks every ' +
    'question the sheet answers with <i>why</i> each is asked, shows what the node reads and ' +
    'feeds and the known values that already hold it, and saves a filled copy of itself. It ' +
    'needs no connection and nothing installed.</p>' +
    '<ol class="nform-steps">' +
      '<li><a class="ctl nform-dl" href="/v1/form/' + encodeURIComponent(id) + '" download="' +
        esc(file) + '">download the node form</a></li>' +
      '<li>Fill it in a browser — or give it to an assistant to fill; the content is a plain ' +
        'TOML block. Press <b>save a filled copy</b>.</li>' +
      '<li>Send the saved file to the developer team. They apply it with ' +
        '<code>cargo run -p xtask -- intake ' + esc(file) + ' --apply</code>, and the next ' +
        'release carries it.</li>' +
    '</ol>' +
    '<p><label class="ctl nform-up-l">check a filled form…<input type="file" class="nform-up" ' +
      'accept=".html,text/html" hidden></label> <span class="muted">shows what it would ' +
      'change. Nothing is written here.</span></p>' +
    '<div class="nform-out"></div>';
  $('.nform-up', host).onchange = async e => {
    const f = e.target.files && e.target.files[0];
    e.target.value = '';
    if (f) await checkForm($('.nform-out', host), f.name, await f.text());
  };
}

/**
 * A returned form, checked: what it would change, every interface it declares,
 * what is still open, and the command a developer applies it with. Writes
 * nothing. Shared by a node's page and the Forms page.
 */
export async function checkForm(out, name, html) {
  out.innerHTML = '<p class="muted">reading ' + esc(name) + '…</p>';
  let r;
  try {
    r = await (await fetch('/v1/form/check', { ...POST, body: new URLSearchParams({ html }).toString() })).json();
  } catch (e) {
    r = { ok: false, message: 'the engine did not answer: ' + e };
  }
  if (!r.ok) {
    out.innerHTML = '<div class="blocked"><b>This file cannot be read as a node form.</b><div>' +
      esc(r.message || '') + '</div></div>';
    return;
  }
  const by = r.filled_by;
  const tag = { apply: 'will apply', already: 'already so', conflict: 'CONFLICT', refused: 'REFUSED' };
  const what = r.new
    ? 'A <b>new node</b>, <code>' + esc(r.new.id || '(no id)') + '</code>, under <code>' +
      esc(r.new.parent || '(no group)') + '</code>, a ' + esc(r.new.kind) + ' row'
    : 'For <code>' + esc(r.node) + '</code>';
  let h = '<div class="nform-file"><h4>' + esc(name) + '</h4><p>' + what + ', ' +
    'filled by <b>' + esc(by.name || 'nobody named') + '</b>' + (by.team ? ' (' + esc(by.team) + ')' : '') +
    (by.date ? ', ' + esc(by.date) : '') + '; assistant: ' + esc(by.ai) + '. ' +
    (r.new ? 'The developers build it in its place in the tree when they apply it.'
      : r.base_current ? 'The node is still the version the form was made from.'
      : '<b>The node has changed since the form was made</b> — a change to the same thing is a conflict.') +
    '</p>';
  h += r.items.length
    ? '<div class="ri-wrap"><table class="fx"><thead><tr><th></th><th>what</th><th>now</th><th>the form says</th>' +
      '</tr></thead><tbody>' + r.items.map(i =>
        '<tr class="nform-' + esc(i.verdict) + '"><td>' + esc(tag[i.verdict] || i.verdict) + '</td><td><code>' +
        esc(i.what) + '</code></td><td>' + esc(i.from) + '</td><td>' + esc(i.to) +
        (i.why ? '<div class="nform-why">' + esc(i.why) + '</div>' : '') + '</td></tr>').join('') +
      '</tbody></table></div>'
    : '<p>The form changes nothing in the node.</p>';
  if (r.interfaces && r.interfaces.length) {
    h += '<h4>Interfaces — what each input reads</h4><div class="ri-wrap"><table class="fx"><tbody>' +
      r.interfaces.map(i => '<tr class="nform-' + (i.why ? 'refused' : 'apply') + '"><td>' +
        (i.why ? 'REFUSED' : 'connects') + '</td><td><code>' + esc(i.binding) + '</code> ← <code>' +
        esc(i.var) + '</code></td><td>' + (i.why ? esc(i.why) : esc(i.label) + ' — ' + esc(i.have) + ' in ' +
        esc(i.unit)) + '</td></tr>').join('') + '</tbody></table></div>';
  }
  if (r.open && r.open.length) {
    h += '<h4>Still for the developer to settle</h4><ul class="nform-open">' +
      r.open.map(o => '<li>' + esc(o) + '</li>').join('') + '</ul>';
  }
  h += '<p><b>' + r.applicable + '</b> change(s) can be applied, <b>' + r.blocked + '</b> cannot.</p>';
  if (r.notes && r.notes.trim()) h += '<p class="muted">From the filler: ' + esc(r.notes) + '</p>';
  if (r.known) {
    h += '<details><summary>' + r.known + ' known value(s) — a request for whoever records fixtures, ' +
      'never applied from a form</summary><pre class="nform-fx">' + esc(r.fixture_request) + '</pre></details>';
  }
  if (r.applicable) {
    h += '<p class="muted">A developer applies it with <code>cargo run -p xtask -- intake ' + esc(name) +
      ' --apply</code>' + (r.blocked ? ' <code>--partial</code>' : '') + ', which regenerates the node, gates ' +
      'it and puts everything back on a refusal.</p>';
  }
  out.innerHTML = h + '</div>';
}
