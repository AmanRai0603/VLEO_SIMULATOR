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
import { S, SIZES, HOWTO, load, layerRoot, subsystemLayers, subtreeAll } from './state.js';
import { drawFigure, drawCaptions, drawStepper, drawStatus, drawFoot } from './figure.js';
import { openNode } from './node.js';
import { renderRun } from './run.js';
import { drawArchitecture } from './architecture.js';
import { drawSolar } from './solar.js';

// ---------------------------------------------------------------------------
// boot

async function boot() {
  try {
    await load();
  } catch (e) {
    $('#status').textContent = 'the engine did not answer — reload';
    return;
  }
  fillSubsys();
  $('#howto-steps').innerHTML = HOWTO.map(t => '<li>' + t + '</li>').join('');
  wire();
  setLayer(1);
}

function fillSubsys() {
  const kids = subsystemLayers();
  $('#subsys').innerHTML = kids.map(id =>
    '<option value="' + id + '">' + S.G.get(id).label + ' · ' + subtreeAll(id).length +
    '</option>').join('');
  $('#subsys').value = S.subsys;
}

// ---------------------------------------------------------------------------
// one draw, four views

export function draw() {
  document.documentElement.style.setProperty('--cell', SIZES[S.size] + 'px');

  const v = S.view;
  $('#figure').hidden   = v !== 'layer';
  $('#nodeview').hidden = v !== 'node';
  $('#runview').hidden  = v !== 'run';
  $('#archview').hidden = v !== 'arch';
  $('#solarview').hidden = v !== 'solar';
  $('#caption-a').hidden = $('#caption-b').hidden = (v === 'node' || v === 'arch' || v === 'solar');
  $('#controls').style.display = v === 'layer' && S.layer !== 4 ? '' : 'none';
  $('#caserow').style.display = v === 'arch' || v === 'solar' || S.layer === 4 ? 'none' : '';
  $('#subsys-grp').style.display = S.layer === 3 ? '' : 'none';
  $('#concept-grp').style.display = S.layer === 1 ? '' : 'none';

  $('#arch-tab').classList.toggle('sel', v === 'arch');
  $('#solar-tab').classList.toggle('sel', v === 'solar');
  $$('.tab[data-layer]').forEach(b =>
    b.classList.toggle('sel', v !== 'arch' && v !== 'solar' && +b.dataset.layer === S.layer));
  $$('.ctl.sz').forEach(b => b.classList.toggle('sel', b.dataset.size === S.size));
  $$('.ctl.case').forEach(b => b.classList.toggle('sel', b.dataset.case === S.caseSel));
  $$('.ctl.cpt').forEach(b => b.classList.toggle('sel', b.dataset.concept === S.concept));
  $('#concept-tag').textContent = S.concept === 'single' ? 'Single satellite' : 'Constellation';

  drawCaptions();
  drawStepper();
  drawFoot();

  if (v === 'arch') { drawArchitecture(); drawStatus([]); return; }
  if (v === 'solar') { drawSolar(); drawStatus([]); return; }
  if (v === 'run')  { drawRunView(); drawStatus([]); return; }
  if (v === 'node') { drawStatus(S.disp); return; }
  drawStatus(drawFigure());
}

function drawRunView() {
  const id = S.runTarget || (S.byId.has('prop_thrust_to_drag') ? 'prop_thrust_to_drag' : S.rows[0].id);
  S.runTarget = id;
  renderRun($('#run-body'), S.byId.get(id), true);
}

// ---------------------------------------------------------------------------
// navigation

function select(id) {
  if (!S.dispIndex.has(id) && !S.byId.has(id)) return;
  S.selected = id;
  S.view = 'layer';
  draw();
  const el = $('#matrix-cells .mcell.diag.sel');
  if (el) el.scrollIntoView({ block: 'nearest', inline: 'nearest' });
}

function toggle(id) {
  if (S.expanded.has(id)) S.expanded.delete(id); else S.expanded.add(id);
  S.selected = id;
  S.view = 'layer';
  draw();
}

function setLayer(n) {
  S.layer = n;
  S.view = n === 4 ? 'run' : 'layer';
  S.expanded = new Set([layerRoot()]);
  S.selected = null;
  resetScroll();
  draw();
}

function setArch(section) {
  S.view = 'arch';
  if (section) S.arch = section;
  draw();
}

function resetScroll() {
  const g = $('.grid');
  if (g) { g.scrollTop = 0; g.scrollLeft = 0; }
}

/** A node the current layer does not show is still reachable. Following a
    cross-reference moves the layer to wherever that node lives. */
function goTo(id) {
  const r = S.byId.get(id);
  if (!r) return;
  if (r.layer !== S.layer) { S.layer = r.layer; S.expanded = new Set(); }
  if (r.layer === 3) {
    let g = S.G.get(r.parent);
    while (g && g.parent !== 'root') g = S.G.get(g.parent);
    if (g) { S.subsys = g.id; $('#subsys').value = g.id; }
  }
  S.expanded.add(layerRoot());
  let g = S.G.get(r.parent);
  while (g) { S.expanded.add(g.id); g = S.G.get(g.parent); }
  S.selected = id;
  S.view = 'layer';
  resetScroll();
  draw();
}

function goToGroup(gid) {
  const g = S.G.get(gid);
  if (!g) return;
  if (g.layer && g.layer !== S.layer) { S.layer = g.layer; S.expanded = new Set(); }
  if (g.layer === 3) {
    let t = g;
    while (t && t.parent !== 'root') t = S.G.get(t.parent);
    if (t) { S.subsys = t.id; $('#subsys').value = t.id; }
  }
  S.expanded.add(layerRoot());
  let t = S.G.get(gid);
  while (t) { S.expanded.add(t.id); t = S.G.get(t.parent); }
  S.selected = gid;
  S.view = 'layer';
  resetScroll();
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
  $('#arch-tab').onclick = () => setArch();
  $('#solar-tab').onclick = () => { S.view = 'solar'; resetScroll(); draw(); };
  $$('.tab[data-layer]').forEach(b => b.onclick = () => setLayer(+b.dataset.layer));
  $('#prev').onclick = () => setLayer(Math.max(1, S.layer - 1));
  $('#next').onclick = () => setLayer(Math.min(4, S.layer + 1));

  $('#expand-all').onclick = () => {
    const add = gid => { S.expanded.add(gid); (S.gkids.get(gid) || []).forEach(add); };
    add(layerRoot());
    draw();
  };
  $('#collapse-all').onclick = () => { S.expanded = new Set([layerRoot()]); resetScroll(); draw(); };
  $('#howto').onclick = () => { $('#howto-panel').hidden = !$('#howto-panel').hidden; };
  $('#howto-close').onclick = () => { $('#howto-panel').hidden = true; };

  $$('.ctl.sz').forEach(b => b.onclick = () => { S.size = b.dataset.size; draw(); });
  $$('.ctl.cpt').forEach(b => b.onclick = () => { S.concept = b.dataset.concept; draw(); });
  $$('.ctl.case').forEach(b => b.onclick = () => { S.caseSel = b.dataset.case; draw(); });
  $('#subsys').onchange = e => { S.subsys = e.target.value; setLayer(3); };
  $('#back').onclick = () => { S.view = 'layer'; draw(); };

  $('#arch-nav').addEventListener('click', e => {
    const b = e.target.closest('[data-arch]');
    if (b) setArch(b.dataset.arch);
  });

  $('#tree').addEventListener('click', e => {
    const row = e.target.closest('.trow');
    if (!row) return;
    const d = S.disp[+row.dataset.i];
    if (!d) return;
    if (d.kind === 'group' && d.hasKids && (d.id === S.selected || e.target.closest('.tri'))) toggle(d.id);
    else select(d.id);
  });
  $('#tree').addEventListener('dblclick', e => {
    const row = e.target.closest('.trow');
    if (!row) return;
    const d = S.disp[+row.dataset.i];
    if (d && d.kind === 'node') open(d.id);
    else if (d) toggle(d.id);
  });
  $('#paths-dots').addEventListener('click', e => {
    const dot = e.target.closest('.pdot');
    if (!dot) return;
    const i = Array.prototype.indexOf.call(dot.parentNode.children, dot);
    if (S.disp[i]) select(S.disp[i].id);
  });
  $('#matrix-cells').addEventListener('click', e => {
    const c = e.target.closest('.mcell.diag');
    if (!c) return;
    const d = S.disp[+c.dataset.i];
    if (!d) return;
    if (d.kind === 'node') open(d.id); else toggle(d.id);
  });

  // Cross-references, wherever they appear. One rule, one place.
  document.addEventListener('click', e => {
    const x = e.target.closest('.xref');
    if (!x) return;
    e.preventDefault();
    if (x.dataset.goto) goTo(x.dataset.goto);
    else if (x.dataset.group) goToGroup(x.dataset.group);
  });

  document.addEventListener('keydown', e => {
    if (e.target.tagName === 'INPUT' || e.target.tagName === 'SELECT') return;
    if (S.view !== 'layer') {
      if (e.key === 'Escape') { S.view = 'layer'; draw(); }
      return;
    }
    const i = S.dispIndex.get(S.selected);
    if (i == null) return;
    if (e.key === 'ArrowDown' && i + 1 < S.disp.length) { e.preventDefault(); select(S.disp[i + 1].id); }
    if (e.key === 'ArrowUp' && i > 0) { e.preventDefault(); select(S.disp[i - 1].id); }
    if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') {
      const d = S.disp[i];
      if (d.kind === 'group' && d.hasKids) { e.preventDefault(); toggle(d.id); }
    }
    if (e.key === 'Enter') {
      e.preventDefault();
      const d = S.disp[i];
      if (d.kind === 'node') open(d.id); else toggle(d.id);
    }
  });
}

// The tool is opened by a person, so a hand on the console is a person too.
window.S = S;
window.select = select;
window.openNode = open;
window.draw = draw;

boot();
