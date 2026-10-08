/*
  Signing a group folder, and sealing it for the developer.

  A FINGERPRINT is a SHA-256 over every file in a scope — a node's folder, or
  the whole group — by path and content, so the same files give the same
  fingerprint on every machine and any change at all gives a different one.
  A sign-off is recorded against a fingerprint, in reviews.csv. Change a
  file after signing and the sign-off is stale, and the page says so: nobody
  signs something other than what is there.

  SEALING writes one file the developer receives: a zip of the whole folder
  with a MANIFEST.csv (every path, its size and its SHA-256) and a SEAL.csv
  (the group, the version, the fingerprint, who signed, when). The developer
  checks the manifest before anything else, so what is built is exactly what
  the group reviewed.

  No library and no network: the hash and the zip are written out here.
*/
'use strict';

import { parseCsv, records, toCsv } from './csv.js';

// ── SHA-256 ────────────────────────────────────────────────────────────────

const K = new Uint32Array([
  0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
  0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
  0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
  0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
  0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
  0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
  0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
  0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2]);

/** The SHA-256 of bytes, as 64 hex characters. */
export function sha256(bytes) {
  const b = bytes instanceof Uint8Array ? bytes : new TextEncoder().encode(String(bytes));
  const n = b.length, padded = new Uint8Array(((n + 9 + 63) >> 6) << 6);
  padded.set(b); padded[n] = 0x80;
  const bits = n * 8;
  const dv = new DataView(padded.buffer);
  dv.setUint32(padded.length - 8, Math.floor(bits / 2 ** 32));
  dv.setUint32(padded.length - 4, bits >>> 0);
  const H = new Uint32Array([0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]);
  const W = new Uint32Array(64);
  const r = (x, s) => (x >>> s) | (x << (32 - s));
  for (let o = 0; o < padded.length; o += 64) {
    for (let i = 0; i < 16; i++) W[i] = dv.getUint32(o + i * 4);
    for (let i = 16; i < 64; i++) {
      const s0 = r(W[i - 15], 7) ^ r(W[i - 15], 18) ^ (W[i - 15] >>> 3);
      const s1 = r(W[i - 2], 17) ^ r(W[i - 2], 19) ^ (W[i - 2] >>> 10);
      W[i] = (W[i - 16] + s0 + W[i - 7] + s1) >>> 0;
    }
    let [a, bb, c, d, e, f, g, h] = H;
    for (let i = 0; i < 64; i++) {
      const t1 = (h + (r(e, 6) ^ r(e, 11) ^ r(e, 25)) + ((e & f) ^ (~e & g)) + K[i] + W[i]) >>> 0;
      const t2 = ((r(a, 2) ^ r(a, 13) ^ r(a, 22)) + ((a & bb) ^ (a & c) ^ (bb & c))) >>> 0;
      h = g; g = f; f = e; e = (d + t1) >>> 0; d = c; c = bb; bb = a; a = (t1 + t2) >>> 0;
    }
    H[0] += a; H[1] += bb; H[2] += c; H[3] += d; H[4] += e; H[5] += f; H[6] += g; H[7] += h;
  }
  return [...H].map(v => (v >>> 0).toString(16).padStart(8, '0')).join('');
}

// ── fingerprints and sign-offs ─────────────────────────────────────────────

/** Files that are the folder's record of itself, never part of what is signed. */
const unsigned = p => p === 'reviews.csv' || p.startsWith('packages/') || p.startsWith('issues/');

/** `{ fingerprint, files: [{path, bytes, sha}] }` over the paths a scope covers. */
export async function fingerprint(folder, scope) {
  const paths = folder.list(scope === 'group' ? '' : 'nodes/' + scope + '/').filter(p => !unsigned(p));
  const files = [];
  for (const p of paths) {
    const bytes = await folder.bytes(p);
    files.push({ path: p, bytes: bytes.length, sha: sha256(bytes) });
  }
  return { fingerprint: sha256(files.map(f => f.path + '\u0000' + f.sha).join('\n')), files };
}

/** The sign-offs, each marked current or stale against the fingerprint it was given for. */
export async function reviewState(folder, model) {
  const rows = model.group.reviews;
  const prints = new Map();
  const fp = async s => { if (!prints.has(s)) prints.set(s, (await fingerprint(folder, s)).fingerprint); return prints.get(s); };
  const out = [];
  for (const r of rows) out.push({ ...r, current: r.fingerprint === await fp(r.scope || 'group') });
  return { reviews: out, fingerprintOf: fp };
}

/** Who may sign a scope: the owner signs the group; a node's engineers (or `*`) sign it. */
export function maySign(model, name, scope) {
  const m = model.group.members.find(x => x.name === name);
  if (!m) return false;
  if (scope === 'group') return m.role === 'owner';
  const nodes = String(m.nodes || '').split(/\s+/);
  return m.role === 'owner' || nodes.includes('*') || nodes.includes(scope);
}

/** Add a sign-off to reviews.csv; answers 'saved' or 'downloaded'. */
export async function sign(folder, model, { name, scope, verdict, note }) {
  const { fingerprint: fpv } = await fingerprint(folder, scope);
  const head = ['name', 'scope', 'version', 'fingerprint', 'date', 'verdict', 'note'];
  const old = await folder.text('reviews.csv');
  const t = old ? parseCsv(old) : { head, rows: [] };
  const rows = old ? records(t).map(r => head.map(h => r[h] ?? '')) : [];
  rows.push([name, scope, model.meta.version || '', fpv, new Date().toISOString().slice(0, 10), verdict, note || '']);
  return folder.write('reviews.csv', toCsv(head, rows), 'text/csv');
}

/**
 * What still stands between this folder and a seal: errors, and every node
 * and the group without a current ok from somebody who may sign it.
 */
export async function sealBlockers(folder, model, findings) {
  const out = [];
  const errors = findings.filter(f => f.level === 'error').length;
  if (errors) out.push(errors + ' error(s) in the checks');
  const { reviews } = await reviewState(folder, model);
  const ok = scope => reviews.some(r => r.scope === scope && r.verdict === 'ok' && r.current && maySign(model, r.name, scope));
  for (const id of model.order) if (!ok(id)) out.push(id + ' has no current sign-off from its node engineer');
  if (!ok('group')) out.push('the group has no current sign-off from its owner');
  return out;
}

// ── the sealed package ─────────────────────────────────────────────────────

const CRC = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) { let c = n; for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1; t[n] = c >>> 0; }
  return t;
})();
function crc32(b) { let c = 0xffffffff; for (let i = 0; i < b.length; i++) c = CRC[(c ^ b[i]) & 0xff] ^ (c >>> 8); return (c ^ 0xffffffff) >>> 0; }

/** A zip of `[{path, data: Uint8Array}]`, stored without compression, readable everywhere. */
export function zip(entries) {
  const enc = new TextEncoder();
  const parts = [], central = [];
  let offset = 0;
  for (const e of entries) {
    const name = enc.encode(e.path), crc = crc32(e.data), size = e.data.length;
    const h = new DataView(new ArrayBuffer(30));
    h.setUint32(0, 0x04034b50, true); h.setUint16(4, 20, true); h.setUint16(6, 0x0800, true);
    h.setUint16(8, 0, true); h.setUint16(10, 0, true); h.setUint16(12, 0x21, true);
    h.setUint32(14, crc, true); h.setUint32(18, size, true); h.setUint32(22, size, true);
    h.setUint16(26, name.length, true); h.setUint16(28, 0, true);
    parts.push(new Uint8Array(h.buffer), name, e.data);
    const c = new DataView(new ArrayBuffer(46));
    c.setUint32(0, 0x02014b50, true); c.setUint16(4, 20, true); c.setUint16(6, 20, true); c.setUint16(8, 0x0800, true);
    c.setUint16(10, 0, true); c.setUint16(12, 0, true); c.setUint16(14, 0x21, true);
    c.setUint32(16, crc, true); c.setUint32(20, size, true); c.setUint32(24, size, true);
    c.setUint16(28, name.length, true); c.setUint32(42, offset, true);
    central.push(new Uint8Array(c.buffer), name);
    offset += 30 + name.length + size;
  }
  const csize = central.reduce((s, p) => s + p.length, 0);
  const end = new DataView(new ArrayBuffer(22));
  end.setUint32(0, 0x06054b50, true); end.setUint16(8, entries.length, true); end.setUint16(10, entries.length, true);
  end.setUint32(12, csize, true); end.setUint32(16, offset, true);
  return new Blob([...parts, ...central, new Uint8Array(end.buffer)], { type: 'application/zip' });
}

/** Seal the folder: answers `{ name, how, fingerprint }`. Call only when sealBlockers is empty. */
export async function seal(folder, model, sealedBy) {
  const fp = await fingerprint(folder, 'group');
  const { reviews } = await reviewState(folder, model);
  const id = model.meta.id || folder.name, version = model.meta.version || '0';
  const enc = new TextEncoder();
  const entries = [];
  for (const f of fp.files) entries.push({ path: f.path, data: await folder.bytes(f.path) });
  if (folder.has('reviews.csv')) entries.push({ path: 'reviews.csv', data: await folder.bytes('reviews.csv') });
  entries.push({ path: 'MANIFEST.csv', data: enc.encode(toCsv(['path', 'bytes', 'sha256'], fp.files.map(f => [f.path, f.bytes, f.sha]))) });
  const signed = reviews.filter(r => r.current && r.verdict === 'ok').map(r => r.name + ':' + r.scope).join(' ');
  entries.push({ path: 'SEAL.csv', data: enc.encode(toCsv(['group', 'version', 'fingerprint', 'sealed', 'sealed_by', 'signed', 'spec'],
    [[id, version, fp.fingerprint, new Date().toISOString(), sealedBy, signed, String((window.VLEO_GROUP_SPEC || {}).version || '')]])) });
  const name = 'packages/' + id + '-' + version + '.zip';
  const how = await folder.write(name, zip(entries), 'application/zip');
  return { name, how, fingerprint: fp.fingerprint };
}

/** Write an issue a member raises while looking at the folder or the design. */
export async function raiseIssue(folder, { by, where, text, version }) {
  const date = new Date().toISOString().slice(0, 10);
  const slug = String(text).toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 40) || 'issue';
  const body = '# ' + String(text).split('\n')[0].slice(0, 100) + '\n\n' +
    '- raised by: ' + by + '\n- on: ' + date + '\n- folder version: ' + (version || '') + '\n- where: ' + where + '\n\n' + text + '\n';
  return { path: 'issues/' + date + '-' + slug + '.md', how: await folder.write('issues/' + date + '-' + slug + '.md', body, 'text/markdown') };
}
