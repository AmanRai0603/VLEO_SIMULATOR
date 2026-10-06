'use strict';
(function () {
  // The page exactly as it arrived, before anything is drawn into it. A saved
  // copy is this with its data block replaced — so the copy is the same form,
  // not a snapshot of whatever the page happened to be showing.
  const PRISTINE = '<!doctype html>\n' + document.documentElement.outerHTML;
  const $ = s => document.querySelector(s);
  const esc = s => String(s == null ? '' : s).replace(/&/g, '&amp;').replace(/</g, '&lt;')
    .replace(/>/g, '&gt;').replace(/"/g, '&quot;');

  // ---- TOML, the subset these blocks use ---------------------------------
  function parseToml(src) {
    const root = {}; let cur = root; let i = 0; const n = src.length;
    const ws = () => { while (i < n && (src[i] === ' ' || src[i] === '\t')) i++; };
    const eol = () => { ws(); if (src[i] === '#') while (i < n && src[i] !== '\n') i++; };
    const fail = m => { throw new Error(m + ' (at character ' + i + ')'); };
    function esc1() {
      const c = src[i++];
      const m = { b: '\b', t: '\t', n: '\n', f: '\f', r: '\r', '"': '"', '\\': '\\' };
      if (c in m) return m[c];
      if (c === 'u' || c === 'U') {
        const len = c === 'u' ? 4 : 8; const h = src.substr(i, len); i += len;
        return String.fromCodePoint(parseInt(h, 16));
      }
      if (c === '\n' || c === ' ' || c === '\t' || c === '\r') { // line-ending backslash
        while (i < n && /[\s]/.test(src[i])) i++; return '';
      }
      fail('bad escape \\' + c);
    }
    function value() {
      if (src.startsWith('"""', i)) {
        i += 3; if (src[i] === '\n') i++; else if (src.startsWith('\r\n', i)) i += 2;
        let o = '';
        while (i < n) {
          if (src.startsWith('"""', i)) {
            let q = 3; while (src[i + q] === '"') q++; // up to two quotes may end the value
            o += '"'.repeat(q - 3); i += q; return o;
          }
          const c = src[i++]; o += c === '\\' ? esc1() : c;
        }
        fail('unclosed """');
      }
      if (src.startsWith("'''", i)) {
        i += 3; if (src[i] === '\n') i++;
        const e = src.indexOf("'''", i); if (e < 0) fail("unclosed '''");
        const o = src.slice(i, e); i = e + 3; return o;
      }
      if (src[i] === '"') {
        i++; let o = '';
        while (i < n && src[i] !== '"') { if (src[i] === '\n') fail('newline in a string'); const c = src[i++]; o += c === '\\' ? esc1() : c; }
        i++; return o;
      }
      if (src[i] === "'") { const e = src.indexOf("'", i + 1); const o = src.slice(i + 1, e); i = e + 1; return o; }
      const m = /^[^\s#,\]}]+/.exec(src.slice(i)); if (!m) fail('a value was expected');
      i += m[0].length; const t = m[0];
      if (t === 'true') return true; if (t === 'false') return false;
      const v = Number(t.replace(/_/g, '')); if (!isFinite(v)) fail('not a value: ' + t); return v;
    }
    function key() {
      if (src[i] === '"') return value();
      const m = /^[A-Za-z0-9_-]+/.exec(src.slice(i)); if (!m) fail('a key was expected');
      i += m[0].length; return m[0];
    }
    function path(close) {
      const parts = []; for (;;) { ws(); parts.push(key()); ws(); if (src[i] === '.') { i++; continue; } break; }
      if (!src.startsWith(close, i)) fail('expected ' + close); i += close.length; return parts;
    }
    while (i < n) {
      ws(); const c = src[i];
      if (c === '\n' || c === '\r') { i++; continue; }
      if (c === '#') { eol(); continue; }
      if (src.startsWith('[[', i)) {
        i += 2; const p = path(']]'); let t = root;
        for (const k of p.slice(0, -1)) t = t[k] = t[k] || {};
        const last = p[p.length - 1]; (t[last] = t[last] || []).push(cur = {}); eol(); continue;
      }
      if (c === '[') {
        i++; const p = path(']'); let t = root; for (const k of p) t = t[k] = t[k] || {}; cur = t; eol(); continue;
      }
      const k = key(); ws(); if (src[i] !== '=') fail('expected = after ' + k); i++; ws();
      cur[k] = value(); eol();
    }
    return root;
  }
  function tq(v) {
    v = String(v == null ? '' : v);
    const multi = v.indexOf('\n') >= 0;
    let o = '';
    for (let k = 0; k < v.length; k++) {
      const c = v[k], code = c.charCodeAt(0);
      if (c === '\\') o += '\\\\';
      else if (c === '"') o += '\\"';
      else if (c === '\n') o += multi ? '\n' : '\\n';
      else if (c === '\t') o += '\\t';
      else if (c === '\r') o += '\\r';
      else if (c === '<' && (v[k + 1] === '/' || v[k + 1] === '!')) o += '\\u003C';
      else if (code < 0x20 || code === 0x7f) o += '\\u' + code.toString(16).toUpperCase().padStart(4, '0');
      else o += c;
    }
    return multi ? '"""\n' + o + '"""' : '"' + o + '"';
  }

  const SCHEMA = JSON.parse($('#vleo-node-schema').textContent);
  const ORIG = parseToml($('#vleo-node-original').textContent);
  let DATA;
  try { DATA = parseToml($('#vleo-node-form').textContent); }
  catch (e) {
    $('#nf').innerHTML = '<p class="nf-warn"><b>The form\'s content block could not be read:</b> ' +
      esc(e.message) + '. It has been edited into something that is not TOML; fix the block ' +
      'marked vleo-node-form in a text editor.</p>';
    return;
  }
  DATA.fields = DATA.fields || {}; DATA.filled_by = DATA.filled_by || {}; DATA.notes = DATA.notes || {};
  DATA.derisk = DATA.derisk || {};
  ORIG.fields = ORIG.fields || {};
  const S = v => (v == null ? '' : String(v));

  function toToml() {
    let o = 'format = ' + tq(SCHEMA.format) + '\nnode = ' + tq(DATA.node) + '\nbase = ' + tq(DATA.base) + '\n';
    if (DATA.new) o += '\n[new]\nid = ' + tq(DATA.new.id) + '\nparent = ' + tq(DATA.new.parent) + '\nkind = ' + tq(DATA.new.kind) + '\n';
    const by = DATA.filled_by;
    o += '\n[filled_by]\nname = ' + tq(by.name) + '\nteam = ' + tq(by.team) + '\ndate = ' + tq(by.date) +
      '\nai = ' + tq(by.ai || 'none') + '\n';
    o += '\n[fields]\n';
    for (const f of SCHEMA.fields) if (f.field in DATA.fields) o += f.field + ' = ' + tq(DATA.fields[f.field]) + '\n';
    if (DATA.view) o += '\n[view]\nkind = ' + tq(DATA.view.kind) + '\nover = ' + tq(DATA.view.over) +
      '\npoints = ' + tq(DATA.view.points) + '\n';
    for (const a of SCHEMA.arrays) for (const r of (DATA[a.name] || [])) {
      o += '\n[[' + a.name + ']]\n';
      for (const c of a.columns) if (c.key in r) o += c.key + ' = ' + tq(r[c.key]) + '\n';
    }
    for (const r of (DATA.known_value || [])) {
      o += '\n[[known_value]]\n';
      for (const k of ['label', 'inputs', 'expected', 'tolerance', 'provenance', 'source']) o += k + ' = ' + tq(r[k]) + '\n';
    }
    o += '\n[derisk]\n';
    for (const [k] of SCHEMA.derisk) o += k + ' = ' + tq(DATA.derisk[k]) + '\n';
    o += '\n[notes]\ntext = ' + tq(DATA.notes.text) + '\n';
    return o;
  }

  // ---- what changed, against the node as the form was made -------------
  function changes() {
    const out = [];
    if (DATA.new && ORIG.new) for (const k of ['id', 'parent', 'kind'])
      if (S(DATA.new[k]) !== S(ORIG.new[k])) out.push('new node ' + k);
    for (const f of SCHEMA.fields) {
      const a = S(ORIG.fields[f.field]), b = S(DATA.fields[f.field]);
      if (a !== b) out.push(f.field);
    }
    for (const a of SCHEMA.arrays) {
      if (JSON.stringify(ORIG[a.name] || []) !== JSON.stringify(DATA[a.name] || [])) out.push(a.label);
    }
    if (ORIG.view && DATA.view && JSON.stringify(ORIG.view) !== JSON.stringify(DATA.view)) out.push('how it is drawn');
    if ((DATA.known_value || []).length) out.push((DATA.known_value || []).length + ' known value(s)');
    if (SCHEMA.derisk.some(([k]) => S(DATA.derisk[k]).trim())) out.push('why it is changing');
    if (S(DATA.notes.text).trim()) out.push('a note for the developers');
    return out;
  }
  // WHICH DECISIONS THE FORM MOVES, as the developers' intake will see them:
  // the same field-to-kind table, sent from the tool.
  function decisions() {
    const kinds = new Set();
    // A new node is one decision, and its first version says so.
    if (DATA.new) return ['node'];
    for (const f of SCHEMA.fields) {
      if (SCHEMA.about[f.field] && S(ORIG.fields[f.field]) !== S(DATA.fields[f.field])) kinds.add(SCHEMA.about[f.field]);
    }
    for (const a of SCHEMA.arrays) {
      if (SCHEMA.about[a.name] && JSON.stringify(ORIG[a.name] || []) !== JSON.stringify(DATA[a.name] || [])) kinds.add(SCHEMA.about[a.name]);
    }
    if (ORIG.view && DATA.view && JSON.stringify(ORIG.view) !== JSON.stringify(DATA.view)) kinds.add('visualisation');
    return [...kinds];
  }
  function recordMissing() {
    const need = DATA.new ? ['rests_on', 'breaks_if'] : ['believed', 'tested', 'learned', 'changed', 'rests_on', 'breaks_if'];
    return need.filter(k => !S(DATA.derisk[k]).trim());
  }
  function paintCount() {
    const c = changes();
    $('#nf-count').textContent = c.length ? c.length + ' change(s): ' + c.join(', ') : 'nothing changed yet';
    const st = $('#nf-dr-state');
    if (!st) return;
    const kinds = decisions(), miss = recordMissing();
    st.className = 'nf-dr-state' + (kinds.length && miss.length ? ' bad' : '');
    st.textContent = !kinds.length
      ? 'Your changes so far are wording only: no record is needed for them.'
      : (DATA.new ? 'A new node. ' : 'Your changes move: ' + kinds.join(', ') + '. ') + (miss.length
        ? 'Still to answer below: ' + miss.map(k => (SCHEMA.derisk.find(d => d[0] === k) || [k, k])[1]).join('; ') +
          (DATA.new ? ' — a new node is not built without them.' : ' — without them the developers apply only your wording.')
        : 'The record is complete: this becomes version ' + (SCHEMA.version + 1) + ' of the node.');
    if (typeof scheduleCheck === 'function') scheduleCheck();
  }

  // ---- controls --------------------------------------------------------
  function control(shape, options, value, onchange, aria) {
    let el;
    const opts = shape === 'quantity' ? SCHEMA.choices.type : shape === 'unit' ? SCHEMA.choices.unit : options;
    if (opts && opts.length) {
      el = document.createElement('select');
      const vals = opts.indexOf(value) >= 0 || value === '' ? opts : [value].concat(opts);
      el.innerHTML = (value === '' ? '<option value=""></option>' : '') +
        vals.map(o => '<option' + (o === value ? ' selected' : '') + '>' + esc(o) + '</option>').join('');
    } else if (shape === 'prose') {
      el = document.createElement('textarea');
      el.value = value; el.rows = Math.min(14, Math.max(3, value.split('\n').length + 1));
    } else if (shape === 'code') {
      // Code: every space kept, no wrapping, and Tab indents instead of
      // leaving the box.
      el = document.createElement('textarea'); el.className = 'nf-code';
      el.value = value; el.spellcheck = false; el.setAttribute('wrap', 'off');
      el.rows = Math.min(24, Math.max(6, value.split('\n').length + 1));
      el.addEventListener('keydown', e => {
        if (e.key !== 'Tab' || e.shiftKey || e.ctrlKey || e.metaKey || e.altKey) return;
        e.preventDefault();
        const a = el.selectionStart, b = el.selectionEnd;
        el.value = el.value.slice(0, a) + '  ' + el.value.slice(b);
        el.selectionStart = el.selectionEnd = a + 2;
        el.dispatchEvent(new Event('input'));
      });
    } else {
      el = document.createElement('input'); el.type = 'text'; el.value = value;
      if (shape === 'number' || shape === 'count') el.inputMode = 'decimal';
      if (shape === 'row') el.setAttribute('list', 'nf-rows');
      if (/(^| )source$/.test(aria || '')) {
        el.setAttribute('list', 'nf-sources');
        const hint = document.createElement('div'); hint.className = 'nf-why nf-src';
        const say = () => {
          const v = el.value.trim(), s = SOURCES.get(v);
          hint.textContent = !v ? 'pick one of the ' + SOURCES.size + ' works in sources/, or name a new one'
            : s ? s[1] + ' — ' + s[2]
            : 'not in sources/ yet: say the full reference here or in the notes, and the developers add its entry before the row is published';
          hint.classList.toggle('bad', !!v && !s);
        };
        el.addEventListener('input', say);
        const box = document.createElement('span'); box.className = 'nf-srcbox';
        box.appendChild(el); box.appendChild(hint); say();
        el.setAttribute('aria-label', aria || '');
        el.addEventListener('input', () => { onchange(el.value); paintCount(); });
        return box;
      }
    }
    el.setAttribute('aria-label', aria || '');
    el.addEventListener(el.tagName === 'SELECT' ? 'change' : 'input', () => { onchange(el.value); paintCount(); });
    return el;
  }

  function question(f) {
    const box = document.createElement('div');
    box.className = 'nf-q'; box.dataset.field = f.field;
    const was = S(ORIG.fields[f.field]);
    box.innerHTML = '<label>' + esc(f.ask.charAt(0).toUpperCase() + f.ask.slice(1)) +
      (f.required ? '<span class="nf-tag req">needed</span>' : '') +
      (f.relation ? '<span class="nf-tag rel">the relation</span>' : '') +
      (SCHEMA.about[f.field] ? '<span class="nf-tag dr" title="changing it needs why it is changing">' +
        esc(SCHEMA.about[f.field]) + '</span>' : '') +
      ' <span class="nf-tag">' + esc(f.field) + '</span></label>' +
      '<div class="nf-why">' + esc(f.why) + '</div>';
    const paint = () => {
      const now = S(DATA.fields[f.field]);
      box.classList.toggle('changed', now !== was);
      let w = box.querySelector('.nf-was');
      if (now !== was) {
        if (!w) { w = document.createElement('div'); w.className = 'nf-was'; box.appendChild(w); }
        w.textContent = 'was: ' + (was || '(blank)');
      } else if (w) w.remove();
    };
    box.appendChild(control(f.shape, f.options, S(DATA.fields[f.field]),
      v => { DATA.fields[f.field] = v; paint(); }, f.field));
    const ex = (SCHEMA.example && SCHEMA.example.fields || {})[f.field];
    if (ex) box.appendChild(exampleBox(ex));
    paint();
    return box;
  }

  // "SHOW THE EXAMPLE": the same question answered for one worked node, so what
  // a good answer looks like is on the page beside the question.
  function exampleBox(text) {
    const d = document.createElement('details'); d.className = 'nf-exbox';
    d.innerHTML = '<summary>Show the example <span class="nf-tag">' + esc(SCHEMA.example.tag) + '</span></summary>' +
      '<pre>' + esc(text) + '</pre>';
    return d;
  }
  function exampleBlocks(a) {
    const bl = (SCHEMA.example && SCHEMA.example.arrays || {})[a.name];
    if (!bl || !bl.length) return null;
    return exampleBox(bl.map((b, i) => a.name + ' ' + (i + 1) + '\n' +
      Object.entries(b).map(([k, v]) => '  ' + k + ': ' + String(v).replace(/\n/g, '\n    ')).join('\n')).join('\n\n'));
  }

  // ---- a case's inputs: one number per input of the node, in SI ----------
  function bindings() {
    return (DATA.input || []).filter(r => S(r.binding).trim()).map(r => [S(r.binding).trim(), S(r.type)]);
  }
  function readInputs(v) {
    const m = {}; const re = /([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([^,}\s]+)/g; let x;
    while ((x = re.exec(S(v)))) m[x[1]] = x[2];
    return m;
  }
  function inputsOk(v) {
    const m = readInputs(v), b = bindings();
    return b.length > 0 && b.every(([k]) => k in m && isFinite(Number(m[k])) && m[k] !== '');
  }
  function inputsEditor(r, onchange) {
    const box = document.createElement('div'); box.className = 'nf-ins';
    const b = bindings();
    if (!b.length) { box.innerHTML = '<span class="nf-muted">add this node\'s inputs first — a case gives each one a value</span>'; return box; }
    const m = readInputs(r.inputs);
    for (const [k, ty] of b) {
      const lab = document.createElement('label'); lab.textContent = k + ' [' + ((SCHEMA.si || {})[ty] || '?') + ']';
      const inp = document.createElement('input'); inp.type = 'text'; inp.inputMode = 'decimal'; inp.value = S(m[k]);
      inp.setAttribute('aria-label', 'case input ' + k);
      inp.addEventListener('input', () => {
        const cur = readInputs(r.inputs); cur[k] = inp.value.trim();
        r.inputs = '{ ' + b.map(([n]) => n).filter(n => S(cur[n]) !== '').map(n => n + ' = ' + cur[n]).join(', ') + ' }';
        inp.style.borderColor = inp.value.trim() && !isFinite(Number(inp.value)) ? 'var(--warn)' : '';
        onchange(); paintCount();
      });
      box.appendChild(lab); box.appendChild(inp);
    }
    return box;
  }

  function blocks(a) {
    const wrap = document.createElement('section');
    const rows = DATA[a.name] = DATA[a.name] || [];
    const draw = () => {
      wrap.innerHTML = '<h2>' + esc(a.label.charAt(0).toUpperCase() + a.label.slice(1)) +
        (a.relation ? ' <span class="nf-tag rel">the relation</span>' : '') + '</h2>' +
        '<p class="nf-why">' + esc(a.why) + '</p>';
      const exb = exampleBlocks(a); if (exb) wrap.appendChild(exb);
      rows.forEach((r, i) => {
        const b = document.createElement('div'); b.className = 'nf-block';
        const last = i === rows.length - 1;
        b.innerHTML = '<div class="nf-block-h"><span>' + esc(a.name) + ' ' + (i + 1) + '</span></div>';
        const rm = document.createElement('button'); rm.type = 'button'; rm.textContent = 'remove';
        rm.disabled = a.end_only && !last;
        rm.title = rm.disabled ? 'Only the last can be removed: each is numbered, and the number is a place in the generated code.' : '';
        rm.onclick = () => { rows.splice(i, 1); draw(); paintCount(); };
        b.firstChild.appendChild(rm);
        const hint = document.createElement('div'); hint.className = 'nf-was';
        // WHAT THIS INPUT CONNECTS TO, as it is typed: the row, its quantity and
        // unit — and a warning when the quantity is not the one expected.
        const paintHint = () => {
          if (a.name !== 'input') return;
          const row = ROWS.get(S(r.var));
          hint.className = 'nf-was' + (row && S(r.type) && row[2] && row[2] !== S(r.type) ? ' nf-bad' : '');
          hint.textContent = !S(r.var) ? 'choose a row from the list' : !row
            ? 'there is no row "' + S(r.var) + '" in the tree — a row that is needed first comes in on its own form'
            : 'reads ' + row[1] + ' — a ' + (row[2] || '?') + ' in ' + (row[3] || '?') +
              (S(r.type) && row[2] && row[2] !== S(r.type) ? ', NOT the ' + S(r.type) + ' this block expects' : '');
        };
        for (const c of a.columns) {
          const col = document.createElement('div'); col.className = 'nf-col';
          if (c.managed) { col.innerHTML = '<span>' + esc(c.key) + ': ' + esc(r[c.key] || '(assigned on apply)') + '</span>'; b.appendChild(col); continue; }
          let ask = c.ask || c.key;
          if (a.name === 'case' && c.key === 'expect') ask = 'the answer your code gave, in ' + ((SCHEMA.si || {})[S(DATA.fields.type) || SCHEMA.type] || 'SI');
          col.innerHTML = '<span>' + esc(ask) + (c.required ? ' <span class="nf-tag req">needed</span>' : '') + '</span>';
          if (a.name === 'case' && (c.key === 'expect' || c.key === 'tolerance')) col.dataset.answer = '1';
          if (c.shape === 'inputs') {
            col.appendChild(inputsEditor(r, () => {}));
          } else {
            const opts = c.shape === 'choice' && a.name === 'case' && c.key === 'refuse' ? ['no', 'yes'] : [];
            col.appendChild(control(c.shape, opts, S(r[c.key]), v => {
              r[c.key] = v;
              if (a.name === 'input' && c.key === 'var' && !S(r.type) && ROWS.get(v)) {
                r.type = ROWS.get(v)[2]; draw(); return;
              }
              if (a.name === 'case' && c.key === 'refuse') showAnswer();
              paintHint();
            }, a.name + ' ' + (i + 1) + ' ' + c.key));
          }
          b.appendChild(col);
        }
        // A case that must be refused has no answer to give.
        const showAnswer = () => {
          if (a.name !== 'case') return;
          for (const el of b.querySelectorAll('[data-answer]')) el.hidden = S(r.refuse) === 'yes';
        };
        showAnswer();
        if (a.name === 'case') {
          const res = document.createElement('div'); res.className = 'nf-was nf-case-res'; res.dataset.case = String(i);
          b.appendChild(res);
        }
        if (a.name === 'input') { b.appendChild(hint); paintHint(); }
        wrap.appendChild(b);
      });
      const add = document.createElement('button'); add.type = 'button';
      add.textContent = 'add ' + (a.end_only ? 'one at the end' : 'one');
      add.onclick = () => {
        const r = {}; for (const c of a.columns) r[c.key] = c.managed ? String(rows.length + 1) : '';
        if (a.name === 'case') { r.refuse = 'no'; r.tolerance = '1e-6'; r.inputs = '{ }'; }
        rows.push(r); draw(); paintCount();
      };
      wrap.appendChild(add);
    };
    draw();
    return wrap;
  }

  function known() {
    const wrap = document.createElement('section');
    const rows = DATA.known_value = DATA.known_value || [];
    const draw = () => {
      wrap.innerHTML = '<h2>Known values</h2><p class="nf-why">An answer you know at given inputs, and where ' +
        'it comes from. A developer records it as a check the node must pass — it is never taken from the ' +
        'tool itself, and never from an assistant.</p>';
      if (SCHEMA.fixtures.length) {
        wrap.insertAdjacentHTML('beforeend', '<p class="nf-muted">Already held by:</p><table class="nf-fx"><tr><th>label</th>' +
          '<th>inputs</th><th>expected</th><th>from</th></tr>' + SCHEMA.fixtures.map(x => '<tr><td>' + esc(x.label) +
          '</td><td>' + esc(x.inputs) + '</td><td>' + esc(x.expect) + ' ± ' + esc(x.tolerance) + '</td><td>' +
          esc(x.provenance) + ' — ' + esc(x.source) + '</td></tr>').join('') + '</table>');
      }
      rows.forEach((r, i) => {
        const b = document.createElement('div'); b.className = 'nf-block';
        b.innerHTML = '<div class="nf-block-h"><span>known value ' + (i + 1) + '</span></div>';
        const rm = document.createElement('button'); rm.type = 'button'; rm.textContent = 'remove';
        rm.onclick = () => { rows.splice(i, 1); draw(); paintCount(); };
        b.firstChild.appendChild(rm);
        const cols = [['label', 'what case this is'], ['inputs', 'the inputs it holds at, each with its unit — e.g. orbit_altitude = 250 km'],
          ['expected', 'the answer, in the node\'s unit'], ['tolerance', 'how close counts, as a fraction (default 1e-6)'],
          ['provenance', 'where it comes from'], ['source', 'the reference — book, paper, page, or who measured it']];
        for (const [k, ask] of cols) {
          const col = document.createElement('div'); col.className = 'nf-col';
          col.innerHTML = '<span>' + esc(ask) + '</span>';
          col.appendChild(control('line', k === 'provenance' ? SCHEMA.provenances : [], S(r[k]), v => { r[k] = v; }, 'known ' + (i + 1) + ' ' + k));
          b.appendChild(col);
        }
        wrap.appendChild(b);
      });
      const add = document.createElement('button'); add.type = 'button'; add.textContent = 'add a known value';
      add.onclick = () => { rows.push({ label: '', inputs: '', expected: '', tolerance: '', provenance: SCHEMA.provenances[0], source: '' }); draw(); paintCount(); };
      wrap.appendChild(add);
    };
    draw();
    return wrap;
  }

  // ---- the page --------------------------------------------------------
  const ROWS = new Map((SCHEMA.rows || []).map(r => [r[0], r]));
  const dl = document.createElement('datalist'); dl.id = 'nf-rows';
  dl.innerHTML = (SCHEMA.rows || []).map(r => '<option value="' + esc(r[0]) + '">' + esc(r[1]) + ' — ' +
    esc(r[2] || '') + ' ' + esc(r[3] || '') + '</option>').join('');
  document.body.appendChild(dl);
  const SOURCES = new Map((SCHEMA.sources || []).map(r => [r[0], r]));
  const ds = document.createElement('datalist'); ds.id = 'nf-sources';
  ds.innerHTML = (SCHEMA.sources || []).map(r => '<option value="' + esc(r[0]) + '">' + esc(r[1]) + '</option>').join('');
  document.body.appendChild(ds);
  // A field that belongs to one kind of node, shown only on that kind.
  const ONLY = { sense: ['required'], declared_value: ['declared', 'required'] };
  const main = $('#nf');
  main.innerHTML = '<div class="nf-bar"><button type="button" class="nf-primary" id="nf-save">save a filled copy</button>' +
    '<button type="button" id="nf-print">print</button><span class="nf-count" id="nf-count"></span></div>';

  const who = document.createElement('section');
  who.innerHTML = '<h2>Who is filling this</h2>';
  const g = document.createElement('div'); g.className = 'nf-grid2';
  for (const [k, ask] of [['name', 'your name'], ['team', 'your group or company'], ['date', 'date (filled in on save if blank)']]) {
    const q = document.createElement('div'); q.className = 'nf-q'; q.innerHTML = '<label>' + esc(ask) + '</label>';
    q.appendChild(control('line', [], S(DATA.filled_by[k]), v => { DATA.filled_by[k] = v; }, k)); g.appendChild(q);
  }
  const ai = document.createElement('div'); ai.className = 'nf-q';
  ai.innerHTML = '<label>did an assistant help?</label><div class="nf-why">none · wording (the text, not the maths) · ' +
    'relation (the equation, its steps or its derivation — those are then derived by a developer, not taken from this form)</div>';
  ai.appendChild(control('choice', SCHEMA.ai_help, S(DATA.filled_by.ai || 'none'), v => { DATA.filled_by.ai = v; }, 'ai'));
  g.appendChild(ai); who.appendChild(g); main.appendChild(who);

  if (SCHEMA.new) {
    DATA.new = DATA.new || { id: '', parent: '', kind: 'computed' };
    const w = document.createElement('section');
    w.innerHTML = '<h2>Where it goes</h2><p class="nf-why">A new node hangs under one group of the tree, in one ' +
      'of its four layers, and is one kind of row. The developers check this first: where a node sits decides ' +
      'who owns it and what it may read.</p>';
    const g3 = document.createElement('div'); g3.className = 'nf-grid2';
    const idq = document.createElement('div'); idq.className = 'nf-q';
    idq.innerHTML = '<label>its id <span class="nf-tag req">needed</span></label><div class="nf-why">lowercase words ' +
      'joined by underscores, starting with a letter — e.g. <code>pay_sensor_mass</code>. It is the answer\'s name ' +
      'everywhere in the design.</div>';
    idq.appendChild(control('line', [], S(DATA.new.id), v => { DATA.new.id = v; }, 'new id')); g3.appendChild(idq);
    const kq = document.createElement('div'); kq.className = 'nf-q';
    kq.innerHTML = '<label>what kind of row <span class="nf-tag req">needed</span></label><div class="nf-why">computed ' +
      '— worked out from what it reads · declared — a number somebody chose · required — a bound the design must ' +
      'meet · achieved — what the design reaches against one · kpi — a figure the programme reports</div>';
    kq.appendChild(control('choice', SCHEMA.kinds, S(DATA.new.kind || 'computed'), v => { DATA.new.kind = v; paintKind(); }, 'new kind'));
    g3.appendChild(kq); w.appendChild(g3);
    const pq = document.createElement('div'); pq.className = 'nf-q';
    pq.innerHTML = '<label>under which group <span class="nf-tag req">needed</span></label><div class="nf-why">every ' +
      'group of the tree, by layer: 1 management · 2 the system · 3 subsystem · 4 the run</div>';
    const sel = document.createElement('select'); sel.setAttribute('aria-label', 'new parent');
    sel.innerHTML = '<option value=""></option>' + SCHEMA.groups.map(g => '<option value="' + esc(g[0]) + '"' +
      (g[0] === S(DATA.new.parent) ? ' selected' : '') + '>Layer ' + g[2] + ' · ' + esc(g[1]) + ' (' + esc(g[0]) + ')</option>').join('');
    sel.addEventListener('change', () => { DATA.new.parent = sel.value; paintCount(); });
    pq.appendChild(sel); w.appendChild(pq);
    main.appendChild(w);
  }
  function paintKind() {
    const kind = DATA.new ? S(DATA.new.kind) : SCHEMA.kind;
    for (const [f, kinds] of Object.entries(ONLY)) {
      const box = document.querySelector('.nf-q[data-field="' + f + '"]');
      if (box) box.hidden = kinds.indexOf(kind) < 0;
    }
  }

  const ctx = document.createElement('section');
  ctx.innerHTML = '<h2>This node</h2><dl class="nf-ctx"><dt>id</dt><dd>' + esc(SCHEMA.node) + '</dd><dt>kind</dt><dd>' +
    esc(SCHEMA.kind) + '</dd><dt>subsystem</dt><dd>' + esc(SCHEMA.subsystem) + ' — owner ' + esc(SCHEMA.owner) +
    '</dd><dt>state</dt><dd>' + esc(SCHEMA.state) + '</dd><dt>reads</dt><dd>' + (esc(SCHEMA.reads.join(', ')) || 'nothing') +
    '</dd><dt>feeds</dt><dd>' + (esc(SCHEMA.feeds.join(', ')) || 'nothing yet') + '</dd></dl>' +
    '<p class="nf-muted">Its place in the tree, its kind and its owner are not on this form: moving a node is a ' +
    'developer\'s decision, taken in the repository.</p>';
  if (!SCHEMA.new) main.appendChild(ctx);

  // THE METHOD, YOUR CODE, YOUR CASES — and the check that runs one against
  // the others while you type, with the same checker intake and the gate use.
  const INTRO = {
    'the method': 'Your relation once more, as a few lines the tool can check, run and translate into the code it ships. ' +
      'It reads the inputs by their names, gives every number its unit, and ends every path with return or refuse.',
    'your code': 'The code you wrote and tested — in MATLAB, Python, C or anything else — and the script that ran it on ' +
      'your test cases. Your cases decide: the method, and the code the tool generates from it, must reproduce every one.'
  };
  function methodRef() {
    const m = SCHEMA.method || {};
    const d = document.createElement('details'); d.className = 'nf-exbox nf-ref';
    d.innerHTML = '<summary>The method language on one page (version ' + esc(m.version) + ')</summary>' +
      '<table>' + (m.statements || []).map(x => '<tr><td><code>' + esc(x[0]) + '</code></td><td>' + esc(x[1]) +
      '<pre>' + esc(x[2]) + '</pre></td></tr>').join('') + '</table>' +
      '<p><b>Functions:</b> ' + (m.functions || []).map(x => '<code title="' + esc(x[1]) + '">' + esc(x[0]) + '</code>').join(' · ') + '</p>' +
      '<p><b>Constants:</b> ' + (m.constants || []).map(x => '<code title="' + esc(x[2]) + '">' + esc(x[0]) + '</code> [' + esc(x[1]) + ']').join(' · ') + '</p>' +
      '<p class="nf-muted">Units go in brackets straight after a number: <code>250 [km]</code>, <code>30 [deg]</code>, ' +
      '<code>3.986e14 [m^3/s^2]</code>. A bare 0 is zero of anything. The full reference is docs/PSEUDOCODE.md.</p>';
    return d;
  }
  const check = document.createElement('section'); check.className = 'nf-check'; check.id = 'nf-check';
  let placedCases = false, placedInputs = false;
  const groups = [];
  for (const f of SCHEMA.fields) if (groups.indexOf(f.group) < 0) groups.push(f.group);
  for (const grp of groups) {
    // What the node reads comes before the method, which reads it by name.
    if (grp === 'the method') {
      const ia = SCHEMA.arrays.find(a => a.name === 'input');
      if (ia) { main.appendChild(blocks(ia)); placedInputs = true; }
    }
    const sec = document.createElement('section');
    sec.innerHTML = '<h2>' + esc(grp.charAt(0).toUpperCase() + grp.slice(1)) + '</h2>' +
      (INTRO[grp] ? '<p class="nf-why">' + esc(INTRO[grp]) + '</p>' : '');
    if (grp === 'the method') sec.appendChild(methodRef());
    for (const f of SCHEMA.fields.filter(x => x.group === grp)) sec.appendChild(question(f));
    main.appendChild(sec);
    if (grp === 'your code') {
      const ca = SCHEMA.arrays.find(a => a.name === 'case');
      if (ca) { main.appendChild(blocks(ca)); main.appendChild(check); placedCases = true; }
    }
  }
  for (const a of SCHEMA.arrays) {
    if (a.name === 'flight' || (a.name === 'case' && placedCases) || (a.name === 'input' && placedInputs)) continue;
    main.appendChild(blocks(a));
    if (a.name === 'case') main.appendChild(check);
  }
  for (const a of SCHEMA.arrays) if (a.name === 'flight') main.appendChild(blocks(a));

  // ---- the check: the method run on your cases, in this page -------------
  let VM = null, VMerr = '';
  async function vm() {
    if (VM || VMerr) return VM;
    try {
      const bin = atob((($('#vleo-method-wasm') || {}).textContent || '').trim());
      if (!bin) throw new Error('this form was made without its checker, web/method.wasm.gz');
      const gz = new Uint8Array(bin.length);
      for (let k = 0; k < bin.length; k++) gz[k] = bin.charCodeAt(k);
      // Carried compressed, to keep the form small; unpacked by the browser.
      if (typeof DecompressionStream !== 'function') throw new Error('this browser is too old to unpack the checker');
      const bytes = await new Response(new Blob([gz]).stream().pipeThrough(new DecompressionStream('gzip'))).arrayBuffer();
      VM = (await WebAssembly.instantiate(bytes, {})).instance.exports;
    } catch (e) { VMerr = String(e && e.message || e); }
    return VM;
  }
  async function runReport(text) {
    const v = await vm(); if (!v) return null;
    const enc = new TextEncoder().encode(text);
    const p = v.vleo_alloc(enc.length);
    new Uint8Array(v.memory.buffer, p, enc.length).set(enc);
    const out = v.vleo_report(p, enc.length), n = v.vleo_report_len();
    return JSON.parse(new TextDecoder().decode(new Uint8Array(v.memory.buffer, out, n)));
  }
  const num = v => { const x = Number(S(v).trim()); return isFinite(x) && S(v).trim() !== '' ? String(x) : null; };
  // Only a case whose answer YOU have typed is run: the method's value is shown
  // beside yours, never before it, so the tool can never be where your answer
  // came from.
  function checkToml() {
    // The plain form `method::report_plain` reads — see its doc comment.
    const one = v => S(v).replace(/[\r\n]+/g, ' ').trim();
    let o = 'output ' + one(DATA.fields.type) + '\n';
    for (const [k, ty] of bindings()) o += 'input ' + one(k) + ' ' + one(ty) + '\n';
    const used = [], waiting = [];
    (DATA.case || []).forEach((r, i) => {
      const refuse = S(r.refuse) === 'yes';
      if ((!refuse && num(r.expect) == null) || !inputsOk(r.inputs)) { waiting.push(i); return; }
      const m = readInputs(r.inputs);
      const ins = bindings().map(([k]) => k + '=' + Number(m[k])).join(';');
      o += 'case ' + (refuse ? '1 - - ' : '0 ' + num(r.expect) + ' ' + (num(r.tolerance) || '0') + ' ') +
        ins + ' ' + (one(r.label) || 'case ' + (i + 1)) + '\n';
      used.push(i);
    });
    o += 'method\n' + S(DATA.fields.method_text);
    return { text: o, used, waiting };
  }
  let timer = null, seq = 0;
  function scheduleCheck() { clearTimeout(timer); timer = setTimeout(doCheck, 350); }
  async function doCheck() {
    const my = ++seq;
    const head = '<h3>Check: the method against your cases</h3>';
    const resEls = [...document.querySelectorAll('.nf-case-res')];
    for (const el of resEls) { el.textContent = ''; el.className = 'nf-was nf-case-res'; }
    if (!S(DATA.fields.method_text).trim()) {
      check.innerHTML = head + '<p class="nf-why">Write the method above, and your cases, and this runs one on the ' +
        'other as you type — the same check the developers run when the form arrives.</p>';
      return;
    }
    const { text, used, waiting } = checkToml();
    const r = await runReport(text);
    if (my !== seq) return;
    if (!r) {
      check.innerHTML = head + '<p class="nf-warn">This browser could not start the checker (' + esc(VMerr) +
        '). The developers run the same check when the form arrives; nothing is lost.</p>';
      return;
    }
    let h = head;
    if (r.error) h += '<p class="bad">' + esc(r.error) + '</p>';
    if (r.diags.length) h += '<ul>' + r.diags.map(d => '<li class="' + (d.severity === 'error' ? 'bad' : '') + '">' +
      (d.line ? 'line ' + d.line + ': ' : '') + (d.severity === 'note' ? 'note: ' : '') + esc(d.msg) + '</li>').join('') + '</ul>';
    else if (!r.error) h += '<p class="ok">The method reads, and its units agree.</p>';
    const agree = r.cases.filter(c => c.agrees).length;
    if (r.cases.length) h += '<p>' + agree + ' of ' + r.cases.length + ' case(s) agree with your code.</p><ul>' +
      r.cases.map(c => '<li class="' + (c.agrees ? 'ok' : 'bad') + '">' + esc(c.label) + ': ' + esc(c.text) +
        (c.got != null && c.agrees && !c.refuse ? ' — the method gives ' + esc(String(Number(c.got))) : '') + '</li>').join('') + '</ul>';
    if (waiting.length) h += '<p class="nf-muted">' + waiting.length + ' case(s) not run yet: give every input a number, and ' +
      'type the answer your code gave (or say it must be refused). The method\'s value is shown only after yours.</p>';
    if (r.shortfall.length) h += '<ul>' + r.shortfall.map(x => '<li class="bad">' + esc(x) + '</li>').join('') + '</ul>';
    h += '<p class="nf-sound' + (r.sound ? '' : ' bad') + '">' + (r.sound
      ? 'Sound: the method checks, and agrees with every one of your cases.'
      : 'Not sound yet — the developers will see exactly what is shown here.') + '</p>';
    check.innerHTML = h;
    used.forEach((i, k) => {
      const c = r.cases[k], el = resEls.find(e => e.dataset.case === String(i));
      if (!c || !el) return;
      el.className = 'nf-was nf-case-res ' + (c.agrees ? 'ok' : 'bad');
      el.textContent = (c.agrees ? '✓ ' : '✗ ') + c.text +
        (c.got != null && c.agrees && !c.refuse ? ' — the method gives ' + String(Number(c.got)) : '');
    });
    for (const i of waiting) {
      const el = resEls.find(e => e.dataset.case === String(i));
      if (el) el.textContent = 'not run yet: every input needs a number, and your code\'s answer (or refuse = yes)';
    }
  }

  if (DATA.view) {
    const v = document.createElement('section');
    v.innerHTML = '<h2>How the answer is drawn</h2><p class="nf-why">A number, a line over one of its inputs, or bars.</p>';
    const g2 = document.createElement('div'); g2.className = 'nf-grid2';
    for (const [k, ask, sh, opts] of [['kind', 'drawn as', 'choice', SCHEMA.view_kinds], ['over', 'over which input (a line), or which row (bars)', 'line', []],
      ['points', 'how many points along the line', 'count', []]]) {
      const q = document.createElement('div'); q.className = 'nf-q'; q.innerHTML = '<label>' + esc(ask) + '</label>';
      q.appendChild(control(sh, opts, S(DATA.view[k]), val => { DATA.view[k] = val; }, 'view ' + k)); g2.appendChild(q);
    }
    v.appendChild(g2); main.appendChild(v);
  }
  // ---- why it is changing: the de-risking record ----------------------
  const dr = document.createElement('section'); dr.className = 'nf-dr';
  dr.innerHTML = '<h2>Why it is changing</h2><p class="nf-why">' + (SCHEMA.new
    ? 'A new node is a belief nobody has tested yet. Say what it rests on and what would break it — the ' +
      'rest can wait for the first time it changes.'
    : 'A node changes because a belief broke: somebody tested it and learned otherwise. The answers here ' +
      'become version ' + (SCHEMA.version + 1) + ' of this node, and one row of the programme\'s ' +
      'de-risking narrative. Wording alone needs none of it.') + '</p><p class="nf-dr-state" id="nf-dr-state"></p>';
  for (const [k, ask, why] of SCHEMA.derisk) {
    if (SCHEMA.new && ['believed', 'tested', 'learned', 'cost'].indexOf(k) >= 0) continue;
    const q = document.createElement('div'); q.className = 'nf-q'; q.dataset.derisk = k;
    q.innerHTML = '<label>' + esc(ask.charAt(0).toUpperCase() + ask.slice(1)) + '</label><div class="nf-why">' + esc(why) + '</div>';
    q.appendChild(control(k === 'cost' ? 'line' : 'prose', [], S(DATA.derisk[k]), v => { DATA.derisk[k] = v; }, 'why ' + k));
    dr.appendChild(q);
  }
  dr.insertAdjacentHTML('beforeend', '<details class="nf-ex"><summary>A worked example ' +
    '<span class="nf-tag">illustrative — not this node</span></summary><dl>' +
    '<dt>What we believed</dt><dd>That the drag coefficient could be bounded from modelling alone, without ' +
    'reconciliation against flight decay data.</dd>' +
    '<dt>What we tested</dt><dd>Free-molecular gas-surface interaction runs against two density models, ' +
    'cross-checked against published on-orbit decay in the 350–380 km band.</dd>' +
    '<dt>What we now know</dt><dd>The 1-sigma band narrowed from 28% to 19%, but the residual is dominated by ' +
    'energy accommodation, which only flight data will settle. Ground work alone will not reach the 15% target.</dd>' +
    '<dt>What it cost</dt><dd>$310k</dd>' +
    '<dt>What changes, and what it gains</dt><dd>Propellant margin held at 30% to CDR rather than released; the ' +
    'air-breathing propulsion decision moved to Q2-27.</dd>' +
    '<dt>Risks</dt><dd>R-09 closed · R-01 L5-&gt;L4</dd>' +
    '<dt>Rests on now</dt><dd>The 19% band, until flight decay data exists.</dd>' +
    '<dt>Would break if</dt><dd>The first flight decay residual falls outside the 19% band.</dd></dl></details>');
  main.appendChild(dr);

  main.appendChild(known());

  const notes = document.createElement('section');
  notes.innerHTML = '<h2>Anything else the developers should know</h2><p class="nf-why">What you changed and why, what ' +
    'you were unsure of, what you would like to see when you run it.</p>';
  const nq = document.createElement('div'); nq.className = 'nf-q';
  nq.appendChild(control('prose', [], S(DATA.notes.text), v => { DATA.notes.text = v; }, 'notes'));
  notes.appendChild(nq); main.appendChild(notes);

  paintKind();
  $('#nf-print').onclick = () => window.print();
  $('#nf-save').onclick = () => {
    if (!S(DATA.filled_by.date)) DATA.filled_by.date = new Date().toISOString().slice(0, 10);
    const open = '<script type="application/toml" id="vleo-node-form">';
    const at = PRISTINE.indexOf(open);
    const end = PRISTINE.indexOf('</' + 'script>', at);
    const html = PRISTINE.slice(0, at + open.length) + '\n' + toToml() + PRISTINE.slice(end);
    const a = document.createElement('a');
    a.href = URL.createObjectURL(new Blob([html], { type: 'text/html' }));
    a.download = (SCHEMA.new ? 'new-' + (S(DATA.new.id) || 'node') : SCHEMA.node) + '.node-form.html';
    document.body.appendChild(a); a.click(); a.remove();
    setTimeout(() => URL.revokeObjectURL(a.href), 4000);
  };
  paintCount();
  doCheck();
})();
