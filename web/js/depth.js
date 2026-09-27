/*
  How deep the page goes: Learn · Read · Expert.

  One page serves three readers (docs/EXPLAINING.md, E9). Guidance that helps
  somebody new gets in an expert's way, so the same content is shown at three
  depths rather than written three times. LEARN keeps every step, including
  the ones that ask you to predict before you look. READ shows the answer
  first, the plain-words explanation and the real thing. EXPERT keeps the
  answer, the relation, the limits and the reference, and drops the plain
  words.

  The page marks its blocks, not this module: `d-l` is shown only to a
  learner, `d-lr` to a learner and a reader. Everything unmarked is for all
  three. The choice is remembered in this browser only.
*/
'use strict';

import { $$ } from './dom.js';

const DEPTHS = ['learn', 'read', 'expert'];
const KEY = 'vleo.depth';

export function initDepth() {
  let d = 'read';
  try { const s = localStorage.getItem(KEY); if (DEPTHS.includes(s)) d = s; } catch { /* no storage */ }
  set(d);
  $$('#depth .dp').forEach(b => b.addEventListener('click', () => set(b.dataset.depth, true)));
}

function set(d, remember) {
  document.documentElement.dataset.depth = d;
  $$('#depth .dp').forEach(b => {
    b.classList.toggle('sel', b.dataset.depth === d);
    b.setAttribute('aria-pressed', String(b.dataset.depth === d));
  });
  if (remember) { try { localStorage.setItem(KEY, d); } catch { /* no storage */ } }
}
