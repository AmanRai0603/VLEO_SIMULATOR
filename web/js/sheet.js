/*
  The sheet, as a form a person can fill in.

  `xtask declare` asks nine questions and says what each one is for. This is
  that form in a browser, because most of the people who have to fill a row in
  are not going to be at a terminal — and a row nobody can fill is a row that
  stays empty.

  The questions are not written here. They come from `/v1/declare/<id>`, which
  is the same list the terminal prints, so a field added to the sheet appears in
  both or in neither.

  WHAT THIS DELIBERATELY CANNOT DO. It cannot add a row, delete one, or touch a
  structural field — the identifier, the parent, the order, the layer, the kind,
  the owner. Those move the tree or renumber a row's neighbours, and they are a
  developer's act at a terminal. The server refuses them too; this only means a
  reader is never offered something that will be rejected. Each locked field is
  shown WITH THE REASON it is locked, because a reader told "no" and not "why"
  goes looking for a way round.

  It also cannot edit a hole. A hole body is Rust between numbered markers, and
  a hand edit outside one is discarded by the next generation pass.
*/
'use strict';

import { $, $$, esc } from './dom.js';

/** The form for one row, or null when the daemon will not serve it. */
async function load(id) {
  try {
    const r = await fetch('/v1/declare/' + encodeURIComponent(id));
    if (!r.ok) return null;
    return await r.json();
  } catch { return null; }
}

/**
 * One question.
 *
 * The `why` is not a tooltip. A field whose consequence is hidden is a field
 * somebody fills with anything to make the form go green, so the consequence
 * sits under the input where it cannot be missed.
 */
function fieldHtml(f) {
  const big = f.field === 'question' || f.field.startsWith('reason_');
  const id = 'sf-' + f.field;
  return '<div class="sf-field' + (f.open ? ' open' : '') + '" data-field="' + esc(f.field) + '">' +
    '<label for="' + id + '">' +
      '<span class="sf-mark">' + (f.open ? '?' : '·') + '</span>' +
      esc(f.ask) +
    '</label>' +
    (big
      ? '<textarea id="' + id + '" rows="3" spellcheck="true">' + esc(f.value) + '</textarea>'
      : '<input id="' + id + '" type="text" value="' + esc(f.value) + '">') +
    '<p class="sf-why"><code>' + esc(f.field) + '</code> — without it: ' + esc(f.why) + '</p>' +
    '<p class="sf-said" hidden></p>' +
  '</div>';
}

/** The fields a face may not write, each with the reason. */
function lockedHtml(st, reasons) {
  return '<div class="sf-locked"><h4>Not editable here</h4>' +
    '<p class="muted">These move the tree or renumber a row\'s neighbours, so they are ' +
    'changed in a checkout rather than in a browser.</p><dl>' +
    Object.keys(st).map(k =>
      '<dt><code>' + esc(k) + '</code> = ' + esc(st[k] || '—') + '</dt>' +
      '<dd>' + esc(reasons[k] || 'structural') + '</dd>').join('') +
    '</dl></div>';
}

/**
 * Why each structural field is locked.
 *
 * The server holds the same sentences and is what actually refuses; these are
 * for the reader. If the two ever disagree the server wins, and the reader has
 * been told something slightly wrong rather than allowed something harmful.
 */
const LOCKED_WHY = {
  id: 'the identifier is the folder and the variable name; renaming it is a move',
  folder: 'the identifier is the folder and the variable name; renaming it is a move',
  parent: 'the parent is the tree’s shape — moving a row moves everyone who reads it',
  order: 'order decides position among siblings, and the block may be packed solid; ' +
         'inserting renumbers its neighbours',
  layer: 'the layer decides which contract the row sits under',
  kind: 'kind decides whether the row declares a value or computes one, which changes ' +
        'what is generated for it',
  subsystem: 'ownership is generated into CODEOWNERS and decides who reviews it',
  owner: 'ownership is generated into CODEOWNERS and decides who reviews it',
  tier: 'it changes what the gate demands of the row',
  state: 'it changes what the gate demands of the row',
};

/**
 * Mount the editor for one row.
 *
 * Saves one field at a time, which is how `xtask confirm` works and for the same
 * reason: an edit that changes several things at once is an edit nobody can read
 * in a diff. Each field keeps its own save button and its own message.
 */
export async function mountSheetEditor(host, id) {
  const d = await load(id);
  if (!d) {
    host.innerHTML = '<p class="empty">The daemon will not serve this row’s form.</p>';
    return;
  }
  // One hash for the whole form, refreshed after every save. It is the FILE
  // hash, not the sheet hash: a bound's reason is outside the sheet hash, so two
  // people rewording one would both look current and the second would overwrite
  // the first.
  let base = d.file_hash;

  host.innerHTML =
    '<p class="sf-head">' + (d.open
      ? '<b>' + d.open + '</b> of ' + d.fields.length + ' still open'
      : 'every question answered') +
      ' · <span class="muted">a save regenerates this row and runs the gate on it; ' +
      'if the gate refuses, nothing changes</span></p>' +
    '<div class="sf-who"><label for="sf-by">your name, for anything you attribute' +
      '</label><input id="sf-by" type="text" placeholder="A. Person" autocomplete="name">' +
      '<p class="sf-why">An agent may never supply mathematics, and this field is the only ' +
      'thing that can tell whether one did. The server refuses an agent’s name.</p></div>' +
    d.fields.map(fieldHtml).join('') +
    lockedHtml(d.structural, LOCKED_WHY) +
    '<p class="sf-foot muted">Holes are not edited here: a hole body is Rust between ' +
    'numbered markers, and an edit outside one is discarded by the next generation pass.</p>';

  // Each field saves itself, and says what happened where it happened.
  $$('.sf-field', host).forEach(box => {
    const field = box.dataset.field;
    const inp = $('input,textarea', box);
    const said = $('.sf-said', box);
    const was = inp.value;
    const btn = document.createElement('button');
    btn.className = 'ctl sf-save';
    btn.textContent = 'save';
    btn.disabled = true;
    box.insertBefore(btn, said);
    inp.addEventListener('input', () => { btn.disabled = inp.value === was; });

    btn.addEventListener('click', async () => {
      const by = ($('#sf-by', host) || {}).value || '';
      btn.disabled = true;
      // A save regenerates the row and runs the gate, which takes a moment. An
      // optimistic tick here would be a lie for as long as it takes.
      said.hidden = false;
      said.className = 'sf-said waiting';
      said.textContent = 'writing the sheet, regenerating and gating…';
      const body = new URLSearchParams({ field, value: inp.value, base, by });
      let res = null;
      try {
        const r = await fetch('/v1/sheet/' + encodeURIComponent(id), { method: 'POST', body });
        res = await r.json();
      } catch (e) {
        said.className = 'sf-said bad';
        said.textContent = 'the daemon did not answer: ' + e;
        btn.disabled = false;
        return;
      }
      if (res.ok) {
        base = res.file_hash;
        said.className = 'sf-said good';
        said.textContent = 'saved · ' + res.regenerated + ' artefact(s) regenerated';
        box.classList.remove('open');
        $('.sf-mark', box).textContent = '·';
        return;
      }
      // A conflict is not the reader's mistake and must never read like one.
      if (res.stale) {
        said.className = 'sf-said stale';
        said.textContent = 'somebody else changed this row since you opened it. ' +
          'Nothing was written. Reopen the row to see it as it is now, then reapply your edit.';
        return;
      }
      said.className = 'sf-said bad';
      said.textContent = res.message || 'refused, with no reason given';
      btn.disabled = false;
    });
  });
}
