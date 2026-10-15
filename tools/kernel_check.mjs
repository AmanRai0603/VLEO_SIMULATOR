#!/usr/bin/env node
/*
  The engine compiled for the browser answers as the tool does.

      cargo run -p xtask -- assemble          # generated/index.json: which rows compute
      cargo build --release --target wasm32-unknown-unknown \
        --manifest-path crates/vleo-kernel-wasm/Cargo.toml
      cargo run -p vleo-files --example design_rows -- <design.rows>
      node tools/kernel_check.mjs <design.rows> [--answers <file.tsv>]

  crates/vleo-kernel-wasm is the whole kernel built for WebAssembly. This
  loads it as a page would, asking it for nothing (no imports), and checks
  that it says what it is; runs nothing before a design is open; opens the
  design's files; answers every computed row it can on the declared values;
  refuses a value supplied for a computed row; refuses a row that reads
  reference data as data-missing, since none travels with it; and sweeps.

  With --answers, every answer is written as `<question>\t<the engine's
  JSON>`, and `cargo test -- --include-ignored` in crates/vleo-kernel-wasm,
  given the file in VLEO_BROWSER_ANSWERS, asks the same engine built natively
  the same questions: the answers must be the same text, byte for byte,
  because the maths is portable (pmath) and so must not move between builds.

  Exit 0 when every check held; 1 with what did not.
*/
import { readFileSync, writeFileSync } from 'node:fs';
import { join, dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const WASM = join(ROOT, 'crates/vleo-kernel-wasm/target/wasm32-unknown-unknown/release/vleo_kernel_wasm.wasm');
const args = process.argv.slice(2);
const design = args[0];
const answersAt = args.indexOf('--answers') >= 0 ? args[args.indexOf('--answers') + 1] : '';
if (!design) {
  console.error('usage: node tools/kernel_check.mjs <design.rows> [--answers <file.tsv>]');
  process.exit(2);
}

const index = JSON.parse(readFileSync(join(ROOT, 'generated/index.json'), 'utf8'));
const rows = index.rows.filter(r => r.kind === 'computed' && r.state === 'published').map(r => r.id);

const mod = await WebAssembly.compile(readFileSync(WASM));
const imports = WebAssembly.Module.imports(mod).map(i => i.module + '.' + i.name);
const v = (await WebAssembly.instantiate(mod, {})).exports;
const read = at => new TextDecoder().decode(new Uint8Array(v.memory.buffer, at, v.vleo_out_len()));
const callBytes = (fn, enc) => {
  const p = v.vleo_alloc(enc.length);
  new Uint8Array(v.memory.buffer, p, enc.length).set(enc);
  return read(v[fn](p, enc.length));
};
const call = (fn, text) => callBytes(fn, new TextEncoder().encode(text));

const bad = [];
const json = (what, text) => {
  try { return JSON.parse(text); } catch (e) { bad.push(what + ' is not JSON: ' + String(text).slice(0, 120)); return {}; }
};

if (imports.length) bad.push('the engine asks its host for ' + imports.join(', ') + ': a page opened from a file has nothing to give');
const before = json('the answer before a design is open', call('vleo_run', 'node sw_ap_design_long\n'));
if (before.ok || !String(before.message || '').includes('no design is open')) bad.push('the engine ran before a design was open: ' + JSON.stringify(before).slice(0, 200));
const opened = json('the answer to opening the design', callBytes('vleo_open', new Uint8Array(readFileSync(design))));
if (!opened.ok || !opened.rows) bad.push('the design did not open: ' + JSON.stringify(opened).slice(0, 300));
// What it is, once it runs a design: its kernel, and the graph that design makes.
const ident = json('the engine\'s identity', read(v.vleo_identity()));
if (!ident.kernel) bad.push('the engine does not say which kernel it is');
if (!ident.graph) bad.push('the engine does not say which graph it runs');

const runs = {};
let answered = 0, missingData = 0;
for (const id of rows) {
  runs[id] = call('vleo_run', 'node ' + id + '\n');
  const r = json(id, runs[id]);
  if (!r.ok) { bad.push(id + ': the engine failed rather than answering or refusing: ' + r.message); continue; }
  if ((r.values || []).some(x => x.id === id)) answered++;
  const blocked = (r.blocked || []).filter(b => b.id === id);
  if (blocked.length && blocked[0].kind === 'data-missing') missingData++;
}
if (!answered) bad.push('no computed row answered');
if (!missingData) bad.push('no row refused for want of reference data: none travels with the engine, so some must');
const supplied = call('vleo_run', 'node sw_ap_design_long\nset sw_ap_design_long 3\n');
if (json('the refusal of a supplied computed row', supplied).ok) bad.push('the engine took a value for a computed row: ' + supplied);
const sweep = call('vleo_sweep', 'node sw_ap_design_long\nover sw_ap_central_expectation\nfrom 5\nto 35\npoints 7\n');
const w = json('the sweep', sweep);
const ys = ((w.figure || {}).series || [{}])[0].y || [];
if (!w.ok || ys.length !== 7 || ys.some(y => y == null)) bad.push('the sweep did not answer 7 of 7 points: ' + sweep.slice(0, 200));
else if (new Set(ys).size < 2) bad.push('the sweep answered the same value at every point: ' + ys);

if (answersAt) {
  const lines = rows.map(id => id + '\t' + runs[id]);
  lines.push('@supplied\t' + supplied, '@sweep\t' + sweep);
  writeFileSync(answersAt, lines.join('\n') + '\n');
}
console.log(answered + ' of ' + rows.length + ' computed rows answered, ' + missingData + ' refused for want of data');
for (const b of bad) console.log('FAIL  ' + b);
process.exit(bad.length ? 1 : 0);
