/*
  The checks, read from the pattern itself (groups/SPEC.toml).

  Which files a folder must have, which columns each table carries and what
  values they allow, which headings each text needs and in what order — all
  of it comes from the spec, so the rule a group is told and the rule it is
  checked against are the same text. The checks this file adds are the ones
  that cross files: an input that comes from a node that does not exist, a
  result that leaves out the defaults, an embed that names nothing, theory
  that repeats the explanation.

  Three levels. An ERROR stops the folder being sealed. A WARNING is shown and
  does not. A NOTE is something worth knowing. Every finding names its file
  and, where it can, its line.
*/
'use strict';

import { parseCsv, records, splitUnit, num, column } from './csv.js';
import { sections, longSentences } from './md.js';
import { texToMathml, pseudocodeEquations } from './texmath.js';
import { readMethod } from './gmethod.js';
import { findEquation, findFigure, resultColumns } from './gmodel.js';

const ID = /^[a-z][a-z0-9_]*$/;

export async function checkGroup(model, spec) {
  const out = [];
  const add = (level, where, msg, line) => out.push({ level, where, msg, line: line || 0 });
  const f = model.folder;
  const files = spec.file || [];

  // ── files, by level ──
  for (const rule of files.filter(r => r.level === 'group' || r.level === 'both')) {
    if (rule.path.endsWith('/')) continue;
    if (rule.required && !f.has(rule.path)) add('error', rule.path, 'is missing — ' + rule.says);
  }
  for (const n of model.nodes.values()) {
    if (!n.present) { add('error', n.dir, 'has no folder, though nodes.csv lists ' + n.id); continue; }
    for (const rule of files.filter(r => r.level === 'node')) {
      if (rule.path.endsWith('/')) continue;
      const kinds = rule.kinds || null;
      if (kinds && !kinds.includes(n.row.kind)) continue;
      if (rule.required && !f.has(n.dir + rule.path)) add('error', n.dir + rule.path, 'is missing — ' + firstLine(rule.says));
    }
  }
  for (const id of model.stray) add('warning', 'nodes/' + id + '/', 'is a folder nodes.csv does not list');

  // ── tables: columns and allowed values ──
  const checkTable = (path, t, rule) => {
    if (!t) return;
    for (const p of t.problems) add('error', path, p.msg, p.line);
    if (!rule || !rule.columns) return;
    for (const c of rule.columns) {
      if (c.prefix) {
        if (!t.head.some(h => splitUnit(h).name.startsWith(c.name))) add('error', path, 'has no ' + c.name + ' column — ' + c.says, 1);
        continue;
      }
      const k = column(t, c.name);
      if (k < 0) { if (!c.optional) add('error', path, 'has no column ' + c.name + ' — ' + c.says, 1); continue; }
      if (c.one_of) t.rows.forEach((r, i) => {
        if (r[k] !== '' && !c.one_of.includes(r[k])) add('error', path, c.name + ' is "' + r[k] + '"; it must be one of ' + c.one_of.join(', '), t.lines[i]);
      });
      // A refusal has no answer and so no tolerance: blank is right there.
      const refK = column(t, 'refuses');
      if (!c.optional && !c.one_of) t.rows.forEach((r, i) => {
        if (r[k] === '' && !(refK >= 0 && r[refK] === 'yes')) add('warning', path, c.name + ' is blank', t.lines[i]);
      });
    }
    if (rule.rows === 'one' && t.rows.length !== 1) add('error', path, 'must have exactly one row; it has ' + t.rows.length);
  };
  const ruleFor = (path, level) => files.find(r => r.path === path && (r.level === level || r.level === 'both'));
  for (const [p, t] of Object.entries(model.g.files)) checkTable(p, t, ruleFor(p, 'group'));
  for (const n of model.nodes.values()) for (const [p, t] of Object.entries(n.files)) checkTable(n.dir + p, t, ruleFor(p, 'node'));

  // ── the group's identity and people ──
  const meta = model.meta;
  if (meta.id && !ID.test(meta.id)) add('error', 'group.csv', 'the id "' + meta.id + '" is not lower case letters, digits and underscores');
  const versions = model.group.versions;
  if (meta.version && versions.length && !versions.some(v => v.version === meta.version)) {
    add('error', 'versions.csv', 'has no row for version ' + meta.version + ' — say what this version changed');
  }
  const members = model.group.members;
  if (model.g.files['members.csv'] && !members.some(m => m.role === 'owner')) add('error', 'members.csv', 'names no owner — somebody must sign the whole group');
  for (const m of members) for (const id of String(m.nodes || '').split(/\s+/).filter(x => x && x !== '*')) {
    if (!model.nodes.has(id)) add('error', 'members.csv', m.name + ' is the author of "' + id + '", which is not a node', m._line);
  }

  // ── nodes.csv ──
  const seen = new Set();
  for (const r of records(model.g.files['nodes.csv'] || { head: [], rows: [], lines: [] })) {
    if (!ID.test(r.id || '')) add('error', 'nodes.csv', 'the id "' + (r.id || '') + '" is not lower case letters, digits and underscores', r._line);
    if (seen.has(r.id)) add('error', 'nodes.csv', r.id + ' is listed twice', r._line);
    seen.add(r.id);
    if (!r.question || !/\?\s*$/.test(r.question)) add('warning', 'nodes.csv', r.id + ': the question should be a question, ending in ?', r._line);
    if (r.kind === 'declared' && num(r.value) === null) add('error', 'nodes.csv', r.id + ' is declared, so it needs a value', r._line);
    const lo = num(r.lower), hi = num(r.upper);
    if (lo !== null && hi !== null && lo > hi) add('error', 'nodes.csv', r.id + ': lower is above upper', r._line);
  }

  // ── a transcription names its source and the person who checked it ──
  const declRule = files.find(r => r.path === 'declaration.csv') || {};
  const assistants = declRule.assistants || [];
  for (const n of model.nodes.values()) {
    const t = n.files['declaration.csv'];
    if (!t) continue;
    for (const r of records(t)) {
      if (r.ai !== 'transcribed') continue;
      const where = n.dir + 'declaration.csv';
      if (!String(r.source || '').trim()) add('error', where, 'says transcribed but names no source — say what the relation was copied from', r._line);
      const who = String(r.checked_by || '').trim();
      const lower = who.toLowerCase();
      if (!who) add('error', where, 'says transcribed but names nobody who checked the copy against its source', r._line);
      else if (assistants.some(a => lower === a || lower.startsWith(a + ' ') || lower.includes(a + '/'))) {
        add('error', where, '"' + who + '" is an assistant\'s name; the copy is checked by a person', r._line);
      }
    }
  }

  // ── the texts: headings, what each may contain, embeds, maths ──
  const textRules = spec.text || [];
  const checkText = (path, text, role, node) => {
    if (text === null || text === undefined) return;
    const rule = textRules.find(t => t.role === role);
    if (!rule) return;
    const s = sections(text);
    const titles = s.sections.map(x => x.title.toLowerCase());
    let last = -1;
    for (const h of rule.headings) {
      const k = titles.indexOf(h.toLowerCase());
      if (k < 0) add('error', path, 'has no "## ' + h + '" section');
      else if (k < last) add('warning', path, '"## ' + h + '" is out of order');
      else {
        last = k;
        if (!s.sections[k].body.trim()) add('error', path, 'says nothing under "## ' + h + '"', s.sections[k].line);
      }
    }
    for (const bad of rule.forbid || []) {
      const at = text.split('\n').findIndex(l => l.includes(bad));
      if (at >= 0) add('error', path, 'contains ' + bad + ' — ' + rule.forbid_says, at + 1);
    }
    text.split('\n').forEach((l, i) => {
      for (const m of l.matchAll(/\{\{\s*([a-z]+)\s+([^}]*?)\s*\}\}/g)) {
        const [, name, arg] = m;
        if (name === 'eq' && !findEquation(model, node, arg)) add('error', path, '{{eq ' + arg + '}} names no equation in equations.csv', i + 1);
        else if (name === 'fig' && !findFigure(model, node, arg)) add('error', path, '{{fig ' + arg + '}} names no row of figures.csv', i + 1);
        else if (name === 'node' && !model.nodes.has(arg)) add('error', path, '{{node ' + arg + '}} names no node', i + 1);
        else if (name === 'guess' && !arg.includes('||')) add('error', path, '{{guess …}} needs the question, then ||, then the answer', i + 1);
        else if (!['eq', 'fig', 'node', 'guess'].includes(name)) add('error', path, '{{' + name + '}} is not one of eq, fig, node, guess', i + 1);
      }
      for (const m of l.matchAll(/\$\$?([^$]+)\$\$?/g)) {
        const r = texToMathml(m[1]);
        for (const p of r.problems) add('warning', path, 'maths: ' + p, i + 1);
      }
    });
  };
  const checkOverlap = (where, a, b) => {
    if (!a || !b) return;
    const theory = new Set(longSentences(b));
    const dup = longSentences(a).filter(s => theory.has(s));
    for (const s of dup.slice(0, 3)) add('warning', where, 'says the same sentence in the explanation and the theory: "' + s.slice(0, 80) + '…"');
  };
  checkText('explanation.md', model.g.text['explanation.md'], 'explanation', null);
  checkText('theory.md', model.g.text['theory.md'], 'theory', null);
  checkOverlap('the group', model.g.text['explanation.md'], model.g.text['theory.md']);
  for (const n of model.nodes.values()) {
    checkText(n.dir + 'explanation.md', n.text['explanation.md'], 'explanation', n);
    checkText(n.dir + 'theory.md', n.text['theory.md'], 'theory', n);
    checkOverlap(n.dir, n.text['explanation.md'], n.text['theory.md']);
  }
  for (const e of model.group.equations) {
    for (const p of texToMathml(e.latex).problems) add('warning', 'equations.csv', e.id + ': ' + p, e._line);
  }

  // ── each node's algorithm, inputs and results ──
  for (const n of model.nodes.values()) {
    if (n.row.kind !== 'computed' || !n.present) continue;
    const code = n.text['pseudocode.txt'];
    if (code) {
      if (!/^\s*return\b/m.test(code)) add('error', n.dir + 'pseudocode.txt', 'never returns an answer — every path ends in return or refuse');
      for (const inp of n.inputs) {
        if (inp.name && !new RegExp('\\b' + inp.name + '\\b').test(code)) add('warning', n.dir + 'pseudocode.txt', 'never uses the input ' + inp.name);
      }
      for (const e of pseudocodeEquations(code, n.row.output)) for (const p of e.problems) add('note', n.dir + 'pseudocode.txt', 'shown as an equation with a gap: ' + p, e.line);
      // The language's own reading: every line parses and every unit agrees.
      // The members a node publishes beside its answer are its results' answer.<member> columns.
      const members = resultColumns(n.files['results/isolation.csv']).answers
        .filter(a => a.name.startsWith('answer.')).map(a => ({ name: a.name.slice(7), unit: a.unit }));
      const r = await readMethod(code, n.inputs.map(i => ({ name: i.name, unit: i.unit })), n.row.unit, members);
      if (r && r.error) add('error', n.dir + 'pseudocode.txt', r.error);
      if (r) for (const d of r.diags) add(d.severity === 'error' ? 'error' : d.severity === 'warning' ? 'warning' : 'note', n.dir + 'pseudocode.txt', d.msg, d.line);
    }
    // Inputs: where each comes from, and its default in its range.
    for (const inp of n.inputs) {
      const where = n.dir + 'inputs.csv';
      const from = String(inp.from || '').trim();
      if (!from) add('error', where, inp.name + ' does not say where it comes from', inp._line);
      else if (from !== 'case' && !model.nodes.has(from) && !/^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$/.test(from)) {
        add('error', where, inp.name + ' comes from "' + from + '", which is no node here; another group\'s node is written group.node', inp._line);
      }
      const d = num(inp.default), lo = num(inp.min), hi = num(inp.max);
      if (d === null) add('error', where, inp.name + ' has no default value', inp._line);
      if (d !== null && lo !== null && d < lo) add('error', where, inp.name + ': the default is below min', inp._line);
      if (d !== null && hi !== null && d > hi) add('error', where, inp.name + ': the default is above max', inp._line);
      if (model.nodes.has(from)) {
        const up = model.nodes.get(from).row;
        if (up.unit && inp.unit && up.unit !== inp.unit) add('warning', where, inp.name + ' is in ' + inp.unit + ' but ' + from + ' answers in ' + up.unit, inp._line);
      }
    }
    // The isolation results: the reference the developer's code must match.
    const t = n.files['results/isolation.csv'];
    if (!t) continue;
    const where = n.dir + 'results/isolation.csv';
    const cols = resultColumns(t);
    for (const inp of n.inputs) {
      const c = cols.inputs.find(x => x.name === inp.name);
      if (!c) add('error', where, 'has no column for the input ' + inp.name + ' [' + (inp.unit || 'unit') + ']');
      else if (inp.unit && c.unit !== inp.unit) add('error', where, inp.name + ' is in [' + c.unit + '] here and [' + inp.unit + '] in inputs.csv');
    }
    for (const c of cols.inputs) if (!n.inputs.some(i => i.name === c.name)) add('warning', where, 'has a column ' + c.name + ' that is not an input');
    if (cols.answers.length && n.row.unit && cols.answers.length === 1 && cols.answers[0].unit !== n.row.unit) {
      add('error', where, 'the answer is in [' + cols.answers[0].unit + '] here and [' + n.row.unit + '] in nodes.csv');
    }
    const refK = cols.other.refuses, tolK = cols.other.tolerance, oriK = cols.other.origin;
    let answers = 0, refusals = 0, atDefaults = false;
    const cover = new Map(n.inputs.map(i => [i.name, { lo: false, hi: false }]));
    t.rows.forEach((r, i) => {
      const line = t.lines[i];
      const refuses = refK !== undefined && r[refK] === 'yes';
      // A refusal may test a value that is not a finite number: the node must refuse that too.
      const odd = v => refuses && /^[-+]?(nan|inf|infinity)$/i.test(String(v).trim());
      for (const c of cols.inputs) if (num(r[c.k]) === null && !odd(r[c.k])) add('error', where, c.name + ' is not a number', line);
      if (refuses) {
        refusals++;
        if (cols.answers.some(a => r[a.k] !== '')) add('warning', where, 'a refusal should leave the answer blank', line);
      } else {
        answers++;
        for (const a of cols.answers) if (num(r[a.k]) === null) add('error', where, a.name + ' is not a number', line);
        if (tolK !== undefined && !(num(r[tolK]) > 0)) add('error', where, 'the tolerance must be a number above zero', line);
      }
      if (n.inputs.length && n.inputs.every(inp => {
        const c = cols.inputs.find(x => x.name === inp.name);
        return c && num(r[c.k]) !== null && num(r[c.k]) === num(inp.default);
      })) atDefaults = true;
      for (const inp of n.inputs) {
        const c = cols.inputs.find(x => x.name === inp.name);
        if (!c) continue;
        const v = num(r[c.k]);
        if (v !== null && num(inp.min) !== null && v === num(inp.min)) cover.get(inp.name).lo = true;
        if (v !== null && num(inp.max) !== null && v === num(inp.max)) cover.get(inp.name).hi = true;
      }
    });
    if (!atDefaults && n.inputs.length) add('error', where, 'has no row with every input at its default');
    if (answers < 3) add('error', where, 'has ' + answers + ' ordinary case(s); it needs at least 3');
    if (refusals < 1) add('warning', where, 'has no refusal — a row where the node must refuse, with refuses = yes');
    for (const inp of n.inputs) {
      const c = cover.get(inp.name);
      if (num(inp.min) !== null && !c.lo) add('warning', where, 'never tests ' + inp.name + ' at its min, ' + inp.min);
      if (num(inp.max) !== null && !c.hi) add('warning', where, 'never tests ' + inp.name + ' at its max, ' + inp.max);
    }
    if (oriK !== undefined && t.rows.some(r => r[oriK] === 'code') && !n.text['results/how-run.md']) {
      add('error', n.dir + 'results/how-run.md', 'is missing — these results came from code, so say how it was run');
    }
  }

  // ── the group's own results: the developer's group test ──
  const gt = model.g.files['results/group.csv'];
  if (gt) {
    const where = 'results/group.csv';
    const cols = resultColumns(gt);
    const unitOf = id => (model.nodes.get(id) || { row: {} }).row.unit;
    for (const c of cols.inputs) {
      const node = model.nodes.get(c.name);
      if (!node) add('error', where, 'the column ' + c.name + ' is no node of the group; an input column is named by its node id');
      else if (node.row.kind !== 'declared') add('warning', where, c.name + ' is ' + node.row.kind + ', not declared: a group test sets what the group is given');
      else if (unitOf(c.name) && c.unit !== unitOf(c.name)) add('error', where, c.name + ' is in [' + c.unit + '] here and [' + unitOf(c.name) + '] in nodes.csv');
    }
    if (!cols.answers.length) add('error', where, 'has no answer column — write answer.<node id> [unit] for each node the test reads');
    for (const a of cols.answers) {
      const id = a.name.replace(/^answer\.?\s*/, '');
      if (!model.nodes.has(id)) add('error', where, a.name + ' names no node: an answer column is answer.<node id>');
      else if (unitOf(id) && a.unit !== unitOf(id)) add('error', where, a.name + ' is in [' + a.unit + '] here and [' + unitOf(id) + '] in nodes.csv');
    }
    const refK = cols.other.refuses, tolK = cols.other.tolerance;
    gt.rows.forEach((r, i) => {
      const line = gt.lines[i];
      const refuses = refK !== undefined && r[refK] === 'yes';
      for (const c of cols.inputs) if (num(r[c.k]) === null) add('error', where, c.name + ' is not a number', line);
      if (!refuses) {
        for (const a of cols.answers) if (num(r[a.k]) === null) add('error', where, a.name + ' is not a number', line);
        if (tolK === undefined || !(num(r[tolK]) > 0)) add('error', where, 'the tolerance must be a number above zero', line);
      }
    });
  }

  // ── requirements, loops, publishes, flow, sources, figures ──
  for (const r of model.group.requirements) {
    for (const side of ['required', 'achieved']) {
      const node = model.nodes.get(r[side]);
      if (!node) add('error', 'requirements.csv', r.id + ': the ' + side + ' node "' + r[side] + '" does not exist', r._line);
      else if (node.row.kind !== side) add('warning', 'requirements.csv', r.id + ': ' + r[side] + ' is ' + node.row.kind + ', not ' + side, r._line);
    }
  }
  for (const l of model.group.loops) for (const id of String(l.nodes || '').split(/\s+/).filter(Boolean)) {
    if (!model.nodes.has(id)) add('error', 'loops.csv', 'the loop names "' + id + '", which is not a node', l._line);
  }
  for (const p of model.group.publishes) if (!model.nodes.has(p.node)) add('error', 'publishes.csv', '"' + p.node + '" is not a node', p._line);
  const flow = model.g.text['flow.txt'];
  if (flow) {
    const declared = new Set();
    flow.split('\n').forEach((l, i) => {
      const s = l.replace(/#.*$/, '').trim();
      if (!s) return;
      const m = /^([a-z][a-z0-9_]*)\s*<-\s*(.*)$/.exec(s);
      if (!m) { add('warning', 'flow.txt', 'a step is written `node <- input, input`', i + 1); return; }
      if (!model.nodes.has(m[1])) add('error', 'flow.txt', '"' + m[1] + '" is not a node', i + 1);
      for (const from of m[2].split(/[,\s]+/).filter(Boolean)) {
        declared.add(from + '>' + m[1]);
        if (!model.nodes.has(from) && from !== 'case' && !from.includes('.')) add('error', 'flow.txt', '"' + from + '" is not a node', i + 1);
      }
    });
    for (const e of model.edges) if (!declared.has(e.from + '>' + e.to)) {
      add('warning', 'flow.txt', 'does not show ' + e.from + ' feeding ' + e.to + ', which ' + e.to + '/inputs.csv says it does');
    }
  }
  const sourceIds = new Set(model.group.sources.map(s => s.id));
  for (const s of model.group.sources) if (s.file && !f.has(s.file) && !f.has('sources/' + s.file)) add('error', 'sources.csv', s.id + ': the file ' + s.file + ' is not in the folder', s._line);
  // A node's own sources count for the whole group: a paper cited once is cited.
  for (const n of model.nodes.values()) {
    for (const s of n.files['sources.csv'] ? records(n.files['sources.csv']) : []) {
      if (s.file && !f.has(n.dir + s.file) && !f.has(n.dir + 'sources/' + s.file)) add('error', n.dir + 'sources.csv', s.id + ': the file ' + s.file + ' is not in the folder', s._line);
      if (s.id) sourceIds.add(s.id);
    }
  }
  const cite = (where, rows) => rows.forEach(r => { if (r.source && !sourceIds.has(r.source)) add('error', where, 'cites "' + r.source + '", which sources.csv does not list', r._line); });
  cite('equations.csv', model.group.equations);
  cite('constants.csv', model.group.constants);
  for (const n of model.nodes.values()) {
    cite(n.dir + 'equations.csv', n.files['equations.csv'] ? records(n.files['equations.csv']) : []);
    cite(n.dir + 'evidence.csv', n.files['evidence.csv'] ? records(n.files['evidence.csv']) : []);
  }
  const checkFigures = async (rows, base, where) => {
    for (const fg of rows) {
      const path = base + fg.file;
      if (!f.has(path)) { add('error', where, fg.id + ': ' + fg.file + ' is not in the folder', fg._line); continue; }
      if (['image', 'video'].includes(fg.kind)) continue;
      const t = parseCsv(await f.text(path));
      const need = { line: ['x', 'y'], scatter: ['x', 'y'], bar: ['x', 'y'], heatmap: ['x', 'y', 'z'], animation: ['x', 'y', 'z'] }[fg.kind] || [];
      for (const key of need) for (const col of String(fg[key] || '').split(/\s+/).filter(Boolean)) {
        if (column(t, col) < 0) add('error', where, fg.id + ': ' + fg.file + ' has no column ' + col, fg._line);
      }
      for (const key of need) if (!fg[key]) add('error', where, fg.id + ': a ' + fg.kind + ' needs the column ' + key, fg._line);
      if (fg.kind === 'flow' && (column(t, 'from') < 0 || column(t, 'to') < 0)) add('error', where, fg.id + ': a flow is a table with from and to', fg._line);
      if (fg.kind === 'steps' && (column(t, 'step') < 0 || column(t, 'caption') < 0)) add('error', where, fg.id + ': steps are a table with step and caption', fg._line);
      if (fg.kind === 'steps' && fg.on && !findFigure(model, null, fg.on) && !rows.some(r => r.id === fg.on)) add('error', where, fg.id + ': walks through "' + fg.on + '", which is not a figure', fg._line);
      if (fg.kind === 'scene3d' && ['body', 'x', 'y', 'z'].some(c => column(t, c) < 0)) add('error', where, fg.id + ': a 3D scene is a table with body, x, y, z', fg._line);
    }
  };
  await checkFigures(model.group.figures, '', 'figures.csv');
  for (const n of model.nodes.values()) {
    await checkFigures(n.files['figures.csv'] ? records(n.files['figures.csv']) : [], n.dir, n.dir + 'figures.csv');
  }

  // ── what the folder may hold ──
  const lim = spec.limits || {};
  for (const p of f.list()) {
    const file = f.file(p);
    const ext = p.split('.').pop().toLowerCase();
    if (lim.file_bytes && file.size > lim.file_bytes) add('error', p, 'is ' + (file.size / 1048576).toFixed(1) + ' MB — ' + lim.file_bytes_says);
    if ((lim.refused_types || []).includes(ext)) add('error', p, 'is a .' + ext + ' — ' + lim.refused_says);
  }

  const order = { error: 0, warning: 1, note: 2 };
  out.sort((a, b) => order[a.level] - order[b.level] || a.where.localeCompare(b.where) || a.line - b.line);
  return out;
}

function firstLine(s) { return String(s || '').trim().split('\n')[0]; }
