/*
  A page of the readers' docs folder: one row's page, and its lesson, read
  with no tool running — from a shared drive or any internal web server.

  `xtask readers` bundles this with the modules it imports into one classic
  script (a page opened from a file cannot load modules), and writes beside it
  the engine compiled for the browser. So a lesson here is drawn by the same
  component library as in the tool, and its widgets are answered by the same
  relations: the engine runs in the page, on the declared values, and a row
  that needs reference data refuses by name — the page carries none.
*/
'use strict';

import { $, $$ } from './dom.js';
import { S } from './state.js';
import { renderLesson, useEngine } from './components.js';
import { initDepth } from './depth.js';

/** The engine in the page: crates/vleo-kernel-wasm, carried by kernel.js. */
async function kernel() {
  const b64 = window.VLEO_KERNEL || '';
  if (!b64) throw new Error('this folder was built without its engine (assets/kernel.js)');
  const bin = atob(b64);
  const gz = new Uint8Array(bin.length);
  for (let k = 0; k < bin.length; k++) gz[k] = bin.charCodeAt(k);
  if (typeof DecompressionStream !== 'function') throw new Error('this browser is too old to unpack the engine');
  const bytes = await new Response(new Blob([gz]).stream().pipeThrough(new DecompressionStream('gzip'))).arrayBuffer();
  const v = (await WebAssembly.instantiate(bytes, {})).instance.exports;
  const call = (fn, text) => {
    const enc = new TextEncoder().encode(text);
    const p = v.vleo_alloc(enc.length);
    new Uint8Array(v.memory.buffer, p, enc.length).set(enc);
    const at = v[fn](p, enc.length), n = v.vleo_out_len();
    return JSON.parse(new TextDecoder().decode(new Uint8Array(v.memory.buffer, at, n)));
  };
  // The request the engine reads: `node`, the sweep's `over`/`from`/`to`/
  // `points`, and a `set <id> <si>` for each supplied value.
  const request = p => {
    let t = 'node ' + (p.get('node') || '') + '\n';
    for (const k of ['over', 'from', 'to', 'points']) if (p.get(k) != null) t += k + ' ' + p.get(k) + '\n';
    for (const s of p.getAll('set')) {
      const i = s.lastIndexOf(':');
      t += 'set ' + s.slice(0, i) + ' ' + s.slice(i + 1) + '\n';
    }
    return t;
  };
  // A row that reads reference data refuses here, and says why in words that
  // fit a page: there is no store to sync, only the tool to open.
  const here = r => {
    for (const b of r.blocked || []) {
      if (b.kind === 'data-missing') b.message = b.id + ': reads reference data, which this page does not carry — open the row in the tool to run it';
    }
    return r;
  };
  return {
    run: async p => here(call('vleo_run', request(p))),
    sweep: async q => call('vleo_sweep', request(q)),
  };
}

function tabs() {
  // The row's page is the tool's own generated page; its tabs switch here.
  $$('.tabs .tab').forEach(t => {
    t.onclick = () => {
      $$('.tabs .tab').forEach(x => x.classList.remove('sel'));
      $$('[data-panel]').forEach(x => x.classList.remove('sel'));
      t.classList.add('sel');
      const p = $('[data-panel="' + t.dataset.tab + '"]');
      if (p) p.classList.add('sel');
    };
  });
}

/** A link to another row, in the tool's own pages, goes to that row's page here. */
function links() {
  document.addEventListener('click', e => {
    const a = e.target.closest('[data-goto]');
    if (!a) return;
    e.preventDefault();
    const here = /\/rows\//.test(location.pathname);
    location.href = (here ? '' : 'rows/') + encodeURIComponent(a.dataset.goto) + '.html';
  });
}

async function boot() {
  initDepth();
  tabs();
  links();
  const rowsEl = $('#vleo-rows'), lessonEl = $('#vleo-lesson-json'), host = $('#row-lesson');
  if (!lessonEl || !host) return;
  S.byId = new Map(JSON.parse(rowsEl.textContent).map(r => [r.id, r]));
  const l = JSON.parse(lessonEl.textContent);
  try {
    useEngine(await kernel());
  } catch (e) {
    const why = String(e && e.message || e);
    useEngine({
      run: async () => ({ ok: false, message: 'the engine could not start in this page: ' + why }),
      sweep: async () => ({ ok: false, message: why }),
    });
  }
  renderLesson(host, l);
}

if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', boot);
else boot();
