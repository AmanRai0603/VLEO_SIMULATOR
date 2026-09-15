/*
  The derivation, walked one line at a time.

  A derivation read all at once is a wall. Read one line at a time it is an
  argument, and the reader notices the line they do not believe — which is the
  whole point of writing it down instead of only writing the formula.

  PROGRESSIVE, NOT REQUIRED. The generated page already contains every line,
  visible, in order. This takes a page that is complete and makes it steppable;
  it never makes a complete page incomplete. So `page.html` opened straight off
  the disk with no engine and no script still carries the whole argument, and if
  this module fails to load nothing is hidden.

  It adds no content. Every line it reveals was authored on the sheet, and there
  is nothing here that can say something the sheet does not.
*/
'use strict';

import { $, $$ } from './dom.js';

export function mountTheory(host) {
  const items = $$('li', host);
  if (items.length < 2) return;   // one line is not a walk

  const bar = document.createElement('div');
  bar.className = 'derivectl';
  bar.innerHTML =
    '<button class="ctl d-step">derive it, a line at a time</button>' +
    '<button class="ctl d-back" hidden>← back</button>' +
    '<button class="ctl d-all" hidden>show the whole derivation</button>' +
    '<span class="muted d-count"></span>';
  host.insertBefore(bar, host.firstChild);

  const step = $('.d-step', bar), back = $('.d-back', bar),
        all = $('.d-all', bar), count = $('.d-count', bar);

  // -1 means "not walking": every line is shown, which is how the page arrived.
  let at = -1;

  const paint = () => {
    if (at < 0) {
      items.forEach(li => { li.hidden = false; li.classList.remove('now'); });
      step.hidden = false; back.hidden = true; all.hidden = true;
      count.textContent = items.length + ' lines, all shown';
      return;
    }
    items.forEach((li, i) => {
      li.hidden = i > at;
      li.classList.toggle('now', i === at);
    });
    step.hidden = false;
    step.textContent = at >= items.length - 1 ? 'start again' : 'next line →';
    back.hidden = at === 0;
    all.hidden = false;
    count.textContent = 'line ' + (at + 1) + ' of ' + items.length;
  };

  step.onclick = () => { at = at >= items.length - 1 ? 0 : at + 1; paint(); };
  back.onclick = () => { at = Math.max(0, at - 1); paint(); };
  all.onclick = () => { at = -1; paint(); };

  // The arrow keys, once the walk has started. Bound on the host rather than the
  // document so a reader scrolling the page with the keyboard does not
  // accidentally drive a derivation they are not looking at.
  host.tabIndex = 0;
  host.onkeydown = ev => {
    if (at < 0) return;
    if (ev.key === 'ArrowRight' || ev.key === ' ') { step.onclick(); ev.preventDefault(); }
    else if (ev.key === 'ArrowLeft') { back.onclick(); ev.preventDefault(); }
  };

  paint();
}
