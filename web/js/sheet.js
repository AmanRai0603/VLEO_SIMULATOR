/*
  The sheet, as a form a person can fill in.

  `xtask declare` asks the sheet's questions and says what each one is for. This
  is that form in a browser, because most of the people who have to fill a row in
  are not going to be at a terminal — and a row nobody can fill is a row that
  stays empty.

  The questions are not written here. Neither is the shape of any answer, the
  order the groups come in, or the reason a locked field is locked. All of it
  comes from `/v1/declare/<id>`, which is the same list the terminal prints from
  the same table, so a field added to the sheet appears in both faces or in
  neither. Nine of the questions block generation; the rest decide whether the
  row answers at all, and the form does not pretend the second group is optional
  reading.

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
/**
 * The box for one answer, whatever holds it.
 *
 * WHICH CONTROL IS THE SERVER'S ANSWER, not a guess from the field's name. This
 * used to test `field === 'question' || field.startsWith('reason_')` for "is it
 * a paragraph", which was right for the nine questions it was written for and
 * wrong for every field added after them. The shape comes from the same table
 * that decides how the value is written into the file, and a key inside a
 * repeated block draws from it too.
 */
function controlHtml(id, shape, options, value, choices) {
  value = value || '';
  // A CLOSED SET IS OFFERED, NOT TYPED. A `type` goes straight into the
  // generated signature, so a name that is not a quantity stops the tree
  // compiling; a `unit` decides what converts at a face boundary; a `sense`
  // read the wrong way reports a comfortable margin for a spacecraft about to
  // be destroyed. The server refuses all three either way — this is so nobody
  // meets that refusal by typing a plausible word.
  const set = shape === 'quantity' ? (choices && choices.type)
    : shape === 'unit' ? (choices && choices.unit)
    : options && options.length ? options
    : null;
  if (set) {
    return '<select id="' + id + '" class="ctl sf-pick">' +
      (value && !set.includes(value)
        ? '<option value="' + esc(value) + '" selected>' + esc(value) +
          ' — not one this system has</option>'
        : '<option value=""' + (value ? '' : ' selected') + '>—</option>') +
      set.map(o => '<option value="' + esc(o) + '"' +
        (o === value ? ' selected' : '') + '>' + esc(o) + '</option>').join('') +
      '</select>';
  }
  if (shape === 'prose') {
    return '<textarea id="' + id + '" rows="' +
      Math.min(12, Math.max(2, Math.ceil(value.length / 90))) +
      '" spellcheck="true">' + esc(value) + '</textarea>';
  }
  if (shape === 'number' || shape === 'count') {
    // A number box, so a phone offers a number pad and a stray letter is caught
    // before it is a round trip to the server and back.
    return '<input id="' + id + '" class="sf-num" type="number" ' +
      (shape === 'count' ? 'step="1" min="0"' : 'step="any"') +
      ' value="' + esc(value) + '">';
  }
  return '<input id="' + id + '" type="text" value="' + esc(value) + '">';
}

function fieldHtml(f, choices) {
  const id = 'sf-' + f.field;
  const control = controlHtml(id, f.shape, f.options, f.value, choices);
  return '<div class="sf-field' + (f.open ? ' open' : '') + '" data-field="' + esc(f.field) + '">' +
    '<label for="' + id + '">' +
      '<span class="sf-mark">' + (f.open ? '?' : '·') + '</span>' +
      esc(f.ask) +
    '</label>' +
    control +
    '<p class="sf-why"><code>' + esc(f.field) + '</code> — without it: ' + esc(f.why) + '</p>' +
    '<p class="sf-said" hidden></p>' +
  '</div>';
}

/**
 * The fields a face may not write, each with the reason.
 *
 * Both halves come from the server. A copy of these sentences lived here and had
 * already drifted: it still said a `state` "changes what the gate demands",
 * which is true of a tier and is not why a state may not be typed into a box.
 */
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
 * One repeated block's editor: what is there, and one place to add another.
 *
 * A BLOCK IS NOT A FIELD. It is added and removed as well as edited, and which
 * of those a particular array allows is the server's answer, not this file's: a
 * numbered array — the algorithm steps — grows and shrinks only at its end,
 * because each number is a hole holding somebody's hand-written Rust and
 * inserting in the middle moves every body after it onto a different step.
 *
 * Each cell saves itself, the same as a field does, for the same reason: an
 * edit that changes several things at once is an edit nobody can read in a diff.
 */
function arrayHtml(a, choices) {
  const cells = (row, i) => a.columns.filter(c => !c.managed).map(c =>
    '<div class="sf-cell" data-key="' + esc(c.key) + '">' +
      '<label for="sf-' + esc(a.name) + '-' + i + '-' + esc(c.key) + '">' +
        esc(c.ask) + '</label>' +
      controlHtml('sf-' + a.name + '-' + i + '-' + c.key, c.shape, c.options,
                  row ? row[c.key] : '', choices) +
    '</div>').join('');
  const numbered = a.columns.find(c => c.managed);
  return '<section class="sf-array" data-array="' + esc(a.name) + '">' +
    '<h4 class="sf-group">' + esc(a.label) +
      '<span class="sf-group-open">' + a.rows.length + '</span></h4>' +
    '<p class="sf-why">' + esc(a.why) + '</p>' +
    (a.rows.length
      ? '<ol class="sf-blocks">' + a.rows.map((r, i) =>
          '<li data-i="' + i + '">' +
            '<div class="sf-blockhead"><span class="sf-blocknum">' +
              (numbered ? esc(r[numbered.key]) : String(i + 1)) + '</span>' +
              '<button class="ctl sf-block-rm" title="remove this block">remove</button>' +
            '</div>' +
            cells(r, i) +
            '<p class="sf-said" hidden></p>' +
          '</li>').join('') + '</ol>'
      : '<p class="sf-said">none yet.</p>') +
    '<details class="sf-add"><summary>add one</summary>' +
      (a.mode === 'end'
        ? '<p class="sf-why">Added at the end. Its number is the form\u2019s to assign: ' +
          'each one is a hole in the generated model holding somebody\u2019s Rust, and ' +
          'inserting in the middle would move every body after it onto a different step.</p>'
        : '') +
      cells(null, 'new') +
      '<button class="ctl sf-block-add">add it</button>' +
      '<p class="sf-said sf-add-said" hidden></p>' +
    '</details>' +
  '</section>';
}

/**
 * How the answer is drawn.
 *
 * ONE TABLE, NOT THREE FIELDS. The kind decides which of the other keys exist —
 * a number has none, a line has the row and the point count, a bar has the row
 * — so offering them separately would mean setting a key on a row where it does
 * not exist and then setting the kind afterwards. Two saves, either order
 * wrong. The whole state goes at once.
 */
function viewHtml(v) {
  if (!v.kinds.includes(v.kind)) {
    return '<section class="sf-view"><h4 class="sf-group">how the answer is drawn</h4>' +
      '<p class="sf-why">This row draws a <code>' + esc(v.kind) + '</code>, which this form ' +
      'does not write. It has more axes than the three kinds here, and turning it into one ' +
      'of them would throw one away.</p></section>';
  }
  return '<section class="sf-view"><h4 class="sf-group">how the answer is drawn</h4>' +
    '<div class="sf-cell"><label for="sf-vkind">what shape</label>' +
      '<select id="sf-vkind" class="ctl sf-pick">' +
        v.kinds.map(k => '<option value="' + esc(k) + '"' +
          (k === v.kind ? ' selected' : '') + '>' + esc(k) + '</option>').join('') +
      '</select></div>' +
    '<div class="sf-cell sf-vover"><label for="sf-vover">which row it is drawn against</label>' +
      '<input id="sf-vover" type="text" value="' + esc(v.over) + '"></div>' +
    '<div class="sf-cell sf-vpoints"><label for="sf-vpoints">over how many points</label>' +
      '<input id="sf-vpoints" class="sf-num" type="number" step="1" min="2" value="' +
      esc(String(v.points || 60)) + '"></div>' +
    '<button class="ctl sf-view-go">save how it is drawn</button>' +
    '<p class="sf-why">A number is drawn from the row\u2019s own answer. A line and a bar ' +
    'are drawn against another row, and the run sweeps it \u2014 so that row has to exist, ' +
    'which nothing else in the tree checks.</p>' +
    '<p class="sf-said sf-view-said" hidden></p></section>';
}

/**
 * Publishing the row.
 *
 * A STATE IS NOT A FIELD. It decides whether anything is generated from the row
 * at all: a seeded row generates a page and its metadata, a published one
 * generates the model, its contract, its module and its evidence. Flipping it
 * half-written would emit four files that say nothing, so it is an action with
 * preconditions rather than a box, and every unmet one is named.
 */
function publishHtml(p) {
  if (p.state !== 'empty') {
    return '<section class="sf-publish"><h4 class="sf-group">state</h4>' +
      '<p class="sf-why">This row is <b>' + esc(p.state) + '</b>. Publishing moves a seeded ' +
      'row and nothing else; the states after it are a developer\u2019s at a terminal.</p>' +
      '</section>';
  }
  return '<section class="sf-publish"><h4 class="sf-group">state</h4>' +
    '<p class="sf-why">This row is <b>seeded</b>: it is on the tree, every tab opens and each ' +
    'says what goes in it, but nothing is generated from it and it cannot run. Publishing ' +
    'is what makes the generator write its model, its contract and its evidence.</p>' +
    (p.possible
      ? '<button class="ctl sf-publish-go">publish this row</button>'
      : '<p class="sf-why"><b>Not yet:</b></p><ul class="sf-dropped">' +
        p.why.map(w => '<li>' + esc(w) + '</li>').join('') + '</ul>') +
    '<p class="sf-said sf-publish-said" hidden></p></section>';
}

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
      ? '<b>' + d.open + '</b> of ' + d.fields.filter(f => f.available).length +
        ' still block generation'
      : 'nothing left that blocks generation') +
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
    // GROUPED, in the order the server gives them. Sixteen questions in one
    // column is a form people abandon halfway; the groups are the few decisions
    // the questions actually belong to.
    d.groups.map(g => {
      const mine = d.fields.filter(f => f.group === g && f.available);
      if (!mine.length) return '';
      const open = mine.filter(f => f.open).length;
      return '<h4 class="sf-group">' + esc(g) +
        (open ? ' <span class="sf-group-open">' + open + ' open</span>' : '') + '</h4>' +
        mine.map(f => fieldHtml(f, d.choices)).join('');
    }).join('') +
    // A field the sheet has not got and the form may not add is named rather
    // than silently missing. A `sense` belongs to a requirement and a declared
    // number to a row that declares one, so offering either here would be a
    // question with no right answer — but so is leaving a reader to wonder where
    // it went.
    (d.fields.some(f => !f.available)
      ? '<p class="sf-why sf-absent">Not on this row: ' +
        d.fields.filter(f => !f.available).map(f => '<code>' + esc(f.field) + '</code>')
          .join(', ') +
        '. The sheet has not got the key, and this form does not decide where a new ' +
        'one belongs \u2014 add it in a checkout and it appears here.</p>'
      : '') +
    (d.arrays || []).map(a => arrayHtml(a, d.choices)).join('') +
    viewHtml(d.view) +
    publishHtml(d.publish) +
    lockedHtml(d.structural, d.structural_why || {}) +
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

  /**
   * Say what happened, in the place it happened.
   *
   * ADDS AND REMOVES CLASSES; IT DOES NOT ASSIGN `className`. Assigning it wiped
   * every other class the element carried — including `sf-add-said`, which is
   * the class the element is FOUND by — so the first save through one of these
   * messages made the next lookup of it return null. The symptom was a
   * successful write followed by a TypeError, which reads like a failed save and
   * is not one.
   */
  function tell(said, state, text) {
    ['waiting', 'good', 'bad', 'stale'].forEach(c => said.classList.remove(c));
    if (state) said.classList.add(state);
    said.hidden = false;
    said.textContent = text;
  }

  /**
   * One write, and what it did.
   *
   * Every writing endpoint answers the same three ways — written, refused, or
   * somebody else moved the row — so saying so is here once. A save regenerates
   * the row and runs the gate, which takes a moment; an optimistic tick would
   * be a lie for as long as it takes.
   */
  async function write(url, body, said, onOk) {
    tell(said, 'waiting', 'writing the sheet, regenerating and gating…');
    let res;
    try {
      res = await (await fetch(url, {
        method: 'POST', body: new URLSearchParams(Object.assign({ base }, body)),
      })).json();
    } catch (e) {
      tell(said, 'bad', 'the daemon did not answer: ' + e);
      return false;
    }
    if (res.ok) {
      base = res.file_hash;
      tell(said, 'good', 'saved · ' + res.regenerated + ' artefact(s) regenerated');
      if (onOk) onOk(res);
      return true;
    }
    // A conflict is not the reader's mistake and must never read like one.
    if (res.stale) {
      tell(said, 'stale', 'somebody else changed this row since you opened it. Nothing was ' +
        'written. Reopen the row to see it as it is now, then reapply your edit.');
      return false;
    }
    tell(said, 'bad', res.message || 'refused, with no reason given');
    return false;
  }

  /** What one block's boxes currently say. */
  function cellsOf(scope) {
    const out = {};
    $$('.sf-cell', scope).forEach(c => {
      const inp = $('input,textarea,select', c);
      if (inp) out[c.dataset.key] = inp.value;
    });
    return out;
  }

  // The repeated blocks. A cell saves itself; a block is removed whole; a new
  // one is added from the boxes under "add one".
  $$('.sf-array', host).forEach(sec => {
    const name = sec.dataset.array;
    const url = '/v1/block/' + encodeURIComponent(id);
    $$('.sf-blocks > li', sec).forEach(li => {
      const said = $('.sf-said', li);
      const i = li.dataset.i;
      $$('.sf-cell', li).forEach(cell => {
        const inp = $('input,textarea,select', cell);
        const was = inp.value;
        const btn = document.createElement('button');
        btn.className = 'ctl sf-save';
        btn.textContent = 'save';
        btn.disabled = true;
        cell.appendChild(btn);
        ['input', 'change'].forEach(e =>
          inp.addEventListener(e, () => { btn.disabled = inp.value === was; }));
        btn.addEventListener('click', async () => {
          btn.disabled = true;
          const body = { array: name, op: 'set', index: i, key: cell.dataset.key };
          body[cell.dataset.key] = inp.value;
          if (!await write(url, body, said)) btn.disabled = false;
        });
      });
      $('.sf-block-rm', li).addEventListener('click', async () => {
        const rm = $('.sf-block-rm', li);
        rm.disabled = true;
        const ok = await write(url, { array: name, op: 'remove', index: i }, said);
        if (ok) li.classList.add('gone'); else rm.disabled = false;
        if (ok) said.textContent += ' · reopen the row to renumber what is left';
      });
    });
    const add = $('.sf-block-add', sec);
    if (add) add.addEventListener('click', async () => {
      add.disabled = true;
      const body = Object.assign({ array: name, op: 'add', index: 0 },
                                 cellsOf($('.sf-add', sec)));
      const said = $('.sf-add-said', sec);
      const ok = await write(url, body, said);
      if (ok) said.textContent += ' · reopen the row to see it in the list';
      add.disabled = false;
    });
  });

  // How the answer is drawn: one save for the whole table, and the boxes that
  // do not apply to the chosen kind are hidden rather than sent and refused.
  const vsec = $('.sf-view', host);
  if (vsec && $('#sf-vkind', vsec)) {
    const kind = $('#sf-vkind', vsec);
    const showing = () => {
      $('.sf-vover', vsec).hidden = kind.value === 'number';
      $('.sf-vpoints', vsec).hidden = kind.value !== 'line';
    };
    kind.addEventListener('change', showing);
    showing();
    $('.sf-view-go', vsec).addEventListener('click', async () => {
      const go = $('.sf-view-go', vsec);
      go.disabled = true;
      await write('/v1/view/' + encodeURIComponent(id), {
        kind: kind.value,
        over: kind.value === 'number' ? '' : $('#sf-vover', vsec).value,
        points: $('#sf-vpoints', vsec).value,
      }, $('.sf-view-said', vsec));

      go.disabled = false;
    });
  }

  const pub = $('.sf-publish-go', host);
  if (pub) pub.addEventListener('click', async () => {
    const said = $('.sf-publish-said', host);
    pub.disabled = true;
    const ok = await write('/v1/publish/' + encodeURIComponent(id), {}, said);
    if (ok) said.textContent +=
      ' · this row now generates its model, its contract and its evidence';
    else pub.disabled = false;
  });

  // Each field saves itself, and says what happened where it happened.
  $$('.sf-field', host).forEach(box => {
    const field = box.dataset.field;
    const inp = $('input,textarea,select', box);
    const said = $('.sf-said', box);
    const was = inp.value;
    const btn = document.createElement('button');
    btn.className = 'ctl sf-save';
    btn.textContent = 'save';
    btn.disabled = true;
    box.insertBefore(btn, said);
    ['input', 'change'].forEach(e =>
      inp.addEventListener(e, () => { btn.disabled = inp.value === was; }));

    btn.addEventListener('click', async () => {
      btn.disabled = true;
      const ok = await write('/v1/sheet/' + encodeURIComponent(id),
                             { field, value: inp.value }, said, () => {
        box.classList.remove('open');
        $('.sf-mark', box).textContent = '·';
      });
      if (!ok && !said.classList.contains('stale')) btn.disabled = false;
    });
  });
}
