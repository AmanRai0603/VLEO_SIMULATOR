/*
  What a group sees of its own folder: every page drawn from the files.

  The group's page and each node's page are laid out the way the application
  lays out a node — the answer first, then the explanation's stations, then
  the mathematics, the algorithm, the inputs, the results — so the group
  reviews its work in the shape everyone will read it in. Each part says
  which file it came from, so a reader who wants something changed knows
  exactly where to change it.

  NOTHING HERE RUNS. The pseudocode is shown, and shown again as equations;
  the results are the group's own, plotted. Computing is the developer's
  engine's job, after the folder is sealed.
*/
'use strict';

import { esc, answerFirst } from './dom.js';
import { renderMd, sections } from './md.js';
import { texToMathml, pseudocodeEquations } from './texmath.js';
import { parseCsv, records, splitUnit, num } from './csv.js';
import { findEquation, findFigure, symbolsFor, resultColumns } from './gmodel.js';
import { figureFromCsv, flowFromCsv, flowSvg, mountSteps, wiringFlow, reach, captioned } from './gdraw.js';
import { drawFigureInto } from './figures.js';

const src = path => '<span class="gsrc" title="the file this comes from">' + esc(path) + '</span>';

// ── equations and embeds ───────────────────────────────────────────────────

/** An equation by id, typeset, with the symbols it uses explained. */
function equationBlock(ctx, node, id) {
  const e = findEquation(ctx.model, node, id);
  if (!e) return '<p class="gmiss">No equation ' + esc(id) + '.</p>';
  const m = texToMathml(e.latex, true);
  const used = symbolsFor(ctx.model, node).filter(s => s.symbol && e.latex.includes(s.symbol));
  return '<figure class="geq" id="eq-' + esc(id) + '"><div class="geq-row"><span class="geq-id">' + esc(id) + '</span>' + m.html + '</div>' +
    (e.says ? '<figcaption>' + esc(e.says) + (e.source ? ' <span class="claim claim-sourced">' + esc(e.source) + '</span>' : '') + '</figcaption>' : '') +
    (used.length ? '<dl class="geq-where"><dt>where</dt>' + used.map(s =>
      '<dd>' + texToMathml(s.symbol).html + ' — ' + esc(s.name || '') + (s.unit ? ' [' + esc(s.unit) + ']' : '') +
      (s.means ? ': ' + esc(s.means) : '') + '</dd>').join('') + '</dl>' : '') + '</figure>';
}

/** Markdown with the folder's embeds filled in. Figures are drawn after insertion (`mountFigures`). */
export function md(ctx, node, text) {
  return renderMd(text, {
    math: (tex, display) => texToMathml(tex, display).html,
    embed: (name, arg, block) => {
      if (name === 'eq') return block ? equationBlock(ctx, node, arg) : '<a class="geq-ref" href="#eq-' + esc(arg) + '">' + esc(arg) + '</a>';
      if (name === 'fig') return '<div class="gfig" data-fig="' + esc(arg) + '" data-node="' + esc(node ? node.id : '') + '"></div>';
      if (name === 'node') return '<a class="gnode-link" href="#/node/' + esc(arg) + '">' + esc(arg) + '</a>';
      if (name === 'guess') {
        const [q, a] = arg.split('||').map(s => s.trim());
        return '<div class="gguess"><p class="gguess-q"><span class="gguess-k">Guess first</span> ' + esc(q || '') + '</p>' +
          '<button class="ctl gguess-go" type="button">I have my answer — show me</button>' +
          '<p class="gguess-a" hidden>' + esc(a || '') + '</p></div>';
      }
      return esc('{{' + name + ' ' + arg + '}}');
    },
  });
}

/** Draw every figure placeholder inside `host`, and wire the guess buttons. */
export async function mountFigures(ctx, host) {
  host.querySelectorAll('.gguess-go').forEach(b => b.addEventListener('click', () => {
    b.hidden = true; b.parentElement.querySelector('.gguess-a').hidden = false;
  }));
  for (const el of host.querySelectorAll('.gfig')) {
    const node = el.dataset.node ? ctx.model.nodes.get(el.dataset.node) : null;
    await drawFigure(ctx, el, node, el.dataset.fig);
  }
}

async function drawFigure(ctx, el, node, id) {
  const hit = findFigure(ctx.model, node, id);
  if (!hit) { el.innerHTML = '<p class="gmiss">No figure ' + esc(id) + ' in figures.csv.</p>'; return; }
  const { fig, base } = hit;
  const path = base + fig.file;
  const f = ctx.folder;
  const cap = '<p class="gfig-cap">' + esc(fig.title || '') + ' ' + src(path) + '</p>';
  if (!f.has(path)) { el.innerHTML = '<p class="gmiss">' + esc(path) + ' is not in the folder.</p>'; return; }
  if (fig.kind === 'image') { el.innerHTML = '<img class="gimg" alt="' + esc(fig.title || '') + '" src="' + f.url(path) + '">' + cap; return; }
  if (fig.kind === 'video') { el.innerHTML = '<video class="gimg" controls preload="metadata" src="' + f.url(path) + '"></video>' + cap; return; }
  const text = await f.text(path);
  if (fig.kind === 'flow') { const fl = flowFromCsv(text); el.innerHTML = '<div class="gflow-host">' + flowSvg(fl, captioned(fl, { label: fig.title })) + '</div>' + cap; return; }
  if (fig.kind === 'steps') {
    const on = fig.on ? findFigure(ctx.model, node, fig.on) : null;
    const flowText = on && f.has(on.base + on.fig.file) ? await f.text(on.base + on.fig.file) : 'from,to,label\n';
    el.innerHTML = '<div class="gsteps"></div>' + cap;
    mountSteps(el.querySelector('.gsteps'), flowFromCsv(flowText), text, { label: fig.title });
    return;
  }
  const desc = figureFromCsv(fig, text);
  if (!desc) { el.innerHTML = '<p class="gmiss">' + esc(fig.kind) + ' is not a kind of figure this page draws.</p>'; return; }
  // The player states the title above the chart; beneath it, only where it came from.
  el.innerHTML = '<div class="gfig-host"></div><p class="gfig-cap">' + src(path) + '</p>';
  drawFigureInto(el.querySelector('.gfig-host'), desc, fig.id);
}

// ── the explanation, as stations ───────────────────────────────────────────

const STATION_DEPTH = { 'in one line': '', 'said simply': 'd-lr', 'picture it': '', 'guess first': 'd-l', 'where it breaks': '', 'common misreading': 'd-lr' };

function stations(ctx, node, text, path) {
  if (text === null || text === undefined) return '<p class="gmiss">' + esc(path) + ' is missing.</p>';
  const s = sections(text);
  return (s.intro.trim() ? '<div class="prose">' + md(ctx, node, s.intro) + '</div>' : '') +
    s.sections.map((x, i) => '<section class="station gst ' + (STATION_DEPTH[x.title.toLowerCase()] || '') + '">' +
      '<h3 class="st-h"><span class="st-n">' + (i + 1) + '</span> ' + esc(x.title) + '</h3><div class="prose">' + md(ctx, node, x.body) + '</div></section>').join('') +
    '<p class="gfrom">' + src(path) + '</p>';
}

function theory(ctx, node, text, path) {
  if (text === null || text === undefined) return '<p class="gmiss">' + esc(path) + ' is missing.</p>';
  const s = sections(text);
  return (s.intro.trim() ? '<div class="prose">' + md(ctx, node, s.intro) + '</div>' : '') +
    s.sections.map(x => '<section class="gth"><h3>' + esc(x.title) + '</h3><div class="prose">' + md(ctx, node, x.body) + '</div></section>').join('') +
    '<p class="gfrom">' + src(path) + '</p>';
}

// ── tables ─────────────────────────────────────────────────────────────────

function table(t, opts = {}) {
  if (!t || !t.head.length) return '<p class="muted">' + (opts.empty || 'None.') + '</p>';
  return '<div class="ri-wrap"><table class="fx gtable"><thead><tr>' + t.head.map(h => '<th>' + esc(h) + '</th>').join('') +
    '</tr></thead><tbody>' + t.rows.map((r, i) => '<tr' + (opts.rowClass ? ' class="' + opts.rowClass(r, i) + '"' : '') + '>' +
      r.map((c, k) => '<td>' + (opts.cell ? opts.cell(c, k, r) : esc(c)) + '</td>').join('') + '</tr>').join('') + '</tbody></table></div>';
}

// ── the group's page ───────────────────────────────────────────────────────

export function groupPage(ctx) {
  const { model, findings } = ctx;
  const m = model.meta;
  const errors = findings.filter(f => f.level === 'error').length, warnings = findings.filter(f => f.level === 'warning').length;
  const computed = [...model.nodes.values()].filter(n => n.row.kind === 'computed').length;
  return answerFirst(esc(m.summary || 'This group has not yet said what it answers in one sentence (group.csv, summary).'), [
    esc(model.nodes.size + ' nodes') + ', ' + esc(computed + ' computed') + ' — ' + esc(model.edges.length + ' connections inside the group, ' + model.external.length + ' from other groups'),
    errors ? '<b>' + errors + ' error(s)</b> and ' + warnings + ' warning(s) before this folder can be sealed — <a href="#/checks">see the checks</a>'
      : 'No errors' + (warnings ? '; ' + warnings + ' warning(s)' : '') + ' — <a href="#/sign">sign and seal</a>',
    'Owner ' + esc(m.owner || '—') + ' · version ' + esc(m.version || '—'),
  ], 'explanation') +
    '<h2 class="gh">The group, understood</h2>' + stations(ctx, null, model.g.text['explanation.md'], 'explanation.md') +
    '<h2 class="gh">How the nodes connect</h2><p class="muted">Drawn from every node\'s inputs.csv. Point at a node to see what feeds it ' +
    '(<span class="gkey-in">violet</span>) and what it feeds (<span class="gkey-out">amber</span>); click it to open it.</p>' +
    '<div class="gwire">' + wiringSvg(model) + '</div>' +
    '<h2 class="gh">The flow, as the group wrote it</h2><pre class="md-code gflowtxt"><code>' + esc(model.g.text['flow.txt'] || '(flow.txt is missing)') + '</code></pre>' + '<p class="gfrom">' + src('flow.txt') + '</p>' +
    '<h2 class="gh">Theory &amp; maths across the nodes</h2>' + theory(ctx, null, model.g.text['theory.md'], 'theory.md') +
    groupTables(ctx) +
    '<h2 class="gh">The group\'s own results</h2>' + resultsBlock(ctx, model.g.files['results/group.csv'], 'results/group.csv', null) +
    '<h2 class="gh">Sources</h2>' + table(model.g.files['sources.csv'], { empty: 'sources.csv is missing.' }) +
    '<h2 class="gh">Versions</h2>' + versions(model.group.versions);
}

function wiringSvg(model) {
  const flow = wiringFlow(model);
  return flowSvg(flow, {
    label: 'how the nodes of this group connect',
    kind: id => model.nodes.has(id) ? 'k-' + model.nodes.get(id).row.kind : (id.startsWith('→') ? 'k-out' : 'k-ext'),
    sub: id => model.nodes.has(id) ? (model.nodes.get(id).row.output || '') + (model.nodes.get(id).row.unit ? ' [' + model.nodes.get(id).row.unit + ']' : '') : '',
    name: id => id,
    title: id => model.nodes.has(id) ? model.nodes.get(id).row.question : id,
    link: true,
  });
}

/** Hover and click on a wiring drawing. */
export function wireUp(ctx, host) {
  host.querySelectorAll('.gwire').forEach(w => {
    w.querySelectorAll('.gbox').forEach(g => {
      const id = g.dataset.id;
      g.addEventListener('mouseenter', () => {
        if (!ctx.model.nodes.has(id)) return;
        const { up, down } = reach(ctx.model, id);
        w.querySelectorAll('.gbox').forEach(b => {
          b.classList.toggle('up', up.has(b.dataset.id));
          b.classList.toggle('down', down.has(b.dataset.id));
          b.classList.toggle('on', b.dataset.id === id);
        });
        w.classList.add('lit');
      });
      g.addEventListener('mouseleave', () => {
        w.classList.remove('lit');
        w.querySelectorAll('.gbox').forEach(b => b.classList.remove('up', 'down', 'on'));
      });
      const go = () => { if (ctx.model.nodes.has(id)) location.hash = '#/node/' + id; };
      g.addEventListener('click', go);
      g.addEventListener('keydown', e => { if (e.key === 'Enter') go(); });
    });
  });
}

function groupTables(ctx) {
  const g = ctx.model.g.files;
  const part = (title, path, empty) => '<h3 class="gh3">' + title + ' ' + src(path) + '</h3>' + table(g[path], { empty });
  return '<h2 class="gh">What the group promises, shares and loops</h2>' +
    part('Requirements, and which way each binds', 'requirements.csv', 'No requirements.') +
    part('What the group sends on', 'publishes.csv', 'Nothing is sent on.') +
    part('Declared loops', 'loops.csv', 'No loops.') +
    part('Shared constants', 'constants.csv', 'No shared constants.') +
    part('Equations', 'equations.csv', 'No group-level equations.');
}

function versions(rows) {
  if (!rows.length) return '<p class="muted">versions.csv is missing or empty.</p>';
  return '<ol class="gver">' + rows.slice().reverse().map(v => '<li><p class="version-h"><b>' + esc(v.version) + '</b> · ' + esc(v.date || '') + ' · ' + esc(v.by || '') + '</p>' +
    '<dl class="gdl"><dt>We believed</dt><dd>' + esc(v.believed || '—') + '</dd><dt>We tested</dt><dd>' + esc(v.tested || '—') + '</dd>' +
    '<dt>We now know</dt><dd>' + esc(v.learned || '—') + '</dd><dt>What changed</dt><dd>' + esc(v.changed || '—') + '</dd>' +
    (v.risks ? '<dt>Risks</dt><dd>' + esc(v.risks) + '</dd>' : '') + '</dl></li>').join('') + '</ol>';
}

// ── a node's page ──────────────────────────────────────────────────────────

export const NODE_TABS = [
  ['explain', 'Explanation'], ['theory', 'Theory & maths'], ['algorithm', 'Algorithm'], ['io', 'Inputs & output'],
  ['results', 'Results'], ['evidence', 'Evidence'], ['code', 'Code'], ['pictures', 'Pictures'], ['checks', 'Checks'],
];

export function nodePage(ctx, id, tab) {
  const n = ctx.model.nodes.get(id);
  if (!n) return '<p class="gmiss">There is no node ' + esc(id) + ' in nodes.csv.</p>';
  const r = n.row;
  const mine = ctx.findings.filter(f => f.where.startsWith(n.dir) || (f.msg.includes(id) && !f.where.startsWith('nodes/')));
  const errors = mine.filter(f => f.level === 'error').length;
  const authors = ctx.model.group.members.filter(m => String(m.nodes || '').split(/\s+/).some(x => x === id || x === '*')).map(m => m.name);
  const range = (r.lower || r.upper) ? ' · from ' + esc(r.lower || '—') + ' to ' + esc(r.upper || '—') + ' ' + esc(r.unit || '') : '';
  const head = answerFirst('<span class="af-q">' + esc(r.question || '') + '</span><br>' +
    '<b>' + esc(r.output || id) + '</b>' + (r.unit ? ' [' + esc(r.unit) + ']' : '') + range +
    (r.kind === 'declared' ? ' · declared value <b>' + esc(r.value || '—') + '</b>' : ''), [
    '<span class="gkind gk-' + esc(r.kind) + '">' + esc(r.kind || '?') + '</span> · authors: ' + esc(authors.join(', ') || 'nobody yet — members.csv'),
    errors ? '<b>' + errors + ' error(s)</b> in this node — <a href="#/node/' + esc(id) + '/checks">see them</a>' : 'No errors in this node',
  ], 'explanation');
  const tabs = '<div class="tabs gtabs" role="tablist">' + NODE_TABS.map(([k, label]) =>
    '<a class="tab' + (k === tab ? ' sel' : '') + '" role="tab" href="#/node/' + esc(id) + '/' + k + '">' + esc(label) +
    (k === 'checks' && mine.length ? ' <span class="gcount">' + mine.length + '</span>' : '') + '</a>').join('') + '</div>';
  const body = {
    explain: () => '<p class="dx dx-explanation">explanation</p>' + stations(ctx, n, n.text['explanation.md'], n.dir + 'explanation.md'),
    theory: () => '<p class="dx dx-reference">reference</p>' + theory(ctx, n, n.text['theory.md'], n.dir + 'theory.md') + equationsList(ctx, n),
    algorithm: () => algorithm(ctx, n),
    io: () => io(ctx, n),
    results: () => resultsBlock(ctx, n.files['results/isolation.csv'], n.dir + 'results/isolation.csv', n) +
      (n.text['results/how-run.md'] ? '<h3 class="gh3">How these results were made ' + src(n.dir + 'results/how-run.md') + '</h3><div class="prose">' + md(ctx, n, n.text['results/how-run.md']) + '</div>' : ''),
    evidence: () => '<p class="muted">Values from OUTSIDE any code — a paper, a handbook, another tool. They check the physics; the results check the code.</p>' +
      table(n.files['evidence.csv'], { empty: 'No evidence yet (evidence.csv).' }) + '<p class="gfrom">' + src(n.dir + 'evidence.csv') + '</p>',
    code: () => codeTab(ctx, n),
    pictures: () => {
      const figs = n.files['figures.csv'] ? records(n.files['figures.csv']) : [];
      return figs.length ? figs.map(f => '<h3 class="gh3">' + esc(f.id) + ' <span class="muted">' + esc(f.kind) + '</span></h3><div class="gfig" data-fig="' + esc(f.id) + '" data-node="' + esc(n.id) + '"></div>').join('')
        : '<p class="muted">No pictures yet (figures.csv).</p>';
    },
    checks: () => findingsList(mine, 'No findings for this node.'),
  }[tab] || (() => '');
  return head + tabs + '<div class="gtab-body">' + body() + '</div>';
}

function equationsList(ctx, n) {
  const own = n.files['equations.csv'] ? records(n.files['equations.csv']) : [];
  if (!own.length) return '';
  return '<h3 class="gh3">Every equation of this node ' + src(n.dir + 'equations.csv') + '</h3>' + own.map(e => equationBlock(ctx, n, e.id)).join('');
}

const KEYWORDS = /\b(let|set|const|if|then|else|end|for|to|return|refuse|and|or|not)\b/g;

function algorithm(ctx, n) {
  const code = n.text['pseudocode.txt'];
  if (code === null || code === undefined) return n.row.kind === 'computed' ? '<p class="gmiss">pseudocode.txt is missing — every computed node needs one.</p>' : '<p class="muted">A ' + esc(n.row.kind) + ' node has no algorithm.</p>';
  const eqs = pseudocodeEquations(code, n.row.output || 'answer');
  const lines = code.replace(/\r/g, '').replace(/\n+$/, '').split('\n').map((l, i) => {
    // Code first, then its comment: a keyword inside a comment is just a word.
    const hash = l.indexOf('#');
    const body = hash < 0 ? l : l.slice(0, hash), note = hash < 0 ? '' : l.slice(hash);
    const hl = esc(body).replace(KEYWORDS, '<span class="pc-k">$1</span>').replace(/\[([^\]]*)\]/g, '<span class="pc-u">[$1]</span>') +
      (note ? '<span class="pc-c">' + esc(note) + '</span>' : '');
    return '<span class="pc-line" data-line="' + (i + 1) + '"><span class="pc-n">' + (i + 1) + '</span>' + (hl || ' ') + '</span>';
  }).join('');
  return '<p class="muted">The algorithm, exactly as the developer will build it. <b>Nothing here runs it</b>: the developer\'s code is generated from these lines and must give the numbers in Results.</p>' +
    '<div class="galgo"><pre class="md-code gpc"><code>' + lines + '</code></pre>' +
    '<div class="galgo-eq"><p class="gh3">The same lines, as equations</p>' +
    (eqs.length ? eqs.map(e => '<div class="galgo-row" data-line="' + e.line + '"><span class="pc-n">' + e.line + '</span>' + texToMathml(e.tex, true).html + '</div>').join('')
      : '<p class="muted">No let or return lines to show.</p>') + '</div></div><p class="gfrom">' + src(n.dir + 'pseudocode.txt') + '</p>';
}

/** Point at a pseudocode line and its equation lights up, and the other way round. */
export function wireAlgorithm(host) {
  const all = host.querySelectorAll('[data-line]');
  all.forEach(el => {
    el.addEventListener('mouseenter', () => all.forEach(o => o.classList.toggle('on', o.dataset.line === el.dataset.line)));
    el.addEventListener('mouseleave', () => all.forEach(o => o.classList.remove('on')));
  });
}

function io(ctx, n) {
  const { up, down } = reach(ctx.model, n.id);
  const near = ctx.model.edges.filter(e => e.to === n.id || e.from === n.id);
  const flow = { boxes: [...new Set([n.id, ...near.map(e => e.from), ...near.map(e => e.to), ...ctx.model.external.filter(e => e.to === n.id).map(e => e.from)])].map(id => ({ id, caption: '' })),
    arrows: near.map(e => ({ from: e.from, to: e.to, label: e.input })).concat(ctx.model.external.filter(e => e.to === n.id).map(e => ({ from: e.from, to: e.to, label: e.input }))) };
  return '<h3 class="gh3">Its inputs ' + src(n.dir + 'inputs.csv') + '</h3>' +
    table(n.files['inputs.csv'], { empty: n.row.kind === 'computed' ? 'inputs.csv is missing.' : 'A ' + esc(n.row.kind) + ' node takes no inputs.',
      cell: (c, k, r) => {
        const h = splitUnit(n.files['inputs.csv'].head[k]).name;
        return h === 'from' && ctx.model.nodes.has(c) ? '<a href="#/node/' + esc(c) + '">' + esc(c) + '</a>' : esc(c);
      } }) +
    '<h3 class="gh3">Where it sits</h3><p class="muted">' + up.size + ' node(s) feed it, directly or not; it feeds ' + down.size + '.</p>' +
    (flow.arrows.length ? '<div class="gwire">' + flowSvg(flow, { label: 'the node and its neighbours', link: true,
      kind: id => id === n.id ? 'k-self' : ctx.model.nodes.has(id) ? 'k-' + ctx.model.nodes.get(id).row.kind : 'k-ext' }) + '</div>' : '<p class="muted">Nothing in this group connects to it.</p>');
}

function codeTab(ctx, n) {
  const files = n.code;
  if (!files.length) return '<p class="muted">No code kept with this node (code/). The results came from: ' +
    esc([...new Set((n.files['results/isolation.csv'] ? records(n.files['results/isolation.csv']) : []).map(r => r.origin).filter(Boolean))].join(', ') || 'nothing yet') + '.</p>';
  return '<p class="muted">The author\'s own code, kept for the record. Nothing runs it here; what it gave is in Results.</p>' +
    files.map(p => '<details class="gcode" data-path="' + esc(p) + '"><summary>' + esc(p.slice(n.dir.length)) + '</summary><pre class="md-code"><code>…</code></pre></details>').join('');
}

/** Fill the code files when opened. */
export function wireCode(ctx, host) {
  host.querySelectorAll('details.gcode').forEach(d => d.addEventListener('toggle', async () => {
    if (!d.open || d.dataset.done) return;
    d.dataset.done = '1';
    const t = await ctx.folder.text(d.dataset.path);
    d.querySelector('code').textContent = t === null ? '(not readable)' : t;
  }));
}

// ── results ────────────────────────────────────────────────────────────────

function resultsBlock(ctx, t, path, node) {
  if (!t) return '<p class="muted">' + esc(path) + (node && node.row.kind === 'computed' ? ' is missing — the developer\'s code has nothing to be checked against.' : ' — none.') + '</p>';
  const cols = resultColumns(t);
  const ref = cols.other.refuses;
  const inputs = node ? node.inputs : [];
  const isDefault = r => inputs.length && inputs.every(inp => {
    const c = cols.inputs.find(x => x.name === inp.name);
    return c && num(r[c.k]) !== null && num(r[c.k]) === num(inp.default);
  });
  const answered = t.rows.filter(r => !(ref !== undefined && r[ref] === 'yes')).length;
  const refused = t.rows.length - answered;
  const pick = (cols.inputs.length > 1 ? '<label class="gpick">across <select class="gres-x">' +
    cols.inputs.map((c, i) => '<option value="' + i + '">' + esc(c.name) + '</option>').join('') + '</select></label> ' : '') +
    (cols.answers.length > 1 ? '<label class="gpick">showing <select class="gres-y">' +
    cols.answers.map((c, i) => '<option value="' + i + '">' + esc(c.name) + '</option>').join('') + '</select></label>' : '');
  return '<p>' + esc(answered + ' answer(s) and ' + refused + ' refusal(s).') + (node ? ' These are the numbers the developer\'s code must give, within each row\'s tolerance.' : '') + '</p>' +
    (cols.inputs.length && cols.answers.length ? '<div class="gres-plot" data-path="' + esc(path) + '">' + pick + '<div class="gres-fig"></div></div>' : '') +
    table(t, { rowClass: r => (ref !== undefined && r[ref] === 'yes' ? 'grow-refuse' : '') + (isDefault(r) ? ' grow-default' : ''),
      cell: (c, k) => k === ref && c === 'yes' ? '<b>refuses</b>' : esc(c) }) +
    '<p class="muted gkeyline"><span class="grow-default-k">default row</span> <span class="grow-refuse-k">refusal</span></p>' +
    '<p class="gfrom">' + src(path) + '</p>';
}

/** Plot each results table: the answer against the chosen input. */
export async function mountResults(ctx, host) {
  for (const el of host.querySelectorAll('.gres-plot')) {
    const t = parseCsv(await ctx.folder.text(el.dataset.path));
    const cols = resultColumns(t);
    const ref = cols.other.refuses;
    let xi = cols.inputs[0], yi = cols.answers[0];
    const draw = () => {
      const rows = t.rows.filter(r => !(ref !== undefined && r[ref] === 'yes'));
      const fig = {
        id: 'results', kind: 'scatter', says: yi.name + ' in each row of the table, against ' + xi.name + '. Refusals are not drawn.',
        x: { id: xi.name, label: xi.name, unit: xi.unit || '1' },
        y: { id: yi.name, label: yi.name, unit: yi.unit || '1' },
        series: [yi].map(a => {
          const pts = rows.map(r => [num(r[xi.k]), num(r[a.k])]).filter(p => p[0] !== null && p[1] !== null).sort((p, q) => p[0] - q[0]);
          return { name: a.name, x: pts.map(p => p[0]), y: pts.map(p => p[1]) };
        }),
        notes: [], gaps: [],
      };
      drawFigureInto(el.querySelector('.gres-fig'), fig, 'results-' + yi.name + '-' + xi.name);
    };
    const sx = el.querySelector('.gres-x'), sy = el.querySelector('.gres-y');
    if (sx) sx.addEventListener('change', () => { xi = cols.inputs[+sx.value]; draw(); });
    if (sy) sy.addEventListener('change', () => { yi = cols.answers[+sy.value]; draw(); });
    draw();
  }
}

// ── findings ───────────────────────────────────────────────────────────────

export function findingsList(list, empty) {
  if (!list.length) return '<p class="gok">' + esc(empty) + '</p>';
  return '<ul class="gfind">' + list.map(f => {
    const node = /^nodes\/([^/]+)\//.exec(f.where);
    const href = node ? '#/node/' + node[1] + '/' + tabFor(f.where) : '#/';
    return '<li class="gf-' + f.level + '"><span class="gf-l">' + esc(f.level) + '</span> <a href="' + href + '">' + esc(f.where) +
      (f.line ? ':' + f.line : '') + '</a> ' + esc(f.msg) + '</li>';
  }).join('') + '</ul>';
}

function tabFor(path) {
  if (path.includes('explanation')) return 'explain';
  if (path.includes('theory') || path.includes('equations')) return 'theory';
  if (path.includes('pseudocode')) return 'algorithm';
  if (path.includes('inputs')) return 'io';
  if (path.includes('results')) return 'results';
  if (path.includes('evidence')) return 'evidence';
  if (path.includes('figures')) return 'pictures';
  return 'checks';
}

