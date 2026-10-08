/*
  The forms: every way something reaches this tool from outside it.

  There are two, and both are yours. THE INPUTS: set on the Inputs page or
  uploaded as a CSV, saved on this machine, and every run is on them.
  RESULTS: saved from a run, uploaded from somebody else, shown without
  running. A NODE — what it asks, how it computes, what it reads — is the
  design, and it does not arrive here: its owner writes it in their own file.

  This page is where to find each of them. Nothing on it changes a node.
*/
'use strict';

import { answerFirst } from './dom.js';

export function renderForms(host) {
  if (!host) return;
  host.innerHTML =
    '<div class="node-head"><h2>Forms — how anything reaches this tool</h2></div>' +
    answerFirst('Two things come in from outside: your inputs, and a result you kept. ' +
      'Neither changes the design — a node is written by its owner, in their own file.',
      ['What you run on and what you keep are yours, on this machine.'],
      'how-to') +
    '<table class="fx forms-who"><thead><tr><th>form</th><th>who fills it</th><th>what it changes</th>' +
      '<th>who applies it</th></tr></thead><tbody>' +
      '<tr><td><b>the inputs</b> — a CSV</td><td>anyone using the tool</td><td>the values every run is on</td>' +
        '<td>you, on the Inputs page — saved on this machine</td></tr>' +
      '<tr><td><b>a result</b> — a CSV, or its report</td><td>saved from a run</td><td>nothing: it is a record</td>' +
        '<td>you, on the Results page — shown without running</td></tr>' +
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
    '<p><button class="ctl xref" data-results>open the Results page</button></p></section>';
}
