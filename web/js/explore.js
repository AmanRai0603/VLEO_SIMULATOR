/*
  Explore: which design is open, where it breaks, and what differs.

  docs/PLAN_F.md, stage F2. Three questions anyone on the programme asks of the
  design every day, each answered by the engine and only drawn here:

  * WHICH DESIGN IS THIS. Today's design is built from every group's latest
    sealed release on the drive; the design's own files are something else.
    A screen that does not say which it shows invites a decision made on the
    wrong one (docs/OPERATING_1_0.md, section 5), so the answer opens the page.
  * WHERE DOES IT BREAK. The health map: every node's state, each group as bad
    as its worst node, the spacecraft as bad as its worst group. A closure that
    does not close is walked down to the node that causes it — with its group,
    its owner and, when a release changed it, which release.
  * WHAT DIFFERS. Two files of the design, block by block, in the same words
    the library uses everywhere else.

  * RAISE AN ISSUE. Any mark that needs a person can be raised, addressed to
    its group and its node (W11). Only where the page is the application's:
    the issue is signed with the open key, so the host hands this module the
    one function that asks the program to write it.

  NOTHING HERE DECIDES A STATE. Every state, cause, margin and difference is
  the engine's (`/v1/today`, `/v1/health`, `/v1/health/trace`,
  `/v1/compare`); this module only lays them out. A colour the page chose for
  itself would be a second health map, and the two would drift.
*/
'use strict';

import { $, $$, esc, fmt, plural, answerFirst } from './dom.js';
import { withOverrides, overrideCount } from './inputs.js';

const PAGE = { open: null, trace: null, a: '', b: '', raise: null };

/** Worst first, as the engine orders them (vleo_modules::health::State). */
const ORDER = ['fails', 'refused', 'blocked', 'unproven', 'open', 'tight', 'closes'];

/** A branch opens by itself when something under it needs a person: it
 *  fails, refuses, waits or is not proven. Open and closing branches stay
 *  shut until clicked, so the map opens on where it breaks. */
const needsLook = s => ORDER.indexOf(s) < ORDER.indexOf('open');

async function get(url) {
  try { return await (await fetch(url)).json(); } catch (e) { return { ok: false, message: 'the engine did not answer: ' + e }; }
}

/** The case every run on the page is on: the saved case, and any what-if
 *  set in this browser on top, as a run sends them. */
const onTheCase = () => withOverrides(new URLSearchParams()).toString();

const chip = s => '<span class="hm hm-' + esc(s) + '">' + esc(s) + '</span>';
const pct = m => m == null ? '—' : (m >= 0 ? '+' : '') + (m * 100).toFixed(1) + '%';
const short = fp => fp ? fp.slice(0, 16) + '…' : '—';
const goto = id => '<a class="xref" href="#" data-goto="' + esc(id) + '">' + esc(id) + '</a>';

/** Draw the Explore page into `host`. `opts.raise(params)`, when given, asks
 *  the program to write an issue and answers what it said. */
export async function renderExplore(host, opts = {}) {
  if (!host) return;
  PAGE.raise = opts.raise || null;
  host.innerHTML = '<p class="hint-inline">asking the engine…</p>';
  const [today, map] = await Promise.all([get('/v1/today'), get('/v1/health?' + onTheCase())]);
  if (!today.ok || !map.ok) {
    const why = (!today.ok ? today : map).message || '';
    host.innerHTML = '<div class="blocked"><b>no answer</b><div>' + esc(why) + '</div></div>';
    return;
  }
  host.innerHTML =
    opening(today, map) +
    '<section class="ex-sec" id="ex-design">' + whichDesign(today) + '</section>' +
    '<section class="ex-sec" id="ex-health">' + healthMap(map) + '</section>' +
    '<section class="ex-sec" id="ex-closures">' + closures(map) + '</section>' +
    '<section class="ex-sec" id="ex-compare">' + compareForm(today) + '</section>';
  wire(host, map);
}

function opening(today, map) {
  const which = today.today
    ? 'This is <b>today’s design</b>, built from ' +
      plural(today.groups.filter(g => g.taken).length, 'group release') +
      ' on the drive ' + esc(today.drive) + ', fingerprint <code>' + esc(short(today.design.fingerprint)) + '</code>.'
    : 'This is <b>the design’s own files</b>, not today’s design: no drive was named when the tool started.';
  const counts = Object.entries(map.counts).filter(([, n]) => n > 0)
    .map(([s, n]) => n + ' ' + s).join(' · ');
  return answerFirst(
    which + ' The spacecraft ' + chip(map.spacecraft) + ', as bad as its worst group.',
    [
      counts + ', of ' + plural(map.nodes.length, 'node') + '.',
      'Branches that fail, refuse or wait are open; click any group to open or close it and see its nodes. Trace any closure to the node that causes it.',
      'On the saved case' + (overrideCount() ? ', with ' + plural(overrideCount(), 'what-if') + ' set in this browser' : '') +
        '. Every state here is the engine’s; the page draws it and decides none of it.',
    ],
    'reference');
}

/** A release's or a design's base: the released design it was built on. */
const baseOf = (on, n) => on ? esc(on) + (n == null ? '' : ' (the ' + n + nth(n) + ')') : (n === 0 ? 'none released yet' : '—');
const nth = n => (n % 100 >= 11 && n % 100 <= 13) ? 'th' : ({ 1: 'st', 2: 'nd', 3: 'rd' }[n % 10] || 'th');

function whichDesign(t) {
  const d = t.design;
  let h = '<h3>Which design</h3><dl class="ex-kv">' +
    '<dt>shown</dt><dd class="ex-which">' + esc(d.which) + '</dd>' +
    '<dt>fingerprint</dt><dd class="ex-fp" title="' + esc(d.fingerprint) + '"><code>' + esc(short(d.fingerprint)) + '</code></dd>' +
    '<dt>its base</dt><dd class="ex-base">' + esc(d.base) + '</dd>' +
    '<dt>from</dt><dd>' + esc(d.from) + '</dd>' +
    '<dt>rows</dt><dd>' + fmt(d.rows) + '</dd>' +
    (t.today ? '<dt>its own files</dt><dd title="' + esc(d.files_fingerprint) + '"><code>' +
      esc(short(d.files_fingerprint)) + '</code>, with each release below taken into them</dd>' : '') + '</dl>';
  if (!t.today) return h;
  h += '<table class="ex-tbl ex-releases"><thead><tr><th>group</th><th>release</th><th>sealed</th><th>its seal</th>' +
    '<th>based on</th><th>what to know</th></tr></thead><tbody>';
  for (const g of t.groups) {
    const k = g.taken;
    const notes = (k ? k.notes : []).map(n => '<li>' + esc(n) + '</li>');
    const refused = g.refused.map(r => '<li class="ex-refused">' + esc(r.version) + ' refused: ' + esc(r.why) + '</li>');
    h += '<tr data-group="' + esc(g.id) + '"' + (g.refused.length ? ' class="ex-row-refused"' : '') + '><td>' + esc(g.id) + '</td><td>' +
      (k ? esc(k.version) + (g.refused.length ? ' <span class="ex-mark">its last good one</span>' : '') : '<i>none passes</i>') + '</td><td>' +
      (k ? esc(k.sealed) + ' by <b>' + esc(k.sealed_by) + '</b>' : '—') + '</td><td class="ex-chain">' +
      (k ? esc(k.chain) : '—') + '</td><td>' + (k ? baseOf(k.based_on, k.base_version) : '—') + '</td><td><ul>' +
      refused.join('') + notes.join('') + '</ul></td></tr>';
  }
  h += '</tbody></table>';
  if (t.ignored.length) {
    h += '<p class="ex-ignored">not taken from the drive:</p><ul>' +
      t.ignored.map(i => '<li>' + esc(i) + '</li>').join('') + '</ul>';
  }
  return h;
}

/** The groups as a tree, each as bad as its worst node, worst first. */
function healthMap(map) {
  const kids = new Map();
  for (const g of map.groups) {
    if (!kids.has(g.parent)) kids.set(g.parent, []);
    kids.get(g.parent).push(g);
  }
  const ids = new Set(map.groups.map(g => g.id));
  const tops = map.groups.filter(g => !ids.has(g.parent));
  if (!PAGE.open) {
    PAGE.open = new Set(map.groups.filter(g => !ids.has(g.parent) || needsLook(g.state)).map(g => g.id));
  }
  const worst = (a, b) => ORDER.indexOf(a.state) - ORDER.indexOf(b.state) || a.id.localeCompare(b.id);
  const branch = g => {
    const under = (kids.get(g.id) || []).slice().sort(worst);
    const open = PAGE.open.has(g.id);
    const named = g.worst.slice(0, 3).map(esc).join(', ') + (g.worst.length > 3 ? ' and ' + (g.worst.length - 3) + ' more' : '');
    return '<li><button class="ex-grp" data-group="' + esc(g.id) + '" aria-expanded="' + open + '">' +
      (open ? '\u25be ' : '\u25b8 ') + chip(g.state) + ' <b>' + esc(g.id) + '</b> ' +
      esc(g.label) + '</button> <span class="hint-inline">' + named + '</span>' +
      (open ? nodesOf(map, g.id) + (under.length ? '<ul>' + under.map(branch).join('') + '</ul>' : '') : '') + '</li>';
  };
  return '<h3>Where it breaks</h3><ul class="ex-tree">' + tops.sort(worst).map(branch).join('') + '</ul>' +
    (map.notes.length ? '<ul class="ex-notes">' + map.notes.map(n => '<li>' + esc(n) + '</li>').join('') + '</ul>' : '');
}

function nodesOf(map, group) {
  const rows = map.nodes.filter(n => n.parent === group);
  if (!rows.length) return '';
  return '<table class="ex-tbl ex-nodes"><tbody>' + rows.map(n =>
    '<tr><td>' + chip(n.state) + '</td><td>' + goto(n.id) + '</td><td>' +
    (n.closure ? pct(n.margin) + ' ' : '') + esc(n.why) +
    (n.blocked_by ? ' — waits on ' + goto(n.blocked_by) : '') + '</td><td>' + raiseButton(n.id, n.state) +
    '</td></tr>').join('') +
    '</tbody></table>';
}

function closures(map) {
  const cs = map.nodes.filter(n => n.closure);
  const answer = cs.filter(n => n.margin != null);
  answer.sort((a, b) => ORDER.indexOf(a.state) - ORDER.indexOf(b.state) || a.margin - b.margin);
  return '<h3>Closures</h3><p class="hint-inline">' + plural(answer.length, 'closure') + ' answer, ' +
    (cs.length - answer.length) + ' do not. Trace one to what decides it.</p>' +
    '<table class="ex-tbl"><tbody>' + answer.map(n =>
      '<tr><td>' + chip(n.state) + '</td><td>' + goto(n.id) + '</td><td>' + pct(n.margin) +
      '</td><td><button class="ctl ex-trace" data-closure="' + esc(n.id) + '">trace</button></td></tr>').join('') +
    '</tbody></table>' +
    '<p><label>trace any closure <input class="ex-trace-id" list="ex-closure-ids" placeholder="l3_solar_ach_01"></label> ' +
    '<button class="ctl ex-trace-go">trace</button></p><datalist id="ex-closure-ids">' +
    cs.map(n => '<option value="' + esc(n.id) + '">').join('') + '</datalist>' +
    '<div class="ex-trace-out"></div>';
}

async function trace(el, id) {
  PAGE.trace = id;
  el.innerHTML = '<p class="hint-inline">tracing ' + esc(id) + '…</p>';
  const t = await get('/v1/health/trace?closure=' + encodeURIComponent(id) + '&' + onTheCase());
  if (!t.ok) {
    el.innerHTML = '<div class="blocked"><b>' + esc(id) + '</b><div>' + esc(t.message || '') + '</div></div>';
    return;
  }
  const lever = l => '<tr><td>' + goto(l.id) + '</td><td>' + pct(l.at_lower) + ' to ' + pct(l.at_upper) +
    '</td><td>' + (l.closes_at == null ? 'the same side across its range'
      : 'closes from ' + fmt(l.closes_at / l.factor) + (l.unit && l.unit !== '-' ? ' ' + esc(l.unit) : '')) +
    '</td><td>' + esc(l.group) + ' · ' + esc(l.owner) + '</td></tr>';
  el.innerHTML = '<h4>' + esc(t.closure) + ' ' + chip(t.state) + ' ' + pct(t.margin) + ' ' +
    raiseButton(t.closure, t.state) + '</h4>' +
    (t.why ? '<p>' + esc(t.why) + '</p>' : '') +
    (t.causes.length
      ? '<p><b>caused by, nearest first</b></p><table class="ex-tbl"><tbody>' + t.causes.map(c =>
          '<tr><td>' + chip(c.state) + '</td><td>' + goto(c.id) + '</td><td>' + esc(c.group) + ' · ' +
          esc(c.owner) + '</td><td>' + esc(c.why) + '</td></tr>').join('') + '</tbody></table>'
      : '<p>no row it reads is refused, open or unproven, and no release changed one</p>') +
    '<p><b>over its open values:</b> ' + esc(t.range.said) + '</p>' +
    (t.range.tornado.length ? '<table class="ex-tbl"><tbody>' + t.range.tornado.map(lever).join('') + '</tbody></table>' : '') +
    '<p class="hint-inline">the least mature value it rests on is ' + esc(t.range.least_mature) + ': ' +
    t.range.least_mature_rows.slice(0, 3).map(esc).join(', ') +
    (t.range.least_mature_rows.length > 3 ? ' and ' + (t.range.least_mature_rows.length - 3) + ' more' : '') + '</p>' +
    (t.levers.length ? '<p><b>what moves its margin, most first, each across its declared range</b></p>' +
      '<table class="ex-tbl"><tbody>' + t.levers.map(lever).join('') + '</tbody></table>' : '');
}

/** *raise issue*, where the page can write one and the mark needs a person. */
function raiseButton(id, state) {
  if (!PAGE.raise || !needsLook(state)) return '';
  return '<button class="ctl ex-raise" data-node="' + esc(id) + '">raise issue</button>';
}

/** The form under a mark: the person's words, and the issue raised with the
 *  engine's evidence on the same case as the map. */
function raiseForm(button) {
  const id = button.dataset.node;
  const at = button.closest('tr') || button.closest('h4');
  const next = at.nextElementSibling;
  if (next && next.classList.contains('ex-raising')) { next.remove(); return; }
  const form = document.createElement(at.tagName === 'TR' ? 'tr' : 'div');
  form.className = 'ex-raising';
  form.innerHTML = (at.tagName === 'TR' ? '<td colspan="4">' : '') +
    '<label>what is wrong with <b>' + esc(id) + '</b> <textarea class="ex-what" rows="2" cols="60"></textarea></label> ' +
    '<button class="ctl ex-raise-go">raise</button> <span class="ex-raise-said"></span>' +
    '<p class="hint-inline">Written into your copy\'s issues/, addressed to its group and to ' + esc(id) +
    ', with what the engine says of it on this case, signed with your key.</p>' +
    (at.tagName === 'TR' ? '</td>' : '');
  at.after(form);
  form.querySelector('.ex-raise-go').onclick = async () => {
    const p = new URLSearchParams(onTheCase());
    p.set('node', id);
    p.set('what', form.querySelector('.ex-what').value);
    const said = form.querySelector('.ex-raise-said');
    said.textContent = 'raising…';
    const r = await PAGE.raise(p);
    said.className = 'ex-raise-said ' + (r.ok ? 'ex-raised' : 'ex-raise-refused');
    said.textContent = r.message || (r.ok ? 'raised' : 'not raised');
    if (r.ok) form.dataset.file = r.file;
  };
}

function compareForm(t) {
  return '<h3>What differs</h3><p class="hint-inline">Two files of the design, block by block. Name each as ' +
    '<code>design:&lt;path&gt;</code>' + (t.today ? ' or <code>drive:&lt;path&gt;</code>' : '') +
    ', such as <code>design:groups/l3_solar/nodes/sw_ap_design.vnode</code>.</p>' +
    '<p><input class="ex-a" size="56" placeholder="design:groups/…" value="' + esc(PAGE.a) + '"> ' +
    '<input class="ex-b" size="56" placeholder="drive:groups/…" value="' + esc(PAGE.b) + '"> ' +
    '<button class="ctl ex-compare-go">compare</button></p><div class="ex-compare-out"></div>';
}

async function compare(el, a, b) {
  PAGE.a = a; PAGE.b = b;
  el.innerHTML = '<p class="hint-inline">comparing…</p>';
  const c = await get('/v1/compare?a=' + encodeURIComponent(a) + '&b=' + encodeURIComponent(b));
  if (!c.ok) {
    el.innerHTML = '<div class="blocked"><b>' + esc(c.node || '') + '</b><div>' + esc(c.message || '') + '</div></div>';
    return;
  }
  const list = ds => ds.length ? '<ul>' + ds.map(d => '<li>' + esc(d) + '</li>').join('') + '</ul>' : '';
  el.innerHTML = '<p><b>' + (c.same_content ? 'the same content' : esc(c.summary)) + '</b></p>' +
    (c.file.length ? '<p>in the file itself</p>' + list(c.file) : '') +
    c.blocks.map(b => '<div class="ex-blk"><b>' + esc(b.id) + '</b> ' + esc(b.status) +
      (b.before && b.after && b.before !== b.after ? ' (was ' + esc(b.before) + ')' : '') +
      list(b.differences) + '</div>').join('') +
    (c.history.some(h => h.only_first || h.only_second)
      ? '<p class="hint-inline">history, counted not compared: ' + c.history.filter(h => h.only_first || h.only_second)
          .map(h => esc(h.table) + ' ' + h.only_first + '/' + h.only_second).join(' · ') + '</p>'
      : '');
}

function wire(host, map) {
  $$('.ex-grp', host).forEach(b => b.onclick = () => {
    const id = b.dataset.group;
    if (PAGE.open.has(id)) PAGE.open.delete(id); else PAGE.open.add(id);
    $('#ex-health', host).innerHTML = healthMap(map);
    wire(host, map);
  });
  const out = $('.ex-trace-out', host);
  $$('.ex-trace', host).forEach(b => b.onclick = () => trace(out, b.dataset.closure));
  $('.ex-trace-go', host).onclick = () => {
    const id = $('.ex-trace-id', host).value.trim();
    if (id) trace(out, id);
  };
  $('.ex-compare-go', host).onclick = () =>
    compare($('.ex-compare-out', host), $('.ex-a', host).value.trim(), $('.ex-b', host).value.trim());
  if (PAGE.trace && !out.innerHTML) trace(out, PAGE.trace);
  // One listener for every raise button, those a trace draws later included.
  if (!host.dataset.raising) {
    host.dataset.raising = '1';
    host.addEventListener('click', e => {
      const b = e.target.closest('.ex-raise');
      if (b && host.contains(b)) raiseForm(b);
    });
  }
}
