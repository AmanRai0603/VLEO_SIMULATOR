/*
  One node, opened.

  Three segments in a fixed order, because the order is the thing that has to
  be repeatable: what the node is *made of*, what it is *connected to*, and
  what it *says*. The third of those is the generated page; the first two are
  computed here from the folder on disk and from the graph, so neither can
  claim something the repository does not hold.
*/
'use strict';

import { $, $$, esc, plural } from './dom.js';
import { S, isSeeded, isUndefined, isUnconfirmed, isUnread } from './state.js';
import { renderRun } from './run.js';
import { mountRelation } from './relation.js';
import { mountTheory } from './theory.js';
import { figuresForRow, drawRowFigure } from './solar.js';
import { isInput, inputControl, mountInput } from './inputs.js';

export async function openNode(id) {
  const r = S.byId.get(id);
  if (!r) return;
  const body = $('#node-body');
  body.innerHTML = '<p class="muted">loading the node…</p>';
  const figs = figuresForRow(id);

  const [meta, fragment] = await Promise.all([
    fetch('/v1/node/' + encodeURIComponent(id)).then(x => x.json()).catch(() => null),
    fetch('/v1/fragment/' + encodeURIComponent(id)).then(x => x.text()).catch(() => null),
  ]);

  body.innerHTML =
    // AND THIS COMES BEFORE EVEN THAT. Whether the row answers is the first
    // thing a reader needs, because every number further down the page is
    // either the design's or nobody's and they look identical. It is a banner
    // rather than a segment: the segments say what the node IS, and this says
    // what it currently DOES.
    activeBanner(r) +
    // THE CONTROL COMES BEFORE THE DESCRIPTION, and it is not a numbered
    // segment. The four segments say what this node IS; a declared row is also
    // a thing a reader can move, and that is a different kind of statement. It
    // is drawn only where it can do something — see isInput.
    (isInput(r) ? '<section class="seg ovr-panel"></section>' : '') +
    '<section class="seg" data-seg="assembly"><h3 class="seg-h">' +
      '<span class="seg-n">1</span>assembly — what this node is made of</h3>' +
      assemblyHtml(r, meta) + '</section>' +
    '<section class="seg" data-seg="connectivity"><h3 class="seg-h">' +
      '<span class="seg-n">2</span>connectivity — what it is joined to, as declared</h3>' +
      connectivityHtml(r) + '</section>' +
    '<section class="seg" data-seg="sheet"><h3 class="seg-h">' +
      '<span class="seg-n">3</span>the sheet — what it says</h3>' +
      (fragment || '<p class="empty">The sheet for <code>' + esc(id) +
        '</code> is not on disk. Run <code>cargo xtask docs</code>.</p>') + '</section>' +
    // A FIGURE IS NOT A ROW, and it is not a place of its own either. The eight
    // study figures used to live behind a sixth item in a navigation whose own
    // subtitle says there are four layers, showing solar weather in a second
    // place that disagreed with layer 3. They belong to the rows they argue
    // about, which every one of them already declared; this segment is that
    // declaration read the other way round.
    (figs.length
      ? '<section class="seg" data-seg="figure"><h3 class="seg-h">' +
        '<span class="seg-n">4</span>the figure — the picture this claim is argued from</h3>' +
        (figs.length > 1
          ? '<div class="tabrow sub sw-tabs">' + figs.map((f, i) =>
              '<button class="ctl sw-tab' + (i ? '' : ' sel') + '" data-fig="' + esc(f.id) +
              '">' + esc(f.label) + '</button>').join('') + '</div>'
          : '') +
        '<div class="row-figure"></div></section>'
      : '');

  if (isInput(r)) mountInput($('.ovr-panel', body), r);

  if (figs.length) {
    const hostFig = $('.row-figure', body);
    // The row is passed so a panel with one view per row can open on this one.
    // See optsFor: a figure reached from a row should be about that row.
    drawRowFigure(hostFig, figs[0].id, r.id);
    $$('.sw-tabs .sw-tab', body).forEach(b => {
      b.onclick = () => {
        $$('.sw-tabs .sw-tab', body).forEach(x => x.classList.remove('sel'));
        b.classList.add('sel');
        drawRowFigure(hostFig, b.dataset.fig, r.id);
      };
    });
  }

  // The generated fragment styles its tabs as `.tab`; inside `.tabs` that is
  // the underline tab, not a layer tab.
  $$('.tabs .tab', body).forEach(t => {
    t.classList.add('tab-btn');
    t.onclick = () => {
      $$('.tabs .tab', body).forEach(x => x.classList.remove('sel'));
      $$('[data-panel]', body).forEach(x => x.classList.remove('sel'));
      t.classList.add('sel');
      const p = $('[data-panel="' + t.dataset.tab + '"]', body);
      if (p) p.classList.add('sel');
      // The relation tab asks the engine, so it is mounted when it is opened
      // and not before: a page that swept every node on load would ask a
      // question nobody had.
      const rel = p && $('.relation-host', p);
      if (rel && !rel.dataset.mounted) { rel.dataset.mounted = '1'; mountRelation(rel); }
      // The derivation is already complete in the fragment; this only adds the
      // controls that walk it, so mounting late costs the reader nothing.
      const th = p && $('.theory-walk', p);
      if (th && !th.dataset.mounted) { th.dataset.mounted = '1'; mountTheory(th); }
    };
  });

  renderRun($('#run-panel'), r);
}

/**
 * The folder, read off the disk.
 *
 * This is the template: the same eight files for every one of the rows, seven
 * of them generated. It is what makes adding the next node a copy rather than
 * a decision, and it is shown rather than described because a described layout
 * goes stale the first time the layout changes.
 */
function assemblyHtml(r, meta) {
  if (!meta || !meta.ok) {
    return '<p class="empty">The engine did not describe this folder.</p>';
  }
  const present = meta.artefacts.filter(a => a.present).length;
  return '<p class="seg-lead"><code>' + esc(meta.folder) + '</code> — ' +
    plural(present, 'file') + ' of ' + meta.artefacts.length +
    '. Everything about one node is in one directory: adding one is a copy, deleting one is a ' +
    'remove, its history is the log of a directory, and ownership is a path rule.</p>' +
    '<table class="asm-t"><thead><tr><th>file</th><th>written</th><th>what it is</th>' +
    '<th class="num">bytes</th></tr></thead><tbody>' +
    meta.artefacts.map(a =>
      '<tr class="' + (a.present ? '' : 'absent') + '">' +
      '<td><code>' + esc(a.name) + '</code></td>' +
      '<td><span class="by ' + (a.written === 'by hand' ? 'hand' : 'gen') + '">' +
        esc(a.written) + '</span></td>' +
      '<td>' + esc(a.what) + '</td>' +
      '<td class="num">' + (a.present ? Math.round(a.bytes) : '—') + '</td></tr>').join('') +
    '</tbody></table>' +
    '<p class="seg-note">' +
      'sheet <b>' + esc(meta.sheet_hash) + '</b> · implementation <b>' + esc(meta.impl_hash) + '</b> · ' +
      plural(meta.steps, 'algorithm step') + ', and one numbered <code>HOLE</code> in ' +
      '<code>model.rs</code> for each. A hand edit outside a hole is discarded by the next ' +
      'regeneration and fails the regeneration diff, which is what makes the generated region ' +
      'genuinely owned by the generator rather than merely labelled that way.' +
    '</p>' +
    (isSeeded(r)
      ? '<p class="empty">Seeded: the two hand-written files exist and are unfilled, so nothing ' +
        'has been generated from them. That is the normal state of most of a tree for most of a ' +
        'programme, and it is a reported state rather than a failing one.</p>'
      : '');
}

/**
 * The connectivity definition.
 *
 * Three graphs, never merged. Each edge is declared exactly once, by the end
 * that the edge *changes*: a derivation edge by the consuming node, because
 * knowing its inputs changes its implementation; a contribution edge by the
 * contributing variable; a relation edge by the layer file. Everything else —
 * who consumes this, what feeds a KPI — is derived here and never stored.
 */
function connectivityHtml(r) {
  const ins  = S.producers[r.i].map(i => S.rows[i]);
  const outs = S.consumers[r.i].map(i => S.rows[i]);
  const link = x => '<a class="xref" data-goto="' + esc(x.id) + '">' + esc(x.id) + '</a>' +
    ' <span class="muted">' + esc(x.label) + '</span>';

  const rows = [
    ['reads · declared here',
     'the derivation graph, declared by this node because knowing its inputs changes its implementation',
     ins.length ? ins.map(link) : null,
     'Nothing. This is a declared value or a leaf of the design.'],
    ['publishes',
     'one small question, one answer — the variable id is the node id',
     [ '<code>' + esc(r.id) + '</code>' + (r.symbol ? ' <b>' + esc(r.symbol) + '</b>' : '') +
       ' <span class="muted">' + esc(r.unit) + '</span>' ],
     ''],
    ['read by · derived',
     'never stored, so it cannot go stale',
     outs.length ? outs.map(link) : null,
     'Nothing reads this yet. Every one of these is a leaf of the design, or an oversight.'],
    ['contributes to · declared here',
     'the contribution graph, declared by the variable — coverage, never execution',
     r.kpi.length ? r.kpi.map(k => '<a class="xref" data-goto="' + esc(k) + '">' + esc(k) + '</a>') : null,
     'No KPI names this row.'],
  ];
  if (r.crosses) {
    rows.push(['crosses upward · declared here',
      'the one route out of this layer — a subsystem is reached through its interface node, never by reaching into it',
      ['<a class="xref" data-group="' + esc(r.crosses) + '">' + esc(r.crosses) + '</a>'], '']);
  }

  return '<table class="conn-t"><tbody>' + rows.map(([k, why, items, none]) =>
    '<tr><th>' + esc(k) + '<span class="why">' + esc(why) + '</span></th><td>' +
    (items ? items.join('<br>') : '<span class="muted">' + esc(none) + '</span>') +
    '</td></tr>').join('') + '</tbody></table>';
}

/**
 * Whether this row answers, said once and at the top.
 *
 * Three cases, and the middle one is the reason this is not a single flag:
 *
 *   INACTIVE      a function whose relation is stated and never derived. The
 *                 engine refuses it, so every number on the rest of the page is
 *                 an example rather than an answer.
 *   UNCONFIRMED   derived, and nobody has read the relation against its source.
 *                 It answers, and reports 1 of 4 for mathematics. Worth saying,
 *                 not worth withholding.
 *   an input      a default is a definition. Nothing is drawn: the override
 *                 control below says everything there is to say about it.
 */
function activeBanner(r) {
  if (isUndefined(r)) {
    return '<div class="banner inactive"><b>INACTIVE</b> — this row does not answer. ' +
      'Its relation is stated and never derived, so there is nothing on the sheet a reader can ' +
      'check it against. The engine refuses it under its own name and everything downstream ' +
      'blocks on it, named. ' +
      '<span class="muted">Fixed by a <code>[theory]</code> block on the sheet — written by ' +
      'somebody who knows where the relation came from. An agent may never supply mathematics, ' +
      'which is the reason this refusal exists.</span></div>';
  }
  // Reach and trust are different facts, so both can show. A row can answer
  // correctly, be read by nobody, AND have nobody's name on its relation; each
  // is a separate piece of work for a different person.
  let h = '';
  if (isUnread(r)) {
    h += '<div class="banner unread"><b>UNREAD</b> — this row answers and its number reaches ' +
      'no KPI closure. The tree exists to move twelve of them, and nothing downstream of this ' +
      'row ends at one, so the answer is computed and the design is not reading it.' +
      '<span class="muted">Not a defect in the row, and not something the row can fix: a ' +
      'crossing nothing reads gets wired up, retired, or kept on purpose as a reference beside ' +
      'the design point. Which one is a decision with a person\'s name on it.</span></div>';
  }
  if (isUnconfirmed(r)) {
    h += '<div class="banner unconfirmed"><b>UNCONFIRMED</b> — this row answers, and nobody ' +
      'has read its relation against its source. It reports <b>1 of 4</b> for mathematics until ' +
      'somebody does. <span class="muted">A citation says the paper exists; ' +
      '<code>[maths] confirmed_by</code> says a person read the relation in it.</span></div>';
  }
  return h;
}
