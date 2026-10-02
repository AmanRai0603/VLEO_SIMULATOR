/*
  A group folder, opened from wherever the group keeps it.

  Three ways in, one result. The folder picker of every browser (a folder
  chosen with the file dialog), a folder dragged onto the page, and — where
  the browser offers it — a folder opened for reading AND writing, so a
  sign-off, a sealed package or an issue is saved straight back into it. A
  folder synced from Google Drive is an ordinary folder to all three.

  Where the browser cannot write into the folder, everything that would be
  written is offered as a download instead, named for where it belongs, and
  the page says so. Nothing is ever sent anywhere: there is no network here.
*/
'use strict';

/** A folder: every file by its path inside the folder, and how to write back. */
export class Folder {
  constructor(name, files, handle = null) {
    this.name = name;
    this.files = files;          // Map<path, File>
    this.handle = handle;        // a directory handle when it can be written
    this.urls = new Map();
  }
  has(path) { return this.files.has(path); }
  file(path) { return this.files.get(path) || null; }
  /** Paths under a prefix, sorted. */
  list(prefix = '') { return [...this.files.keys()].filter(p => p.startsWith(prefix)).sort(); }
  async text(path) {
    const f = this.files.get(path);
    if (!f) return null;
    return await f.text();
  }
  async bytes(path) {
    const f = this.files.get(path);
    return f ? new Uint8Array(await f.arrayBuffer()) : null;
  }
  /** An address the page can show an image or a video from. */
  url(path) {
    if (!this.files.has(path)) return '';
    if (!this.urls.has(path)) this.urls.set(path, URL.createObjectURL(this.files.get(path)));
    return this.urls.get(path);
  }
  get writable() { return !!this.handle; }
  /**
   * Save `body` at `path` in the folder, or offer it as a download. Answers
   * 'saved' or 'downloaded', so the caller can say which happened.
   */
  async write(path, body, type = 'text/plain') {
    const blob = body instanceof Blob ? body : new Blob([body], { type });
    if (this.handle) {
      try {
        const parts = path.split('/');
        let dir = this.handle;
        for (const p of parts.slice(0, -1)) dir = await dir.getDirectoryHandle(p, { create: true });
        const fh = await dir.getFileHandle(parts[parts.length - 1], { create: true });
        const w = await fh.createWritable();
        await w.write(blob);
        await w.close();
        this.files.set(path, new File([blob], parts[parts.length - 1], { type }));
        return 'saved';
      } catch { /* fall through to a download */ }
    }
    download(path.replace(/\//g, '__'), blob);
    // The page carries on from what it wrote, so a member can sign and then
    // seal in one sitting; the download is what makes it last.
    this.files.set(path, new File([blob], path.split('/').pop(), { type }));
    return 'downloaded';
  }
}

export function download(name, blob) {
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = name;
  document.body.appendChild(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(a.href), 4000);
}

/** The folder a file dialog returned (`<input type=file webkitdirectory>`). */
export function fromFileList(list) {
  const all = [...list];
  if (!all.length) return null;
  const top = (all[0].webkitRelativePath || all[0].name).split('/')[0];
  const files = new Map();
  for (const f of all) {
    const rel = f.webkitRelativePath || f.name;
    const path = rel.split('/').slice(1).join('/') || f.name;
    if (!ignored(path)) files.set(path, f);
  }
  return new Folder(top, files);
}

/** The folder dropped on the page. */
export async function fromDrop(dt) {
  const items = [...(dt.items || [])].map(i => i.webkitGetAsEntry && i.webkitGetAsEntry()).filter(Boolean);
  const dir = items.find(e => e.isDirectory);
  if (!dir) return null;
  const files = new Map();
  const walk = (entry, prefix) => new Promise(resolve => {
    if (entry.isFile) {
      entry.file(f => { const p = prefix + entry.name; if (!ignored(p)) files.set(p, f); resolve(); }, () => resolve());
      return;
    }
    const reader = entry.createReader();
    const batch = [];
    const more = () => reader.readEntries(async es => {
      if (!es.length) { for (const e of batch) await walk(e, prefix + entry.name + '/'); resolve(); return; }
      batch.push(...es); more();
    }, () => resolve());
    more();
  });
  const reader = dir.createReader();
  const top = [];
  await new Promise(res => {
    const more = () => reader.readEntries(es => { if (!es.length) return res(); top.push(...es); more(); }, () => res());
    more();
  });
  for (const e of top) await walk(e, '');
  return new Folder(dir.name, files);
}

/** A folder opened for reading and writing, where the browser allows it. */
export async function fromPicker() {
  if (!window.showDirectoryPicker) return null;
  return fromHandle(await window.showDirectoryPicker({ mode: 'readwrite' }));
}

/** Read (again) a folder already opened for saving. */
export async function fromHandle(handle) {
  const files = new Map();
  const walk = async (dir, prefix) => {
    for await (const [name, h] of dir.entries()) {
      const p = prefix + name;
      if (ignored(p)) continue;
      if (h.kind === 'file') files.set(p, await h.getFile());
      else await walk(h, p + '/');
    }
  };
  await walk(handle, '');
  return new Folder(handle.name, files, handle);
}

export const canPick = () => typeof window !== 'undefined' && !!window.showDirectoryPicker;

/** Files no group writes: the operating system's and the sync client's own. */
function ignored(path) {
  const base = path.split('/').pop();
  return base.startsWith('.') || base === 'Thumbs.db' || base === 'desktop.ini' || base.endsWith('~') ||
    path.split('/').some(p => p.startsWith('.'));
}
