/*
  The lesson form: the lesson drawn as fields, checked as it is typed by the
  gate's own check (vleo_sheet::lesson::report, in the checker this page
  carries), and saved as a copy with its blocks rewritten. No network.
*/
'use strict';
(function () {
  const $ = (s, r = document) => r.querySelector(s);
  const S = v => (v == null ? '' : String(v));
  const esc = v => S(v).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  const schema = JSON.parse($('#vleo-lesson-schema').textContent);
  const KINDS = schema.kinds.split(','), STATIONS = schema.stations.split(','), CLAIMS = schema.claims.split(',');
  const ROWS = $('#vleo-lesson-rows').textContent;
  const NODE = (ROWS.match(/^node (\S+)/m) || [])[1] || '';
  const L = JSON.parse($('#vleo-lesson-json').textContent);
  for (const k of ['stations', 'equations', 'widgets', 'checks', 'references']) L[k] = L[k] || [];
  const BLANK = {
    stations: () => ({ kind: 'simply', title: '', text: '', claim: 'derived', source: '' }),
    equations: () => ({ text: '', says: '', claim: 'sourced', source: '' }),
    widgets: () => ({ title: 'Try it', inputs: [], outputs: [], sweep: '' }),
    checks: () => ({ question: '', options: ['', ''], answer: 1, why: '' }),
  };
  if (!L.stations.length) L.stations.push(BLANK.stations());

  // ---- TOML, written the one way intake reads it -------------------------
  // '<' is written as \u003C, so nothing the author types can end the script
  // element the lesson travels in.
  const q = v => '"' + S(v).replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\n/g, '\\n')
    .replace(/\r/g, '').replace(/\t/g, '\\t').replace(/</g, '\\u003C') + '"';
  const arr = a => '[' + a.map(q).join(', ') + ']';
  function toml() {
    let o = '[lesson]\ntitle = ' + q(L.title) + '\nby = ' + q(L.by) + '\nanswer = ' + q(L.answer) +
      '\nkind = ' + q(L.kind) + '\n';
    for (const s of L.stations) o += '\n[[station]]\nkind = ' + q(s.kind) + '\ntitle = ' + q(s.title) +
      '\ntext = ' + q(s.text) + '\nclaim = ' + q(s.claim) + (s.source ? '\nsource = ' + q(s.source) : '') + '\n';
    for (const e of L.equations) o += '\n[[equation]]\ntext = ' + q(e.text) + '\nsays = ' + q(e.says) +
      '\nclaim = ' + q(e.claim) + (e.source ? '\nsource = ' + q(e.source) : '') + '\n';
    for (const w of L.widgets) o += '\n[[widget]]\ntitle = ' + q(w.title) + '\ninputs = ' + arr(w.inputs) +
      '\noutputs = ' + arr(w.outputs) + (w.sweep ? '\nsweep = ' + q(w.sweep) : '') + '\n';
    for (const c of L.checks) o += '\n[[check]]\nquestion = ' + q(c.question) + '\noptions = ' + arr(c.options) +
      '\nanswer = ' + (parseInt(c.answer, 10) || 0) + '\nwhy = ' + q(c.why) + '\n';
    for (const r of L.references) o += '\n[[reference]]\ntext = ' + q(r) + '\n';
    return o;
  }

  // ---- the fields --------------------------------------------------------
  const field = (label, key, obj, kind, extra = '') => {
    const v = obj[key];
    const id = 'f' + Math.random().toString(36).slice(2);
    let input;
    if (kind === 'area') input = '<textarea id="' + id + '">' + esc(v) + '</textarea>';
    else if (Array.isArray(kind)) input = '<select id="' + id + '">' + kind.map(k =>
      '<option' + (k === v ? ' selected' : '') + '>' + esc(k) + '</option>').join('') + '</select>';
    else input = '<input type="' + (kind || 'text') + '" id="' + id + '" value="' + esc(v) + '"' + extra + '>';
    return { html: '<div><label for="' + id + '">' + esc(label) + '</label>' + input + '</div>', id, key, obj };
  };
  function block(title, fields, remove) {
    const el = document.createElement('section');
    el.className = 'ls-block';
    el.innerHTML = '<div class="ls-head"><b>' + esc(title) + '</b>' +
      (remove ? '<button type="button" class="ls-rm">remove</button>' : '') + '</div>' +
      '<div class="ls-row">' + fields.map(f => f.html).join('') + '</div>';
    for (const f of fields) {
      const inp = el.querySelector('#' + f.id);
      inp.addEventListener('input', () => { f.set ? f.set(inp.value) : (f.obj[f.key] = inp.value); changed(); });
    }
    if (remove) el.querySelector('.ls-rm').onclick = () => { remove(); draw(); changed(); };
    return el;
  }
  const lines = (obj, key, label, list) => {
    const f = field(label, key, { [key]: obj[key].join('\n') }, 'area');
    f.set = v => { obj[key] = v.split('\n').map(x => x.trim()).filter(Boolean); };
    if (list) f.html = f.html.replace('<textarea', '<textarea placeholder="one row id per line"');
    return f;
  };
  function draw() {
    const main = $('#ls-main');
    main.innerHTML = '';
    const h = t => { const e = document.createElement('h2'); e.textContent = t; main.appendChild(e); };
    const add = (what, list, make) => {
      const b = document.createElement('button');
      b.type = 'button'; b.className = 'ls-add'; b.textContent = 'add ' + what;
      b.onclick = () => { list.push(make()); draw(); changed(); };
      main.appendChild(b);
    };
    h('The lesson');
    main.appendChild(block('answer first', [
      field('title', 'title', L), field('written by — your name', 'by', L),
      field('the one sentence a reader takes away', 'answer', L, 'area'), field('kind of reading', 'kind', L, KINDS),
    ]));
    h('Stations — said simply, the real thing, where it breaks, a story');
    L.stations.forEach((s, i) => main.appendChild(block('station ' + (i + 1), [
      field('kind', 'kind', s, STATIONS), field('title', 'title', s), field('text', 'text', s, 'area'),
      field('what the claim rests on', 'claim', s, CLAIMS), field('source (for a sourced claim)', 'source', s),
    ], () => L.stations.splice(i, 1))));
    add('a station', L.stations, BLANK.stations);
    h('Equations — as a reader sees them, with their source');
    L.equations.forEach((e, i) => main.appendChild(block('equation ' + (i + 1), [
      field('the equation, in plain text', 'text', e), field('what it says', 'says', e, 'area'),
      field('what the claim rests on', 'claim', e, CLAIMS), field('source', 'source', e),
    ], () => L.equations.splice(i, 1))));
    add('an equation', L.equations, BLANK.equations);
    h('Try it — sliders for rows a person picks; the tool answers the rows you show');
    L.widgets.forEach((w, i) => main.appendChild(block('widget ' + (i + 1), [
      field('title', 'title', w), lines(w, 'inputs', 'input rows the reader moves (declared rows)', true),
      lines(w, 'outputs', 'rows it shows', true), field('sweep one input across its range (optional)', 'sweep', w),
    ], () => L.widgets.splice(i, 1))));
    add('a widget', L.widgets, BLANK.widgets);
    h('Check yourself');
    L.checks.forEach((c, i) => main.appendChild(block('check ' + (i + 1), [
      field('question', 'question', c, 'area'), lines(c, 'options', 'options, one per line'),
      field('the right option, counted from 1', 'answer', c, 'number', ' min="1"'),
      field('why it is right', 'why', c, 'area'),
    ], () => L.checks.splice(i, 1))));
    add('a check', L.checks, BLANK.checks);
    h('References');
    main.appendChild(block('references', [lines(L, 'references', 'one per line')]));
  }

  // ---- the check: the gate's own, in this page ---------------------------
  let VM = null, VMerr = '';
  async function vm() {
    if (VM || VMerr) return VM;
    try {
      const bin = atob((($('#vleo-method-wasm') || {}).textContent || '').trim());
      if (!bin) throw new Error('this form was made without its checker, web/method.wasm.gz');
      const gz = new Uint8Array(bin.length);
      for (let k = 0; k < bin.length; k++) gz[k] = bin.charCodeAt(k);
      if (typeof DecompressionStream !== 'function') throw new Error('this browser is too old to unpack the checker');
      const bytes = await new Response(new Blob([gz]).stream().pipeThrough(new DecompressionStream('gzip'))).arrayBuffer();
      VM = (await WebAssembly.instantiate(bytes, {})).instance.exports;
      if (!VM.vleo_lesson_report) { VM = null; throw new Error('this form carries a checker from before lessons'); }
    } catch (e) { VMerr = String(e && e.message || e); }
    return VM;
  }
  async function check() {
    const v = await vm();
    const out = $('#ls-problems');
    if (!v) { out.innerHTML = '<p class="nf-warn">The check could not run here: ' + esc(VMerr) +
      '. The lesson is still saved as you wrote it; the developer who applies it runs the same check.</p>'; return; }
    const enc = new TextEncoder().encode(ROWS + '---\n' + toml());
    const p = v.vleo_alloc(enc.length);
    new Uint8Array(v.memory.buffer, p, enc.length).set(enc);
    const at = v.vleo_lesson_report(p, enc.length), n = v.vleo_report_len();
    const r = JSON.parse(new TextDecoder().decode(new Uint8Array(v.memory.buffer, at, n)));
    $('#ls-state').textContent = r.ok ? 'passes the check' : (r.read ? 'does not read' : r.problems.length + ' to fix');
    out.innerHTML = r.ok ? '<p class="ls-ok">It passes the check the gate runs. Save a filled copy and send it back.</p>'
      : '<ul class="ls-bad">' + (r.read ? '<li>' + esc(r.read) + '</li>' : '') +
        r.problems.map(x => '<li>' + esc(x) + '</li>').join('') + '</ul>';
  }
  let t = null;
  function changed() { clearTimeout(t); t = setTimeout(check, 250); }

  // ---- saving ------------------------------------------------------------
  function download(name, text, type) {
    const a = document.createElement('a');
    a.href = URL.createObjectURL(new Blob([text], { type }));
    a.download = name;
    document.body.appendChild(a); a.click();
    setTimeout(() => { URL.revokeObjectURL(a.href); a.remove(); }, 1000);
  }
  $('#ls-save').onclick = () => {
    // '<' never travels raw: as \u003c in the JSON, and escaped by the
    // browser in the textarea, so nothing typed can end an element.
    $('#vleo-lesson-json').textContent = JSON.stringify(L).replace(/</g, '\\u003c');
    $('#vleo-lesson').textContent = toml();
    const main = $('#ls-main'), kept = main.innerHTML;
    main.innerHTML = '';
    const html = '<!doctype html>\n' + document.documentElement.outerHTML;
    main.innerHTML = kept; draw();
    download(NODE + '.lesson-form.html', html, 'text/html');
  };
  $('#ls-toml').onclick = () => download('lesson.toml', toml(), 'text/plain');

  draw();
  check();
  window.VLEO_LESSON = { toml, check, lesson: L };
})();
