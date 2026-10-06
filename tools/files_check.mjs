#!/usr/bin/env node
/*
  The design-file library in the page checks what the installed one checks.

      cargo run -p vleo-files --example a_signed_release -- <dir>
      node tools/files_check.mjs <dir>

  The example writes a programme's file and a release signed through the
  chain, and the installed library's own answer (expected.json). This reads
  the two files with the page's SQLite (web/vendor/sqlite) through the page's
  module (web/js/files.js), asks the library the page carries
  (web/files.wasm.gz), and holds the answer to the installed one, byte for
  byte. Then it changes the files the way someone might after they were
  signed, and each change must be refused.

  docs/PLAN_1_0.md, phase C: "a release signed afresh checks through the
  chain, installed and in the page".
*/
import { readFileSync, readdirSync, copyFileSync, writeFileSync, mkdtempSync, rmSync } from 'node:fs';
import { join, dirname, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { gunzipSync } from 'node:zlib';
import { pathToFileURL, fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const dir = process.argv[2];
if (!dir) { console.error('usage: node tools/files_check.mjs <dir written by the a_signed_release example>'); process.exit(2); }

// The page's globals, as the page sets them.
globalThis.window = globalThis;
if (!globalThis.location) globalThis.location = new URL('file:///');
const sqliteMod = await import(pathToFileURL(join(ROOT, 'web/vendor/sqlite/sqlite3.mjs')).href);
window.sqlite3InitModule = sqliteMod.default;
window.VLEO_SQLITE_WASM = readFileSync(join(ROOT, 'web/vendor/sqlite/sqlite3.wasm.gz')).toString('base64');

// web/js is browser modules with no package.json of its own; a copy beside
// one that says so lets Node load them unchanged.
const tmp = mkdtempSync(join(tmpdir(), 'vleo-files-check-'));
for (const f of readdirSync(join(ROOT, 'web/js'))) if (f.endsWith('.js')) copyFileSync(join(ROOT, 'web/js', f), join(tmp, f));
writeFileSync(join(tmp, 'package.json'), '{"type":"module"}\n');
const files = await import(pathToFileURL(join(tmp, 'files.js')).href);
const { sqlite } = await import(pathToFileURL(join(tmp, 'gstore.js')).href);

const lib = await files.library(gunzipSync(readFileSync(join(ROOT, 'web/files.wasm.gz'))));
const anchor = readFileSync(join(dir, 'anchor.txt'), 'utf8');
const programme = new Uint8Array(readFileSync(join(dir, 'programme.vleo')));
const release = new Uint8Array(readFileSync(join(dir, 'release.vleo')));
const expected = readFileSync(join(dir, 'expected.json'), 'utf8');

let failed = 0;
const check = (what, ok, detail) => {
  console.log((ok ? 'ok    ' : 'FAIL  ') + what + (ok ? '' : '\n      ' + detail));
  if (!ok) failed++;
};

/** The file's bytes after `sql` ran on a copy of it, through the page's SQLite. */
async function changed(bytes, sql) {
  const s3 = await sqlite();
  const db = new s3.oo1.DB();
  const p = s3.wasm.allocFromTypedArray(bytes);
  s3.capi.sqlite3_deserialize(db.pointer, 'main', p, bytes.length, bytes.length,
    s3.capi.SQLITE_DESERIALIZE_FREEONCLOSE | s3.capi.SQLITE_DESERIALIZE_RESIZEABLE);
  db.exec(sql);
  const out = s3.capi.sqlite3_js_db_export(db.pointer);
  db.close();
  return out;
}

check('the page carries format ' + lib.format + ', the format the files are in', lib.format === 2, 'format ' + lib.format);

const answer = lib.checkRelease(anchor, await files.rowsOf(programme), await files.rowsOf(release));
check('the release signed afresh checks through the chain in the page',
  answer.holds === true && answer.signed.length === 2, JSON.stringify(answer));
check('and the page answers exactly what the installed library answers',
  JSON.stringify(answer) === expected, 'page: ' + JSON.stringify(answer) + '\n      installed: ' + expected);

const tampered = lib.checkRelease(anchor, await files.rowsOf(programme),
  await files.rowsOf(await changed(release, "UPDATE text SET body = 'result = 2' WHERE scope = 'b-ap'")));
check('a method changed after the seal is refused, by the seal',
  tampered.holds === false && /^the seal:/.test(tampered.refused[0] || ''), JSON.stringify(tampered));

const stripped = lib.checkRelease(anchor, await files.rowsOf(programme),
  await files.rowsOf(await changed(release, "DELETE FROM signature WHERE scope = 'b-ap'")));
check("a node's signature taken out after the seal is refused, by the seal",
  stripped.holds === false && /^the seal:/.test(stripped.refused[0] || ''), JSON.stringify(stripped));

const other = lib.checkRelease('0'.repeat(64), await files.rowsOf(programme), await files.rowsOf(release));
check('a programme file not signed with the anchored key is refused',
  other.kind === 'Signature' && /START HERE/.test(other.error || ''), JSON.stringify(other));

const notOurs = lib.checkRelease(anchor, await files.rowsOf(programme),
  await files.rowsOf(await changed(release, "DELETE FROM meta WHERE key = 'file_kind'")));
check('a file that does not say what it is is refused by name',
  /does not say what kind of file it is/.test(notOurs.error || ''), JSON.stringify(notOurs));

// What a release holds, checked in the page as intake checks it: Solar 1.0,
// which intake refused for six nodes with no case their method must refuse,
// and Solar 1.1, which it took — each the installed library's answer, byte
// for byte.
for (const v of ['1.0', '1.1']) {
  const found = lib.checkContent(await files.rowsOf(new Uint8Array(readFileSync(join(dir, 'l3_solar-' + v + '.vleo')))));
  const want = readFileSync(join(dir, 'expected-' + v + '.json'), 'utf8');
  const errors = (found.findings || []).filter(f => f.level === 'error');
  if (v === '1.0') {
    check('Solar 1.0, which intake refused, is refused in the page for the six nodes intake named',
      found.holds === false && new Set(errors.map(e => e.place)).size === 6 && errors.every(e => /no case the node must refuse/.test(e.what)), JSON.stringify(errors));
  } else {
    check('Solar 1.1, which intake took, holds in the page', found.holds === true && errors.length === 0, JSON.stringify(errors));
  }
  check('and the page finds in Solar ' + v + ' exactly what the installed library finds', JSON.stringify(found) === want,
    'page: ' + JSON.stringify(found).slice(0, 300) + '\n      installed: ' + want.slice(0, 300));
}

// Solar 1.0 against 1.1, block by block, compared in the page: the installed
// library's answer, byte for byte, and the seven nodes that changed.
{
  const read = v => files.rowsOf(new Uint8Array(readFileSync(join(dir, 'l3_solar-' + v + '.vleo'))));
  const found = lib.compare(await read('1.0'), await read('1.1'));
  const want = readFileSync(join(dir, 'expected-compare.json'), 'utf8');
  check('Solar 1.0 against 1.1, compared in the page: ' + found.summary, !found.error &&
    (found.blocks || []).map(b => b.id).join(' ') === 'sw_activity_band sw_ap_daily_band_drop sw_ap_design sw_exceedance_rate sw_horizon_persistence sw_kp_scenarios sw_regime',
    JSON.stringify(found).slice(0, 300));
  check('and the page compares them exactly as the installed library does', JSON.stringify(found) === want,
    'page: ' + JSON.stringify(found).slice(0, 300) + '\n      installed: ' + want.slice(0, 300));
  const self = lib.compare(await read('1.1'), await read('1.1'));
  check('and a file against itself is the same', !self.error && self.blocks.length === 0 && self.file.length === 0, self.summary);
}

// The group folder's checks, said by the library, against the page's own
// checker (web/js/gcheck.js) on the same folders, with the pattern the page
// carries (the spec inlined in web/group.html): the same findings, at the
// same level, place, words and line. The maths reader (texmath.js) stays in
// the page for now, and its warnings are left out of the comparison.
window.VLEO_GROUP_SCHEMA = readFileSync(join(ROOT, 'groups/schema.sql'), 'utf8');
window.VLEO_METHOD_WASM = readFileSync(join(ROOT, 'web/method.wasm.gz')).toString('base64');
const groupHtml = readFileSync(join(ROOT, 'web/group.html'), 'utf8');
const specAt = groupHtml.indexOf('window.VLEO_GROUP_SPEC = ');
const spec = JSON.parse(groupHtml.slice(specAt + 'window.VLEO_GROUP_SPEC = '.length, groupHtml.indexOf('\n', specAt)).replace(/;\s*$/, ''));
const gdb = await import(pathToFileURL(join(tmp, 'gdb.js')).href);
const gstore = await import(pathToFileURL(join(tmp, 'gstore.js')).href);
const { loadGroup } = await import(pathToFileURL(join(tmp, 'gmodel.js')).href);
const { checkGroup } = await import(pathToFileURL(join(tmp, 'gcheck.js')).href);
const maths = f => /^maths: /.test(f.msg) || /^shown as an equation with a gap: /.test(f.msg) || (f.where === 'equations.csv' && f.level === 'warning');
const key = f => f.level + ' | ' + f.where + ' | ' + f.msg + ' | ' + (f.line || 0);
async function parity(what, bytes) {
  const model = await loadGroup(gdb.folderFromDb(await gstore.open(bytes)));
  const page = (await checkGroup(model, spec)).filter(f => !maths(f)).map(key).sort();
  const answer = lib.checkFolder(await files.rowsOf(bytes));
  const mine = (answer.findings || []).map(key).sort();
  const only = (a, b) => a.filter(x => !b.includes(x));
  const same = JSON.stringify(page) === JSON.stringify(mine);
  check(what + ': the library finds what the page checker finds — ' + page.length + ' finding(s), ' +
    page.filter(k => k.startsWith('error')).length + ' error(s)', same && !answer.error,
    (answer.error ? 'error: ' + answer.error + '\n      ' : '') +
    'only the page: ' + JSON.stringify(only(page, mine).slice(0, 8)) + '\n      only the library: ' + JSON.stringify(only(mine, page).slice(0, 8)));
  return page;
}
const solar10 = new Uint8Array(readFileSync(join(dir, 'l3_solar-1.0.vleo')));
const solar11 = new Uint8Array(readFileSync(join(dir, 'l3_solar-1.1.vleo')));
await parity('Solar 1.0', solar10);
const clean = await parity('Solar 1.1', solar11);
check('Solar 1.1, sealed, has no error by either', !clean.some(k => k.startsWith('error')), JSON.stringify(clean.filter(k => k.startsWith('error'))));
// A copy of 1.1 broken in many places at once: every check that finds
// something must find the same thing in both.
const broken = await changed(solar11, `
  UPDATE doc SET body = body || '\n{{eq NOPE}} {{bogus x}} {{node nope}} {{guess q}} {{fig nofig}}\n' WHERE scope = 'sw_regime' AND kind = 'explanation';
  UPDATE doc SET body = replace(body, '## Validity', '## Validity, later') WHERE scope = 'sw_regime' AND kind = 'theory';
  UPDATE doc SET body = body || '\n$$x$$\n' WHERE scope = 'sw_regime' AND kind = 'explanation';
  DELETE FROM doc WHERE scope = 'sw_ap_design' AND kind = 'theory';
  UPDATE doc SET body = body || '\nnot a step\nsw_regime <- nowhere_node, case\n' WHERE scope = 'group' AND kind = 'flow';
  UPDATE node SET question = 'no question here' WHERE uid = 'sw_regime';
  UPDATE node SET lower = '5', upper = '1' WHERE uid = 'sw_band_confidence';
  UPDATE input SET dflt = '', source = 'Nowhere Else' WHERE node_uid = 'sw_regime';
  UPDATE tbl SET csv = csv || 'a,b\n' WHERE scope = 'sw_regime' AND path = 'results/isolation.csv';
  UPDATE tbl SET csv = replace(csv, 'Aman Rai,transcribed', 'Aman Rai,transcribed-ish') WHERE scope = 'sw_regime' AND path = 'declaration.csv';
  UPDATE tbl SET csv = replace(csv, ',<=,', ',=,') WHERE scope = 'group' AND path = 'requirements.csv';
  UPDATE member SET role = 'author';
  INSERT INTO media (scope, path, type, sha256, bytes) VALUES ('group', 'notes.xlsx', 'application/octet-stream', '', x'00');
  INSERT INTO tbl (scope, path, csv) VALUES ('group', 'loops.csv', 'nodes,converge_on,tolerance,max_iter,seed_node,seed_value,source\nsw_regime ghost,sw_regime,1e-6,10,sw_regime,1,s\n');
`);
const found = await parity('Solar 1.1, broken in many places', broken);
check('and the broken copy is found broken — at least 15 errors', found.filter(k => k.startsWith('error')).length >= 15,
  found.filter(k => k.startsWith('error')).length + ' errors');

// The seal, said by the library, against the page's own seal rules
// (web/js/gseal.js: reviewState, sealBlockers) on the same folders: the same
// fingerprint for the group and every node, every sign-off current or stale
// alike, and the same things in the way of a seal, in the same words.
const gseal = await import(pathToFileURL(join(tmp, 'gseal.js')).href);
async function sealParity(what, bytes) {
  const folder = gdb.folderFromDb(await gstore.open(bytes));
  const model = await loadGroup(folder);
  const { reviews, fingerprintOf } = await gseal.reviewState(folder, model);
  const scopes = [];
  for (const s of ['group'].concat(model.order)) scopes.push({ scope: s, fingerprint: await fingerprintOf(s) });
  const page = {
    scopes,
    reviews: reviews.map(r => [r.name, r.scope, r.fingerprint, r.verdict, r.current]),
    blockers: await gseal.sealBlockers(folder, model, await checkGroup(model, spec)),
  };
  const answer = lib.sealState(await files.rowsOf(bytes));
  const mine = answer.error ? answer : {
    scopes: answer.scopes,
    reviews: answer.reviews.map(r => [r.name, r.scope, r.fingerprint, r.verdict, r.current]),
    blockers: answer.blockers,
  };
  check(what + ': the library seals as the page seals — ' + page.reviews.filter(r => !r[4]).length + ' stale sign-off(s), ' +
    page.blockers.length + ' in the way', JSON.stringify(page) === JSON.stringify(mine),
    'page: ' + JSON.stringify(page.blockers).slice(0, 300) + '\n      library: ' + JSON.stringify(mine.blockers || mine).slice(0, 300));
  return page;
}
for (const [v, bytes] of [['1.0', solar10], ['1.1', solar11]]) {
  const page = await sealParity('Solar ' + v, bytes);
  const sealed = (await gstore.open(bytes)).meta('fingerprint');
  check('Solar ' + v + ', as sealed: nothing in the way, and the fingerprint the seal recorded',
    !page.blockers.length && page.scopes[0].fingerprint === sealed, JSON.stringify(page.blockers) + ' ' + page.scopes[0].fingerprint + ' / ' + sealed);
}
const stale = await sealParity('Solar 1.1, one node changed after signing', await changed(solar11,
  "UPDATE doc SET body = body || ' ' WHERE scope = 'sw_regime' AND kind = 'explanation';"));
check('and that node and the group wait for a new sign-off', stale.blockers.join(' | ') ===
  'sw_regime has no current sign-off from its node engineer | the group has no current sign-off from its owner', JSON.stringify(stale.blockers));
// A reviewer, not the owner, gives the group a current ok: it is current, and
// it does not count, because only the owner signs the whole group.
const withReviewer = await changed(solar11, "INSERT INTO member (name, role) VALUES ('Somebody Else', 'reviewer');");
const groupNow = (await gseal.fingerprint(gdb.folderFromDb(await gstore.open(withReviewer)), 'group')).fingerprint;
const notOwner = await sealParity('Solar 1.1, the group signed by a reviewer', await changed(withReviewer,
  `UPDATE review SET name = 'Somebody Else', fingerprint = '${groupNow}' WHERE scope = 'group';`));
check('and the reviewer\'s ok, current, does not seal the group', notOwner.reviews.some(r => r[1] === 'group' && r[4]) &&
  notOwner.blockers.join(' | ') === 'the group has no current sign-off from its owner', JSON.stringify(notOwner.blockers));
const unowned = await sealParity('Solar 1.1, broken in many places', broken);
check('and nothing in it may be sealed', unowned.blockers.length === 1 + unowned.scopes.length, unowned.blockers.length + ' in the way');

rmSync(tmp, { recursive: true, force: true });
console.log(failed ? failed + ' check(s) failed' : 'every check holds');
process.exit(failed ? 1 : 0);
