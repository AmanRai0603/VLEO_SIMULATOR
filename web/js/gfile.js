/*
  A database file, opened from wherever the group keeps it and saved back.

  Where the browser can (Chrome and Edge), a file is opened for reading AND
  writing: Save writes it in place, on the disk or in the folder Google Drive
  syncs. Elsewhere it is read from the file dialog or a drop, and Save offers
  it as a download, named as it was, for the member to put back. The page says
  which, and never sends a file anywhere.

  The three kinds a group keeps are told apart by their name's ending as well
  as by what they say inside: <group>.vgroup (the structure), <node>.vnode (one
  node), <group>-<version>.vleo (a release).
*/
'use strict';

import { download } from './gfolder.js';
import { open } from './gstore.js';

export const ENDINGS = { structure: '.vgroup', node: '.vnode', release: '.vleo' };
const ACCEPT = { 'application/x-sqlite3': ['.vgroup', '.vnode', '.vleo'] };

/** Whether this browser can open a file for saving in place. */
export function canSaveInPlace() { return typeof window.showOpenFilePicker === 'function'; }

/** A file the page has open: its database, its name, and how to save it. */
export class DbFile {
  constructor(db, name, handle = null) {
    this.db = db;
    this.name = name;
    this.handle = handle;
    this.dirty = false;
  }
  get kind() { return this.db.meta('file_kind'); }
  /** Write the file. Answers 'saved' (in place) or 'downloaded'. */
  async save() {
    const blob = new Blob([this.db.bytes()], { type: 'application/x-sqlite3' });
    if (this.handle) {
      try {
        const w = await this.handle.createWritable();
        await w.write(blob);
        await w.close();
        this.dirty = false;
        return 'saved';
      } catch { /* the permission was withdrawn: download instead */ }
    }
    download(this.name, blob);
    this.dirty = false;
    return 'downloaded';
  }
  /** Save under a new name, chosen in the save dialog where there is one. */
  async saveAs(name) {
    if (typeof window.showSaveFilePicker === 'function') {
      try {
        const h = await window.showSaveFilePicker({ suggestedName: name, types: [{ description: 'VLEO database', accept: ACCEPT }] });
        this.handle = h;
        this.name = h.name;
        return this.save();
      } catch (e) { if (e && e.name === 'AbortError') return 'cancelled'; }
    }
    this.handle = null;
    this.name = name;
    return this.save();
  }
}

/** Save bytes as a new file: the save dialog where there is one, a download elsewhere. */
export async function saveBytesAs(name, bytes) {
  const f = new DbFile({ bytes: () => bytes }, name);
  return f.saveAs(name);
}

/** Open a database from a File (the dialog or a drop). */
export async function fromFile(file, handle = null) {
  const db = await open(new Uint8Array(await file.arrayBuffer()));
  return new DbFile(db, file.name, handle);
}

/** Open a database for saving in place. Answers null when cancelled. */
export async function pickForSaving() {
  try {
    const [h] = await window.showOpenFilePicker({ types: [{ description: 'VLEO database', accept: ACCEPT }] });
    return fromFile(await h.getFile(), h);
  } catch (e) {
    if (e && e.name === 'AbortError') return null;
    throw e;
  }
}

/** Several node files at once — the dialog, a drop, or a whole folder. */
export async function openMany(files) {
  const out = [], refused = [];
  for (const f of files) {
    if (!/\.(vnode|vgroup|vleo)$/i.test(f.name)) continue;
    try { out.push(await fromFile(f)); } catch (e) { refused.push(f.name + ': ' + e.message); }
  }
  return { files: out, refused };
}

/** A safe file name from a node or group id. */
export function fileName(id, kind, version) {
  const base = String(id || 'group').replace(/[^A-Za-z0-9_.-]+/g, '_');
  return kind === 'release' ? base + '-' + (version || '0') + ENDINGS.release : base + ENDINGS[kind];
}
