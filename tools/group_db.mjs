#!/usr/bin/env node
/*
  A group folder, made into the database files a group keeps on its drive.

      node tools/group_db.mjs <folder> [--out <dir>]
      node tools/group_db.mjs --unpack <file.vleo> [--out <dir>]

  The first writes, under <dir> (default target/groups/<group>-db):

      <group>.vgroup                 the structure: contracts, people, the group's own text
      nodes/<node>.vnode             one node file per node, issued from the release
      releases/<group>-<v>.vleo      the release, assembled from those node files — not sealed

  It runs the group application's own modules (web/js/gdb.js, gstore.js and the
  SQLite the page carries, web/vendor/sqlite), so a file made here is the file
  the page would make. Sealing is a person's act and is left to the page.

  --unpack is the developer's way in: a release written out as the folder it
  holds (default target/groups/<group>-<v>), with RELEASE.toml beside it saying
  what the file said of itself — sealed, by whom, and the fingerprint every
  sign-off was given for. `cargo run -p xtask -- group-intake <dir>` reads it.
*/
import { readFileSync, writeFileSync, mkdirSync, readdirSync, statSync, copyFileSync, mkdtempSync, rmSync } from 'node:fs';
import { join, relative, dirname, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { pathToFileURL, fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const args = process.argv.slice(2);
const folderArg = args.find(a => !a.startsWith('--'));
if (!folderArg) { console.error('usage: node tools/group_db.mjs <folder> [--out <dir>] | --unpack <file.vleo> [--out <dir>]'); process.exit(2); }
const unpack = args.includes('--unpack');
const outAt = args.indexOf('--out');

// The page's globals, as the page sets them.
globalThis.window = globalThis;
// The engine probes for the browser's private file storage, which a file
// opened in Node has not got; with an address to read it says so quietly.
if (!globalThis.location) globalThis.location = new URL('file:///');
const sqlite = await import(pathToFileURL(join(ROOT, 'web/vendor/sqlite/sqlite3.mjs')).href);
window.sqlite3InitModule = sqlite.default;
window.VLEO_SQLITE_WASM = readFileSync(join(ROOT, 'web/vendor/sqlite/sqlite3.wasm.gz')).toString('base64');
window.VLEO_GROUP_SCHEMA = readFileSync(join(ROOT, 'groups/schema.sql'), 'utf8');

// web/js is written as browser modules with no package.json of its own; a
// copy beside one that says so lets Node load them unchanged.
const tmp = mkdtempSync(join(tmpdir(), 'vleo-group-db-'));
for (const f of readdirSync(join(ROOT, 'web/js'))) if (f.endsWith('.js')) copyFileSync(join(ROOT, 'web/js', f), join(tmp, f));
writeFileSync(join(tmp, 'package.json'), '{"type":"module"}\n');
const gdb = await import(pathToFileURL(join(tmp, 'gdb.js')).href);
const { Folder } = await import(pathToFileURL(join(tmp, 'gfolder.js')).href);

if (unpack) {
  const { open } = await import(pathToFileURL(join(tmp, 'gstore.js')).href);
  const db = await open(new Uint8Array(readFileSync(resolve(folderArg))));
  const kind = db.meta('file_kind');
  if (kind !== 'release') { console.error(folderArg + ' is a ' + (kind || 'database') + ' file, not a release'); process.exit(1); }
  const folder = gdb.folderFromDb(db);
  const id = db.meta('group_id'), version = db.meta('version');
  const out = outAt >= 0 ? resolve(args[outAt + 1]) : join(ROOT, 'target/groups', id + '-' + version);
  rmSync(out, { recursive: true, force: true });
  for (const p of folder.list()) {
    mkdirSync(dirname(join(out, p)), { recursive: true });
    writeFileSync(join(out, p), await folder.bytes(p));
  }
  const q = v => '"' + String(v ?? '').replace(/\\/g, '\\\\').replace(/"/g, '\\"') + '"';
  const keys = ['group_id', 'group_name', 'version', 'file_kind', 'sealed', 'sealed_by', 'fingerprint', 'spec', 'format'];
  const nodes = db.all('SELECT id, revision, contract_version, author FROM node WHERE archived = 0 ORDER BY ord, id');
  writeFileSync(join(out, 'RELEASE.toml'), '# What ' + folderArg.split(/[\\/]/).pop() + ' said of itself. Written by tools/group_db.mjs --unpack.\n' +
    keys.map(k => k + ' = ' + q(db.meta(k))).join('\n') + '\n' +
    nodes.map(n => '\n[[node]]\nid = ' + q(n.id) + '\nrevision = ' + Number(n.revision) + '\ncontract_version = ' + Number(n.contract_version) + '\nauthor = ' + q(n.author)).join('\n') + '\n');
  rmSync(tmp, { recursive: true, force: true });
  console.log('wrote ' + out + ' — ' + folder.list().length + ' files, and RELEASE.toml (' + (db.meta('sealed') ? 'sealed ' + db.meta('sealed').slice(0, 10) + ' by ' + db.meta('sealed_by') : 'NOT sealed') + ')');
  process.exit(0);
}

const dir = resolve(folderArg);
const files = new Map();
const walk = d => {
  for (const f of readdirSync(d).sort()) {
    const p = join(d, f);
    if (statSync(p).isDirectory()) walk(p);
    else files.set(relative(dir, p).split('\\').join('/'), new File([readFileSync(p)], f));
  }
};
walk(dir);
const folder = new Folder(dir.split(/[\\/]/).pop(), files);

const release = await gdb.dbFromFolder(folder, 'release');
const id = release.meta('group_id') || folder.name, version = release.meta('version') || '0';
const out = outAt >= 0 ? resolve(args[outAt + 1]) : join(ROOT, 'target/groups', id + '-db');
mkdirSync(join(out, 'nodes'), { recursive: true });
mkdirSync(join(out, 'releases'), { recursive: true });
const save = (path, db) => { writeFileSync(join(out, path), db.bytes()); return path; };

const structure = await gdb.structureOf(release);
const nodes = release.all('SELECT uid, id, contract_version FROM node WHERE archived = 0 ORDER BY ord, id');
const issued = [];
for (const n of nodes) {
  const nf = await gdb.issueNode(release, n.uid);
  nf.setMeta('issued_from', id + '-' + version + ' (from the folder ' + folder.name + ')');
  save('nodes/' + n.id + '.vnode', nf);
  issued.push(nf);
  structure.setMeta('issued:' + n.uid, String(n.contract_version));
}
gdb.logChange(structure, 'tools/group_db.mjs', 'group', 'made from the folder ' + folder.name, null, null);
save(id + '.vgroup', structure);
const { db: assembled, notes } = await gdb.assemble(structure, issued);
save('releases/' + id + '-' + version + '.vleo', assembled);
rmSync(tmp, { recursive: true, force: true });

console.log('wrote ' + out);
console.log('  ' + id + '.vgroup — the structure, ' + nodes.length + ' nodes');
console.log('  nodes/ — ' + issued.length + ' node files');
console.log('  releases/' + id + '-' + version + '.vleo — assembled, not sealed');
for (const n of notes.filter(x => x.level !== 'note')) console.log('  ' + n.level + ': ' + n.msg);
