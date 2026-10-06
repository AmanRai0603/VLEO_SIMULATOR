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

rmSync(tmp, { recursive: true, force: true });
console.log(failed ? failed + ' check(s) failed' : 'every check holds');
process.exit(failed ? 1 : 0);
