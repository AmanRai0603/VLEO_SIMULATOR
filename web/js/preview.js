/**
 * A PREVIEW BUILD, AND ITS AUTHOR'S APPROVAL.
 *
 * A preview is the tool built from one author's form branch, before their
 * change is approved (docs/roles/maintainer.html). It must never be mistaken for
 * a release, so every view carries a banner saying whose change it is, which
 * nodes it touches and which build it is.
 *
 * The author tries the changed nodes here and, when they are right, presses
 * Approve. That saves a small file — the branch, the commit and the build it
 * was given for, their name and what they checked — which they send back to
 * the maintainer. `xtask approve` refuses it for any other build, so an
 * approval cannot be carried to a change it was not given for. It is a record,
 * not a password: it proves which build was approved, not who pressed the
 * button, and the maintainer who received it vouches for that.
 */

import { esc } from './dom.js';
import { S, CONTRACT } from './state.js';

const slug = s => String(s || '').toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');

/** A TOML basic string. A JSON string is one, escapes included. */
const tomlString = s => JSON.stringify(String(s == null ? '' : s));

/** The approval file, as `xtask approve` reads it. */
export function approvalToml(p, by, checked, note, at) {
  return [
    '# A preview approval. Send this file back to whoever sent you the preview;',
    '# it is for this exact build and no other.',
    'kind = "vleo-approval"',
    'version = 1',
    'branch = ' + tomlString(p.branch),
    'commit = ' + tomlString(p.commit),
    'run = ' + tomlString(p.run),
    'nodes = [' + (p.nodes || []).map(tomlString).join(', ') + ']',
    'approved_by = ' + tomlString(by),
    'approved_at = ' + tomlString(at),
    'checked = ' + (checked ? 'true' : 'false'),
    'note = ' + tomlString(note),
    '',
  ].join('\n');
}

function save(name, text) {
  const a = document.createElement('a');
  a.href = URL.createObjectURL(new Blob([text], { type: 'application/toml' }));
  a.download = name;
  document.body.appendChild(a);
  a.click();
  setTimeout(() => { URL.revokeObjectURL(a.href); a.remove(); }, 1000);
}

function nodeLinks(nodes) {
  return (nodes || []).map(n =>
    '<a href="#" class="pv-node" data-node="' + esc(n) + '"><code>' + esc(n) + '</code></a>').join(', ');
}

/**
 * Say so when the engine speaks another contract than this page was built for
 * — a page from one release against an engine from another. The page carries
 * on, because most of an older contract still reads; but every answer it shows
 * is now in doubt, and a reader must know that before trusting one.
 */
export function showContract() {
  const said = S.version && S.version.contract;
  if (!S.version || said === CONTRACT || document.getElementById('contract-bar')) return;
  const bar = document.createElement('div');
  bar.id = 'contract-bar';
  bar.setAttribute('role', 'alert');
  bar.innerHTML = '<b>MISMATCH</b> — this page was built for contract ' + esc(CONTRACT) + ', and the engine ' +
    (said ? 'speaks contract ' + esc(said) : 'does not say which contract it speaks') + '. The page and the ' +
    'engine come from different releases: start the tool from one release, or reload after updating it.';
  document.body.insertBefore(bar, document.body.firstChild);
}

/**
 * Say so when the answers come from the mock engine (tools/mock_engine.py):
 * recorded examples, for working on the page without building the engine.
 * Every number on such a page is an example, and one run on another row shows
 * the recorded row's numbers — so the page must never look like the tool.
 */
export function showMock() {
  if (!S.version || S.version.endpoint !== 'mock-engine' || document.getElementById('mock-bar')) return;
  const bar = document.createElement('div');
  bar.id = 'mock-bar';
  bar.setAttribute('role', 'status');
  bar.innerHTML = '<b>MOCK ENGINE</b> — every answer on this page is a recorded example from ' +
    '<code>contract/examples</code>, not the design. For working on the page; run the tool for numbers.';
  document.body.insertBefore(bar, document.body.firstChild);
}

/** Show the banner, if this is a preview build. Does nothing for a release. */
export function showPreview() {
  const p = S.version && S.version.preview;
  if (!p || document.getElementById('preview-bar')) return;
  const bar = document.createElement('div');
  bar.id = 'preview-bar';
  bar.setAttribute('role', 'status');
  bar.innerHTML =
    '<b>PREVIEW</b> of ' + esc(p.author_name || p.author) + '’s change to ' + (nodeLinks(p.nodes) || 'the design') +
    ' · build ' + esc(p.run) + ' · <b>not a release</b> — for trying the change before it is approved. ' +
    '<button class="ctl pv-approve" type="button">Approve this preview…</button>';
  document.body.insertBefore(bar, document.body.firstChild);

  const dlg = document.createElement('dialog');
  dlg.id = 'approve-dlg';
  dlg.innerHTML =
    '<form method="dialog" class="pv-form">' +
    '<h3>Approve this preview</h3>' +
    '<p class="pv-answer">Approve only when each node below gives what you expect. You will save a small ' +
    'file for this build — send it back to whoever sent you the preview.</p>' +
    '<p><b>What this preview changes:</b> ' + (nodeLinks(p.nodes) || '—') + '</p>' +
    '<ol class="pv-steps"><li>Open each node above and read its page.</li>' +
    '<li>Run it, and move its inputs; check the answers against your own.</li>' +
    '<li>If anything is wrong, do not approve — tell the maintainer what.</li></ol>' +
    '<label>Your name <input name="by" required value="' + esc(p.author_name || p.author || '') + '"></label>' +
    '<label class="pv-check"><input type="checkbox" name="checked"> ' +
    'I opened each changed node, ran it, and it gives what I expect</label>' +
    '<label>Anything the maintainer should know <span class="muted">(optional)</span>' +
    '<textarea name="note" rows="3"></textarea></label>' +
    '<p class="pv-btns"><button class="ctl pv-save" type="button" disabled>Save the approval file</button> ' +
    '<button class="ctl" value="cancel">Not yet</button></p>' +
    '<p class="muted pv-build">branch ' + esc(p.branch) + ' · commit ' + esc(String(p.commit || '').slice(0, 10)) +
    ' · build ' + esc(p.run) + '</p>' +
    '</form>';
  document.body.appendChild(dlg);

  const f = dlg.querySelector('form');
  const ready = () => {
    f.querySelector('.pv-save').disabled = !(f.checked.checked && f.by.value.trim());
  };
  f.addEventListener('input', ready);
  f.addEventListener('change', ready);
  f.querySelector('.pv-save').onclick = () => {
    const by = f.by.value.trim();
    const text = approvalToml(p, by, f.checked.checked, f.note.value.trim(), new Date().toISOString());
    save('vleo-approval-' + slug(p.author) + '-build' + slug(p.run) + '.toml', text);
    f.querySelector('.pv-build').textContent = 'Saved. Send the file back to whoever sent you the preview.';
  };
  bar.querySelector('.pv-approve').onclick = () => dlg.showModal();
  document.addEventListener('click', e => {
    const a = e.target.closest && e.target.closest('.pv-node');
    if (!a) return;
    e.preventDefault();
    if (dlg.open) dlg.close();
    if (window.openNode) window.openNode(a.dataset.node);
  });
}
