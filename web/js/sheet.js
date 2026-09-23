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
 * Paste a sibling's sheet body.
 *
 * This is how twenty rows that share a pattern get filled without retyping, and
 * it is also how one stale source citation gets dragged through thirty of them.
 * So a paste is never applied: the server parses it, drops every structural
 * key, and returns what WOULD change. A person looks at that list, and only
 * then does each field go through the same one-at-a-time save as any other
 * edit.
 */
function pasteHtml() {
  return '<details class="sf-paste"><summary>paste a sheet body from another row</summary>' +
    '<p class="sf-why">Nothing is written by pasting. The structural keys — the ' +
    'identifier, the parent, the order — are dropped, and you are shown what ' +
    'would change before anything happens.</p>' +
    '<textarea class="sf-paste-in" rows="6" spellcheck="false" ' +
      'placeholder="[maths]&#10;expression = &quot;…&quot;&#10;source = &quot;…&quot;"></textarea>' +
    '<button class="ctl sf-paste-go">show me what this would change</button>' +
    '<div class="sf-paste-out"></div></details>';
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
 * Put the edited rows on a branch.
 *
 * A sheet edit is a source change and belongs in history — unlike a run's
 * inputs, which are a question somebody asked and are never committed. So edits
 * made here do not sit in a working tree waiting to be noticed: they go onto a
 * branch of their own, where CODEOWNERS routes them to whoever owns those rows.
 *
 * IT COMMITS AND DOES NOT PUSH. Pushing puts the work where other people and
 * the pipeline see it, under whatever credentials the checkout holds. The
 * branch and the command come back instead, so the person who made the edits is
 * the one who shares them.
 */
function proposeHtml() {
  const kinds = ['docs', 'fix', 'feat', 'refactor', 'chore'];
  return '<details class="sf-propose"><summary>put these edits on a branch</summary>' +
    '<p class="sf-why">Commits the rows you have changed, on a new branch. It does ' +
    'not push — you are given the branch and the command.</p>' +
    '<label for="sf-kind">what kind of change</label>' +
    '<select id="sf-kind" class="ctl">' +
      kinds.map(k => '<option value="' + k + '">' + k + '</option>').join('') +
    '</select>' +
    '<label for="sf-sum">one line saying what changed and why</label>' +
    '<input id="sf-sum" type="text" placeholder="reword the lower bound\u2019s reason">' +
    '<button class="ctl sf-propose-go">commit them to a branch</button>' +
    '<div class="sf-propose-out"></div></details>';
}

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
    // SHOWN, NOT ASKED FOR. A name typed into a box is a name somebody chose for
    // that box; this is the one their commits already carry, so the sheet and
    // the history agree about who did the work.
    '<div class="sf-who">' +
      (d.identity
        ? '<p class="sf-ident">Anything you attribute will be signed <b>' +
          esc(d.identity) + '</b><span class="muted"> \u2014 this checkout\u2019s ' +
          '<code>git config user.name</code></span></p>' +
          '<p class="sf-why">An agent may never supply mathematics, and this is the only ' +
          'thing that can tell whether one did. A save that changes the relation is refused ' +
          'if this name is an agent\u2019s.</p>'
        : '<p class="sf-said bad">' + esc(d.identity_why || 'this checkout has no identity') +
          '</p>') +
    '</div>' +
    pasteHtml() +
    d.fields.map(fieldHtml).join('') +
    lockedHtml(d.structural, LOCKED_WHY) +
    proposeHtml() +
    '<p class="sf-foot muted">Holes are not edited here: a hole body is Rust between ' +
    'numbered markers, and an edit outside one is discarded by the next generation pass.</p>';

  // The paste. Preview first, always; applying is the ordinary per-field save,
  // so a pasted value goes through exactly the checks a typed one does.
  const pgo = $('.sf-paste-go', host);
  if (pgo) pgo.addEventListener('click', async () => {
    const out = $('.sf-paste-out', host);
    const text = $('.sf-paste-in', host).value;
    if (!text.trim()) { out.innerHTML = '<p class="sf-said bad">nothing pasted</p>'; return; }
    out.innerHTML = '<p class="sf-said waiting">reading it…</p>';
    let r;
    try {
      r = await (await fetch('/v1/preview/' + encodeURIComponent(id), {
        method: 'POST', body: new URLSearchParams({ body: text }),
      })).json();
    } catch (e) { out.innerHTML = '<p class="sf-said bad">' + esc(String(e)) + '</p>'; return; }
    if (r.ok === false) {
      out.innerHTML = '<p class="sf-said bad">' + esc(r.message || 'refused') + '</p>';
      return;
    }
    // What would change, and what was ignored. A key silently dropped is a key
    // somebody believes they set.
    out.innerHTML =
      (r.changes.length
        ? '<table class="sf-diff"><thead><tr><th></th><th>field</th><th>from</th>' +
          '<th>to</th></tr></thead><tbody>' +
          r.changes.map((c, i) =>
            '<tr><td><input type="checkbox" checked data-i="' + i + '"></td>' +
            '<td><code>' + esc(c.field) + '</code></td>' +
            '<td class="was">' + esc(c.from || '(empty)') + '</td>' +
            '<td class="now">' + esc(c.to) + '</td></tr>').join('') +
          '</tbody></table>' +
          '<button class="ctl sf-paste-apply">apply the ticked ones</button>'
        : '<p class="sf-said">nothing in that paste would change this row.</p>') +
      (r.dropped.length
        ? '<p class="sf-why"><b>Ignored:</b></p><ul class="sf-dropped">' +
          r.dropped.map(x => '<li>' + esc(x) + '</li>').join('') + '</ul>'
        : '');
    const apply = $('.sf-paste-apply', out);
    if (apply) apply.addEventListener('click', async () => {
      apply.disabled = true;
      const picked = $$('.sf-diff input:checked', out).map(x => r.changes[+x.dataset.i]);
      const said = [];
      // One at a time, through the same endpoint a typed edit uses. Sequential
      // because each save moves the hash the next one has to send.
      for (const c of picked) {
        const res = await (await fetch('/v1/sheet/' + encodeURIComponent(id), {
          method: 'POST',
          body: new URLSearchParams({ field: c.field, value: c.to, base }),
        })).json();
        if (res.ok) { base = res.file_hash; said.push(c.field + ' — saved'); }
        else { said.push(c.field + ' — ' + (res.message || 'refused')); break; }
      }
      out.insertAdjacentHTML('beforeend',
        '<ul class="sf-applied">' + said.map(x => '<li>' + esc(x) + '</li>').join('') + '</ul>' +
        '<p class="sf-why">Reopen the row to see the form as it now stands.</p>');
    });
  });

  const prop = $('.sf-propose-go', host);
  if (prop) prop.addEventListener('click', async () => {
    const out = $('.sf-propose-out', host);
    prop.disabled = true;
    out.innerHTML = '<p class="sf-said waiting">committing\u2026</p>';
    let r;
    try {
      r = await (await fetch('/v1/propose', {
        method: 'POST',
        body: new URLSearchParams({
          summary: $('#sf-sum', host).value,
          kind: $('#sf-kind', host).value,
        }),
      })).json();
    } catch (e) {
      out.innerHTML = '<p class="sf-said bad">' + esc(String(e)) + '</p>';
      prop.disabled = false; return;
    }
    if (!r.ok) {
      out.innerHTML = '<p class="sf-said ' + (r.nothing ? '' : 'bad') + '">' +
        esc(r.message || 'refused') + '</p>';
      prop.disabled = false; return;
    }
    // The branch exists. What is left is the person's to do, so it is shown as
    // something to copy rather than described.
    out.innerHTML =
      '<p class="sf-said good">committed ' + esc(r.commit) + ' \u2014 ' +
        esc(String(r.files)) + ' file(s) on <code>' + esc(r.branch) + '</code></p>' +
      '<p class="sf-why">To share it:</p><pre class="sf-cmd">' + esc(r.push) + '</pre>' +
      (r.compare
        ? '<p class="sf-why">then open it: <a href="' + esc(r.compare) +
          '" target="_blank" rel="noopener">' + esc(r.compare) + '</a></p>'
        : '<p class="sf-why">This remote has no pull-request page that could be ' +
          'linked to, so nothing is guessed here.</p>');
  });

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
      btn.disabled = true;
      // A save regenerates the row and runs the gate, which takes a moment. An
      // optimistic tick here would be a lie for as long as it takes.
      said.hidden = false;
      said.className = 'sf-said waiting';
      said.textContent = 'writing the sheet, regenerating and gating…';
      const body = new URLSearchParams({ field, value: inp.value, base });
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
