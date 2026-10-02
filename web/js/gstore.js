/*
  The group's database files, opened, made and saved in the browser.

  SQLite itself runs in the page (web/vendor/sqlite, inlined by `xtask
  group-app`), so a database file on the group's shared drive is opened with no
  server and no network. A file is read whole into memory, worked on there, and
  written back whole — which needs nothing a page opened from a file lacks, and
  means a file on the drive is never left half-written.

  Every file carries the schema's identity (application_id 'VLEO') and its
  format number. A file from a newer application is refused by name rather
  than misread; a file that is not ours at all is refused before anything is
  taken from it.
*/
'use strict';

const APP_ID = 1447838031;
let ready = null;

/** SQLite, started once: the module the page carries, given the wasm it carries. */
export function sqlite() {
  if (ready) return ready;
  ready = (async () => {
    const init = await waitFor(() => window.sqlite3InitModule, 'the database engine did not load');
    const gz = Uint8Array.from(atob(window.VLEO_SQLITE_WASM || ''), c => c.charCodeAt(0));
    const wasm = new Uint8Array(await new Response(new Blob([gz]).stream().pipeThrough(new DecompressionStream('gzip'))).arrayBuffer());
    return init({ wasmBinary: wasm, print: () => {}, printErr: () => {} });
  })();
  return ready;
}

async function waitFor(get, why) {
  for (let i = 0; i < 400; i++) {
    const v = get();
    if (v) return v;
    await new Promise(r => setTimeout(r, 25));
  }
  throw new Error(why);
}

/** The schema's format number, read from groups/schema.sql as the page carries it. */
export function schemaVersion() {
  const m = /PRAGMA user_version\s*=\s*(\d+)/.exec(window.VLEO_GROUP_SCHEMA || '');
  return m ? Number(m[1]) : 0;
}

/** One open database, with the few operations everything else uses. */
export class Db {
  constructor(s3, db) { this.s3 = s3; this.db = db; }
  // SQLite refuses a binding offered to a statement that takes none, so an
  // empty list is never passed on.
  exec(sql, bind) { this.db.exec(bind && bind.length ? { sql, bind } : sql); return this; }
  all(sql, bind) {
    const rows = [];
    this.db.exec({ sql, ...(bind && bind.length ? { bind } : {}), rowMode: 'object', callback: r => { rows.push({ ...r }); } });
    return rows;
  }
  one(sql, bind) { return this.all(sql, bind)[0] || null; }
  value(sql, bind) { return bind && bind.length ? this.db.selectValue(sql, bind) : this.db.selectValue(sql); }
  meta(key) { const r = this.one('SELECT value FROM meta WHERE key = ?', [key]); return r ? r.value : ''; }
  setMeta(key, value) { this.exec('INSERT OR REPLACE INTO meta (key, value) VALUES (?, ?)', [key, String(value)]); }
  /** Everything at once, or nothing. */
  tx(fn) {
    this.exec('BEGIN');
    try { const r = fn(); this.exec('COMMIT'); return r; } catch (e) { this.exec('ROLLBACK'); throw e; }
  }
  /** The file's bytes: an ordinary SQLite database. */
  bytes() { return this.s3.capi.sqlite3_js_db_export(this.db.pointer); }
  close() { this.db.close(); }
}

/** A new, empty database of one kind, in the current format. */
export async function create(kind) {
  const s3 = await sqlite();
  const db = new Db(s3, new s3.oo1.DB(':memory:', 'c'));
  db.exec(window.VLEO_GROUP_SCHEMA);
  db.setMeta('file_kind', kind);
  db.setMeta('format', schemaVersion());
  return db;
}

/** Open a database from its bytes. Refuses what is not ours, or newer than this page. */
export async function open(bytes) {
  const s3 = await sqlite();
  const head = String.fromCharCode(...bytes.slice(0, 15));
  if (head !== 'SQLite format 3') throw new Error('this is not a database file');
  const raw = new s3.oo1.DB();
  const p = s3.wasm.allocFromTypedArray(bytes);
  const rc = s3.capi.sqlite3_deserialize(raw.pointer, 'main', p, bytes.length, bytes.length,
    s3.capi.SQLITE_DESERIALIZE_FREEONCLOSE | s3.capi.SQLITE_DESERIALIZE_RESIZEABLE);
  if (rc !== 0) { raw.close(); throw new Error('the database could not be read (SQLite code ' + rc + ')'); }
  const db = new Db(s3, raw);
  const app = Number(db.value('PRAGMA application_id'));
  const fmt = Number(db.value('PRAGMA user_version'));
  if (app !== APP_ID) { db.close(); throw new Error('this database was not made by a VLEO group application'); }
  if (fmt > schemaVersion()) {
    db.close();
    throw new Error('this file is format ' + fmt + ', newer than this application understands (' + schemaVersion() + '): use the newer application');
  }
  return db;
}

/** A copy of a database, to change without touching the one it came from. */
export async function copy(db) { return open(db.bytes()); }
