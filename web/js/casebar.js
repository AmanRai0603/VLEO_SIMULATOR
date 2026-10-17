/*
  The case and the what-if, said on every page that shows a number.

  Today's tool page carries them above every view, and the Node workspace
  above a node's page (docs/briefs/F3.md, F3-h2): which inputs the numbers are
  for is the first thing a reader of any number needs, and a what-if must
  never be mistaken for the design. One drawing, so the two pages say it alike.
*/
'use strict';

import { $, $$, esc } from './dom.js';
import { S } from './state.js';
import { saveOverridesToCase, savableOverrides } from './case.js';
import { overrideCount, clearAllOverrides, clearOverride, fromSI } from './inputs.js';

/** The case in a phrase: how far its inputs are from the defaults. */
export function caseSaid() {
  const d = S.saved;
  return !d ? 'the case could not be read'
    : !d.stored ? 'every input at its default'
    : d.changed + ' input' + (d.changed === 1 ? '' : 's') + ' changed from default' +
      (d.upgrade ? ' · carried over to this version' + (d.upgrade.set_aside.length
        ? ', ' + d.upgrade.set_aside.length + ' value' + (d.upgrade.set_aside.length === 1 ? '' : 's') +
          ' set aside' : '') : '');
}

/**
 * The what-if bar.
 *
 * Hidden when nothing is overridden — a control strip that is always there
 * teaches a reader to stop seeing it, and the one moment this has to be seen
 * is the moment it appears. Every row it lists is a number the design does not
 * hold, so each one names itself, shows what it is running at, and can be put
 * back on its own without disturbing the others. Drawn into `bar`; `redraw`
 * redraws everything on the page that shows a number, once one is put back.
 */
export function drawWhatIf(bar, redraw) {
  if (!bar) return;
  const n = overrideCount();
  bar.hidden = n === 0;
  if (!n) return;
  const savable = savableOverrides();
  const rows = [];
  for (const [id, si] of S.overrides) {
    const r = S.byId.get(id);
    if (!r) continue;
    // A row's words are a release's, so they are escaped as every other
    // place that draws them escapes them: in `/app` the page holds the word
    // that signs.
    rows.push('<span class="ovr-chip xref" data-goto="' + esc(id) + '">' +
      '<b>' + esc(r.symbol || r.label) + '</b> ' + fmtNum(fromSI(r, si)) + ' ' + esc(r.unit) +
      '<button class="ovr-x" data-drop="' + esc(id) + '" title="put this one back">×</button>' +
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
    redraw();
  });
  $('.ovr-bar-clear', bar).onclick = () => { clearAllOverrides(); S.lastRun = null; redraw(); };
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
