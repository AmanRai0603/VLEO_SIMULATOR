#!/usr/bin/env node
/*
  A group folder, made into the database files a group keeps on its drive.

      node tools/group_db.mjs <folder> [--out <dir>]

  writes, under <dir> (default target/groups/<group>-db):

      <group>.vgroup                 the structure: contracts, people, the group's own text
      nodes/<node>.vnode             one node file per node, issued from the release
      releases/<group>-<v>.vleo      the release, assembled from those node files — not sealed

  It runs the group application's own modules (web/js/gdb.js, gstore.js and the
  SQLite the page carries, web/vendor/sqlite), so a file made here is the file
  the page would make. Sealing is a person's act and is left to the page.
*/
import { readFileSync, writeFileSync, mkdirSync, readdirSync, statSync, copyFileSync, mkdtempSync, rmSync } from 'node:fs';
import { join, relative, dirname, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { pathToFileURL, fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const args = process.argv.slice(2);
const folderArg = args.find(a => !a.startsWith('--'));
if (!folderArg) { console.error('usage: node tools/group_db.mjs <folder> [--out <dir>]'); process.exit(2); }
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
