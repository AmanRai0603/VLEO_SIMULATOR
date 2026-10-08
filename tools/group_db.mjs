#!/usr/bin/env node
/*
  A group folder, made into the database files a group keeps on its drive.

      node tools/group_db.mjs <folder> [--out <dir>]
      node tools/group_db.mjs --unpack <file.vleo> [--out <dir>]
      node tools/group_db.mjs --all <dir> [--out <dir>]

  The first writes, under <dir> (default target/groups/<group>-db):

      <group>.vgroup                 the structure: contracts, people, the group's own text
      nodes/<node>.vnode             one node file per node, issued from the release
      releases/<group>-<v>.vleo      the release, assembled from those node files — not sealed

  It runs the group application's own modules (web/js/gdb.js, gstore.js and the
  SQLite the page carries, web/vendor/sqlite), so a file made here is the file
  the page would make. Sealing is a person's act and is left to the page.

  --unpack writes a release out as the folder it holds (default
  target/groups/<group>-<v>), with RELEASE.toml beside it saying what the file
  said of itself — sealed, by whom, and the fingerprint every sign-off was
  given for.
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

/** One folder of the pattern, made into its structure, node files and release under `out`. */
async function make(dir, out) {
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
  out = out || join(ROOT, 'target/groups', id + '-db');
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
  return { id, version, out, nodes: nodes.length, issued: issued.length, notes, folder };
}

if (args.includes('--all')) {
  // Every group folder under <dir> (as `xtask group-export --all` writes them),
  // each made into its files under <out>/<group>/, and READY.csv: what each
  // group's own checks still ask of it — the page's checks, run here.
  // The pattern exactly as the page carries it: read from the built page.
  const page = readFileSync(join(ROOT, 'web/group.html'), 'utf8');
  const at = page.indexOf('window.VLEO_GROUP_SPEC = ');
  const spec = page.slice(at + 'window.VLEO_GROUP_SPEC = '.length, page.indexOf(';\n', at));
  globalThis.VLEO_GROUP_SPEC = window.VLEO_GROUP_SPEC = JSON.parse(spec);
  const { loadGroup } = await import(pathToFileURL(join(tmp, 'gmodel.js')).href);
  const { checkGroup } = await import(pathToFileURL(join(tmp, 'gcheck.js')).href);
  const base = resolve(folderArg);
  const outAll = outAt >= 0 ? resolve(args[outAt + 1]) : base + '-db';
  // What each group must still give, by kind, from the page's own findings.
  const KINDS = [
    ['empty_sections', f => f.msg.startsWith('says nothing under')],
    ['pseudocode_to_write', f => f.where.endsWith('pseudocode.txt') && f.msg.startsWith('is missing')],
    ['results_to_supply', f => f.where.endsWith('results/isolation.csv') && f.msg.startsWith('is missing')],
    ['values_to_decide', f => f.msg.endsWith('is declared, so it needs a value')],
    ['defaults_to_give', f => f.msg.endsWith('has no default value')],
  ];
  const rows = [['group', 'nodes', 'node_files', 'errors', 'warnings', ...KINDS.map(k => k[0]), 'other']];
  let failed = 0;
  for (const g of readdirSync(base).sort()) {
    const d = join(base, g);
    if (!statSync(d).isDirectory() || !statSync(join(d, 'group.csv'), { throwIfNoEntry: false })) continue;
    try {
      const r = await make(d, join(outAll, g));
      const findings = await checkGroup(await loadGroup(r.folder), window.VLEO_GROUP_SPEC);
      const errors = findings.filter(f => f.level === 'error');
      const by = KINDS.map(([, test]) => errors.filter(test).length);
      const other = errors.filter(f => !KINDS.some(([, test]) => test(f))).length;
      rows.push([r.id, r.nodes, r.issued, errors.length, findings.filter(f => f.level === 'warning').length, ...by, other]);
    } catch (e) {
      failed++;
      rows.push([g, 'FAILED: ' + String(e.message || e).slice(0, 120)]);
    }
  }
  const cell = v => /[",\n]/.test(String(v)) ? '"' + String(v).replace(/"/g, '""') + '"' : String(v);
  mkdirSync(outAll, { recursive: true });
  writeFileSync(join(outAll, 'READY.csv'), rows.map(r => r.map(cell).join(',')).join('\n') + '\n');
  rmSync(tmp, { recursive: true, force: true });
  console.log('wrote ' + (rows.length - 1 - failed) + ' groups\' database files under ' + outAll + ', and READY.csv' + (failed ? ' — ' + failed + ' FAILED' : ''));
  process.exit(failed ? 1 : 0);
}

const r = await make(resolve(folderArg), outAt >= 0 ? resolve(args[outAt + 1]) : null);
rmSync(tmp, { recursive: true, force: true });

console.log('wrote ' + r.out);
console.log('  ' + r.id + '.vgroup — the structure, ' + r.nodes + ' nodes');
console.log('  nodes/ — ' + r.issued + ' node files');
console.log('  releases/' + r.id + '-' + r.version + '.vleo — assembled, not sealed');
for (const n of r.notes.filter(x => x.level !== 'note')) console.log('  ' + n.level + ': ' + n.msg);
