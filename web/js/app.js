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

import { $, $$ } from './dom.js';
import { S, load, theCase, loadSaved, onCaseChange } from './state.js';
import { drawStatus, drawFoot } from './status.js';
import { renderRows } from './rows.js';
import { openNode } from './node.js';
import { drawArchitecture } from './architecture.js';
import { renderManual } from './manual.js';
import { saveOverridesToCase, savableOverrides } from './case.js';
import { renderForms } from './forms.js';
import { renderExplore } from './explore.js';
import { showContract, showMock } from './banners.js';
import { initDepth } from './depth.js';
import { loadOverrides, overrideCount, clearAllOverrides, clearOverride,
         fromSI, onOverrideChange } from './inputs.js';

// ---------------------------------------------------------------------------
// boot

async function boot() {
  initDepth();
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
  // The bar is redrawn wherever an override is set from — the node page's own
  // control sets one without going through a full draw, and a bar that only
  // appeared on the next navigation would be missing at the one moment it has
  // something to say.
  onOverrideChange(() => drawOverrideBar());
  wire();
  setRows();
  // A link to the manual opens it — `#manual`, or `#manual/<section>` to land
  // on the one step somebody was sent to read — and `#row/<id>` opens a row's
  // page.
  const openFromHash = () => {
    const h = location.hash.match(/^#manual(?:\/([\w-]+))?$/);
    if (h) return setManual(h[1]);
    const r = location.hash.match(/^#row\/([\w-]+)$/);
    if (r && S.byId.has(r[1])) open(r[1]);
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
  $('#nodeview').hidden = v !== 'node';
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
  const d = S.saved;
  $('#case-state').textContent = !d ? 'the case could not be read'
    : !d.stored ? 'every input at its default'
    : d.changed + ' input' + (d.changed === 1 ? '' : 's') + ' changed from default' +
      (d.upgrade ? ' · carried over to this version' + (d.upgrade.set_aside.length
        ? ', ' + d.upgrade.set_aside.length + ' value' + (d.upgrade.set_aside.length === 1 ? '' : 's') +
          ' set aside' : '') : '');
}

/** Every row, to open its page. */
function setRows() {
  S.view = 'rows';
  draw();
  renderRows($('#rows-body'));
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

/**
 * The what-if bar.
 *
 * Hidden when nothing is overridden — a control strip that is always there
 * teaches a reader to stop seeing it, and the one moment this has to be seen
 * is the moment it appears. Every row it lists is a number the design does not
 * hold, so each one names itself, shows what it is running at, and can be put
 * back on its own without disturbing the others.
 */
function drawOverrideBar() {
  const bar = $('#ovrbar');
  const n = overrideCount();
  bar.hidden = n === 0;
  if (!n) return;
  const savable = savableOverrides();
  const rows = [];
  for (const [id, si] of S.overrides) {
    const r = S.byId.get(id);
    if (!r) continue;
    rows.push('<span class="ovr-chip xref" data-goto="' + id + '">' +
      '<b>' + (r.symbol || r.label) + '</b> ' + fmtNum(fromSI(r, si)) + ' ' + r.unit +
      '<button class="ovr-x" data-drop="' + id + '" title="put this one back">×</button>' +
      '</span>');
  }
  // THE BAR SAYS WHAT IS TRUE; IT DOES NOT RUN ANYTHING. Update lives beside
  // the field on the node page, which is where the hand already is. A second
  // update button here re-rendered this bar during its own click — the field's
  // blur fires first, the bar redraws, and the button the click was travelling
  // to no longer exists. One action, one place.
  bar.innerHTML =
    // WHAT THIS BAR MAY CLAIM. It used to say "everything below is a what-if",
    // and that is not true of a figure drawn from the record: sw_regime is
    // byte-identical with and without an override, correctly, because the
    // measured record is the measured record whatever the design is set to.
    // A reader looking at that chart under this bar would have been told it
    // reflected their change. Anything the ENGINE computes does follow the
    // override — the relation curves move — so that is what it says.
    '<div class="ovr-bar-h"><b>' + n + ' input' + (n === 1 ? '' : 's') +
      ' overridden.</b> Every number the engine computes below is a what-if. ' +
      'A figure drawn from the measured record is unchanged, because the record ' +
      'is. The sheets on disk are untouched and nothing is written to the ' +
      'repository.</div>' +
    '<div class="ovr-chips">' + rows.join('') +
      '<button class="ctl ovr-bar-clear">put all back</button>' +
      // A what-if worth keeping is made the case here — saved by the daemon,
      // outside the repository, like any other change to the inputs.
      (savable ? '<button class="ctl ovr-bar-save" title="make ' + (savable === n ? 'these' :
        savable + ' of these') + ' part of the saved case: every run on every page then uses ' +
        (savable === 1 ? 'it' : 'them') + '">save ' + (savable === n ? '' : savable + ' ') +
        'to the case</button>' : '') +
      '<span class="why ovr-bar-said"></span></div>';

  $$('.ovr-x', bar).forEach(b => b.onclick = e => {
    e.stopPropagation();
    clearOverride(b.dataset.drop);
    S.lastRun = null;
    draw();
  });
  $('.ovr-bar-clear', bar).onclick = () => { clearAllOverrides(); S.lastRun = null; draw(); };
  const save = $('.ovr-bar-save', bar);
  if (save) save.onclick = async () => {
    save.disabled = true;
    const res = await saveOverridesToCase();
    // On success the case changed and everything showing a number redraws,
    // this bar with it; only a refusal is left to say here.
    if (!res.ok && bar.isConnected) {
      save.disabled = false;
      $('.ovr-bar-said', bar).textContent = 'not saved: ' + (res.message || 'refused');
    }
  };
}

/** Six significant figures, as the rest of the face writes a number. */
const fmtNum = v => (v == null || !isFinite(v)) ? '—' : String(Number(v.toPrecision(6)));

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

async function open(id) {
  S.selected = id;
  S.runTarget = id;
  S.view = 'node';
  draw();
  await openNode(id);
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
  // inputs. The node page is re-opened rather than patched: its input fields,
  // its figures and its run panel all read the case, and re-opening is the one
  // path that is already known to draw all of them from nothing.
  onCaseChange(() => {
    if (S.view === 'node' && S.selected) { draw(); openNode(S.selected); return; }
    draw();
  });
  $('#back').onclick = () => setRows();

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
    else if (x.dataset.group) { setRows(); const g = $('[data-group-id="' + CSS.escape(x.dataset.group) + '"]'); if (g) g.scrollIntoView(); }
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
window.openNode = open;
window.draw = draw;

boot();
