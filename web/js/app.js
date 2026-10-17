/*
  The shell: boot, routing, and every event listener in the tool.

  No view module wires a navigation handler. Views render markup carrying
  `data-` attributes and this file decides what a click means, so the module
  graph stays a tree: app depends on the views, the views depend on the state,
  and nothing depends on app. A view that could navigate would need to import
  the router that imports it, and a cycle in a hundred-line module is a cycle
  nobody notices until it is a thousand.
*/
'use strict';

import { $ } from './dom.js';
import { S, load, theCase, loadSaved, onCaseChange } from './state.js';
import { drawStatus, drawFoot } from './status.js';
import { renderRows } from './rows.js';
import { drawArchitecture } from './architecture.js';
import { renderManual } from './manual.js';
import { renderForms } from './forms.js';
import { renderExplore } from './explore.js';
import { showContract, showMock } from './banners.js';
import { loadOverrides, onOverrideChange } from './inputs.js';
import { caseSaid, drawWhatIf } from './casebar.js';

// ---------------------------------------------------------------------------
// boot

async function boot() {
  try {
    await load();
  } catch (e) {
    $('#status').textContent = 'the engine did not answer — reload';
    return;
  }
  // A page in doubt says so on every view, before anything else is drawn.
  showContract();
  showMock();
  // The saved case, so every status line and run panel can say which inputs
  // the numbers are for before the first run is asked for.
  await loadSaved();
  // What this browser remembered from last time. It is read after the index so
  // an override naming a row that no longer exists can be dropped rather than
  // sent to the engine.
  loadOverrides();
  // The bar is redrawn wherever an override is set or put back from, so it
  // says so at once, not at the next navigation.
  onOverrideChange(() => drawOverrideBar());
  wire();
  setRows();
  // A link to the manual opens it — `#manual`, or `#manual/<section>` to land
  // on the one step somebody was sent to read — `#row/<id>` opens a row's
  // page, and `#group/<id>` the rows of a group (sent from a node's page in
  // the application's Node workspace).
  const openFromHash = () => {
    const h = location.hash.match(/^#manual(?:\/([\w-]+))?$/);
    if (h) return setManual(h[1]);
    const r = location.hash.match(/^#row\/([\w-]+)$/);
    // In this entry's place, so Back does not land here and send the
    // reader on again.
    if (r && S.byId.has(r[1])) return open(r[1], true);
    const g = location.hash.match(/^#group\/([\w-]+)$/);
    if (g && S.G.has(g[1])) openGroup(g[1]);
  };
  openFromHash();
  // AND WHEN ONLY THE HASH CHANGES. A manual link pasted into a tab that already
  // has the tool open does not reload the page, so without this it changed the
  // address and nothing else — the reader was left where they were, told by the
  // address bar that they were somewhere they were not.
  window.addEventListener('hashchange', openFromHash);
}

// ---------------------------------------------------------------------------
// one draw

export function draw() {
  const v = S.view;
  $('#rowsview').hidden = v !== 'rows';
  $('#archview').hidden = v !== 'arch';
  $('#manualview').hidden = v !== 'manual';
  $('#formsview').hidden = v !== 'forms';
  $('#exploreview').hidden = v !== 'explore';
  // The case bar everywhere a number is shown: which inputs every run is on
  // is the first thing a reader of any number needs.
  $('#caserow').style.display = v === 'arch' || v === 'manual' || v === 'forms' ? 'none' : '';

  $('#rows-tab').classList.toggle('sel', v === 'rows');
  $('#arch-tab').classList.toggle('sel', v === 'arch');
  $('#manual-tab').classList.toggle('sel', v === 'manual');
  $('#forms-tab').classList.toggle('sel', v === 'forms');
  $('#explore-tab').classList.toggle('sel', v === 'explore');
  drawCaseBar();
  drawOverrideBar();
  drawFoot();
  drawStatus();
  if (v === 'arch') drawArchitecture();
}

/**
 * The case, across the top: its name, how far its inputs are from the
 * defaults, and the way in to change them.
 *
 * There is one case. This bar once held a button per customer and a menu of
 * conditions, each a file under cases/ — and a customer that needed a file and
 * a commit to exist was a tree that grew with every customer. What varies now
 * is the inputs, set in the application's Explore or uploaded as a CSV there,
 * and saved by the daemon outside the repository.
 */
function drawCaseBar() {
  const c = theCase();
  $('#case-name').textContent = c ? c.label : 'no case';
  $('#case-state').textContent = caseSaid();
}

/** The rows of a group: its own, or, for a group that holds groups, the
 *  first group under it that holds rows. The filter is cleared, so the
 *  group is in the list to be found. */
function openGroup(gid) {
  setRows(gid);
  const under = new Set();
  const walk = g => { under.add(g); (S.gkids.get(g) || []).forEach(walk); };
  walk(gid);
  const at = [...document.querySelectorAll('#rows-body .rows-grp')].find(s => under.has(s.dataset.groupId));
  if (at) at.scrollIntoView();
}

/** Every row, to open its page. */
function setRows(cleared) {
  S.view = 'rows';
  draw();
  renderRows($('#rows-body'), cleared !== undefined);
}

/** Open the Explore page: which design, where it breaks, what differs. */
async function setExplore() {
  S.view = 'explore';
  draw();
  await renderExplore($('#explore-body'));
}

/** Open the Forms page. */
function setForms() {
  S.view = 'forms';
  draw();
  renderForms($('#forms-body'));
}

/** The application's Explore, at its inputs or a result kept: they are kept
 *  there now, not on this page. */
function toExplore(view, file) {
  location.href = '/app#explore/' + view + (file ? '/' + encodeURIComponent(file) : '');
}

/** The what-if bar, above every view (`casebar.js`): put back, and a
 *  what-if made the case, redraw everything that shows a number. */
function drawOverrideBar() {
  drawWhatIf($('#ovrbar'), draw);
}


// ---------------------------------------------------------------------------
// navigation

/**
 * Open the manual — at a layer or a section when one is named.
 *
 * Rendered once per visit rather than on every draw: it is a page to read, and
 * redrawing it under a person's scroll position would lose their place.
 */
function setManual(target) {
  S.view = 'manual';
  draw();
  renderManual($('#manual-body'), target);
}

function setArch(section) {
  S.view = 'arch';
  if (section) S.arch = section;
  draw();
}

/** A row's page: the application's Node workspace, which holds it now
 *  (docs/briefs/F3.md, F3-h3). A link followed is an entry of its own, so
 *  Back returns to the rows; `#row/<id>` is replaced. */
function open(id, replace = false) {
  const to = '/app#node/' + encodeURIComponent(id);
  if (replace) location.replace(to); else location.href = to;
}

// ---------------------------------------------------------------------------
// every listener in the tool

function wire() {
  $('#rows-tab').onclick = () => setRows();
  $('#arch-tab').onclick = () => setArch();
  $('#manual-tab').onclick = () => { history.replaceState(null, '', '#manual'); setManual(); };
  $('#forms-tab').onclick = () => setForms();
  $('#explore-tab').onclick = () => setExplore();
  $('#case-inputs').onclick = () => toExplore('inputs');
  // Whoever changed the saved case, every number on screen was for the old
  // inputs.
  onCaseChange(() => draw());

  $('#arch-nav').addEventListener('click', e => {
    const b = e.target.closest('[data-arch]');
    if (b) setArch(b.dataset.arch);
  });

  // Cross-references, wherever they appear. One rule, one place: a row opens
  // its page; a group, the rows in it; the inputs and the results kept, the
  // application's Explore.
  document.addEventListener('click', e => {
    const x = e.target.closest('.xref');
    if (!x) return;
    e.preventDefault();
    if (x.dataset.goto) { if (S.byId.has(x.dataset.goto)) open(x.dataset.goto); }
    else if (x.dataset.group) openGroup(x.dataset.group);
    else if (x.dataset.inputs !== undefined) toExplore('inputs');
    else if (x.dataset.results !== undefined) toExplore('results', x.dataset.results || '');
  });

  document.addEventListener('keydown', e => {
    if (e.target.tagName === 'INPUT' || e.target.tagName === 'SELECT' || e.target.tagName === 'TEXTAREA') return;
    if (S.view !== 'rows' && e.key === 'Escape') setRows();
  });
}

// The tool is opened by a person, so a hand on the console is a person too.
window.S = S;
window.draw = draw;

boot();
