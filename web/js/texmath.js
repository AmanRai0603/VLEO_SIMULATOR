/*
  Equations, typeset with no library and no network.

  Every browser the tool supports draws MathML itself, so an equation needs
  only to be turned from the LaTeX a group writes into MathML. This does the
  part of LaTeX equations actually use — fractions, roots, powers and
  indices, Greek, operators and relations, functions, big operators with
  limits, accents, brackets that grow, text, cases and matrices — and says
  what it did not understand rather than drawing something else.

  It also reads SHORTHAND, the way people type maths into an email —
  `v = sqrt(mu/r)`, `a_r = 3/2 J2 (R/r)^2` — and the expressions of the method
  language, which are the same shape. That is how a pseudocode line becomes
  an equation on the page without anybody writing it twice, and how the
  equation helper turns plain typing into the LaTeX equations.csv stores.

  The output is built from escaped text only. A folder cannot put markup on
  the page through an equation.
*/
'use strict';

const GREEK = {
  alpha: 'α', beta: 'β', gamma: 'γ', delta: 'δ', epsilon: 'ϵ', varepsilon: 'ε', zeta: 'ζ', eta: 'η',
  theta: 'θ', vartheta: 'ϑ', iota: 'ι', kappa: 'κ', lambda: 'λ', mu: 'μ', nu: 'ν', xi: 'ξ', pi: 'π',
  varpi: 'ϖ', rho: 'ρ', varrho: 'ϱ', sigma: 'σ', varsigma: 'ς', tau: 'τ', upsilon: 'υ', phi: 'ϕ',
  varphi: 'φ', chi: 'χ', psi: 'ψ', omega: 'ω',
  Gamma: 'Γ', Delta: 'Δ', Theta: 'Θ', Lambda: 'Λ', Xi: 'Ξ', Pi: 'Π', Sigma: 'Σ', Upsilon: 'Υ',
  Phi: 'Φ', Psi: 'Ψ', Omega: 'Ω',
};

const SYMBOL = {
  cdot: '·', times: '×', div: '÷', pm: '±', mp: '∓', le: '≤', leq: '≤', ge: '≥', geq: '≥', ne: '≠',
  neq: '≠', approx: '≈', sim: '∼', simeq: '≃', equiv: '≡', propto: '∝', ll: '≪', gg: '≫',
  to: '→', rightarrow: '→', leftarrow: '←', Rightarrow: '⇒', Leftarrow: '⇐', leftrightarrow: '↔',
  Leftrightarrow: '⇔', mapsto: '↦', infty: '∞', partial: '∂', nabla: '∇', in: '∈', notin: '∉',
  subset: '⊂', cup: '∪', cap: '∩', forall: '∀', exists: '∃', ldots: '…', cdots: '⋯', dots: '…',
  degree: '°', circ: '∘', ast: '∗', star: '⋆', oplus: '⊕', otimes: '⊗', perp: '⊥', parallel: '∥',
  angle: '∠', prime: '′', hbar: 'ℏ', ell: 'ℓ', Re: 'ℜ', Im: 'ℑ', odot: '⊙', bullet: '•',
  langle: '⟨', rangle: '⟩', lfloor: '⌊', rfloor: '⌋', lceil: '⌈', rceil: '⌉', vert: '|', Vert: '‖',
  lbrace: '{', rbrace: '}',
};

const FUNCS = ['sin', 'cos', 'tan', 'sec', 'csc', 'cot', 'arcsin', 'arccos', 'arctan', 'sinh', 'cosh',
  'tanh', 'exp', 'ln', 'log', 'lg', 'min', 'max', 'sup', 'inf', 'lim', 'det', 'deg', 'arg', 'gcd', 'mod'];
const LIMITS = { sum: '∑', prod: '∏', coprod: '∐', bigcup: '⋃', bigcap: '⋂', lim: 'lim', max: 'max', min: 'min', sup: 'sup', inf: 'inf' };
const INTEGRALS = { int: '∫', iint: '∬', iiint: '∭', oint: '∮' };
const ACCENT = { hat: '^', widehat: '^', bar: '¯', overline: '¯', vec: '→', dot: '˙', ddot: '¨', tilde: '~', widetilde: '~' };
const SPACE = { ',': '0.17em', ':': '0.22em', ';': '0.28em', '!': '-0.17em', quad: '1em', qquad: '2em', ' ': '0.25em' };
const OPS = '+-=<>,;:!/*|()[]\'';

const x = s => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

// ── LaTeX → MathML ─────────────────────────────────────────────────────────

function tokenize(src) {
  const t = [];
  let i = 0;
  while (i < src.length) {
    const c = src[i];
    if (c === '\\') {
      const m = /^\\([A-Za-z]+|.)/.exec(src.slice(i));
      if (!m) { t.push({ k: 'ch', v: '\\' }); i++; continue; }
      t.push({ k: 'cmd', v: m[1] }); i += m[0].length; continue;
    }
    if (/\s/.test(c)) { t.push({ k: 'sp' }); i++; continue; }
    if (/[0-9.]/.test(c)) {
      const m = /^[0-9]*\.?[0-9]+|^[0-9]+\.?/.exec(src.slice(i));
      t.push({ k: 'num', v: m[0] }); i += m[0].length; continue;
    }
    t.push({ k: 'ch', v: c }); i++;
  }
  return t;
}

class TexParser {
  constructor(src) { this.t = tokenize(src); this.i = 0; this.problems = []; }
  peek() { while (this.t[this.i] && this.t[this.i].k === 'sp') this.i++; return this.t[this.i]; }
  next() { const t = this.peek(); this.i++; return t; }
  /** Raw text up to the matching close brace — for \text{…} keeps its spaces. */
  rawGroup() {
    while (this.t[this.i] && this.t[this.i].k === 'sp') this.i++;
    const open = this.t[this.i];
    if (!open || open.v !== '{') { const a = this.next(); return a ? (a.v || ' ') : ''; }
    this.i++;
    let depth = 1, s = '';
    while (this.i < this.t.length) {
      const tk = this.t[this.i++];
      if (tk.k === 'ch' && tk.v === '{') depth++;
      if (tk.k === 'ch' && tk.v === '}') { depth--; if (!depth) break; }
      s += tk.k === 'sp' ? ' ' : tk.k === 'cmd' ? (tk.v.length === 1 ? tk.v : '\\' + tk.v) : tk.v;
    }
    return s;
  }
  group() {
    const t = this.peek();
    if (t && t.k === 'ch' && t.v === '{') {
      this.next();
      const r = this.row(['}']);
      this.next();
      return r;
    }
    return this.atom() || '<mrow></mrow>';
  }
  row(stops) {
    const parts = [];
    for (;;) {
      const t = this.peek();
      if (!t) break;
      if (t.k === 'ch' && stops.includes(t.v)) break;
      if (t.k === 'cmd' && stops.includes('\\' + t.v)) break;
      const a = this.withScripts();
      if (a === null) break;
      parts.push(a);
    }
    return parts.length === 1 ? parts[0] : '<mrow>' + parts.join('') + '</mrow>';
  }
  withScripts() {
    const start = this.peek();
    const big = start && start.k === 'cmd' && (LIMITS[start.v] !== undefined);
    let base = this.atom();
    if (base === null) return null;
    let sub = null, sup = null;
    for (;;) {
      const t = this.peek();
      if (t && t.k === 'ch' && t.v === '_' && sub === null) { this.next(); sub = this.group(); continue; }
      if (t && t.k === 'ch' && t.v === '^' && sup === null) { this.next(); sup = this.group(); continue; }
      if (t && t.k === 'ch' && t.v === "'" ) { this.next(); sup = (sup ? sup.replace(/<\/mrow>$/, '') : '<mrow>') + '<mo>′</mo></mrow>'; continue; }
      break;
    }
    if (sub === null && sup === null) return base;
    if (big) {
      if (sub !== null && sup !== null) return '<munderover>' + base + sub + sup + '</munderover>';
      return sub !== null ? '<munder>' + base + sub + '</munder>' : '<mover>' + base + sup + '</mover>';
    }
    if (sub !== null && sup !== null) return '<msubsup>' + base + sub + sup + '</msubsup>';
    return sub !== null ? '<msub>' + base + sub + '</msub>' : '<msup>' + base + sup + '</msup>';
  }
  atom() {
    const t = this.next();
    if (!t) return null;
    if (t.k === 'num') return '<mn>' + x(t.v) + '</mn>';
    if (t.k === 'ch') {
      if (t.v === '{') { const r = this.row(['}']); this.next(); return '<mrow>' + r + '</mrow>'; }
      if (t.v === '}') { this.problems.push('an unmatched }'); return '<mrow></mrow>'; }
      if (t.v === '&') return '<mo>&amp;</mo>';
      if (t.v === '~') return '<mspace width="0.25em"></mspace>';
      if (/[A-Za-z]/.test(t.v)) return '<mi>' + x(t.v) + '</mi>';
      if (OPS.includes(t.v)) {
        const v = t.v === '*' ? '∗' : t.v === '-' ? '−' : t.v === "'" ? '′' : t.v;
        const fence = '()[]|'.includes(t.v) ? ' stretchy="false"' : '';
        return '<mo' + fence + '>' + x(v) + '</mo>';
      }
      return '<mo>' + x(t.v) + '</mo>';
    }
    return this.command(t.v);
  }
  command(c) {
    if (GREEK[c]) return '<mi>' + GREEK[c] + '</mi>';
    if (SPACE[c]) return '<mspace width="' + SPACE[c] + '"></mspace>';
    if (c === '\\') return '<mspace linebreak="newline"></mspace>';
    if (c === '{' || c === '}' || c === '%' || c === '$' || c === '#' || c === '_' || c === '&') return '<mo>' + x(c) + '</mo>';
    if (c === '|') return '<mo>‖</mo>';
    if (SYMBOL[c]) return (/[A-Za-z]/.test(SYMBOL[c]) || 'ℏℓℜℑ∞∂∇'.includes(SYMBOL[c]) ? '<mi>' : '<mo>') + SYMBOL[c] +
      (/[A-Za-z]/.test(SYMBOL[c]) || 'ℏℓℜℑ∞∂∇'.includes(SYMBOL[c]) ? '</mi>' : '</mo>');
    if (c === 'frac' || c === 'dfrac' || c === 'tfrac') return '<mfrac>' + this.group() + this.group() + '</mfrac>';
    if (c === 'sqrt') {
      const t = this.peek();
      if (t && t.k === 'ch' && t.v === '[') {
        this.next();
        const n = this.row([']']); this.next();
        return '<mroot>' + this.group() + n + '</mroot>';
      }
      return '<msqrt>' + this.group() + '</msqrt>';
    }
    if (c === 'text' || c === 'textrm' || c === 'mbox') return '<mtext>' + x(this.rawGroup()) + '</mtext>';
    if (c === 'mathrm' || c === 'operatorname' || c === 'rm') return '<mi mathvariant="normal">' + x(this.rawGroup()) + '</mi>';
    if (c === 'mathit') return '<mi>' + x(this.rawGroup()) + '</mi>';
    if (c === 'mathbf' || c === 'boldsymbol' || c === 'bm') return '<mrow mathvariant="bold">' + this.group() + '</mrow>';
    if (c === 'mathcal') return '<mi mathvariant="script">' + x(this.rawGroup()) + '</mi>';
    if (c === 'mathbb') return '<mi mathvariant="double-struck">' + x(this.rawGroup()) + '</mi>';
    if (ACCENT[c]) return '<mover accent="true">' + this.group() + '<mo>' + ACCENT[c] + '</mo></mover>';
    if (c === 'underline') return '<munder>' + this.group() + '<mo>_</mo></munder>';
    if (FUNCS.includes(c)) return '<mi mathvariant="normal">' + c + '</mi>';
    if (LIMITS[c]) return '<mo movablelimits="false">' + LIMITS[c] + '</mo>';
    if (INTEGRALS[c]) return '<mo>' + INTEGRALS[c] + '</mo>';
    if (c === 'left' || c === 'right' || c === 'big' || c === 'Big' || c === 'bigg' || c === 'Bigg') {
      if (c === 'right') { this.problems.push('a \\right with no \\left'); this.next(); return '<mrow></mrow>'; }
      const d = this.delim();
      if (c !== 'left') return '<mo>' + x(d) + '</mo>';
      const inner = this.row(['\\right']);
      const t = this.next();
      const close = t ? this.delim() : '';
      if (!t) this.problems.push('a \\left with no \\right');
      return '<mrow><mo fence="true">' + x(d) + '</mo>' + inner + '<mo fence="true">' + x(close) + '</mo></mrow>';
    }
    if (c === 'begin') return this.environment(this.rawGroup());
    if (c === 'end') { this.rawGroup(); return '<mrow></mrow>'; }
    this.problems.push('\\' + c + ' is not something this page typesets');
    return '<merror><mtext>\\' + x(c) + '</mtext></merror>';
  }
  delim() {
    const t = this.next();
    if (!t) return '';
    if (t.k === 'ch') return t.v === '.' ? '' : t.v;
    if (t.k === 'cmd') return SYMBOL[t.v] || (t.v === '{' ? '{' : t.v === '}' ? '}' : t.v === '|' ? '‖' : '');
    return '';
  }
  environment(name) {
    const fences = { pmatrix: ['(', ')'], bmatrix: ['[', ']'], vmatrix: ['|', '|'], Bmatrix: ['{', '}'],
      cases: ['{', ''], matrix: ['', ''], aligned: ['', ''], align: ['', ''], 'align*': ['', ''], array: ['', ''] };
    if (!fences[name]) { this.problems.push('the environment ' + name + ' is not something this page typesets'); }
    if (name === 'array') this.rawGroup();
    const rows = [[]];
    for (;;) {
      const t = this.peek();
      if (!t) { this.problems.push('\\begin{' + name + '} is never ended'); break; }
      if (t.k === 'cmd' && t.v === 'end') { this.next(); this.rawGroup(); break; }
      if (t.k === 'cmd' && t.v === '\\') { this.next(); rows.push([]); continue; }
      if (t.k === 'ch' && t.v === '&') { this.next(); rows[rows.length - 1].push(null); continue; }
      const cell = rows[rows.length - 1];
      const a = this.withScripts();
      if (a === null) break;
      if (!cell.length || cell[cell.length - 1] === null) cell.push(a); else cell[cell.length - 1] += a;
    }
    const align = name === 'cases' || name.startsWith('align') ? ' columnalign="left"' : '';
    const table = '<mtable' + align + '>' + rows.filter(r => r.length).map(r =>
      '<mtr>' + r.map(c => '<mtd>' + (c || '') + '</mtd>').join('') + '</mtr>').join('') + '</mtable>';
    const [o, cl] = fences[name] || ['', ''];
    return o || cl ? '<mrow><mo fence="true">' + x(o) + '</mo>' + table + '<mo fence="true">' + x(cl) + '</mo></mrow>' : table;
  }
}

/**
 * `{ html, problems }`: MathML for a LaTeX equation. `display` sets it on its
 * own line. Problems name what was not understood; the rest is still drawn.
 */
export function texToMathml(tex, display = false) {
  const p = new TexParser(String(tex || ''));
  let body = '';
  try { body = p.row([]); } catch (e) { p.problems.push(String(e && e.message || e)); }
  while (p.peek()) {
    const t = p.next();
    p.problems.push('an unexpected ' + (t.v || t.k));
    try { body += p.row([]); } catch { break; }
  }
  return {
    html: '<math' + (display ? ' display="block"' : '') + ' class="tm"><semantics>' + (body || '<mrow></mrow>') +
      '<annotation encoding="application/x-tex">' + x(tex) + '</annotation></semantics></math>',
    problems: p.problems,
  };
}

// ── shorthand and method-language expressions → LaTeX ─────────────────────

const NAMED_FUNCS = ['sin', 'cos', 'tan', 'asin', 'acos', 'atan', 'atan2', 'sinh', 'cosh', 'tanh', 'exp', 'ln',
  'log', 'log10', 'min', 'max', 'floor', 'ceil', 'round', 'interp', 'clamp', 'sign', 'hypot', 'mean', 'sum'];

function shortTokens(src) {
  const t = [];
  const re = /\s*(?:(\d+(?:\.\d*)?(?:[eE][+-]?\d+)?|\.\d+(?:[eE][+-]?\d+)?)|([A-Za-z][A-Za-z0-9_]*(?:\.[A-Za-z][A-Za-z0-9_]*)?)|(\[[^\]]*\])|(<=|>=|==|!=|->|<-|\*\*|[-+*/^(),=<>!_|]))/y;
  let m, last = 0;
  re.lastIndex = 0;
  while ((m = re.exec(src)) !== null) {
    const sp = /^\s/.test(m[0]);
    if (m[1] !== undefined) t.push({ k: 'num', v: m[1], sp });
    else if (m[2] !== undefined) t.push({ k: 'id', v: m[2], sp });
    else if (m[3] !== undefined) t.push({ k: 'unit', v: m[3].slice(1, -1).trim(), sp });
    else t.push({ k: 'op', v: m[4], sp });
    last = re.lastIndex;
    if (last >= src.length) break;
  }
  const rest = src.slice(last).trim();
  return { t, rest };
}

/** An identifier as LaTeX: Greek names become Greek, `a_b` an index, long names upright-italic. */
export function idTex(name) {
  const s = String(name);
  const i = s.indexOf('_');
  const base = i < 0 ? s : s.slice(0, i), sub = i < 0 ? '' : s.slice(i + 1);
  // `mu` and `MU` (a constant's name, MU_EARTH) are both the Greek letter.
  const one = n => GREEK[n] !== undefined ? '\\' + n
    : n.length > 1 && n === n.toUpperCase() && GREEK[n.toLowerCase()] !== undefined ? '\\' + n.toLowerCase()
      : n.length === 1 ? n : '\\mathit{' + n + '}';
  const b = one(base);
  if (!sub) return b;
  const st = sub.split('_').map(p => GREEK[p] !== undefined ? '\\' + p : /^\d+$/.test(p) || p.length === 1 ? p : '\\mathrm{' + p + '}').join(',');
  return '{' + b + '}_{' + st + '}';
}

class ShortParser {
  constructor(src) { const { t, rest } = shortTokens(src); this.t = t; this.i = 0; this.rest = rest; this.problems = []; }
  peek() { return this.t[this.i]; }
  next() { return this.t[this.i++]; }
  is(v) { const t = this.peek(); return t && t.k === 'op' && t.v === v; }
  // Precedence, low to high: relation, + -, * /, unary, ^.
  relation() {
    let l = this.sum();
    for (;;) {
      const t = this.peek();
      if (!t || t.k !== 'op' || !['=', '<', '>', '<=', '>=', '==', '!=', '->', '<-'].includes(t.v)) return l;
      this.next();
      const r = this.sum();
      const m = { '=': '=', '==': '=', '<': '<', '>': '>', '<=': '\\le', '>=': '\\ge', '!=': '\\ne', '->': '\\to', '<-': '\\leftarrow' }[t.v];
      l = { tex: l.tex + ' ' + m + ' ' + r.tex, p: 0 };
    }
  }
  sum() {
    let l = this.product();
    while (this.is('+') || this.is('-')) {
      const op = this.next().v;
      const r = this.product();
      l = { tex: l.tex + ' ' + op + ' ' + (op === '-' && r.p <= 1 ? '\\left(' + r.tex + '\\right)' : r.tex), p: 1 };
    }
    return l;
  }
  product() {
    let l = this.unary();
    for (;;) {
      if (this.is('*') || this.is('/')) {
        const op = this.next().v;
        const r = this.unary();
        if (op === '/') l = { tex: '\\frac{' + l.tex + '}{' + r.tex + '}', p: 3, frac: true };
        else l = { tex: wrap(l, 2) + ' \\, ' + wrap(r, 2), p: 2 };
        continue;
      }
      const t = this.peek();
      // Juxtaposition, as people write `3/2 J2 (R/r)^2`: a product.
      if (t && (t.k === 'id' || t.k === 'num' || (t.k === 'op' && t.v === '('))) {
        const r = this.unary();
        l = { tex: wrap(l, 2) + ' \\, ' + wrap(r, 2), p: 2 };
        continue;
      }
      return l;
    }
  }
  unary() {
    if (this.is('-')) { this.next(); const r = this.unary(); return { tex: '-' + wrap(r, 2), p: 2 }; }
    if (this.is('+')) { this.next(); return this.unary(); }
    if (this.peek() && this.peek().k === 'id' && this.peek().v === 'not') { this.next(); const r = this.unary(); return { tex: '\\lnot ' + wrap(r, 2), p: 2 }; }
    return this.power();
  }
  power() {
    const b = this.postfix();
    if (this.is('^') || this.is('**')) {
      this.next();
      const e = this.unary();
      const base = b.p < 4 || b.frac ? '\\left(' + b.tex + '\\right)' : b.tex;
      return { tex: '{' + base + '}^{' + e.tex + '}', p: 4 };
    }
    return b;
  }
  postfix() {
    let a = this.primary();
    if (this.peek() && this.peek().k === 'unit') {
      const u = this.next().v;
      a = { tex: a.tex + '\\,' + unitTex(u), p: 2 };
    }
    return a;
  }
  primary() {
    const t = this.next();
    if (!t) { this.problems.push('the expression ends too soon'); return { tex: '', p: 5 }; }
    if (t.k === 'num') return { tex: t.v.replace(/[eE]([+-]?\d+)$/, (_, e) => ' \\times 10^{' + Number(e) + '}'), p: /e/i.test(t.v) ? 2 : 5 };
    if (t.k === 'op' && t.v === '(') {
      const e = this.relation();
      if (this.is(')')) this.next(); else this.problems.push('a ( that is never closed');
      return { tex: '\\left(' + e.tex + '\\right)', p: 5 };
    }
    if (t.k === 'op' && t.v === '|') {
      const e = this.sum();
      if (this.is('|')) this.next();
      return { tex: '\\left|' + e.tex + '\\right|', p: 5 };
    }
    if (t.k === 'id') {
      // A call is a known function, or a name with its bracket touching it:
      // `sqrt (x)` and `f(x)` are calls, `J2 (R/r)` is a product.
      const known = ['sqrt', 'abs', 'pow'].includes(t.v) || NAMED_FUNCS.includes(t.v) || FUNCS.includes(t.v);
      if (this.is('(') && (known || !this.peek().sp)) {
        this.next();
        const args = [];
        if (!this.is(')')) {
          args.push(this.relation());
          while (this.is(',')) { this.next(); args.push(this.relation()); }
        }
        if (this.is(')')) this.next(); else this.problems.push('a ( that is never closed after ' + t.v);
        return call(t.v, args);
      }
      if (t.v === 'and' || t.v === 'or') return { tex: t.v === 'and' ? '\\land' : '\\lor', p: 0 };
      return { tex: idTex(t.v), p: 5 };
    }
    if (t.k === 'unit') return { tex: unitTex(t.v), p: 5 };
    this.problems.push('did not expect ' + t.v);
    return { tex: '', p: 5 };
  }
}

function wrap(e, p) { return e.p < p ? '\\left(' + e.tex + '\\right)' : e.tex; }

function unitTex(u) {
  if (!u || u === '1') return '';
  return '\\mathrm{' + u.replace(/\*/g, '\\cdot ').replace(/\^(-?\d+)/g, '^{$1}') + '}';
}

function call(name, args) {
  const a = args.map(e => e.tex);
  if (name === 'sqrt') return { tex: '\\sqrt{' + a[0] + '}', p: 5 };
  if (name === 'abs') return { tex: '\\left|' + a[0] + '\\right|', p: 5 };
  if (name === 'pow') return { tex: '{' + wrap(args[0], 5) + '}^{' + a[1] + '}', p: 4 };
  if (name === 'exp' && args.length === 1 && !/\\frac|\\left/.test(a[0])) return { tex: 'e^{' + a[0] + '}', p: 4 };
  const f = NAMED_FUNCS.includes(name) || FUNCS.includes(name) ? '\\operatorname{' + name + '}' : idTex(name);
  return { tex: f + '\\left(' + a.join(',\\ ') + '\\right)', p: 5 };
}

/** `{ tex, problems }` from shorthand such as `v = sqrt(mu/r)`. */
export function shortToTex(src) {
  const p = new ShortParser(String(src || ''));
  const e = p.peek() ? p.relation() : { tex: '' };
  if (p.peek()) p.problems.push('did not understand from ' + (p.peek().v || ''));
  if (p.rest) p.problems.push('did not understand: ' + p.rest);
  return { tex: e.tex, problems: p.problems };
}

/**
 * The equations a pseudocode states, one per `let`, `set` and `return` line,
 * as `{ line, lhs, tex }`. `answer` names the output a `return` gives. The
 * pseudocode is read, never run.
 */
export function pseudocodeEquations(text, answer = 'answer') {
  const out = [];
  String(text || '').split('\n').forEach((raw, n) => {
    const s = raw.replace(/#.*$/, '').trim();
    let m;
    if ((m = /^(?:let|set)\s+([A-Za-z][A-Za-z0-9_]*)\s*(?::\s*[A-Za-z][A-Za-z0-9_]*)?\s*=\s*(.+)$/.exec(s))) {
      const r = shortToTex(m[2]);
      out.push({ line: n + 1, lhs: m[1], tex: idTex(m[1]) + ' = ' + r.tex, problems: r.problems });
    } else if ((m = /^const\s+([A-Za-z][A-Za-z0-9_]*)\s*=\s*(.+)$/.exec(s))) {
      const r = shortToTex(m[2]);
      out.push({ line: n + 1, lhs: m[1], tex: idTex(m[1]) + ' = ' + r.tex, problems: r.problems, constant: true });
    } else if ((m = /^return\s+(.+)$/.exec(s))) {
      const r = shortToTex(m[1]);
      out.push({ line: n + 1, lhs: answer, tex: idTex(answer) + ' = ' + r.tex, problems: r.problems, returns: true });
    }
  });
  return out;
}
