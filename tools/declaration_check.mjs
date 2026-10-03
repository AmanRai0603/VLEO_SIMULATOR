#!/usr/bin/env node
/*
  The group application's check of a node's declaration, run on the example
  group with one declaration swapped in at a time.

      node tools/declaration_check.mjs

  A transcription is taken only when it names what it was copied from and the
  person who read the copy against that (AGENTS.md, "Transcribing is not
  supplying"). The developer's intake enforces it; this holds the group
  application to the same rule, so a group hears it before it seals rather
  than after. Each case says what the checker must say of it, and the pattern
  is read from the built page, web/group.html, exactly as the page carries it.

  Exit 0 when every case said what it should; 1 with the cases that did not.
*/
import { readFileSync, readdirSync, statSync, copyFileSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { join, relative, dirname, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { pathToFileURL, fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const EXAMPLE = join(ROOT, 'groups/example');
const TARGET = 'nodes/orbit_period/declaration.csv';

// web/js is written as browser modules with no package.json of its own; a
// copy beside one that says so lets Node load them unchanged.
const tmp = mkdtempSync(join(tmpdir(), 'vleo-declaration-'));
for (const f of readdirSync(join(ROOT, 'web/js'))) if (f.endsWith('.js')) copyFileSync(join(ROOT, 'web/js', f), join(tmp, f));
writeFileSync(join(tmp, 'package.json'), '{"type":"module"}\n');
const { Folder } = await import(pathToFileURL(join(tmp, 'gfolder.js')).href);
const { loadGroup } = await import(pathToFileURL(join(tmp, 'gmodel.js')).href);
const { checkGroup } = await import(pathToFileURL(join(tmp, 'gcheck.js')).href);

const page = readFileSync(join(ROOT, 'web/group.html'), 'utf8');
const at = page.indexOf('window.VLEO_GROUP_SPEC = ');
const spec = JSON.parse(page.slice(at + 'window.VLEO_GROUP_SPEC = '.length, page.indexOf(';\n', at)));
globalThis.window = globalThis;
window.VLEO_GROUP_SPEC = spec;

async function findings(declaration) {
  const files = new Map();
  const walk = d => {
    for (const f of readdirSync(d).sort()) {
      const p = join(d, f);
      if (statSync(p).isDirectory()) walk(p);
      else files.set(relative(EXAMPLE, p).split('\\').join('/'), new File([readFileSync(p)], f));
    }
  };
  walk(EXAMPLE);
  files.set(TARGET, new File([declaration], 'declaration.csv'));
  const all = await checkGroup(await loadGroup(new Folder('example', files)), spec);
  return all.filter(x => x.where === TARGET && x.level === 'error').map(x => x.msg);
}

const H = 'author,ai,date,source,checked_by\n';
const CASES = [
  ['a transcription with its source and checker', H + 'Ben Example,transcribed,2026-10-03,orbit.py:12,Ben Example\n', null],
  ['a transcription with no source', H + 'Ben Example,transcribed,2026-10-03,,Ben Example\n', 'names no source'],
  ['a transcription nobody checked', H + 'Ben Example,transcribed,2026-10-03,orbit.py:12,\n', 'names nobody who checked'],
  ['a transcription an assistant checked', H + 'Ben Example,transcribed,2026-10-03,orbit.py:12,Claude\n', "is an assistant's name"],
  ['a declaration with the first three columns only', 'author,ai,date\nBen Example,none,2026-10-03\n', null],
  ['an answer the pattern does not know', 'author,ai,date\nBen Example,copied,2026-10-03\n', 'must be one of none, wording, relation, transcribed'],
];

const failed = [];
for (const [what, csv, expect] of CASES) {
  const got = await findings(csv);
  const ok = expect === null ? got.length === 0 : got.some(m => m.includes(expect));
  console.log((ok ? '  ok    ' : '  FAIL  ') + what + (got.length ? ' — ' + got.join('; ') : ''));
  if (!ok) failed.push(what);
}
rmSync(tmp, { recursive: true, force: true });
if (failed.length) {
  console.log(failed.length + ' case(s) did not say what they should');
  process.exit(1);
}
console.log(CASES.length + ' declarations, each judged as the rule says');
