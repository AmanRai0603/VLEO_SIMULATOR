/*
  A node's pseudocode, read by the repository's own method checker.

  The checker is vleo_sheet::method compiled to WebAssembly (web/method.wasm.gz,
  `cargo run -p xtask -- method-wasm`) — the same reading the developer's tools
  and every node form make, never a second one written here. The page carries
  it (window.VLEO_METHOD_WASM, inlined by `xtask group-app`).

  Only the reading is asked for: does each line parse, does every unit agree,
  is the answer in the unit the contract says. No case is handed over, so
  nothing is run; computing is the developer's engine's work.
*/
'use strict';

let vm = null, failed = '';

async function checker() {
  if (vm || failed) return vm;
  try {
    const b64 = window.VLEO_METHOD_WASM || '';
    if (!b64) throw new Error('this page was built without the method checker');
    const gz = Uint8Array.from(atob(b64), c => c.charCodeAt(0));
    const bytes = await new Response(new Blob([gz]).stream().pipeThrough(new DecompressionStream('gzip'))).arrayBuffer();
    vm = (await WebAssembly.instantiate(bytes, {})).instance.exports;
  } catch (e) { failed = String(e && e.message || e); }
  return vm;
}

/** Whether the checker could be started, and why not. */
export function checkerProblem() { return failed; }

const unit = u => '[' + (String(u || '').trim() || '1') + ']';

/**
 * Read `text` as the method of a node whose inputs are `[{name, unit}]` and
 * whose answer is in `answerUnit`. Answers `{ error, diags: [{line, severity, msg}] }`,
 * or null when the checker could not be started.
 */
export async function readMethod(text, inputs, answerUnit) {
  const v = await checker();
  if (!v) return null;
  let plain = 'output ' + unit(answerUnit) + '\n';
  for (const i of inputs) plain += 'input ' + String(i.name).trim() + ' ' + unit(i.unit) + '\n';
  plain += 'method\n' + String(text || '');
  const enc = new TextEncoder().encode(plain);
  const p = v.vleo_alloc(enc.length);
  new Uint8Array(v.memory.buffer, p, enc.length).set(enc);
  const out = v.vleo_report(p, enc.length), n = v.vleo_report_len();
  const r = JSON.parse(new TextDecoder().decode(new Uint8Array(v.memory.buffer, out, n)));
  return { error: r.error || '', diags: r.diags || [] };
}
