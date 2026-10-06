/*
  The design-file library, in the page.

  Every check a design file must pass is written once, in crates/vleo-files,
  and the page runs that same code compiled to WebAssembly (web/files.wasm.gz,
  built by `cargo run -p xtask -- files-wasm`). The page does only what the
  library cannot do there: read the file with its own SQLite and hand over
  the rows. What the rows mean, and whether a signature counts, is the
  library's to say, so the page and the installed application refuse the same
  files for the same reasons (docs/OPERATING_1_0.md, section 14).

  A file's rows travel in the form crates/vleo-files/src/rows.rs reads:
  "VLEOROWS1", then each table's name, columns and cells, every length and
  number little-endian.
*/
'use strict';

import { sqlite } from './gstore.js';

const enc = new TextEncoder();
const dec = new TextDecoder();

/** Bytes, grown as they are written. */
class Out {
  constructor() { this.buf = new Uint8Array(1 << 16); this.n = 0; }
  room(k) {
    if (this.n + k <= this.buf.length) return;
    let size = this.buf.length * 2;
    while (size < this.n + k) size *= 2;
    const b = new Uint8Array(size); b.set(this.buf.subarray(0, this.n)); this.buf = b;
  }
  u8(v) { this.room(1); this.buf[this.n++] = v; }
  u32(v) { this.room(4); new DataView(this.buf.buffer).setUint32(this.n, v, true); this.n += 4; }
  i64(v) { this.room(8); new DataView(this.buf.buffer).setBigInt64(this.n, BigInt(v), true); this.n += 8; }
  raw(b) { this.room(b.length); this.buf.set(b, this.n); this.n += b.length; }
  bytes(b) { this.u32(b.length); this.raw(b); }
  str(s) { this.bytes(enc.encode(String(s))); }
  done() { return this.buf.slice(0, this.n); }
}

/**
 * A file's rows, read from its bytes with the page's SQLite. Nothing about
 * the file is judged here — not its kind, not its format: the library does
 * that, the same way it does installed.
 */
export async function rowsOf(bytes) {
  const s3 = await sqlite();
  const head = String.fromCharCode(...bytes.slice(0, 15));
  if (head !== 'SQLite format 3') throw new Error('this is not a database file');
  const db = new s3.oo1.DB();
  const p = s3.wasm.allocFromTypedArray(bytes);
  const rc = s3.capi.sqlite3_deserialize(db.pointer, 'main', p, bytes.length, bytes.length,
    s3.capi.SQLITE_DESERIALIZE_FREEONCLOSE | s3.capi.SQLITE_DESERIALIZE_RESIZEABLE);
  if (rc !== 0) { db.close(); throw new Error('the database could not be read (SQLite code ' + rc + ')'); }
  try {
    const out = new Out();
    out.raw(enc.encode('VLEOROWS1'));
    const names = db.selectValues("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name");
    out.u32(names.length);
    for (const name of names) {
      const cols = db.selectValues('SELECT name FROM pragma_table_info(?) ORDER BY cid', [name]);
      out.str(name);
      out.u32(cols.length);
      for (const c of cols) out.str(c);
      const rows = [];
      db.exec({ sql: 'SELECT * FROM "' + name.replace(/"/g, '""') + '" ORDER BY rowid', rowMode: 'array', callback: r => { rows.push(r.slice()); } });
      out.u32(rows.length);
      for (const r of rows) {
        for (const v of r) {
          if (v === null || v === undefined) out.u8(0);
          else if (typeof v === 'bigint' || (typeof v === 'number' && Number.isInteger(v))) { out.u8(1); out.i64(v); }
          // A real number, which no table of the schema holds: as text, as
          // the installed reader takes it.
          else if (typeof v === 'number') { out.u8(2); out.str(String(v)); }
          else if (v instanceof Uint8Array) { out.u8(3); out.bytes(v); }
          else { out.u8(2); out.str(v); }
        }
      }
    }
    return out.done();
  } finally {
    db.close();
  }
}

/** The library, from the WebAssembly bytes the page carries. */
export async function library(wasm) {
  const { instance } = await WebAssembly.instantiate(wasm, {});
  const x = instance.exports;
  const call = (fn, request) => {
    const p = x.vleo_alloc(request.length);
    new Uint8Array(x.memory.buffer, p, request.length).set(request);
    const at = fn(p, request.length);
    return JSON.parse(dec.decode(new Uint8Array(x.memory.buffer, at, x.vleo_out_len())));
  };
  return {
    format: x.vleo_format(),
    /**
     * A release, checked through the signature chain from the anchor in
     * START HERE: `{ holds, signed, refused }`, or `{ error, kind }` when a
     * file itself is refused.
     */
    checkRelease(anchor, programmeRows, releaseRows) {
      const out = new Out();
      out.str(anchor);
      out.bytes(programmeRows);
      out.bytes(releaseRows);
      return call(x.vleo_check_release, out.done());
    },
    /**
     * What one release holds, checked as intake checks it: `{ holds,
     * findings: [{ level, place, what }] }`. A file from before 1.0 is
     * upgraded first, as the application opens it.
     */
    checkContent(releaseRows) {
      return call(x.vleo_check_content, releaseRows);
    },
    /**
     * A group's folder, checked against the pattern (groups/SPEC.toml) as the
     * page's group checker checks it: `{ findings: [{ level, where, msg,
     * line }] }`. The file is a group's file from before 1.0, or one upgraded
     * from it.
     */
    checkFolder(groupRows) {
      return call(x.vleo_check_folder, groupRows);
    },
  };
}

/** The library the page carries, inlined as base64 of its gzipped bytes. */
export async function carried() {
  const gz = Uint8Array.from(atob(window.VLEO_FILES_WASM || ''), c => c.charCodeAt(0));
  const wasm = new Uint8Array(await new Response(new Blob([gz]).stream().pipeThrough(new DecompressionStream('gzip'))).arrayBuffer());
  return library(wasm);
}
