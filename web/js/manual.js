/*
  The manual — how to use this tool, from the browser and from a terminal, as
  somebody reading the design and as somebody building it.

  NOTHING HERE IS WRITTEN IN THIS FILE. The words come from docs/manual.toml,
  served at /v1/manual, and are held true by two checks: one that every name
  the manual uses exists (and everything that exists is in the manual), and one
  that runs every command it tells a person to paste. A second copy of any of
  it here would be the copy nobody checks.

  What IS decided here is what is true of the copy of the tool you are looking
  at right now — whether it will accept an edit, whose name an edit carries,
  how many rows there are — and that is not decided either: it is read from the
  same response, which the daemon fills from the functions that decide it.
*/
'use strict';

import { $, $$, esc } from './dom.js';

let DATA = null;
/** Which part of the manual is showing, and for whom. Kept across visits. */
const VIEW = { layer: 'browser', who: 'all' };

async function load() {
  if (DATA) return DATA;
  const r = await fetch('/v1/manual');
  const d = await r.json();
  if (!d.ok) throw new Error(d.message || 'the manual did not load');
  DATA = d;
  return d;
}

/**
 * The manual's small markup: paragraphs on a blank line, a line break on a
 * single one, `code` and **bold**. Escaped FIRST, so nothing in the file can
 * become markup it did not ask for.
 */
function md(text) {
  return String(text || '').trim().split(/\n\s*\n/).map(p =>
    '<p>' + esc(p)
      .replace(/`([^`]+)`/g, '<code>$1</code>')
      .replace(/\*\*([^*]+)\*\*/g, '<b>$1</b>')
      .replace(/\*([^*\s][^*]*)\*/g, '<i>$1</i>')
      .replace(/\n/g, '<br>') +
    '</p>').join('');
}

function inline(text) {
  return md(text).replace(/^<p>|<\/p>$/g, '');
}

/** Who a section is for, as the badge reads. */
const WHO = { user: 'user', developer: 'developer', everyone: 'everyone' };

/** Whether a section shows under the chosen filter. */
function shows(who) {
  if (VIEW.who === 'all' || who === 'everyone') return true;
  return who === VIEW.who;
}

/**
 * What the pipeline does with a command, in the words a reader needs.
 *
 * Shown on every command because it answers the question a careful person has
 * before pasting anything: will this change something?
 */
function checkNote(step) {
  switch (step.check) {
    case 'exits': return '<span class="man-tag ok">safe to run — only reads</span>';
    case 'fails': return '<span class="man-tag refuse">shows a refusal — it is meant to fail</span>';
    case 'serves': return '<span class="man-tag serve">starts the tool — stop it with Ctrl-C</span>';
    case 'probe': return '<span class="man-tag serve">needs the tool running</span>';
    case 'ci': return '<span class="man-tag ci">the pipeline runs this on every push</span>';
    case 'writes': return '<span class="man-tag writes">changes files: ' + esc(step.why || '') + '</span>';
    default: return '';
  }
}

function cmdBlock(cmd, note) {
  return '<div class="man-cmd"><div class="man-cmd-box"><pre><code>' + esc(cmd) + '</code></pre>' +
    '<button class="ctl man-copy" data-copy="' + esc(cmd) + '" title="copy this command">copy</button></div>' +
    (note ? '<div class="man-cmd-note">' + note + '</div>' : '') + '</div>';
}

function stepHtml(st) {
  return '<li class="man-step">' +
    '<div class="man-say">' + inline(st.say) +
      (st.ui ? ' <kbd class="man-ui">' + esc(st.ui) + '</kbd>' : '') + '</div>' +
    (st.run ? cmdBlock(st.run, checkNote(st)) : '') +
    '</li>';
}

function table(head, rows) {
  if (!rows.length) return '<p class="muted">Nothing for this reader.</p>';
  return '<div class="man-tablewrap"><table class="man-table"><thead><tr>' +
    head.map(h => '<th>' + h + '</th>').join('') + '</tr></thead><tbody>' +
    rows.join('') + '</tbody></table></div>';
}

/** The prefix a person types before a command of each program. */
const PREFIX = {
  xtask: 'cargo run -p xtask -- ',
  vleo: 'cargo run -p vleo-cli --bin vleo -- ',
};

/**
 * The tables some sections carry, drawn from the lists in the manual and from
 * what the running tool reports. Keyed by section id — the check
 * `the_sections_the_page_fills_from_live_data_exist` holds these names to the
 * manual, so a section renamed there fails a test instead of losing its table.
 */
function extra(id, d) {
  const m = d.manual, L = d.live;
  const cannotRows = place => m.cannot.filter(c => c.place === place && shows(c.who)).map(c =>
    '<tr><td><b>' + esc(c.what) + '</b><div class="muted">' + esc(c.why) + '</div></td>' +
    '<td>' + (/^(cargo|python3) /.test(c.instead) ? cmdBlock(c.instead, '') : inline(c.instead)) + '</td></tr>');
  const cmds = tool => m.commands.filter(c => c.tool === tool && shows(c.who)).map(c =>
    '<tr><td><code class="man-usage">' + esc(PREFIX[tool] + c.usage) + '</code></td>' +
    '<td>' + inline(c.what) + '</td>' +
    '<td><span class="man-effect ' + esc(c.effect) + '">' + esc(c.effect) + '</span></td></tr>');
  switch (id) {
    case 'browser-limits':
      return table(['You cannot', 'Where it is done instead'], cannotRows('browser'));
    case 'code-folders':
      return table(['Entry', 'What it is'], m.folders.filter(f => shows(f.who)).map(f =>
        '<tr><td><code>' + esc(f.name) + '</code></td><td>' + inline(f.what) + '</td></tr>'));
    case 'code-docs':
      return table(['Document', 'What it is for'], m.docs.map(x =>
        '<tr><td><code>' + esc(x.path) + '</code></td><td>' + inline(x.what) + '</td></tr>'));
    case 'ref-xtask':
      return table(['Command', 'What it does', 'Effect'], cmds('xtask'));
    case 'ref-vleo':
      return table(['Command', 'What it does', 'Effect'], cmds('vleo'));
    case 'ref-routes':
      return table(['Route', 'What it answers'], m.routes.map(r =>
        '<tr><td><code>' + esc(r.method + ' ' + r.path) + '</code>' +
        (r.writes ? ' <span class="man-effect writes">writes</span>' : '') + '</td>' +
        '<td>' + inline(r.what) + '</td></tr>'));
    case 'ref-env':
      return table(['Setting', 'What it does', 'When unset'], m.env.map(e =>
        '<tr><td><code>' + esc(e.name) + '</code></td><td>' + inline(e.what) + '</td>' +
        '<td class="muted">' + esc(e.default) + '</td></tr>'));
    case 'ref-sheet': {
      const groups = [];
      for (const f of L.fields) if (!groups.includes(f.group)) groups.push(f.group);
      return groups.map(g =>
        '<h4 class="man-sub">' + esc(g) + '</h4>' +
        table(['Field', 'The question', 'Without it'], L.fields.filter(f => f.group === g).map(f =>
          '<tr><td><code>' + esc(f.field) + '</code>' +
          (f.blocks ? ' <span class="man-effect writes" title="nothing is generated from the row until this is answered">blocks</span>' : '') +
          '<div class="muted">' + esc(f.shape) + '</div></td>' +
          '<td>' + esc(f.ask) + '</td><td class="muted">' + esc(f.why) + '</td></tr>'))).join('') +
        '<h4 class="man-sub">lists</h4>' +
        table(['List', 'Its keys', 'What it is'], L.arrays.map(a =>
          '<tr><td><b>' + esc(a.label) + '</b>' +
          (a.end_only ? '<div class="muted">added and removed at the end only</div>' : '') + '</td>' +
          '<td>' + a.keys.map(k => '<code>' + esc(k) + '</code>').join(' ') + '</td>' +
          '<td class="muted">' + esc(a.why) + '</td></tr>'));
    }
    case 'ref-locked':
      return table(['Field', 'Why the browser will not change it'], L.locked.map(x =>
        '<tr><td><code>' + esc(x.field) + '</code></td><td>' + esc(x.why) + '</td></tr>'));
    default:
      return '';
  }
}

/**
 * What is true of this copy of the tool, right now.
 *
 * The first thing on the page because it is what decides whether the rest
 * applies: a person told how to save a sheet, looking at a copy that will
 * refuse every save, needs to be told that before step one.
 */
function liveHtml(L) {
  const write = L.writes_allowed
    ? '<div class="man-live-row good"><b>This copy accepts edits.</b> A saved field is written to ' +
      'the row’s <code>node.toml</code>, the row is regenerated and checked, and a refused ' +
      'save puts everything back.</div>'
    : '<div class="man-live-row"><b>This copy is read-only.</b> You can read everything, run ' +
      'anything and try any what-if; saving a sheet is refused. To edit, stop it and start it with:' +
      cmdBlock('VLEO_ALLOW_WRITE=1 cargo run --release -p vleo-daemon', '') + '</div>';
  const who = L.identity
    ? (L.identity_is_agent
      ? '<div class="man-live-row bad">Edits here would be signed <b>' + esc(L.identity) + '</b>, ' +
        'which is an agent’s name, so a change to a relation will be refused: an agent may ' +
        'never supply mathematics. Set your own name for this checkout, then reload:' +
        cmdBlock('git config user.name "Your Name"', '') + '</div>'
      : '<div class="man-live-row">Edits here are signed <b>' + esc(L.identity) + '</b> ' +
        '<span class="muted">— this checkout’s <code>git config user.name</code>.</span></div>')
    : '<div class="man-live-row bad">' + esc(L.identity_why || 'This checkout has no name to sign with.') + '</div>';
  return '<div class="man-live">' +
    '<div class="man-live-h">This copy of the tool, right now</div>' +
    write + who +
    '<div class="man-live-row muted">' + L.rows + ' rows · ' + L.published +
      ' published · ' + L.seeded + ' seeded and waiting to be filled · serving on port ' +
      L.port + '</div>' +
    '</div>';
}

function sectionHtml(s, d) {
  return '<section class="man-sec" id="man-' + esc(s.id) + '" data-who="' + esc(s.who) + '">' +
    '<h3>' + esc(s.title) + ' <span class="man-who ' + esc(s.who) + '">' + esc(WHO[s.who]) + '</span></h3>' +
    md(s.body) +
    (s.steps.length ? '<ol class="man-steps">' + s.steps.map(stepHtml).join('') + '</ol>' : '') +
    extra(s.id, d) +
    '</section>';
}

function draw(host) {
  const d = DATA;
  const layers = d.manual.layers;
  const L = layers.find(l => l.id === VIEW.layer) || layers[0];
  const secs = L.sections.filter(s => shows(s.who));

  host.innerHTML =
    '<div class="man">' +
    '<aside class="man-side">' +
      '<div class="man-side-h">For</div>' +
      '<div class="man-filter" role="group" aria-label="who this is for">' +
        [['all', 'everyone'], ['user', 'a user'], ['developer', 'a developer']].map(([k, t]) =>
          '<button class="ctl man-f' + (VIEW.who === k ? ' sel' : '') + '" data-who="' + k + '">' +
          t + '</button>').join('') +
      '</div>' +
      '<p class="man-side-note">A <b>user</b> reads the design, asks what-if, fills in a sheet’s ' +
        'words and numbers and proposes changes. A <b>developer</b> also changes the tree itself: ' +
        'rows, Rust, evidence, checks, data.</p>' +
      layers.map(l =>
        '<div class="man-side-h' + (l.id === L.id ? ' cur' : '') + '">' +
          '<a href="#manual/' + esc(l.id) + '" class="man-layer-link" data-layer="' + esc(l.id) + '">' +
          esc(l.title) + '</a></div>' +
        (l.id === L.id
          ? '<nav class="man-toc">' + l.sections.filter(s => shows(s.who)).map(s =>
              '<a href="#manual/' + esc(s.id) + '" data-sec="' + esc(s.id) + '">' + esc(s.title) +
              '</a>').join('') + '</nav>'
          : '')).join('') +
    '</aside>' +
    '<article class="man-main">' +
      liveHtml(d.live) +
      '<div class="man-layers" role="tablist">' + layers.map(l =>
        '<button class="ctl man-lt' + (l.id === L.id ? ' sel' : '') + '" role="tab" data-layer="' +
        esc(l.id) + '">' + esc(l.title) + '</button>').join('') + '</div>' +
      '<h2 class="man-h">' + esc(L.title) + '</h2>' +
      '<div class="man-lede">' + md(L.lede) + '</div>' +
      secs.map(s => sectionHtml(s, d)).join('') +
      (L.id === 'terminal'
        ? '<section class="man-sec"><h3>What the terminal will not do</h3>' +
          table(['You cannot', 'Where it is done instead'],
            d.manual.cannot.filter(c => c.place === 'terminal' && shows(c.who)).map(c =>
              '<tr><td><b>' + esc(c.what) + '</b><div class="muted">' + esc(c.why) + '</div></td>' +
              '<td>' + inline(c.instead) + '</td></tr>')) + '</section>'
        : '') +
      '<p class="man-foot muted">This manual is <code>docs/manual.toml</code>. Every name on it ' +
        'is checked against the code \u2014 both ways, so something added and not written in here ' +
        'fails a test too \u2014 and every command marked safe to run is run by the pipeline ' +
        'exactly as written.</p>' +
    '</article>' +
    '</div>';

  wireIn(host);
}

function wireIn(host) {
  $$('.man-f', host).forEach(b => b.onclick = () => { VIEW.who = b.dataset.who; draw(host); });
  $$('.man-lt, .man-layer-link', host).forEach(b => b.onclick = e => {
    e.preventDefault();
    VIEW.layer = b.dataset.layer;
    draw(host);
    history.replaceState(null, '', '#manual/' + VIEW.layer);
    host.scrollIntoView({ block: 'start' });
  });
  $$('.man-toc a', host).forEach(a => a.onclick = e => {
    e.preventDefault();
    const el = $('#man-' + a.dataset.sec, host);
    if (el) el.scrollIntoView({ block: 'start', behavior: 'smooth' });
    history.replaceState(null, '', '#manual/' + a.dataset.sec);
  });
  // COPY WHAT IS SHOWN, EXACTLY. The command a person copies is the one the
  // pipeline ran, character for character; a copy button that tidied it would
  // hand over something nobody has checked.
  $$('.man-copy', host).forEach(b => b.onclick = async () => {
    const text = b.dataset.copy;
    let ok = false;
    try { await navigator.clipboard.writeText(text); ok = true; } catch { /* below */ }
    if (!ok) {
      // No clipboard permission — an http page on some browsers. Select the
      // text so one more keystroke copies it, rather than pretending.
      const code = b.parentElement.querySelector('code');
      const r = document.createRange(); r.selectNodeContents(code);
      const s = getSelection(); s.removeAllRanges(); s.addRange(r);
    }
    b.textContent = ok ? 'copied' : 'selected — press Ctrl-C';
    setTimeout(() => { b.textContent = 'copy'; }, 1800);
  });
}

/**
 * Open the manual, at a layer or a section if one is named.
 *
 * `#manual/terminal` or `#manual/term-run` in the address opens it there, so a
 * person can be sent a link to exactly the step they need.
 */
export async function renderManual(host, target) {
  host.innerHTML = '<p class="muted">loading the manual…</p>';
  try {
    await load();
  } catch (e) {
    host.innerHTML = '<p class="empty">The manual did not load: ' + esc(String(e.message || e)) +
      '. It is <code>docs/manual.toml</code>; run <code>cargo test -p xtask --test ' +
      'the_manual_is_true</code> to see why.</p>';
    return;
  }
  let sec = null;
  if (target) {
    const layers = DATA.manual.layers;
    if (layers.some(l => l.id === target)) VIEW.layer = target;
    else {
      const l = layers.find(l => l.sections.some(s => s.id === target));
      if (l) { VIEW.layer = l.id; sec = target; VIEW.who = 'all'; }
    }
  }
  draw(host);
  if (sec) {
    const el = $('#man-' + sec, host);
    if (el) el.scrollIntoView({ block: 'start' });
  }
}
