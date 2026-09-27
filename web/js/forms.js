/*
  The forms: every way something reaches this tool from outside it.

  There are three, and each has one owner. THE INPUTS are yours: set on the
  Inputs page or uploaded as a CSV, saved on this machine, and every run is on
  them. RESULTS are yours: saved from a run, uploaded from somebody else, shown
  without running. A NODE — what it asks, how it computes, what it reads — is
  the design, and it changes only through its form: filled by whoever knows
  the answer, checked and applied by the developers, released, and then run
  here with your own inputs.

  This page is where to find each of them. Nothing on it changes a node.
*/
'use strict';

import { $, esc } from './dom.js';
import { S } from './state.js';
import { checkForm } from './nodeform.js';

export function renderForms(host) {
  if (!host) return;
  const rows = S.rows.filter(r => r.state !== 'deprecated');
  host.innerHTML =
    '<div class="node-head"><h2>Forms — how anything reaches this tool</h2>' +
    '<p class="ident">Three kinds of thing come in from outside, and each has one owner. What you run on ' +
    'and what you keep are yours. What the design IS changes only through a node form, applied by the ' +
    'developers — so everyone can see what changed, and why, in the next release.</p></div>' +
    '<table class="fx forms-who"><thead><tr><th>form</th><th>who fills it</th><th>what it changes</th>' +
      '<th>who applies it</th></tr></thead><tbody>' +
      '<tr><td><b>the inputs</b> — a CSV</td><td>anyone using the tool</td><td>the values every run is on</td>' +
        '<td>you, on the Inputs page — saved on this machine</td></tr>' +
      '<tr><td><b>a result</b> — a CSV, or its report</td><td>saved from a run</td><td>nothing: it is a record</td>' +
        '<td>you, on the Results page — shown without running</td></tr>' +
      '<tr><td><b>a node form</b> — one HTML file</td><td>whoever knows what the node should say</td>' +
        '<td>the design: one node, or a new one</td><td>the developers, from a terminal — then a release</td></tr>' +
    '</tbody></table>' +

    '<section class="forms-sec"><h3>1 · The inputs</h3>' +
    '<p>Every input of the design, in two halves — what the customer chooses and the condition it flies in — ' +
      'with its default and its range. Download the template, fill it anywhere, upload it; or change values ' +
      'one by one.</p>' +
    '<p><button class="ctl xref" data-inputs>open the Inputs page</button> ' +
      '<a class="ctl" href="/v1/inputs.csv?inputs=defaults" download="vleo-case-template.csv">download the blank inputs template</a></p></section>' +

    '<section class="forms-sec"><h3>2 · Results</h3>' +
    '<p>What a run returned and the inputs it ran on, kept as a CSV — or as a report page to send to ' +
      'somebody who does not run the tool. Upload one to see it again; nothing runs.</p>' +
    '<p><button class="ctl xref" data-results>open the Results page</button></p></section>' +

    '<section class="forms-sec"><h3>3 · A node form — change a node, or ask for a new one</h3>' +
    '<p>One self-contained HTML file per node. It explains itself, asks every question the node answers — ' +
      'what it asks, its relation and where it comes from, its bounds and why, the steps, what it reads — ' +
      'lists every row it could read, and saves a filled copy of itself. Fill it in a browser, or give it to ' +
      'an assistant: the content is a plain block of text near the end. Send the saved file to the ' +
      'developers.</p>' +
    '<div class="forms-pick"><label>a node that exists <input class="ctl forms-node" list="forms-rows" ' +
      'placeholder="type a row id or name" aria-label="node for its form"></label>' +
      '<a class="ctl forms-dl" download>download its form</a><span class="muted forms-hint"></span></div>' +
    '<datalist id="forms-rows">' + rows.map(r => '<option value="' + esc(r.id) + '">' + esc(r.label) +
      '</option>').join('') + '</datalist>' +
    '<p><a class="ctl forms-new" href="/v1/form/new" download="new-node.node-form.html">download the form for a new node</a> ' +
      '<span class="muted">it also asks where the node goes in the tree and what kind of row it is</span></p>' +
    '<p><label class="ctl forms-up-l">check a filled form…<input type="file" class="forms-up" ' +
      'accept=".html,text/html" hidden></label> <span class="muted">shows what it would change, and every ' +
      'interface it declares. Nothing is written here.</span></p>' +
    '<div class="forms-out"></div></section>';

  const pick = $('.forms-node', host), dl = $('.forms-dl', host), hint = $('.forms-hint', host);
  const paint = () => {
    const r = S.byId.get(pick.value.trim());
    dl.hidden = !r;
    if (r) {
      dl.href = '/v1/form/' + encodeURIComponent(r.id);
      dl.download = r.id + '.node-form.html';
    }
    hint.textContent = !pick.value.trim() ? '' : r ? r.label + ' — ' + r.kind + ', ' + r.state
      : 'no row has that id — for a node the design does not have yet, use the form for a new node';
  };
  pick.oninput = paint;
  paint();
  $('.forms-up', host).onchange = async e => {
    const f = e.target.files && e.target.files[0];
    e.target.value = '';
    if (f) await checkForm($('.forms-out', host), f.name, await f.text());
  };
}
